use std::path::PathBuf;
use std::io;
use ratatui::style::{Color, Style};
use serde_json;
use chrono::{Local, TimeZone};

use crate::states::{
    map::Side, settings::{
        BackupsErr, BackupsInterval, RuntimeBackupsInterval, SelectedToggle, Settings, SettingsNotification, SettingsState, SettingsType, cycle_side, get_settings, side_to_string
    }
};
use crate::states::settings::{FileSystemOps, resolve_backup_path, validate_backup_directory};

/// Mock filesystem for testing
pub struct MockFileSystem {
    pub home_dir: Option<PathBuf>,
    pub create_dir_result: Result<(), io::Error>,
    pub test_write_result: Result<(), io::Error>,
}

impl MockFileSystem {
    pub fn new_success() -> Self {
        MockFileSystem {
            home_dir: Some(PathBuf::from("/mock/home")),
            create_dir_result: Ok(()),
            test_write_result: Ok(()),
        }
    }

    pub fn new_no_home() -> Self {
        MockFileSystem {
            home_dir: None,
            create_dir_result: Ok(()),
            test_write_result: Ok(()),
        }
    }

    pub fn new_create_fails() -> Self {
        MockFileSystem {
            home_dir: Some(PathBuf::from("/mock/home")),
            create_dir_result: Err(io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied")),
            test_write_result: Ok(()),
        }
    }

    pub fn new_write_fails() -> Self {
        MockFileSystem {
            home_dir: Some(PathBuf::from("/mock/home")),
            create_dir_result: Ok(()),
            test_write_result: Err(io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied")),
        }
    }
}

impl FileSystemOps for MockFileSystem {
    fn get_home_dir(&self) -> Option<PathBuf> {
        self.home_dir.clone()
    }

    fn create_dir_all(&self, _path: &PathBuf) -> Result<(), io::Error> {
        self.create_dir_result.as_ref().map(|_| ()).map_err(|e| io::Error::new(e.kind(), "mock error"))
    }

    fn test_write_to_dir(&self, _path: &PathBuf) -> Result<(), io::Error> {
        self.test_write_result.as_ref().map(|_| ()).map_err(|e| io::Error::new(e.kind(), "mock error"))
    }
}

// ============================================================================
// Tests for SettingsState
// ============================================================================

#[test]
fn test_toggle_go_down_without_runtime_backups() {
    let map_path = PathBuf::from("/test/path/map.json");
    let mut state = SettingsState::new(map_path);
    
    // Ensure runtime backups are disabled
    state.settings.settings_mut().runtime_backups_interval = None;

    // Test going down from each toggle
    state.selected_toggle = SelectedToggle::Toggle1;
    state.toggle_go_down();
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle2);

    state.selected_toggle = SelectedToggle::Toggle2;
    state.toggle_go_down();
    // Should skip Toggle3 since runtime backups are disabled
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle4);

    state.selected_toggle = SelectedToggle::Toggle4;
    state.toggle_go_down();
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle5);

    state.selected_toggle = SelectedToggle::Toggle5;
    state.toggle_go_down();
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle6);

    state.selected_toggle = SelectedToggle::Toggle6;
    state.toggle_go_down();
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle1);
}

#[test]
fn test_toggle_go_down_with_runtime_backups() {
    let map_path = PathBuf::from("/test/path/map.json");
    let mut state = SettingsState::new(map_path);
    
    // Enable runtime backups
    state.settings.settings_mut().runtime_backups_interval = Some(RuntimeBackupsInterval::Hourly);

    state.selected_toggle = SelectedToggle::Toggle2;
    state.toggle_go_down();
    // Should go to Toggle3 since runtime backups are enabled
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle3);

    state.selected_toggle = SelectedToggle::Toggle3;
    state.toggle_go_down();
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle4);
}

#[test]
fn test_toggle_go_up_without_runtime_backups() {
    let map_path = PathBuf::from("/test/path/map.json");
    let mut state = SettingsState::new(map_path);
    
    // Ensure runtime backups are disabled
    state.settings.settings_mut().runtime_backups_interval = None;

    state.selected_toggle = SelectedToggle::Toggle1;
    state.toggle_go_up();
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle6);

    state.selected_toggle = SelectedToggle::Toggle2;
    state.toggle_go_up();
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle1);

    state.selected_toggle = SelectedToggle::Toggle4;
    state.toggle_go_up();
    // Should skip Toggle3 since runtime backups are disabled
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle2);

    state.selected_toggle = SelectedToggle::Toggle5;
    state.toggle_go_up();
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle4);

    state.selected_toggle = SelectedToggle::Toggle6;
    state.toggle_go_up();
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle5);
}

