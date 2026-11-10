use std::{collections::HashMap, str::FromStr, sync::Arc};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc::unbounded_channel;

use crate::{
    modules::settings::{
        actions::SettingsAction,
        constants::{DockerField, GeneralField, SettingsTab},
        pages::SettingsPage,
        settings_dto::SettingsDraft,
        settings_service::SettingsService,
    },
    shared::base::{
        base_action::Action,
        base_component::Page,
        icons::IconSet,
    },
    tuikk_core::{
        config::{AppConfig, QueryMode},
        key_map::{Command, KeyMap, KeyScope, KeySeq},
        themes::ThemeColor,
    },
};

#[test]
fn settings_tabs_and_fields_cycle() {
    let t = SettingsTab::General;
    assert_eq!(t.next(), SettingsTab::Docker);
    assert_eq!(t.next().next(), SettingsTab::Keybindings);
    assert_eq!(t.next().next().next(), SettingsTab::General);
    assert_eq!(t.prev(), SettingsTab::Keybindings);

    let g = GeneralField::Theme;
    assert_eq!(g.next(), GeneralField::IconSet);
    assert_eq!(g.prev(), GeneralField::TickRate);

    let d = DockerField::ConnectionType;
    assert_eq!(d.next(), DockerField::AddrOrPath);
    assert_eq!(d.prev(), DockerField::ClientVersion);
}

#[test]
fn settings_draft_cycles_options_and_applies_to_config() {
    let base = AppConfig::default();
    let mut draft = SettingsDraft::from_config(&base);
    assert_eq!(draft.theme, "dark");
    assert_eq!(draft.icon_set, IconSet::Ascii);
    assert_eq!(draft.query_mode, QueryMode::Local);

    draft.cycle_theme();
    assert_eq!(draft.theme, "light");

    draft.cycle_icon_set(true);
    assert_eq!(draft.icon_set, IconSet::Unicode);
    draft.cycle_icon_set(true);
    assert_eq!(draft.icon_set, IconSet::Nerd);

    draft.cycle_query_mode();
    assert_eq!(draft.query_mode, QueryMode::Remote);

    draft.cycle_log_level(true);
    assert_eq!(draft.log_level, "warn");

    draft.cycle_conn_type(true);
    assert_eq!(draft.conn_type_label(), "local");
    draft.cycle_conn_type(true);
    assert_eq!(draft.conn_type_label(), "http");

    draft.adjust_frame_rate(500.0);
    assert_eq!(draft.frame_rate, 240.0);
    draft.adjust_tick_rate(-100.0);
    assert_eq!(draft.tick_rate, 1.0);
    draft.adjust_timeout(30);
    assert_eq!(draft.timeout, 150);

    draft.log_dir = "  /tmp/custom_logs  ".to_owned();
    draft.addr_or_path = "  tcp://127.0.0.1:2375  ".to_owned();
    draft.client_version = "  1.45  ".to_owned();

    let updated = draft.apply_to_config(&base);
    assert_eq!(updated.theme, "light");
    assert_eq!(updated.icon_set, IconSet::Nerd);
    assert_eq!(updated.query_mode, QueryMode::Remote);
    assert_eq!(updated.log_level, "warn");
    assert_eq!(updated.log_dir.as_deref(), Some("/tmp/custom_logs"));
    assert_eq!(
        updated.docker.addr_or_path.as_deref(),
        Some("tcp://127.0.0.1:2375")
    );
    assert_eq!(updated.docker.client_version.as_deref(), Some("1.45"));
    assert_eq!(updated.docker.timeout, 150);
}

#[test]
fn extract_keybindings_sorts_by_scope_and_key_seq() {
    let mut global_map = HashMap::new();
    global_map.insert(KeySeq::from_str("<q>").unwrap(), Command::Quit);
    global_map.insert(KeySeq::from_str("<f5>").unwrap(), Command::Refresh);

    let mut map = HashMap::new();
    map.insert(KeyScope::Global, global_map);
    let keymap = KeyMap(map);

    let rows = SettingsService::extract_keybindings(&keymap);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].scope, "global");
    assert_eq!(rows[0].key_seq, "<f(5)>");
    assert_eq!(rows[1].key_seq, "<q>");
}

#[test]
fn settings_page_keyboard_navigation_and_actions() {
    let svc = Arc::new(SettingsService::new());
    let draft = SettingsDraft::default();
    let (tx, _rx) = unbounded_channel();
    let mut page = SettingsPage::with_draft(svc, draft, Vec::new(), tx, &ThemeColor::dark());

    assert_eq!(page.active_tab(), SettingsTab::General);
    assert!(!page.is_dirty());

    // Press Space on Theme field -> toggles to light and marks dirty
    page.handle_key_event(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
    assert_eq!(page.draft().theme, "light");
    assert!(page.is_dirty());

    // Switch tab to Docker ('2')
    page.handle_key_event(KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE));
    assert_eq!(page.active_tab(), SettingsTab::Docker);

    // Press 'w' -> emits Save action
    let act = page.handle_key_event(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE));
    assert!(matches!(act, Some(Action::Settings(SettingsAction::Save))));

    // Press 't' -> emits PingDocker action
    let act = page.handle_key_event(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE));
    assert!(matches!(
        act,
        Some(Action::Settings(SettingsAction::PingDocker))
    ));
}

