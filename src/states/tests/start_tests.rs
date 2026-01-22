use std::path::PathBuf;
use ratatui::style::{Color, Style};

use crate::{
    input::AppAction,
    states::start::{
        StartState, SelectedStartButton, FocusedInputBox, ErrMsg, 
        RecentPaths, get_recent_paths_with_fs
    },
    utils::filesystem::test_utils::MockFileSystem, 
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
    
    let test_file = PathBuf::from("/test/path.json");
    let mock_fs = MockFileSystem::new().with_existing_path(test_file.clone());
    
    let result = start_state.submit_path_with_fs(Some(test_file.clone()), &mock_fs);
    
    assert_eq!(result, AppAction::LoadMapFile(test_file));
    assert_eq!(start_state.display_err_msg, None);
}

#[test]
fn test_submit_path_with_nonexistent_recent_path() {
    let mut start_state = StartState::new();
    
    let nonexistent_path = PathBuf::from("/nonexistent/path/file.json");
    let mock_fs = MockFileSystem::new();
    
    let result = start_state.submit_path_with_fs(Some(nonexistent_path), &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert_eq!(start_state.display_err_msg, Some(ErrMsg::FileRead));
    assert_eq!(start_state.needs_clear_and_redraw, true);
}

#[test]
fn test_submit_path_create_new_file() {
    let mut start_state = StartState::new();
    
    start_state.input_path_string = Some("test_maps_temp/".to_string());
    start_state.input_path_name = Some("new_map".to_string());
    
    let mock_fs = MockFileSystem::new();
    let result = start_state.submit_path_with_fs(None, &mock_fs);
    
    // Should create a new file since it doesn't exist
    match result {
        AppAction::CreateMapFile(path) => {
            assert!(path.to_string_lossy().contains("new_map.json"));
            assert!(path.to_string_lossy().contains("test_maps_temp"));
            assert_eq!(path, PathBuf::from("/mock/home/test_maps_temp/new_map.json"));
        }
        _ => panic!("Expected CreateMapFile action, got {:?}", result),
    }
}

#[test]
fn test_submit_path_load_existing_file() {
    let mut start_state = StartState::new();
    
    start_state.input_path_string = Some("test_maps_temp2/".to_string());
    start_state.input_path_name = Some("existing_map".to_string());
    
    let expected_path = PathBuf::from("/mock/home/test_maps_temp2/existing_map.json");
    let mock_fs = MockFileSystem::new().with_existing_path(expected_path.clone());
    
    let result = start_state.submit_path_with_fs(None, &mock_fs);
    
    // Should load the existing file
    match result {
        AppAction::LoadMapFile(path) => {
            assert_eq!(path, expected_path);
        }
        _ => panic!("Expected LoadMapFile action, got {:?}", result),
    }
}

#[test]
fn test_submit_path_no_home_dir() {
    let mut start_state = StartState::new();
    
    start_state.input_path_string = Some("maps/".to_string());
    start_state.input_path_name = Some("my_map".to_string());
    
    let mock_fs = MockFileSystem::new().with_home_dir(None);
    let result = start_state.submit_path_with_fs(None, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert_eq!(start_state.display_err_msg, Some(ErrMsg::DirFind));
    // Input fields should be reset
    assert_eq!(start_state.input_path_string, Some(String::new()));
    assert_eq!(start_state.input_path_name, Some(String::new()));
    assert_eq!(start_state.focused_input_box, FocusedInputBox::InputBox1);
}

#[test]
fn test_submit_path_dir_create_failure() {
    let mut start_state = StartState::new();
    
    start_state.input_path_string = Some("maps/".to_string());
    start_state.input_path_name = Some("my_map".to_string());
    
    let mock_fs = MockFileSystem::new().with_dir_create_failure();
    let result = start_state.submit_path_with_fs(None, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert_eq!(start_state.display_err_msg, Some(ErrMsg::DirCreate));
    // Input fields should be reset
    assert_eq!(start_state.input_path_string, Some(String::new()));
    assert_eq!(start_state.input_path_name, Some(String::new()));
    assert_eq!(start_state.focused_input_box, FocusedInputBox::InputBox1);
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

#[test]
fn test_get_recent_paths_no_home_dir() { 
    // Test error when home directory is not available
    let mock_fs = MockFileSystem::new().with_home_dir(None);
    let result = get_recent_paths_with_fs(&mock_fs);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ErrMsg::DirFind);
}

#[test]
fn test_get_recent_paths_dir_create_failure() {
    // Test error when directory creation fails
    let mock_fs = MockFileSystem::new().with_dir_create_failure();
    let result = get_recent_paths_with_fs(&mock_fs);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ErrMsg::DirCreate);
}

#[test]
fn test_recent_paths_save_with_mock() {
    let mut recent_paths = RecentPaths::new();
    recent_paths.add(PathBuf::from("/test/path.json"));
    
    // Test that save_with_fs can be called with MockFileSystem
    // This verifies the method signature works correctly with the trait
    let mock_fs = MockFileSystem::new();
    recent_paths.save_with_fs(&mock_fs);
    
    // The save returns () so we just verify it doesn't panic
    // In production, this would write to ~/.config/tmmpr/recent_paths.json
    // With the mock, it just validates the logic without touching the filesystem
}

#[test]
fn test_recent_paths_save_no_home_dir() {
    let mut recent_paths = RecentPaths::new();
    recent_paths.add(PathBuf::from("/test/path.json"));
    
    // Test that save_with_fs handles missing home directory gracefully
    let mock_fs = MockFileSystem::new().with_home_dir(None);
    recent_paths.save_with_fs(&mock_fs);
    
    // Should return early without panicking when home dir is None
}