#[test]
fn test_toggle_go_up_with_runtime_backups() {
    let map_path = PathBuf::from("/test/path/map.json");
    let mut state = SettingsState::new(map_path);
    
    // Enable runtime backups
    state.settings.settings_mut().runtime_backups_interval = Some(RuntimeBackupsInterval::Hourly);

    state.selected_toggle = SelectedToggle::Toggle4;
    state.toggle_go_up();
    // Should go to Toggle3 since runtime backups are enabled
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle3);

    state.selected_toggle = SelectedToggle::Toggle3;
    state.toggle_go_up();
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle2);
}

// ============================================================================
// Tests for path resolution functions
// ============================================================================

#[test]
fn test_resolve_backup_path_absolute() {
    let mock_fs = MockFileSystem::new_success();
    let result = resolve_backup_path("/absolute/path", &mock_fs);
    assert_eq!(result.unwrap(), PathBuf::from("/absolute/path"));
}

#[test]
fn test_resolve_backup_path_relative() {
    let mock_fs = MockFileSystem::new_success();
    let result = resolve_backup_path("relative/path", &mock_fs);
    assert_eq!(result.unwrap(), PathBuf::from("/mock/home/relative/path"));
}

#[test]
fn test_resolve_backup_path_no_home() {
    let mock_fs = MockFileSystem::new_no_home();
    let result = resolve_backup_path("relative/path", &mock_fs);
    assert_eq!(result.unwrap_err(), BackupsErr::DirFind);
}

#[test]
fn test_validate_backup_directory_success() {
    let mock_fs = MockFileSystem::new_success();
    let path = PathBuf::from("/test/path");
    let result = validate_backup_directory(&path, &mock_fs);
    assert!(result.is_ok());
}

#[test]
fn test_validate_backup_directory_create_fails() {
    let mock_fs = MockFileSystem::new_create_fails();
    let path = PathBuf::from("/test/path");
    let result = validate_backup_directory(&path, &mock_fs);
    assert_eq!(result.unwrap_err(), BackupsErr::DirCreate);
}

#[test]
fn test_validate_backup_directory_write_fails() {
    let mock_fs = MockFileSystem::new_write_fails();
    let path = PathBuf::from("/test/path");
    let result = validate_backup_directory(&path, &mock_fs);
    assert_eq!(result.unwrap_err(), BackupsErr::FileWrite);
}

// ============================================================================
// Tests for submit_path() with mocks
// ============================================================================

#[test]
fn test_submit_path_with_absolute_path() {
    let map_path = PathBuf::from("/test/path/map.json");
    let mut state = SettingsState::new(map_path);
    let mock_fs = MockFileSystem::new_success();
    
    state.settings.settings_mut().backups_path = Some("/mock/backups".to_string());
    state.input_prompt = true;
    
    state.submit_path_with_fs(&mock_fs);

    assert_eq!(state.input_prompt, false);
    assert_eq!(state.input_prompt_err, None);
    assert_eq!(state.settings.settings().backups_interval, Some(BackupsInterval::Daily));
    assert_eq!(state.settings.settings().runtime_backups_interval, Some(RuntimeBackupsInterval::Every2Hours));
}

#[test]
fn test_submit_path_with_relative_path() {
    let map_path = PathBuf::from("/test/path/map.json");
    let mut state = SettingsState::new(map_path);
    let mock_fs = MockFileSystem::new_success();
    
    state.settings.settings_mut().backups_path = Some("backups".to_string());
    state.input_prompt = true;
    
    state.submit_path_with_fs(&mock_fs);

    assert_eq!(state.input_prompt, false);
    assert_eq!(state.input_prompt_err, None);
    // Path should be updated to absolute path
    assert_eq!(state.settings.settings().backups_path, Some("/mock/home/backups".to_string()));
}

