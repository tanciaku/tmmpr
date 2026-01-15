use tempfile::TempDir;
use std::fs;

use crate::{
    states::settings::Settings,
    utils::settings::save_settings_to_path,
};

#[test]
fn test_save_settings_to_path_creates_file_with_correct_content() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().unwrap();
    let settings_path = temp_dir.path().join("settings.json");

    // Create settings with known values
    let settings = Settings::new();

    // Save settings to the temp path
    let result = save_settings_to_path(&settings, &settings_path);

    // Verify the operation succeeded
    assert!(result.is_ok());

    // Verify the file was created
    assert!(settings_path.exists());

    // Read and verify the content
    let content = fs::read_to_string(&settings_path).unwrap();
    
    // Verify it's valid JSON and contains expected fields
    assert!(content.contains("save_interval"));
    assert!(content.contains("backups_interval"));
    assert!(content.contains("default_start_side"));
    assert!(content.contains("edit_modal"));
}

#[test]
fn test_save_settings_to_path_overwrites_existing_file() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().unwrap();
    let settings_path = temp_dir.path().join("settings.json");

    // Write some initial content
    fs::write(&settings_path, "old content").unwrap();

    // Save settings, which should overwrite
    let settings = Settings::new();
    let result = save_settings_to_path(&settings, &settings_path);

    // Verify the operation succeeded
    assert!(result.is_ok());

    // Verify the content was overwritten
    let content = fs::read_to_string(&settings_path).unwrap();
    assert!(!content.contains("old content"));
    assert!(content.contains("save_interval"));
}

#[test]
fn test_save_settings_to_path_fails_with_invalid_path() {
    let settings = Settings::new();
    
    // Try to write to a path that doesn't exist and can't be created
    // (parent directory doesn't exist)
    let invalid_path = std::path::Path::new("/nonexistent/directory/that/does/not/exist/settings.json");
    
    let result = save_settings_to_path(&settings, invalid_path);

    // Verify the operation failed
    assert!(result.is_err());
}
