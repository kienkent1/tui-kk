use std::{collections::HashMap, sync::Arc};

use bollard::{
    Docker,
    models::{
        ContainerStatsResponse, ContainerSummary, ContainerSummaryStateEnum,
        SystemDataUsageResponse, SystemInfo,
    },
    query_parameters::{DataUsageOptions, ListContainersOptions, StatsOptions},
};
use futures_util::{StreamExt, future::join_all};

use crate::{
    modules::dashboard::dashboard_dto::{
        ContainerResourceMetric, DashboardSnapshot, StorageSummary,
    },
    shared::base::base_err::BaseErr,
};

pub struct DashboardService {
    conn: Arc<Docker>,
}

impl DashboardService {
    pub fn new(conn: Arc<Docker>) -> Self {
        Self { conn }
    }

    pub async fn fetch_system_info(&self) -> Result<SystemInfo, BaseErr> {
        Ok(self.conn.info().await?)
    }

    pub async fn fetch_data_usage(&self) -> Result<SystemDataUsageResponse, BaseErr> {
        let opts = DataUsageOptions {
            _type: None,
            verbose: false,
        };
        Ok(self.conn.df(Some(opts)).await?)
    }

    pub async fn fetch_containers_with_size(&self) -> Result<Vec<ContainerSummary>, BaseErr> {
        let query = ListContainersOptions {
            all: true,
            limit: None,
            size: true,
            filters: None,
        };
        Ok(self.conn.list_containers(Some(query)).await?)
    }

    pub async fn fetch_container_stat_once(
        &self,
        id: &str,
    ) -> Result<Option<ContainerStatsResponse>, BaseErr> {
        let opts = StatsOptions {
            stream: false,
            one_shot: false,
        };
        let mut stream = self.conn.stats(id, Some(opts));
        match stream.next().await {
            Some(Ok(stat)) => Ok(Some(stat)),
            Some(Err(e)) => Err(BaseErr::from(e)),
            None => Ok(None),
        }
    }

    /// Collect a complete Dashboard snapshot (Engine info, storage usage, and per-container CPU/RAM/Storage).
    pub async fn fetch_snapshot(&self) -> Result<DashboardSnapshot, BaseErr> {
        let (info_res, df_res, containers_res) = tokio::join!(
            self.fetch_system_info(),
            self.fetch_data_usage(),
            self.fetch_containers_with_size()
        );

        let info = info_res?;
        let df = df_res.unwrap_or_default();
        let summaries = containers_res?;

        let running_ids: Vec<String> = summaries
            .iter()
            .filter(|c| c.state == Some(ContainerSummaryStateEnum::RUNNING))
            .filter_map(|c| c.id.clone())
            .collect();

        let stat_futures = running_ids.iter().map(|id| async move {
            let stat = self.fetch_container_stat_once(id).await.ok().flatten();
            (id.clone(), stat)
        });
        let stats_list = join_all(stat_futures).await;
        let stats_map: HashMap<String, ContainerStatsResponse> = stats_list
            .into_iter()
            .filter_map(|(id, st)| st.map(|s| (id, s)))
            .collect();

        Ok(Self::build_snapshot(&info, &df, &summaries, &stats_map))
    }

