use std::{
    cmp::Ordering,
    fs,
    path::{Component, Path, PathBuf},
    time::UNIX_EPOCH,
};

use crate::{
    modules::logs::{
        constants::{LogDetailTab, LogSort},
        log_dto::{LogEntry, LogFileSummary, LogLevel, parse_log_line},
    },
    shared::base::{base_err::BaseErr, base_filter::BaseFilter},
    tuikk_core::{config::AppConfig, logging::resolve_log_dir},
};

pub struct LogService {
    log_dir: PathBuf,
}

impl LogService {
    pub fn new(log_dir: PathBuf) -> Self {
        Self { log_dir }
    }

    pub fn from_config() -> Self {
        let cfg = AppConfig::global();
        Self::new(resolve_log_dir(&cfg.log_dir))
    }

    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }

    /// Resolve and validate a log file name inside `self.log_dir` (preventing path traversal).
    fn resolve_safe_path(&self, file_name: &str) -> Result<PathBuf, BaseErr> {
        let trimmed = file_name.trim();
        if trimmed.is_empty() {
            return Err(BaseErr::BadRequest {
                message: "Log file name cannot be empty".to_owned(),
            });
        }

        let rel = Path::new(trimmed);
        let mut components = rel.components();
        match (components.next(), components.next()) {
            (Some(Component::Normal(_)), None) => {}
            _ => {
                return Err(BaseErr::BadRequest {
                    message: format!("Invalid log file name '{file_name}'"),
                });
            }
        }

        Ok(self.log_dir.join(rel))
    }

    /// Read all `.log` files in `self.log_dir` and compute summary stats (`error_count`, `warn_count`).
    pub fn fetch_log_files(&self) -> Result<Vec<LogFileSummary>, BaseErr> {
        if !self.log_dir.exists() {
            return Ok(Vec::new());
        }

        let mut summaries = Vec::new();
        for entry_res in fs::read_dir(&self.log_dir)? {
            let entry = entry_res?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            if !name.ends_with(".log") {
                continue;
            }

            let meta = entry.metadata()?;
            let size = meta.len();
            let modified_epoch = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let content = fs::read_to_string(&path).unwrap_or_default();
            let mut total_lines = 0usize;
            let mut error_count = 0usize;
            let mut warn_count = 0usize;
            let mut last_error_preview: Option<String> = None;

            for (i, line) in content.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                total_lines += 1;
                let parsed = parse_log_line(i + 1, line);
                match parsed.level {
                    LogLevel::Error => {
                        error_count += 1;
                        last_error_preview = Some(parsed.message);
                    }
                    LogLevel::Warn => {
                        warn_count += 1;
                        if last_error_preview.is_none() {
                            last_error_preview = Some(parsed.message);
                        }
                    }
                    _ => {}
                }
            }

            summaries.push(LogFileSummary {
                file_name: name.to_owned(),
                path,
                size,
                modified_epoch,
                total_lines,
                error_count,
                warn_count,
                last_error_preview,
            });
        }

        // Default order: newest modified / name descending first
        summaries.sort_by(|a, b| {
            b.modified_epoch
                .cmp(&a.modified_epoch)
                .then_with(|| b.file_name.cmp(&a.file_name))
        });

        Ok(summaries)
    }

    pub fn query_log_files<'a>(
        &self,
        items: &'a [LogFileSummary],
        filter: &BaseFilter,
    ) -> Vec<&'a LogFileSummary> {
        let needle = Self::needle(filter);
        let sort = Self::parse_sort(filter);

        let mut v: Vec<&LogFileSummary> = items
            .iter()
            .filter(|f| needle.is_empty() || self.match_search(f, &needle))
            .collect();

        if sort.is_some() {
            v.sort_by(|a, b| Self::compare(sort, a, b));
        }
        v
    }

    /// Read and parse all non-empty log lines from a specific log file.
    pub fn read_log_entries(&self, file_name: &str) -> Result<Vec<LogEntry>, BaseErr> {
        let path = self.resolve_safe_path(file_name)?;
        if !path.exists() {
            return Err(BaseErr::NotFound {
                resource_type: "LogFile",
                id: file_name.to_owned(),
            });
        }

        let content = fs::read_to_string(&path)?;
        let entries = content
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty())
            .map(|(i, line)| parse_log_line(i + 1, line))
            .collect();
        Ok(entries)
    }

    /// Filter parsed entries according to the active `LogDetailTab` (`WarnAndError`, `ErrorsOnly`, `All`, `Raw`).
    pub fn filter_entries_by_tab<'a>(
        entries: &'a [LogEntry],
        tab: LogDetailTab,
    ) -> Vec<&'a LogEntry> {
        entries
            .iter()
            .filter(|e| match tab {
                LogDetailTab::WarnAndError => e.level.is_warn_or_error(),
                LogDetailTab::ErrorsOnly => e.level.is_error(),
                LogDetailTab::All | LogDetailTab::Raw => true,
            })
            .collect()
    }

    /// Permanently delete a log file from `self.log_dir`.
    pub fn delete_log_file(&self, file_name: &str) -> Result<(), BaseErr> {
        let path = self.resolve_safe_path(file_name)?;
        if !path.exists() {
            return Err(BaseErr::NotFound {
                resource_type: "LogFile",
                id: file_name.to_owned(),
            });
        }
        fs::remove_file(path)?;
        Ok(())
    }

    /// Truncate/clear a log file's contents without deleting the file itself.
    pub fn clear_log_file(&self, file_name: &str) -> Result<(), BaseErr> {
        let path = self.resolve_safe_path(file_name)?;
        if !path.exists() {
            return Err(BaseErr::NotFound {
                resource_type: "LogFile",
                id: file_name.to_owned(),
            });
        }
        fs::write(path, "")?;
        Ok(())
    }

    /// Delete all `.log` files in `self.log_dir` and return the number of deleted files.
    pub fn delete_all_logs(&self) -> Result<usize, BaseErr> {
        if !self.log_dir.exists() {
            return Ok(0);
        }
        let mut deleted = 0usize;
        for entry_res in fs::read_dir(&self.log_dir)? {
            let entry = entry_res?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("log") {
                if fs::remove_file(&path).is_ok() {
                    deleted += 1;
                }
            }
        }
        Ok(deleted)
    }
}