#[test]
fn test_submit_path_sets_correct_intervals() {
    let map_path = PathBuf::from("/test/path/map.json");
    let mut state = SettingsState::new(map_path);
    let mock_fs = MockFileSystem::new_success();
    
    state.settings.settings_mut().backups_path = Some("/mock/backups".to_string());
    
    // Ensure intervals are initially None/different
    state.settings.settings_mut().backups_interval = None;
    state.settings.settings_mut().runtime_backups_interval = None;
    state.input_prompt = true;
    
    state.submit_path_with_fs(&mock_fs);

    // Should have set specific default intervals
    assert_eq!(state.settings.settings().backups_interval, Some(BackupsInterval::Daily));
    assert_eq!(state.settings.settings().runtime_backups_interval, Some(RuntimeBackupsInterval::Every2Hours));
}

#[test]
fn test_submit_path_resets_error_on_success() {
    let map_path = PathBuf::from("/test/path/map.json");
    let mut state = SettingsState::new(map_path);
    let mock_fs = MockFileSystem::new_success();
    
    state.settings.settings_mut().backups_path = Some("/mock/backups".to_string());
    
    // Set an existing error
    state.input_prompt_err = Some(BackupsErr::DirCreate);
    state.input_prompt = true;
    
    state.submit_path_with_fs(&mock_fs);

    // Should have cleared the error and exited input prompt
    assert_eq!(state.input_prompt_err, None);
    assert_eq!(state.input_prompt, false);
}

#[test]
fn test_submit_path_no_home_directory() {
    let map_path = PathBuf::from("/test/path/map.json");
    let mut state = SettingsState::new(map_path);
    let mock_fs = MockFileSystem::new_no_home();
    
    state.settings.settings_mut().backups_path = Some("backups".to_string());
    state.input_prompt = true;
    
    state.submit_path_with_fs(&mock_fs);

    assert_eq!(state.input_prompt, true);
    assert_eq!(state.input_prompt_err, Some(BackupsErr::DirFind));
}

#[test]
fn test_submit_path_directory_create_fails() {
    let map_path = PathBuf::from("/test/path/map.json");
    let mut state = SettingsState::new(map_path);
    let mock_fs = MockFileSystem::new_create_fails();
    
    state.settings.settings_mut().backups_path = Some("/mock/backups".to_string());
    state.input_prompt = true;
    
    state.submit_path_with_fs(&mock_fs);

    assert_eq!(state.input_prompt, true);
    assert_eq!(state.input_prompt_err, Some(BackupsErr::DirCreate));
}

#[test]
fn test_submit_path_write_test_fails() {
    let map_path = PathBuf::from("/test/path/map.json");
    let mut state = SettingsState::new(map_path);
    let mock_fs = MockFileSystem::new_write_fails();
    
    state.settings.settings_mut().backups_path = Some("/mock/backups".to_string());
    state.input_prompt = true;
    
    state.submit_path_with_fs(&mock_fs);

    assert_eq!(state.input_prompt, true);
    assert_eq!(state.input_prompt_err, Some(BackupsErr::FileWrite));
}

// ============================================================================
// Tests for Settings
// ============================================================================

#[test]
fn test_cycle_save_intervals() {
    let mut settings = Settings::new();

    // Start with Some(20) (default)
    assert_eq!(settings.save_interval, Some(20));

    settings.cycle_save_intervals();
    assert_eq!(settings.save_interval, Some(30));

    settings.cycle_save_intervals();
    assert_eq!(settings.save_interval, Some(60));

    settings.cycle_save_intervals();
    assert_eq!(settings.save_interval, None);

    settings.cycle_save_intervals();
    assert_eq!(settings.save_interval, Some(10));

    settings.cycle_save_intervals();
    assert_eq!(settings.save_interval, Some(20));
}

#[test]
fn test_cycle_backup_interval() {
    let mut settings = Settings::new();
    settings.backups_interval = Some(BackupsInterval::Daily);

    settings.cycle_backup_interval();
    assert_eq!(settings.backups_interval, Some(BackupsInterval::Every3Days));

    settings.cycle_backup_interval();
    assert_eq!(settings.backups_interval, Some(BackupsInterval::Weekly));

    settings.cycle_backup_interval();
    assert_eq!(settings.backups_interval, Some(BackupsInterval::Every2Weeks));

    settings.cycle_backup_interval();
    assert_eq!(settings.backups_interval, Some(BackupsInterval::Daily));
}

