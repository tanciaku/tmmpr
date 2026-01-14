use std::time::Duration as StdDuration;
use chrono::Duration as ChronoDuration;

use crate::{
    states::settings::{BackupsInterval, RuntimeBackupsInterval},
    utils::backups::{get_duration, get_duration_rt},
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
// Integration Test Candidates
// ============================================================================
// The following functions should have integration tests rather than unit tests:
//
// handle_on_load_backup():
//   - Requires file I/O (save_map_file, save_settings_to_file)
//   - Complex state mutations on MapState
//   - Filesystem interactions with backup directory
//
// handle_runtime_backup():
//   - Requires file I/O (save_map_file)
//   - Complex state mutations on MapState
//   - Filesystem interactions with backup directory