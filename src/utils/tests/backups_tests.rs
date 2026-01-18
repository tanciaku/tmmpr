use std::time::Duration as StdDuration;
use std::fs;
use std::collections::HashMap;
use chrono::{Duration as ChronoDuration, Local};

use crate::{
    states::{
        MapState,
        settings::{BackupsInterval, RuntimeBackupsInterval},
        map::Notification,
    },
    utils::backups::{get_duration, get_duration_rt, handle_on_load_backup, handle_runtime_backup},
};

// ============================================================================
// Unit Tests for get_duration()
// ============================================================================

#[test]
fn test_get_duration_daily() {
    let interval = BackupsInterval::Daily;
    let duration = get_duration(&interval);
    
    assert_eq!(duration, ChronoDuration::days(1));
}

#[test]
fn test_get_duration_every_3_days() {
    let interval = BackupsInterval::Every3Days;
    let duration = get_duration(&interval);
    
    assert_eq!(duration, ChronoDuration::days(3));
}

#[test]
fn test_get_duration_weekly() {
    let interval = BackupsInterval::Weekly;
    let duration = get_duration(&interval);
    
    assert_eq!(duration, ChronoDuration::weeks(1));
}

#[test]
fn test_get_duration_every_2_weeks() {
    let interval = BackupsInterval::Every2Weeks;
    let duration = get_duration(&interval);
    
    assert_eq!(duration, ChronoDuration::weeks(2));
}

// ============================================================================
// Unit Tests for get_duration_rt()
// ============================================================================

#[test]
fn test_get_duration_rt_hourly() {
    let interval = RuntimeBackupsInterval::Hourly;
    let duration = get_duration_rt(&interval);
    
    assert_eq!(duration, StdDuration::from_secs(3600));
}

#[test]
fn test_get_duration_rt_every_2_hours() {
    let interval = RuntimeBackupsInterval::Every2Hours;
    let duration = get_duration_rt(&interval);
    
    assert_eq!(duration, StdDuration::from_secs(7200));
}

#[test]
fn test_get_duration_rt_every_4_hours() {
    let interval = RuntimeBackupsInterval::Every4Hours;
    let duration = get_duration_rt(&interval);
    
    assert_eq!(duration, StdDuration::from_secs(14400));
}

#[test]
fn test_get_duration_rt_every_6_hours() {
    let interval = RuntimeBackupsInterval::Every6Hours;
    let duration = get_duration_rt(&interval);
    
    assert_eq!(duration, StdDuration::from_secs(21600));
}

#[test]
fn test_get_duration_rt_every_12_hours() {
    let interval = RuntimeBackupsInterval::Every12Hours;
    let duration = get_duration_rt(&interval);
    
    assert_eq!(duration, StdDuration::from_secs(43200));
}

// ============================================================================
// Integration Tests for handle_on_load_backup()
// ============================================================================

#[test]
fn test_handle_on_load_backup_disabled_backups() {
    // Create a temp directory for the map file
    let temp_dir = tempfile::tempdir().unwrap();
    let map_file_path = temp_dir.path().join("test_map.json");
    
    // Create MapState (may load user's actual settings from ~/.config/tmmpr/settings.json)
    let mut map_state = MapState::new(map_file_path.clone());
    
    // Explicitly disable backups for this test
    map_state.settings.backups_path = None;
    map_state.settings.backups_interval = None;
    
    // Call the function
    handle_on_load_backup(&mut map_state);
    
    // Verify no backup was created (backup_res should remain None)
    assert_eq!(map_state.persistence.backup_res, None);
    
    // Verify no notification was set
    assert_eq!(map_state.ui_state.show_notification, None);
}