impl LogService {
    fn contains_ci(&self, v: &str, search: &str) -> bool {
        v.to_lowercase().contains(search)
    }

    fn match_search(&self, f: &LogFileSummary, search: &str) -> bool {
        self.contains_ci(&f.file_name, search)
            || f.last_error_preview
                .as_deref()
                .is_some_and(|p| self.contains_ci(p, search))
    }

    fn needle(f: &BaseFilter) -> String {
        f.search
            .as_deref()
            .map(str::trim)
            .unwrap_or("")
            .to_lowercase()
    }

    fn parse_sort(f: &BaseFilter) -> Option<(LogSort, bool)> {
        f.sort
            .as_deref()
            .filter(|s| !s.is_empty())
            .and_then(LogSort::parse)
    }

    fn compare(
        sort: Option<(LogSort, bool)>,
        a: &LogFileSummary,
        b: &LogFileSummary,
    ) -> Ordering {
        let Some((key, desc)) = sort else {
            return Ordering::Equal;
        };
        let ord = match key {
            LogSort::Modified => a
                .modified_epoch
                .cmp(&b.modified_epoch)
                .then_with(|| a.file_name.cmp(&b.file_name)),
            LogSort::Name => a.file_name.cmp(&b.file_name),
            LogSort::Size => a.size.cmp(&b.size),
            LogSort::Errors => a
                .error_count
                .cmp(&b.error_count)
                .then_with(|| a.warn_count.cmp(&b.warn_count)),
        };
        if desc { ord.reverse() } else { ord }
    }
}

