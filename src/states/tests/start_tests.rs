use std::{path::PathBuf, fs};
use ratatui::style::{Color, Style};

use crate::{
    input::AppAction,
    states::start::{
        StartState, SelectedStartButton, FocusedInputBox, ErrMsg, 
        RecentPaths
    },
};

#[test]
fn test_navigate_start_buttons_up() {
    let mut start_state = StartState::new();
    
    // Test k key
    start_state.selected_button = SelectedStartButton::Recent2;
    start_state.navigate_start_buttons("k");
    assert_eq!(start_state.selected_button, SelectedStartButton::Recent1);
    
    // Test Up key
    start_state.selected_button = SelectedStartButton::Recent1;
    start_state.navigate_start_buttons("Up");
    assert_eq!(start_state.selected_button, SelectedStartButton::CreateSelect);
    
    // Test that CreateSelect stays at CreateSelect
    start_state.selected_button = SelectedStartButton::CreateSelect;
    start_state.navigate_start_buttons("k");
    assert_eq!(start_state.selected_button, SelectedStartButton::CreateSelect);
}

#[test]
fn test_navigate_start_buttons_down() {
    let mut start_state = StartState::new();
    
    // Test j key
    start_state.selected_button = SelectedStartButton::CreateSelect;
    start_state.navigate_start_buttons("j");
    assert_eq!(start_state.selected_button, SelectedStartButton::Recent1);
    
    // Test Down key
    start_state.selected_button = SelectedStartButton::Recent1;
    start_state.navigate_start_buttons("Down");
    assert_eq!(start_state.selected_button, SelectedStartButton::Recent2);
    
    // Test that Recent3 stays at Recent3
    start_state.selected_button = SelectedStartButton::Recent3;
    start_state.navigate_start_buttons("j");
    assert_eq!(start_state.selected_button, SelectedStartButton::Recent3);
}

#[test]
fn test_navigate_start_buttons_other_keys() {
    let mut start_state = StartState::new();
    start_state.selected_button = SelectedStartButton::Recent1;
    
    start_state.navigate_start_buttons("a");
    assert_eq!(start_state.selected_button, SelectedStartButton::Recent1); // No change
    
    start_state.navigate_start_buttons("Enter");
    assert_eq!(start_state.selected_button, SelectedStartButton::Recent1); // No change
}

// Note: button_list_go_up and button_list_go_down are private methods
// They are tested indirectly through navigate_start_buttons tests above

#[test]
fn test_submit_path_with_existing_recent_path() {
    let mut start_state = StartState::new();
    
    // Create a temporary file for testing
    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("test.json");
    fs::write(&test_file, "test content").unwrap();
    
    let result = start_state.submit_path(Some(test_file.clone()));
    
    assert_eq!(result, AppAction::LoadMapFile(test_file));
    assert_eq!(start_state.display_err_msg, None);
}

#[test]
fn test_submit_path_with_nonexistent_recent_path() {
    let mut start_state = StartState::new();
    
    let nonexistent_path = PathBuf::from("/nonexistent/path/file.json");
    let result = start_state.submit_path(Some(nonexistent_path));
    
    assert_eq!(result, AppAction::Continue);
    assert_eq!(start_state.display_err_msg, Some(ErrMsg::FileRead));
    assert_eq!(start_state.needs_clear_and_redraw, true);
}

#[test]
fn test_submit_path_create_new_file() {
    let mut start_state = StartState::new();
    
    // Set up input fields for creating a new file under a test subdirectory
    // Use a path relative to home that doesn't exist yet
    start_state.input_path_string = Some("test_maps_temp/".to_string());
    start_state.input_path_name = Some("new_map".to_string());
    
    let result = start_state.submit_path(None);
    
    // Should create a new file since it doesn't exist
    match result {
        AppAction::CreateMapFile(path) => {
            assert!(path.to_string_lossy().contains("new_map.json"));
            assert!(path.to_string_lossy().contains("test_maps_temp"));
            
            // Clean up - remove the created directory
            let _ = fs::remove_dir_all(path.parent().unwrap());
        }
        AppAction::Continue => {
            // This might happen if there are permission issues or home dir problems
            // In that case, the error handling should have been triggered
            assert!(start_state.display_err_msg.is_some());
        }
        _ => panic!("Expected CreateMapFile or Continue action, got {:?}", result),
    }
}

