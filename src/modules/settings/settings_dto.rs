use crate::{
    shared::base::icons::IconSet,
    tuikk_core::{
        cli::RATE_RANGE,
        config::{AppConfig, QueryMode},
        docker_conn::{DefaultConnectionType, DockerConfig},
        themes::ThemeColor,
        ui_config::UiConfig,
    },
};

#[derive(Debug, Clone)]
pub struct SettingsDraft {
    pub theme: String,
    pub icon_set: IconSet,
    pub query_mode: QueryMode,
    pub log_level: String,
    pub log_dir: String,
    pub frame_rate: f64,
    pub tick_rate: f64,

    pub conn_type: DefaultConnectionType,
    pub addr_or_path: String,
    pub timeout: u64,
    pub client_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeybindingRow {
    pub scope: String,
    pub key_seq: String,
    pub command: String,
}

impl Default for SettingsDraft {
    fn default() -> Self {
        Self::from_config(&AppConfig::default())
    }
}

impl SettingsDraft {
    pub fn from_config(cfg: &AppConfig) -> Self {
        Self {
            theme: cfg.theme.clone(),
            icon_set: cfg.icon_set,
            query_mode: cfg.query_mode,
            log_level: cfg.log_level.clone(),
            log_dir: cfg.log_dir.clone().unwrap_or_default(),
            frame_rate: cfg.ui.frame_rate,
            tick_rate: cfg.ui.tick_rate,
            conn_type: cfg.docker.default_conn_type,
            addr_or_path: cfg.docker.addr_or_path.clone().unwrap_or_default(),
            timeout: cfg.docker.timeout,
            client_version: cfg.docker.client_version.clone().unwrap_or_default(),
        }
    }

    pub fn apply_to_config(&self, base: &AppConfig) -> AppConfig {
        let log_dir_trimmed = self.log_dir.trim();
        let addr_trimmed = self.addr_or_path.trim();
        let ver_trimmed = self.client_version.trim();

        let ui = UiConfig {
            frame_rate: self.frame_rate,
            tick_rate: self.tick_rate,
        }
        .sanitized();

        AppConfig {
            theme: self.theme.clone(),
            theme_color: ThemeColor::default(Some(self.theme.clone())),
            ui,
            icon_set: self.icon_set,
            log_level: self.log_level.clone(),
            log_dir: if log_dir_trimmed.is_empty() {
                None
            } else {
                Some(log_dir_trimmed.to_owned())
            },
            docker: DockerConfig {
                default_conn_type: self.conn_type,
                addr_or_path: if addr_trimmed.is_empty() {
                    None
                } else {
                    Some(addr_trimmed.to_owned())
                },
                timeout: self.timeout.max(1),
                client_version: if ver_trimmed.is_empty() {
                    None
                } else {
                    Some(ver_trimmed.to_owned())
                },
            },
            keybindings: base.keybindings.clone(),
            query_mode: self.query_mode,
        }
    }

    pub fn cycle_theme(&mut self) {
        self.theme = if self.theme == "light" {
            "dark".to_owned()
        } else {
            "light".to_owned()
        };
    }

    pub fn cycle_icon_set(&mut self, forward: bool) {
        self.icon_set = match (self.icon_set, forward) {
            (IconSet::Ascii, true) => IconSet::Unicode,
            (IconSet::Unicode, true) => IconSet::Nerd,
            (IconSet::Nerd, true) => IconSet::Ascii,
            (IconSet::Ascii, false) => IconSet::Nerd,
            (IconSet::Unicode, false) => IconSet::Ascii,
            (IconSet::Nerd, false) => IconSet::Unicode,
        };
    }

    pub fn icon_set_label(&self) -> &'static str {
        match self.icon_set {
            IconSet::Ascii => "ascii",
            IconSet::Unicode => "unicode",
            IconSet::Nerd => "nerd",
        }
    }

    pub fn cycle_query_mode(&mut self) {
        self.query_mode = match self.query_mode {
            QueryMode::Local => QueryMode::Remote,
            QueryMode::Remote => QueryMode::Local,
        };
    }

    pub fn query_mode_label(&self) -> &'static str {
        match self.query_mode {
            QueryMode::Local => "local",
            QueryMode::Remote => "remote",
        }
    }

    pub fn cycle_log_level(&mut self, forward: bool) {
        const LEVELS: [&str; 5] = ["error", "warn", "info", "debug", "trace"];
        let idx = LEVELS
            .iter()
            .position(|&l| l.eq_ignore_ascii_case(&self.log_level))
            .unwrap_or(0);
        let next = if forward {
            (idx + 1) % LEVELS.len()
        } else if idx == 0 {
            LEVELS.len() - 1
        } else {
            idx - 1
        };
        self.log_level = LEVELS[next].to_owned();
    }

    pub fn cycle_conn_type(&mut self, forward: bool) {
        self.conn_type = match (self.conn_type, forward) {
            (DefaultConnectionType::SOCKET, true) => DefaultConnectionType::LOCAL,
            (DefaultConnectionType::LOCAL, true) => DefaultConnectionType::HTTP,
            (DefaultConnectionType::HTTP, true) => DefaultConnectionType::SOCKET,
            (DefaultConnectionType::SOCKET, false) => DefaultConnectionType::HTTP,
            (DefaultConnectionType::LOCAL, false) => DefaultConnectionType::SOCKET,
            (DefaultConnectionType::HTTP, false) => DefaultConnectionType::LOCAL,
        };
    }

    pub fn conn_type_label(&self) -> &'static str {
        match self.conn_type {
            DefaultConnectionType::SOCKET => "socket",
            DefaultConnectionType::LOCAL => "local",
            DefaultConnectionType::HTTP => "http",
        }
    }

    pub fn adjust_frame_rate(&mut self, delta: f64) {
        self.frame_rate = (self.frame_rate + delta).clamp(*RATE_RANGE.start(), *RATE_RANGE.end());
    }

    pub fn adjust_tick_rate(&mut self, delta: f64) {
        self.tick_rate = (self.tick_rate + delta).clamp(*RATE_RANGE.start(), *RATE_RANGE.end());
    }

    pub fn adjust_timeout(&mut self, delta: i64) {
        let next = (self.timeout as i64 + delta).clamp(1, 3600);
        self.timeout = next as u64;
    }
}

