use std::path::PathBuf;

use crate::{
    modules::settings::settings_dto::{KeybindingRow, SettingsDraft},
    tuikk_core::{
        config::AppConfig,
        docker_conn::DockerConnection,
        key_map::{KeyMap, KeyScope},
    },
};

pub struct SettingsService;

impl SettingsService {
    pub fn new() -> Self {
        Self
    }

    pub fn config_file_path(&self) -> PathBuf {
        AppConfig::config_path().unwrap_or_else(|_| PathBuf::from(".config/config.toml"))
    }

    pub fn load_draft(&self) -> SettingsDraft {
        let cfg = AppConfig::global();
        SettingsDraft::from_config(&cfg)
    }

    pub fn reload_from_disk(&self) -> Result<SettingsDraft, String> {
        let cfg = AppConfig::load_from_file().map_err(|e| e.to_string())?;
        Ok(SettingsDraft::from_config(&cfg))
    }

    pub fn save_draft(&self, draft: &SettingsDraft) -> Result<AppConfig, String> {
        let current = AppConfig::global();
        let updated = draft.apply_to_config(&current);
        AppConfig::update_global(updated.clone()).map_err(|e| e.to_string())?;
        Ok(updated)
    }

    pub async fn reload_and_ping_docker(&self, draft: &SettingsDraft) -> Result<String, String> {
        let current = AppConfig::global();
        let cfg = draft.apply_to_config(&current);
        if DockerConnection::is_initialized() {
            DockerConnection::reload_global(cfg.docker.clone()).map_err(|e| e.to_string())?;
        }
        DockerConnection::ping().await.map_err(|e| e.to_string())?;
        Ok(format!(
            "Docker daemon reachable via '{}' (timeout {}s)",
            draft.conn_type_label(),
            draft.timeout
        ))
    }

    pub fn extract_keybindings(keymap: &KeyMap) -> Vec<KeybindingRow> {
        let scope_order = [
            (KeyScope::Global, "global"),
            (KeyScope::Dashboard, "dashboard"),
            (KeyScope::Containers, "containers"),
            (KeyScope::Images, "images"),
            (KeyScope::Logs, "logs"),
            (KeyScope::Settings, "settings"),
        ];

        let mut rows = Vec::new();
        for (scope, scope_name) in scope_order {
            if let Some(binds) = keymap.0.get(&scope) {
                let mut scope_rows: Vec<KeybindingRow> = binds
                    .iter()
                    .map(|(seq, cmd)| KeybindingRow {
                        scope: scope_name.to_owned(),
                        key_seq: seq.to_string(),
                        command: format!("{cmd:?}"),
                    })
                    .collect();
                scope_rows.sort_by(|a, b| a.key_seq.cmp(&b.key_seq));
                rows.extend(scope_rows);
            }
        }
        rows
    }
}