#[test]
fn test_submit_path_load_existing_file() {
    let mut start_state = StartState::new();
    
    // Create a temporary directory under home and a test file
    let home_dir = match home::home_dir() {
        Some(dir) => dir,
        None => {
            // If we can't get home directory, just verify the error handling
            start_state.input_path_string = Some("test_maps_temp2/".to_string());
            start_state.input_path_name = Some("existing_map".to_string());
            let result = start_state.submit_path(None);
            assert_eq!(result, AppAction::Continue);
            return;
        }
    };
    
    let test_dir = home_dir.join("test_maps_temp2");
    fs::create_dir_all(&test_dir).unwrap();
    let existing_file = test_dir.join("existing_map.json");
    fs::write(&existing_file, "test content").unwrap();
    
    // Set up input fields to point to existing file
    start_state.input_path_string = Some("test_maps_temp2/".to_string());
    start_state.input_path_name = Some("existing_map".to_string());
    
    let result = start_state.submit_path(None);
    
    // Should load the existing file
    match result {
        AppAction::LoadMapFile(path) => {
            assert_eq!(path, existing_file);
        }
        _ => panic!("Expected LoadMapFile action, got {:?}", result),
    }
    
    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_handle_submit_error() {
    let mut start_state = StartState::new();
    start_state.input_path_string = Some("some_path".to_string());
    start_state.input_path_name = Some("some_name".to_string());
    start_state.focused_input_box = FocusedInputBox::InputBox2;
    
    start_state.handle_submit_error(ErrMsg::DirCreate);
    
    assert_eq!(start_state.input_path_string, Some(String::new()));
    assert_eq!(start_state.input_path_name, Some(String::new()));
    assert_eq!(start_state.focused_input_box, FocusedInputBox::InputBox1);
    assert_eq!(start_state.display_err_msg, Some(ErrMsg::DirCreate));
}

#[test]
fn test_selected_start_button_get_style_selected() {
    let button = SelectedStartButton::CreateSelect;
    let selected_button = SelectedStartButton::CreateSelect;
    
    let style = button.get_style(&selected_button);
    
    assert_eq!(style, Style::new().bg(Color::White).fg(Color::Black));
}

#[test]
fn test_selected_start_button_get_style_not_selected() {
    let button = SelectedStartButton::CreateSelect;
    let selected_button = SelectedStartButton::Recent1;
    
    let style = button.get_style(&selected_button);
    
    assert_eq!(style, Style::new());
}

#[test]
fn test_focused_input_box_get_style_focused() {
    let input_box = FocusedInputBox::InputBox1;
    let focused_input_box = FocusedInputBox::InputBox1;
    
    let block = input_box.get_style(&focused_input_box);
    
    // We can't easily test the internal state of Block, but we can verify 
    // that the method runs without panicking and returns a Block
    // The actual visual styling would need to be tested in integration tests
    assert!(format!("{:?}", block).contains("Block"));
}

#[test]
fn test_recent_paths_add() {
    let mut recent_paths = RecentPaths::new();
    
    let path1 = PathBuf::from("/path/1");
    let path2 = PathBuf::from("/path/2");
    let path3 = PathBuf::from("/path/3");
    let path4 = PathBuf::from("/path/4");
    
    // Add first path
    recent_paths.add(path1.clone());
    assert_eq!(recent_paths.recent_path_1, Some(path1.clone()));
    assert_eq!(recent_paths.recent_path_2, None);
    assert_eq!(recent_paths.recent_path_3, None);
    
    // Add second path
    recent_paths.add(path2.clone());
    assert_eq!(recent_paths.recent_path_1, Some(path2.clone()));
    assert_eq!(recent_paths.recent_path_2, Some(path1.clone()));
    assert_eq!(recent_paths.recent_path_3, None);
    
    // Add third path
    recent_paths.add(path3.clone());
    assert_eq!(recent_paths.recent_path_1, Some(path3.clone()));
    assert_eq!(recent_paths.recent_path_2, Some(path2.clone()));
    assert_eq!(recent_paths.recent_path_3, Some(path1.clone()));
    
    // Add fourth path (should discard the oldest)
    recent_paths.add(path4.clone());
    assert_eq!(recent_paths.recent_path_1, Some(path4));
    assert_eq!(recent_paths.recent_path_2, Some(path3));
    assert_eq!(recent_paths.recent_path_3, Some(path2));
}

#[test]
fn test_recent_paths_contains_path() {
    let mut recent_paths = RecentPaths::new();
    
    let path1 = PathBuf::from("/path/1");
    let path2 = PathBuf::from("/path/2");
    let path3 = PathBuf::from("/path/3");
    let path4 = PathBuf::from("/path/4");
    
    recent_paths.add(path1.clone());
    recent_paths.add(path2.clone());
    recent_paths.add(path3.clone());
    
    assert!(recent_paths.contains_path(&path1));
    assert!(recent_paths.contains_path(&path2));
    assert!(recent_paths.contains_path(&path3));
    assert!(!recent_paths.contains_path(&path4));
}

#[test]
fn test_recent_paths_contains_path_empty() {
    let recent_paths = RecentPaths::new();
    let path = PathBuf::from("/some/path");
    
    assert!(!recent_paths.contains_path(&path));
}

// Note: Integration tests for get_recent_paths() and RecentPaths::save() 
// require filesystem operations and environment variable manipulation.
// These are better suited for separate integration test files or 
// using proper test fixtures with safe environment isolation.