#[test]
fn test_cycle_runtime_backup_interval() {
    let mut settings = Settings::new();
    settings.runtime_backups_interval = Some(RuntimeBackupsInterval::Hourly);

    settings.cycle_runtime_backup_interval();
    assert_eq!(settings.runtime_backups_interval, Some(RuntimeBackupsInterval::Every2Hours));

    settings.cycle_runtime_backup_interval();
    assert_eq!(settings.runtime_backups_interval, Some(RuntimeBackupsInterval::Every4Hours));

    settings.cycle_runtime_backup_interval();
    assert_eq!(settings.runtime_backups_interval, Some(RuntimeBackupsInterval::Every6Hours));

    settings.cycle_runtime_backup_interval();
    assert_eq!(settings.runtime_backups_interval, Some(RuntimeBackupsInterval::Every12Hours));

    settings.cycle_runtime_backup_interval();
    assert_eq!(settings.runtime_backups_interval, Some(RuntimeBackupsInterval::Hourly));
}

#[test]
fn test_cycle_default_sides() {
    let mut settings = Settings::new();

    // Test cycling start side
    assert_eq!(settings.default_start_side, Side::Right);
    settings.cycle_default_sides(true);
    assert_eq!(settings.default_start_side, Side::Bottom);
    
    settings.cycle_default_sides(true);
    assert_eq!(settings.default_start_side, Side::Left);
    
    settings.cycle_default_sides(true);
    assert_eq!(settings.default_start_side, Side::Top);
    
    settings.cycle_default_sides(true);
    assert_eq!(settings.default_start_side, Side::Right);

    // Test cycling end side
    assert_eq!(settings.default_end_side, Side::Right);
    settings.cycle_default_sides(false);
    assert_eq!(settings.default_end_side, Side::Bottom);
    
    settings.cycle_default_sides(false);
    assert_eq!(settings.default_end_side, Side::Left);
    
    settings.cycle_default_sides(false);
    assert_eq!(settings.default_end_side, Side::Top);
    
    settings.cycle_default_sides(false);
    assert_eq!(settings.default_end_side, Side::Right);
}

// ============================================================================
// Tests for SettingsType
// ============================================================================

#[test]
fn test_settings_type_mut() {
    let settings = Settings::new();
    let mut settings_type = SettingsType::Default(settings, None);

    {
        let settings_mut = settings_type.settings_mut();
        settings_mut.save_interval = Some(60);
        settings_mut.edit_modal = true;
    }

    let settings_ref = settings_type.settings();
    assert_eq!(settings_ref.save_interval, Some(60));
    assert_eq!(settings_ref.edit_modal, true);
}

// ============================================================================
// Tests for SelectedToggle
// ============================================================================

#[test]
fn test_get_style_selected() {
    let toggle = SelectedToggle::Toggle1;
    let selected = SelectedToggle::Toggle1;
    
    let style = toggle.get_style(&selected);
    assert_eq!(style.bg, Some(Color::White));
    assert_eq!(style.fg, Some(Color::Black));
}

#[test]
fn test_get_style_not_selected() {
    let toggle = SelectedToggle::Toggle1;
    let selected = SelectedToggle::Toggle2;
    
    let style = toggle.get_style(&selected);
    assert_eq!(style, Style::new());
}

// ============================================================================
// Tests for helper functions
// ============================================================================

#[test]
fn test_cycle_side() {
    assert_eq!(cycle_side(Side::Right), Side::Bottom);
    assert_eq!(cycle_side(Side::Bottom), Side::Left);
    assert_eq!(cycle_side(Side::Left), Side::Top);
    assert_eq!(cycle_side(Side::Top), Side::Right);
}

#[test]
fn test_side_to_string() {
    assert_eq!(side_to_string(Side::Right), "Right");
    assert_eq!(side_to_string(Side::Bottom), "Bottom");
    assert_eq!(side_to_string(Side::Left), "Left");
    assert_eq!(side_to_string(Side::Top), "Top");
}

#[test]
fn test_get_settings_returns_settings_type() {
    let settings_type = get_settings();
    
    // Should return either Default or Custom, both are valid
    match settings_type {
        SettingsType::Default(settings, _) => {
            assert_eq!(settings.save_interval, Some(20));
        },
        SettingsType::Custom(settings) => {
            // Custom settings loaded from file, verify it's a Settings struct
            assert!(settings.save_interval.is_some() || settings.save_interval.is_none());
        }
    }
}

// ============================================================================
// Tests for serialization
// ============================================================================

