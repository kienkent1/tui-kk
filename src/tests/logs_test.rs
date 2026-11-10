use std::{
    fs,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::sync::mpsc::unbounded_channel;

use crate::{
    modules::logs::{
        actions::LogAction,
        constants::{LogDetailTab, LogSort},
        list_page::LogListPage,
        log_dto::{LogLevel, parse_log_line},
        log_service::LogService,
    },
    shared::base::{
        base_action::Action,
        base_component::Page,
        base_filter::BaseFilter,
    },
    tuikk_core::{
        key_map::Command,
        logging::resolve_log_dir,
        themes::ThemeColor,
    },
};

fn unique_temp_log_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("tuikk_logs_test_{tag}_{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn resolve_log_dir_uses_custom_config_when_provided() {
    let custom = Some("./my-custom-logs".to_owned());
    assert_eq!(resolve_log_dir(&custom), PathBuf::from("./my-custom-logs"));

    let empty = Some("   ".to_owned());
    assert!(resolve_log_dir(&empty).ends_with("logs"));
}

#[test]
fn parse_log_line_extracts_timestamp_level_target_and_message() {
    let raw = "2026-10-19T15:10:47.444708Z  WARN ThreadId(01) tui_kk::shared::helpers::ext_log: Input/output error (os error 5)";
    let entry = parse_log_line(6, raw);
    assert_eq!(entry.line_number, 6);
    assert_eq!(entry.timestamp, "2026-10-19T15:10:47.444708Z");
    assert_eq!(entry.level, LogLevel::Warn);
    assert_eq!(entry.target, "tui_kk::shared::helpers::ext_log");
    assert_eq!(entry.message, "Input/output error (os error 5)");

    let err_raw = "2026-10-19T16:00:00.000000Z ERROR ThreadId(02) tui_kk::docker: Connection refused";
    let err_entry = parse_log_line(10, err_raw);
    assert_eq!(err_entry.level, LogLevel::Error);
    assert!(err_entry.level.is_error());
    assert!(err_entry.level.is_warn_or_error());
    assert_eq!(err_entry.target, "tui_kk::docker");
    assert_eq!(err_entry.message, "Connection refused");
}

#[test]
fn log_sort_and_detail_tab_cycle() {
    assert_eq!(
        LogSort::parse("errors"),
        Some((LogSort::Errors, false))
    );
    assert_eq!(
        LogSort::parse("-size"),
        Some((LogSort::Size, true))
    );
    assert_eq!(LogSort::Modified.next(), LogSort::Errors);
    assert_eq!(LogSort::Errors.next(), LogSort::Size);
    assert_eq!(LogSort::Size.next(), LogSort::Name);
    assert_eq!(LogSort::Name.next(), LogSort::Modified);

    let tab = LogDetailTab::WarnAndError;
    assert_eq!(tab.next(), LogDetailTab::ErrorsOnly);
    assert_eq!(tab.next().next(), LogDetailTab::All);
    assert_eq!(tab.next().next().next(), LogDetailTab::Raw);
    assert_eq!(tab.next().next().next().next(), LogDetailTab::WarnAndError);
}

#[test]
fn log_service_lists_filters_clears_and_deletes_files() {
    let dir = unique_temp_log_dir("service");
    let file_a = dir.join("2026-10-19-14.tuikk.log");
    let file_b = dir.join("2026-10-19-15.tuikk.log");

    fs::write(
        &file_a,
        "2026-10-19T14:00:00Z INFO ThreadId(01) tui_kk: Started\n\
         2026-10-19T14:01:00Z WARN ThreadId(01) tui_kk: Slow response\n",
    )
    .unwrap();

    fs::write(
        &file_b,
        "2026-10-19T15:00:00Z INFO ThreadId(01) tui_kk: Ping ok\n\
         2026-10-19T15:02:00Z ERROR ThreadId(02) tui_kk::docker: Daemon unreachable\n\
         2026-10-19T15:03:00Z ERROR ThreadId(02) tui_kk::docker: Timeout waiting for socket\n",
    )
    .unwrap();

    let svc = LogService::new(dir.clone());
    let files = svc.fetch_log_files().unwrap();
    assert_eq!(files.len(), 2);

    // Sort by -errors (most errors first)
    let filter = BaseFilter {
        sort: Some("-errors".to_owned()),
        ..Default::default()
    };
    let sorted = svc.query_log_files(&files, &filter);
    assert_eq!(sorted[0].file_name, "2026-10-19-15.tuikk.log");
    assert_eq!(sorted[0].error_count, 2);
    assert_eq!(sorted[0].warn_count, 0);
    assert_eq!(
        sorted[0].last_error_preview.as_deref(),
        Some("Timeout waiting for socket")
    );

    assert_eq!(sorted[1].file_name, "2026-10-19-14.tuikk.log");
    assert_eq!(sorted[1].error_count, 0);
    assert_eq!(sorted[1].warn_count, 1);

    // Read entries & filter by tab
    let entries = svc.read_log_entries("2026-10-19-15.tuikk.log").unwrap();
    assert_eq!(entries.len(), 3);
    let errors_only = LogService::filter_entries_by_tab(&entries, LogDetailTab::ErrorsOnly);
    assert_eq!(errors_only.len(), 2);

    // Path traversal is rejected
    assert!(svc.delete_log_file("../outside.log").is_err());

    // Clear file_a -> size becomes 0, file still exists
    svc.clear_log_file("2026-10-19-14.tuikk.log").unwrap();
    assert_eq!(fs::metadata(&file_a).unwrap().len(), 0);

    // Delete file_b -> removed from disk
    svc.delete_log_file("2026-10-19-15.tuikk.log").unwrap();
    assert!(!file_b.exists());

    // Delete all remaining logs
    let deleted = svc.delete_all_logs().unwrap();
    assert_eq!(deleted, 1);
    assert_eq!(svc.fetch_log_files().unwrap().len(), 0);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn log_list_page_navigation_and_delete_command() {
    let dir = unique_temp_log_dir("page");
    fs::write(
        dir.join("2026-10-19-15.tuikk.log"),
        "2026-10-19T15:00:00Z ERROR ThreadId(01) tui_kk: Boom\n",
    )
    .unwrap();

    let svc = Arc::new(LogService::new(dir.clone()));
    let items = svc.fetch_log_files().unwrap();
    let (tx, _rx) = unbounded_channel();
    let mut page = LogListPage::new(svc, tx, &ThemeColor::dark());
    page.set_items(items);

    assert_eq!(page.cursor(), 0);
    assert_eq!(page.selected_file(), Some("2026-10-19-15.tuikk.log"));

    let action = page.handle_command(Command::Delete);
    assert!(matches!(
        action,
        Some(Action::LogsPage(LogAction::DeleteFile(ref f))) if f == "2026-10-19-15.tuikk.log"
    ));

    let _ = fs::remove_dir_all(&dir);
}