#[test]
fn test_handle_on_load_backup_first_backup() {
    // Create temp directories
    let temp_dir = tempfile::tempdir().unwrap();
    let backup_dir = tempfile::tempdir().unwrap();
    let map_file_path = temp_dir.path().join("test_map.json");
    
    // Create MapState (may load user's actual settings from ~/.config/tmmpr/settings.json)
    let mut map_state = MapState::new(map_file_path.clone());
    
    // Enable backups with Daily interval
    map_state.settings.backups_path = Some(backup_dir.path().to_string_lossy().to_string());
    map_state.settings.backups_interval = Some(BackupsInterval::Daily);
    
    // Clear any existing backup_dates to simulate first backup scenario
    map_state.settings.backup_dates.clear();
    
    // Call the function
    handle_on_load_backup(&mut map_state);
     
    // Verify notification was set to BackupSuccess
    assert_eq!(map_state.ui_state.show_notification, Some(Notification::BackupSuccess));
    
    // Verify a backup file was created in the backup directory
    let backup_files: Vec<_> = fs::read_dir(backup_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    
    assert_eq!(backup_files.len(), 1);
    
    // Verify the backup filename format
    let backup_filename = backup_files[0].file_name();
    let backup_filename_str = backup_filename.to_string_lossy();
    assert!(backup_filename_str.starts_with("test_map-load-backup-"));
    assert!(backup_filename_str.ends_with(".json"));
    
    // Verify backup date was recorded in settings
    assert!(map_state.settings.backup_dates.contains_key("test_map"));
    
    // Verify the date is today
    let recorded_date = map_state.settings.backup_dates.get("test_map").unwrap();
    let today = Local::now().date_naive();
    assert_eq!(recorded_date.date_naive(), today);
}

#[test]
fn test_handle_on_load_backup_skip_recent_backup() {
    // Create temp directories
    let temp_dir = tempfile::tempdir().unwrap();
    let backup_dir = tempfile::tempdir().unwrap();
    let map_file_path = temp_dir.path().join("test_map.json");
    
    // Create MapState
    let mut map_state = MapState::new(map_file_path.clone());
    
    // Enable backups with Daily interval
    map_state.settings.backups_path = Some(backup_dir.path().to_string_lossy().to_string());
    map_state.settings.backups_interval = Some(BackupsInterval::Daily);
    
    // Set a recent backup date (today)
    let mut backup_dates = HashMap::new();
    backup_dates.insert("test_map".to_string(), Local::now());
    map_state.settings.backup_dates = backup_dates;
    
    // Call the function
    handle_on_load_backup(&mut map_state);
    
    // Verify no new backup was created
    assert_eq!(map_state.ui_state.show_notification, None);
    
    // Verify no backup files were created
    let backup_files: Vec<_> = fs::read_dir(backup_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    
    assert_eq!(backup_files.len(), 0);
}

#[test]
fn test_handle_on_load_backup_old_backup_triggers_new() {
    // Create temp directories
    let temp_dir = tempfile::tempdir().unwrap();
    let backup_dir = tempfile::tempdir().unwrap();
    let map_file_path = temp_dir.path().join("test_map.json");
    
    // Create MapState
    let mut map_state = MapState::new(map_file_path.clone());
    
    // Enable backups with Daily interval
    map_state.settings.backups_path = Some(backup_dir.path().to_string_lossy().to_string());
    map_state.settings.backups_interval = Some(BackupsInterval::Daily);
    
    // Set an old backup date (2 days ago)
    let mut backup_dates = HashMap::new();
    let old_date = Local::now() - ChronoDuration::days(2);
    backup_dates.insert("test_map".to_string(), old_date);
    map_state.settings.backup_dates = backup_dates;
    
    // Call the function
    handle_on_load_backup(&mut map_state);
    
    // Verify backup was created successfully
    assert_eq!(map_state.ui_state.show_notification, Some(Notification::BackupSuccess));
    
    // Verify a backup file was created
    let backup_files: Vec<_> = fs::read_dir(backup_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    
    assert_eq!(backup_files.len(), 1);
    
    // Verify backup date was updated
    let recorded_date = map_state.settings.backup_dates.get("test_map").unwrap();
    let today = Local::now().date_naive();
    assert_eq!(recorded_date.date_naive(), today);
}

#[test]
fn test_handle_on_load_backup_different_intervals() {
    // Test Weekly interval
    let temp_dir = tempfile::tempdir().unwrap();
    let backup_dir = tempfile::tempdir().unwrap();
    let map_file_path = temp_dir.path().join("test_map.json");
    
    let mut map_state = MapState::new(map_file_path.clone());
    map_state.settings.backups_path = Some(backup_dir.path().to_string_lossy().to_string());
    map_state.settings.backups_interval = Some(BackupsInterval::Weekly);
    
    // Set backup date 6 days ago (should not trigger backup for Weekly)
    let mut backup_dates = HashMap::new();
    let six_days_ago = Local::now() - ChronoDuration::days(6);
    backup_dates.insert("test_map".to_string(), six_days_ago);
    map_state.settings.backup_dates = backup_dates;
    
    handle_on_load_backup(&mut map_state);
    
    // Verify no backup was created (not enough time passed)
    let backup_files: Vec<_> = fs::read_dir(backup_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    assert_eq!(backup_files.len(), 0);
    
    // Now test with 8 days ago (should trigger backup for Weekly)
    let backup_dir2 = tempfile::tempdir().unwrap();
    map_state.settings.backups_path = Some(backup_dir2.path().to_string_lossy().to_string());
    
    let mut backup_dates = HashMap::new();
    let eight_days_ago = Local::now() - ChronoDuration::days(8);
    backup_dates.insert("test_map".to_string(), eight_days_ago);
    map_state.settings.backup_dates = backup_dates;
    
    handle_on_load_backup(&mut map_state);
    
    // Verify backup was created
    let backup_files2: Vec<_> = fs::read_dir(backup_dir2.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    assert_eq!(backup_files2.len(), 1);
}

#[test]
fn test_handle_on_load_backup_invalid_backup_directory() {
    // Create temp directory for map file
    let temp_dir = tempfile::tempdir().unwrap();
    let map_file_path = temp_dir.path().join("test_map.json");
    
    // Create MapState (may load user's actual settings from ~/.config/tmmpr/settings.json)
    let mut map_state = MapState::new(map_file_path.clone());
    
    // Enable backups with an invalid/inaccessible path
    map_state.settings.backups_path = Some("/invalid/nonexistent/path".to_string());
    map_state.settings.backups_interval = Some(BackupsInterval::Daily);
    
    // Clear any existing backup_dates to force a backup attempt
    map_state.settings.backup_dates.clear();
    
    // Call the function
    handle_on_load_backup(&mut map_state);
    
    // Verify backup failed
    assert_eq!(map_state.ui_state.show_notification, Some(Notification::BackupFail));
}

// ============================================================================
// Integration Tests for handle_runtime_backup()
// ============================================================================
// NOTE: The interval timing logic (when to call this function) is handled in
// MapState::on_tick_save_changes(), which checks if enough time has elapsed
// using get_duration_rt(). The handle_runtime_backup() function itself just
// checks that both backups_path and runtime_backups_interval are Some, then
// creates a backup whenever it's called.

#[test]
fn test_handle_runtime_backup_disabled_backups() {
    // Create a temp directory for the map file
    let temp_dir = tempfile::tempdir().unwrap();
    let map_file_path = temp_dir.path().join("test_map.json");
    
    // Create MapState (may load user's actual settings from ~/.config/tmmpr/settings.json)
    let mut map_state = MapState::new(map_file_path.clone());
    
    // Explicitly disable runtime backups for this test
    map_state.settings.backups_path = None;
    map_state.settings.runtime_backups_interval = None;
    
    // Call the function (normally called by on_tick_save_changes when interval elapses)
    handle_runtime_backup(&mut map_state);
    
    // Verify no notification was set
    assert_eq!(map_state.ui_state.show_notification, None);
}

#[test]
fn test_handle_runtime_backup_creates_backup() {
    // Create temp directories
    let temp_dir = tempfile::tempdir().unwrap();
    let backup_dir = tempfile::tempdir().unwrap();
    let map_file_path = temp_dir.path().join("test_map.json");
    
    // Create MapState
    let mut map_state = MapState::new(map_file_path.clone());
    
    // Enable runtime backups (interval value doesn't affect this function's behavior)
    map_state.settings.backups_path = Some(backup_dir.path().to_string_lossy().to_string());
    map_state.settings.runtime_backups_interval = Some(RuntimeBackupsInterval::Hourly);
    
    // Call the function (would normally be called by on_tick_save_changes)
    handle_runtime_backup(&mut map_state);
    
    // Verify notification was set to BackupSuccess
    assert_eq!(map_state.ui_state.show_notification, Some(Notification::BackupSuccess));
    
    // Verify a backup file was created in the backup directory
    let backup_files: Vec<_> = fs::read_dir(backup_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    
    assert_eq!(backup_files.len(), 1);
    
    // Verify the backup filename format (session backup includes time)
    let backup_filename = backup_files[0].file_name();
    let backup_filename_str = backup_filename.to_string_lossy();
    assert!(backup_filename_str.starts_with("test_map-session-backup-"));
    assert!(backup_filename_str.ends_with(".json"));
}

#[test]
fn test_handle_runtime_backup_filename_format() {
    // Create temp directories
    let temp_dir = tempfile::tempdir().unwrap();
    let backup_dir = tempfile::tempdir().unwrap();
    let map_file_path = temp_dir.path().join("my_mindmap.json");
    
    // Create MapState
    let mut map_state = MapState::new(map_file_path.clone());
    
    // Enable runtime backups
    map_state.settings.backups_path = Some(backup_dir.path().to_string_lossy().to_string());
    map_state.settings.runtime_backups_interval = Some(RuntimeBackupsInterval::Every2Hours);
    
    // Call the function
    handle_runtime_backup(&mut map_state);
    
    // Verify the backup filename includes the timestamp with hour and minute
    let backup_files: Vec<_> = fs::read_dir(backup_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    
    assert_eq!(backup_files.len(), 1);
    
    let backup_filename = backup_files[0].file_name();
    let backup_filename_str = backup_filename.to_string_lossy();
    
    // Filename should be: my_mindmap-session-backup-YY-MM-DD-HHMM.json
    assert!(backup_filename_str.starts_with("my_mindmap-session-backup-"));
    assert!(backup_filename_str.ends_with(".json"));
    
    // Verify format contains date and time parts
    let parts: Vec<&str> = backup_filename_str.split('-').collect();
    assert!(parts.len() >= 6); // my, mindmap, session, backup, date parts, time
}

#[test]
fn test_handle_runtime_backup_does_not_update_backup_dates() {
    // Create temp directories
    let temp_dir = tempfile::tempdir().unwrap();
    let backup_dir = tempfile::tempdir().unwrap();
    let map_file_path = temp_dir.path().join("test_map.json");
    
    // Create MapState (will load user's actual settings from ~/.config/tmmpr/settings.json)
    let mut map_state = MapState::new(map_file_path.clone());
    
    // Enable runtime backups
    map_state.settings.backups_path = Some(backup_dir.path().to_string_lossy().to_string());
    map_state.settings.runtime_backups_interval = Some(RuntimeBackupsInterval::Every6Hours);
    
    // Clear any existing backup_dates (may be loaded from user's settings file)
    map_state.settings.backup_dates.clear();
    
    // Verify backup_dates is now empty
    assert!(map_state.settings.backup_dates.is_empty());
    
    // Call the function (would normally be called by on_tick_save_changes)
    handle_runtime_backup(&mut map_state);
    
    // Verify backup_dates HashMap was NOT updated (runtime backups don't update this)
    // This is a key difference from handle_on_load_backup which does update backup_dates
    assert!(map_state.settings.backup_dates.is_empty());
    
    // But verify backup was created
    let backup_files: Vec<_> = fs::read_dir(backup_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    
    assert_eq!(backup_files.len(), 1);
}

#[test]
fn test_handle_runtime_backup_invalid_backup_directory() {
    // Create temp directory for map file
    let temp_dir = tempfile::tempdir().unwrap();
    let map_file_path = temp_dir.path().join("test_map.json");
    
    // Create MapState
    let mut map_state = MapState::new(map_file_path.clone());
    
    // Enable runtime backups with an invalid/inaccessible path
    map_state.settings.backups_path = Some("/invalid/nonexistent/runtime/path".to_string());
    map_state.settings.runtime_backups_interval = Some(RuntimeBackupsInterval::Every12Hours);
    
    // Call the function
    handle_runtime_backup(&mut map_state);
    
    // Verify backup failed
    assert_eq!(map_state.ui_state.show_notification, Some(Notification::BackupFail));
}

#[test]
fn test_handle_runtime_backup_backups_enabled_but_no_interval() {
    // Edge case: backups_path is set but runtime_backups_interval is None.
    // This function requires BOTH to be Some to create a backup.
    // (In practice, on_tick_save_changes wouldn't call this function in this scenario)
    let temp_dir = tempfile::tempdir().unwrap();
    let backup_dir = tempfile::tempdir().unwrap();
    let map_file_path = temp_dir.path().join("test_map.json");
    
    let mut map_state = MapState::new(map_file_path.clone());
    
    // Set backups_path but not runtime_backups_interval
    map_state.settings.backups_path = Some(backup_dir.path().to_string_lossy().to_string());
    map_state.settings.runtime_backups_interval = None;
    
    // Call the function (edge case - wouldn't normally be called in this state)
    handle_runtime_backup(&mut map_state);
    
    // Verify no backup was created (both backups_path and runtime_interval required)
    assert_eq!(map_state.ui_state.show_notification, None);
    
    // Verify no files in backup directory
    let backup_files: Vec<_> = fs::read_dir(backup_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    
    assert_eq!(backup_files.len(), 0);
}