use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::modules::dashboard::constants::{GB, KB, MAX_HISTORY_POINTS, MB, TB};

/// Per-container CPU, RAM, and storage metrics displayed on the Dashboard.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContainerResourceMetric {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub cpu_percent: f64,
    pub mem_usage: u64,
    pub mem_limit: u64,
    pub mem_percent: f64,
    pub storage_rw: u64,
    pub storage_rootfs: u64,
}

/// Summary of Docker disk/storage usage across images, containers, and volumes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageSummary {
    pub images_total_size: u64,
    pub images_reclaimable: u64,
    pub containers_total_size: u64,
    pub containers_reclaimable: u64,
    pub volumes_total_size: u64,
}

impl StorageSummary {
    pub fn total_used(&self) -> u64 {
        self.images_total_size
            .saturating_add(self.containers_total_size)
            .saturating_add(self.volumes_total_size)
    }
}

/// Complete snapshot returned by `DashboardService::fetch_snapshot`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub engine_version: String,
    pub os_arch: String,
    pub ncpu: u64,
    pub host_mem_total: u64,

    pub containers_total: usize,
    pub containers_running: usize,
    pub containers_paused: usize,
    pub containers_stopped: usize,
    pub images_total: usize,

    pub total_cpu_percent: f64,
    pub total_mem_usage: u64,
    pub total_mem_limit: u64,
    pub total_mem_percent: f64,

    pub storage: StorageSummary,
    pub containers: Vec<ContainerResourceMetric>,
}

/// Ring buffer holding time-series history for CPU%, RAM%, and Storage usage.
#[derive(Debug, Clone)]
pub struct MetricHistory {
    pub cpu_points: VecDeque<f64>,
    pub mem_points: VecDeque<f64>,
    pub storage_points: VecDeque<f64>,
    max_len: usize,
}

impl Default for MetricHistory {
    fn default() -> Self {
        Self::new(MAX_HISTORY_POINTS)
    }
}

impl MetricHistory {
    pub fn new(max_len: usize) -> Self {
        let cap = max_len.max(1);
        Self {
            cpu_points: VecDeque::with_capacity(cap),
            mem_points: VecDeque::with_capacity(cap),
            storage_points: VecDeque::with_capacity(cap),
            max_len: cap,
        }
    }

    pub fn push(&mut self, cpu_pct: f64, mem_pct: f64, storage_mb: f64) {
        Self::push_bounded(
            &mut self.cpu_points,
            cpu_pct.clamp(0.0, 100.0),
            self.max_len,
        );
        Self::push_bounded(
            &mut self.mem_points,
            mem_pct.clamp(0.0, 100.0),
            self.max_len,
        );
        Self::push_bounded(&mut self.storage_points, storage_mb.max(0.0), self.max_len);
    }

    fn push_bounded(buf: &mut VecDeque<f64>, val: f64, max_len: usize) {
        if buf.len() >= max_len {
            buf.pop_front();
        }
        buf.push_back(val);
    }

    pub fn cpu_dataset(&self) -> Vec<(f64, f64)> {
        self.cpu_points
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v))
            .collect()
    }

    pub fn mem_dataset(&self) -> Vec<(f64, f64)> {
        self.mem_points
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v))
            .collect()
    }

    pub fn storage_dataset(&self) -> Vec<(f64, f64)> {
        self.storage_points
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v))
            .collect()
    }

    pub fn max_len(&self) -> usize {
        self.max_len
    }
}

/// Format raw bytes into a compact human-readable string (B, KB, MB, GB, TB).
pub fn format_bytes(bytes: u64) -> String {
    let b = bytes as f64;
    if b >= TB {
        format!("{:.2} TB", b / TB)
    } else if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