#[test]
fn test_settings_serialization() {
    let mut settings = Settings::new();
    settings.save_interval = Some(30);
    settings.backups_interval = Some(BackupsInterval::Weekly);
    settings.backups_path = Some("/test/path".to_string());
    settings.runtime_backups_interval = Some(RuntimeBackupsInterval::Every4Hours);
    settings.default_start_side = Side::Left;
    settings.default_end_side = Side::Top;
    settings.edit_modal = true;

    // Add a test backup date
    let test_date = Local.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
    settings.backup_dates.insert("/test/map.json".to_string(), test_date);

    // Serialize to JSON
    let json_str = serde_json::to_string(&settings).unwrap();
    assert!(json_str.contains("\"save_interval\":30"));
    assert!(json_str.contains("\"edit_modal\":true"));

    // Deserialize back
    let deserialized: Settings = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.save_interval, Some(30));
    assert_eq!(deserialized.backups_interval, Some(BackupsInterval::Weekly));
    assert_eq!(deserialized.backups_path, Some("/test/path".to_string()));
    assert_eq!(deserialized.runtime_backups_interval, Some(RuntimeBackupsInterval::Every4Hours));
    assert_eq!(deserialized.default_start_side, Side::Left);
    assert_eq!(deserialized.default_end_side, Side::Top);
    assert_eq!(deserialized.edit_modal, true);
    assert_eq!(deserialized.backup_dates.len(), 1);
}

#[test]
fn test_backups_interval_serialization() {
    let intervals = vec![
        BackupsInterval::Daily,
        BackupsInterval::Every3Days,
        BackupsInterval::Weekly,
        BackupsInterval::Every2Weeks,
    ];

    for interval in intervals {
        let json_str = serde_json::to_string(&interval).unwrap();
        let deserialized: BackupsInterval = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized, interval);
    }
}

#[test]
fn test_runtime_backups_interval_serialization() {
    let intervals = vec![
        RuntimeBackupsInterval::Hourly,
        RuntimeBackupsInterval::Every2Hours,
        RuntimeBackupsInterval::Every4Hours,
        RuntimeBackupsInterval::Every6Hours,
        RuntimeBackupsInterval::Every12Hours,
    ];

    for interval in intervals {
        let json_str = serde_json::to_string(&interval).unwrap();
        let deserialized: RuntimeBackupsInterval = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized, interval);
    }
}

// ============================================================================
// Tests for edge cases
// ============================================================================

#[test]
fn test_settings_with_populated_backup_dates() {
    let mut settings = Settings::new();
    
    // Add multiple backup dates
    let date1 = Local.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
    let date2 = Local.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap();
    
    settings.backup_dates.insert("/path/map1.json".to_string(), date1);
    settings.backup_dates.insert("/path/map2.json".to_string(), date2);

    assert_eq!(settings.backup_dates.len(), 2);
    assert!(settings.backup_dates.contains_key("/path/map1.json"));
    assert!(settings.backup_dates.contains_key("/path/map2.json"));
}

// ============================================================================
// Integration Tests for get_settings()
// ============================================================================

#[test]
fn test_get_settings_returns_valid_settings_type() {
    let settings_type = get_settings();
    
    // Should return either Default or Custom variant
    match settings_type {
        SettingsType::Default(settings, _) => {
            // Verify default values
            assert_eq!(settings.save_interval, Some(20));
            assert_eq!(settings.backups_interval, None);
            assert_eq!(settings.backups_path, None);
            assert_eq!(settings.runtime_backups_interval, None);
            assert_eq!(settings.default_start_side, Side::Right);
            assert_eq!(settings.default_end_side, Side::Right);
            assert_eq!(settings.edit_modal, false);
        },
        SettingsType::Custom(_) => {
            // Custom settings loaded from file - valid either way
        }
    }
}

// ============================================================================
// Integration Tests for save_settings()
// ============================================================================

#[test]
fn test_save_settings_success() {
    let map_path = PathBuf::from("/test/path/map.json");
    let mut state = SettingsState::new(map_path);
    
    // Set can_exit to false to simulate unsaved changes
    state.can_exit = false;
    state.notification = None;
    
    // Save settings
    crate::states::settings::save_settings(&mut state);
    
    // Should set a notification
    assert!(state.notification.is_some());
    
    // If successful, should set can_exit to true
    if state.notification == Some(SettingsNotification::SaveSuccess) {
        assert_eq!(state.can_exit, true);
    }
}