    /// Pure function that builds a `DashboardSnapshot` from raw Docker responses.
    pub fn build_snapshot(
        info: &SystemInfo,
        df: &SystemDataUsageResponse,
        summaries: &[ContainerSummary],
        stats_map: &HashMap<String, ContainerStatsResponse>,
    ) -> DashboardSnapshot {
        let ncpu = info.ncpu.unwrap_or(1).max(1) as u64;
        let host_mem_total = info.mem_total.unwrap_or(0).max(0) as u64;

        let mut metrics: Vec<ContainerResourceMetric> = summaries
            .iter()
            .map(|c| {
                let id = c.id.clone().unwrap_or_default();
                let short_id = id.chars().take(12).collect::<String>();
                let name = c
                    .names
                    .as_ref()
                    .and_then(|n| n.first())
                    .map(|s| s.trim_start_matches('/').to_owned())
                    .unwrap_or_else(|| short_id.clone());
                let image = c.image.clone().unwrap_or_else(|| "-".to_owned());
                let state = c.state.map(|s| s.to_string()).unwrap_or_default();

                let storage_rw = c.size_rw.unwrap_or(0).max(0) as u64;
                let storage_rootfs = c.size_root_fs.unwrap_or(0).max(0) as u64;

                let (cpu_percent, mem_usage, mem_limit, mem_percent) =
                    if let Some(st) = stats_map.get(&id) {
                        let cpu = Self::calculate_cpu_percent(st);
                        let (used, limit, pct) = Self::calculate_mem_usage(st, host_mem_total);
                        (cpu, used, limit, pct)
                    } else {
                        (0.0, 0, host_mem_total, 0.0)
                    };

                ContainerResourceMetric {
                    id: short_id,
                    name,
                    image,
                    state,
                    cpu_percent,
                    mem_usage,
                    mem_limit,
                    mem_percent,
                    storage_rw,
                    storage_rootfs,
                }
            })
            .collect();

        // Sort containers: running & highest CPU/RAM usage first
        metrics.sort_by(|a, b| {
            b.cpu_percent
                .partial_cmp(&a.cpu_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.mem_usage.cmp(&a.mem_usage))
                .then_with(|| b.storage_rootfs.cmp(&a.storage_rootfs))
        });

        let total_cpu_raw: f64 = metrics.iter().map(|m| m.cpu_percent).sum();
        let total_cpu_percent = (total_cpu_raw / ncpu as f64).clamp(0.0, 100.0);

        let total_mem_usage: u64 = metrics.iter().map(|m| m.mem_usage).sum();
        let total_mem_limit = if host_mem_total > 0 {
            host_mem_total
        } else {
            metrics.iter().map(|m| m.mem_limit).max().unwrap_or(0)
        };
        let total_mem_percent = if total_mem_limit > 0 {
            ((total_mem_usage as f64 / total_mem_limit as f64) * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        let storage = Self::extract_storage_summary(df, &metrics);

        let os = info.operating_system.as_deref().unwrap_or("Linux");
        let arch = info.architecture.as_deref().unwrap_or("x86_64");

        DashboardSnapshot {
            engine_version: info
                .server_version
                .clone()
                .unwrap_or_else(|| "unknown".to_owned()),
            os_arch: format!("{os} ({arch})"),
            ncpu,
            host_mem_total,
            containers_total: info.containers.unwrap_or(summaries.len() as i64).max(0) as usize,
            containers_running: info.containers_running.unwrap_or(0).max(0) as usize,
            containers_paused: info.containers_paused.unwrap_or(0).max(0) as usize,
            containers_stopped: info.containers_stopped.unwrap_or(0).max(0) as usize,
            images_total: info.images.unwrap_or(0).max(0) as usize,
            total_cpu_percent,
            total_mem_usage,
            total_mem_limit,
            total_mem_percent,
            storage,
            containers: metrics,
        }
    }

    /// Standard Docker CLI formula for CPU % calculation from `ContainerStatsResponse`.
    pub fn calculate_cpu_percent(stats: &ContainerStatsResponse) -> f64 {
        let Some(cpu_stats) = &stats.cpu_stats else {
            return 0.0;
        };
        let Some(precpu_stats) = &stats.precpu_stats else {
            return 0.0;
        };

        let cur_total = cpu_stats
            .cpu_usage
            .as_ref()
            .and_then(|u| u.total_usage)
            .unwrap_or(0);
        let prev_total = precpu_stats
            .cpu_usage
            .as_ref()
            .and_then(|u| u.total_usage)
            .unwrap_or(0);

        let cur_sys = cpu_stats.system_cpu_usage.unwrap_or(0);
        let prev_sys = precpu_stats.system_cpu_usage.unwrap_or(0);

        let cpu_delta = cur_total.saturating_sub(prev_total) as f64;
        let sys_delta = cur_sys.saturating_sub(prev_sys) as f64;

        if cpu_delta <= 0.0 || sys_delta <= 0.0 {
            return 0.0;
        }

        let online_cpus = cpu_stats
            .online_cpus
            .filter(|&n| n > 0)
            .or_else(|| {
                cpu_stats
                    .cpu_usage
                    .as_ref()
                    .and_then(|u| u.percpu_usage.as_ref().map(|v| v.len() as u32))
                    .filter(|&n| n > 0)
            })
            .unwrap_or(1) as f64;

        ((cpu_delta / sys_delta) * online_cpus * 100.0).max(0.0)
    }

    /// Standard Docker CLI formula for memory usage (subtracting page cache / inactive_file).
    /// Returns `(used_bytes, limit_bytes, percent)`.
    pub fn calculate_mem_usage(
        stats: &ContainerStatsResponse,
        fallback_limit: u64,
    ) -> (u64, u64, f64) {
        let Some(mem) = &stats.memory_stats else {
            return (0, fallback_limit, 0.0);
        };

        let raw_usage = mem.usage.or(mem.privateworkingset).unwrap_or(0);
        let cache = mem
            .stats
            .as_ref()
            .and_then(|m| m.get("inactive_file").or_else(|| m.get("cache")).copied())
            .unwrap_or(0);

        let used = raw_usage.saturating_sub(cache);
        let limit = mem.limit.filter(|&l| l > 0).unwrap_or(fallback_limit);

        let pct = if limit > 0 {
            ((used as f64 / limit as f64) * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        (used, limit, pct)
    }

    pub fn extract_storage_summary(
        df: &SystemDataUsageResponse,
        containers: &[ContainerResourceMetric],
    ) -> StorageSummary {
        let images_total_size = df
            .image_usage
            .as_ref()
            .and_then(|u| u.total_size)
            .unwrap_or(0)
            .max(0) as u64;
        let images_reclaimable = df
            .image_usage
            .as_ref()
            .and_then(|u| u.reclaimable)
            .unwrap_or(0)
            .max(0) as u64;

        let df_containers_size = df
            .container_usage
            .as_ref()
            .and_then(|u| u.total_size)
            .unwrap_or(0)
            .max(0) as u64;
        let containers_reclaimable = df
            .container_usage
            .as_ref()
            .and_then(|u| u.reclaimable)
            .unwrap_or(0)
            .max(0) as u64;

        let sum_rw: u64 = containers.iter().map(|c| c.storage_rw).sum();
        let sum_rootfs: u64 = containers.iter().map(|c| c.storage_rootfs).sum();
        let containers_total_size = if df_containers_size > 0 {
            df_containers_size
        } else if sum_rw > 0 {
            sum_rw
        } else {
            sum_rootfs
        };

        let volumes_total_size = df
            .volume_usage
            .as_ref()
            .and_then(|u| u.total_size)
            .unwrap_or(0)
            .max(0) as u64;

        StorageSummary {
            images_total_size,
            images_reclaimable,
            containers_total_size,
            containers_reclaimable,
            volumes_total_size,
        }
    }
}

