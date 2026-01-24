//! Tests for start screen input handling

#[cfg(test)]
mod tests {
    use crate::input::{start::start_kh, AppAction};
    use crate::states::{
        start::{FocusedInputBox, SelectedStartButton, StartState, RecentPaths},
    };
    use crate::utils::test_utils::MockFileSystem;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::path::PathBuf;

    fn create_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn create_test_start_state() -> StartState {
        let mut state = StartState::new();
        // Override recent_paths with test data to avoid filesystem interactions
        state.recent_paths = Ok(RecentPaths {
            recent_path_1: Some(PathBuf::from("/test/path1.json")),
            recent_path_2: Some(PathBuf::from("/test/path2.json")),
            recent_path_3: Some(PathBuf::from("/test/path3.json")),
        });
        state
    }

    #[test]
    fn test_quit_on_q() {
        let mut state = create_test_start_state();
        let key = create_key_event(KeyCode::Char('q'));
        let mock_fs = MockFileSystem::new();
        let result = start_kh(&mut state, key, &mock_fs);
        assert_eq!(result, AppAction::Quit);
    }

    #[test]
    fn test_navigation_with_k_and_up() {
        let mut state = create_test_start_state();
        state.selected_button = SelectedStartButton::Recent1;

        // Test 'k' key
        let key = create_key_event(KeyCode::Char('k'));
        let mock_fs = MockFileSystem::new();
        start_kh(&mut state, key, &mock_fs);
        assert_eq!(state.selected_button, SelectedStartButton::CreateSelect);

        // Test Up arrow
        state.selected_button = SelectedStartButton::Recent2;
        let key = create_key_event(KeyCode::Up);
        let mock_fs = MockFileSystem::new();
        start_kh(&mut state, key, &mock_fs);
        assert_eq!(state.selected_button, SelectedStartButton::Recent1);
    }

    #[test]
    fn test_navigation_with_j_and_down() {
        let mut state = create_test_start_state();
        
        // Test 'j' key
        let key = create_key_event(KeyCode::Char('j'));
        let mock_fs = MockFileSystem::new();
        start_kh(&mut state, key, &mock_fs);
        assert_eq!(state.selected_button, SelectedStartButton::Recent1);

        // Test Down arrow
        let key = create_key_event(KeyCode::Down);
        let mock_fs = MockFileSystem::new();
        start_kh(&mut state, key, &mock_fs);
        assert_eq!(state.selected_button, SelectedStartButton::Recent2);
    }

    #[test]
    fn test_navigation_boundaries() {
        let mut state = create_test_start_state();
        
        // Test that we can't go up from CreateSelect
        assert_eq!(state.selected_button, SelectedStartButton::CreateSelect);
        let key = create_key_event(KeyCode::Char('k'));
        let mock_fs = MockFileSystem::new();
        start_kh(&mut state, key, &mock_fs);
        assert_eq!(state.selected_button, SelectedStartButton::CreateSelect);

        // Test that we can't go down from Recent3
        state.selected_button = SelectedStartButton::Recent3;
        let key = create_key_event(KeyCode::Char('j'));
        let mock_fs = MockFileSystem::new();
        start_kh(&mut state, key, &mock_fs);
        assert_eq!(state.selected_button, SelectedStartButton::Recent3);
    }

    #[test]
    fn test_enter_on_create_select_opens_input_mode() {
        let mut state = create_test_start_state();
        state.selected_button = SelectedStartButton::CreateSelect;
        
        let key = create_key_event(KeyCode::Enter);
        let mock_fs = MockFileSystem::new();
        let result = start_kh(&mut state, key, &mock_fs);
        
        assert_eq!(result, AppAction::Continue);
        assert!(state.input_path);
        assert!(state.input_path_string.is_some());
        assert!(state.input_path_name.is_some());
        assert_eq!(state.focused_input_box, FocusedInputBox::InputBox1);
        assert!(state.display_err_msg.is_none());
    }

    #[test]
    fn test_enter_on_recent_paths_loads_file() {
        // Note: This test still requires real files because start_kh calls
        // submit_path() which uses the real filesystem. The actual filesystem
        // interaction is tested in the states/tests/start_tests.rs file with mocks.
        // Here we just verify that non-existent files are handled correctly.
        
        let mut state = create_test_start_state();
        let mock_fs = MockFileSystem::new();
        // Use paths that don't exist
        state.recent_paths = Ok(RecentPaths {
            recent_path_1: Some(PathBuf::from("/nonexistent/path1.json")),
            recent_path_2: Some(PathBuf::from("/nonexistent/path2.json")),
            recent_path_3: Some(PathBuf::from("/nonexistent/path3.json")),
        });
        
        // Test Recent1 - should return Continue since file doesn't exist
        state.selected_button = SelectedStartButton::Recent1;
        let key = create_key_event(KeyCode::Enter);
        let result = start_kh(&mut state, key, &mock_fs);
        assert_eq!(result, AppAction::Continue);
        assert_eq!(state.display_err_msg, Some(crate::states::start::ErrMsg::FileRead));

        // Reset error message for next test
        state.display_err_msg = None;

        // Test Recent2
        state.selected_button = SelectedStartButton::Recent2;
        let result = start_kh(&mut state, key, &mock_fs);
        assert_eq!(result, AppAction::Continue);
        assert_eq!(state.display_err_msg, Some(crate::states::start::ErrMsg::FileRead));

        // Reset error message for next test
        state.display_err_msg = None;

        // Test Recent3
        state.selected_button = SelectedStartButton::Recent3;
        let result = start_kh(&mut state, key, &mock_fs);
        assert_eq!(result, AppAction::Continue);
        assert_eq!(state.display_err_msg, Some(crate::states::start::ErrMsg::FileRead));
    }

    #[test]
    fn test_input_mode_escape_exits() {
        let mut state = create_test_start_state();
        let mock_fs = MockFileSystem::new();
        state.input_path = true;
        state.input_path_string = Some("test_path".to_string());
        state.input_path_name = Some("test_name".to_string());
        state.focused_input_box = FocusedInputBox::InputBox2;

        let key = create_key_event(KeyCode::Esc);
        let result = start_kh(&mut state, key, &mock_fs);

        assert_eq!(result, AppAction::Continue);
        assert!(!state.input_path);
        assert_eq!(state.focused_input_box, FocusedInputBox::InputBox1);
        assert!(state.input_path_string.is_none());
        assert!(state.input_path_name.is_none());
    }

    #[test]
    fn test_input_mode_char_input_in_input_box1() {
        let mut state = create_test_start_state();
        let mock_fs = MockFileSystem::new();
        state.input_path = true;
        state.input_path_string = Some(String::new());
        state.input_path_name = Some(String::new());
        state.focused_input_box = FocusedInputBox::InputBox1;

        let key = create_key_event(KeyCode::Char('t'));
        let result = start_kh(&mut state, key, &mock_fs);

        assert_eq!(result, AppAction::Continue);
        assert_eq!(state.input_path_string, Some("t".to_string()));
    }

    #[test]
    fn test_input_mode_char_input_in_input_box2() {
        let mut state = create_test_start_state();
        let mock_fs = MockFileSystem::new();
        state.input_path = true;
        state.input_path_string = Some("path".to_string());
        state.input_path_name = Some(String::new());
        state.focused_input_box = FocusedInputBox::InputBox2;

        let key = create_key_event(KeyCode::Char('n'));
        let result = start_kh(&mut state, key, &mock_fs);

        assert_eq!(result, AppAction::Continue);
        assert_eq!(state.input_path_name, Some("n".to_string()));
    }

    #[test]
    fn test_input_mode_char_input_length_limit() {
        let mut state = create_test_start_state();
        let mock_fs = MockFileSystem::new();
        state.input_path = true;
        state.input_path_string = Some("a".repeat(46)); // Already at max length
        state.input_path_name = Some(String::new());
        state.focused_input_box = FocusedInputBox::InputBox1;

        let key = create_key_event(KeyCode::Char('x'));
        let result = start_kh(&mut state, key, &mock_fs);

        assert_eq!(result, AppAction::Continue);
        // Should still be 46 characters, no new character added
        assert_eq!(state.input_path_string.as_ref().unwrap().len(), 46);
        assert_eq!(state.input_path_string, Some("a".repeat(46)));
    }

    #[test]
    fn test_input_mode_backspace_in_input_box1() {
        let mut state = create_test_start_state();
        let mock_fs = MockFileSystem::new();
        state.input_path = true;
        state.input_path_string = Some("test".to_string());
        state.input_path_name = Some(String::new());
        state.focused_input_box = FocusedInputBox::InputBox1;

        let key = create_key_event(KeyCode::Backspace);
        let result = start_kh(&mut state, key, &mock_fs);

        assert_eq!(result, AppAction::Continue);
        assert_eq!(state.input_path_string, Some("tes".to_string()));
    }

    #[test]
    fn test_input_mode_backspace_in_input_box2() {
        let mut state = create_test_start_state();
        let mock_fs = MockFileSystem::new();
        state.input_path = true;
        state.input_path_string = Some("path".to_string());
        state.input_path_name = Some("name".to_string());
        state.focused_input_box = FocusedInputBox::InputBox2;

        let key = create_key_event(KeyCode::Backspace);
        let result = start_kh(&mut state, key, &mock_fs);

        assert_eq!(result, AppAction::Continue);
        assert_eq!(state.input_path_name, Some("nam".to_string()));
    }

    #[test]
    fn test_input_mode_backspace_on_empty_string() {
        let mut state = create_test_start_state();
        let mock_fs = MockFileSystem::new();
        state.input_path = true;
        state.input_path_string = Some(String::new());
        state.input_path_name = Some(String::new());
        state.focused_input_box = FocusedInputBox::InputBox1;

        let key = create_key_event(KeyCode::Backspace);
        let result = start_kh(&mut state, key, &mock_fs);

        assert_eq!(result, AppAction::Continue);
        assert_eq!(state.input_path_string, Some(String::new()));
    }

    #[test]
    fn test_input_mode_enter_switches_focus() {
        let mut state = create_test_start_state();
        let mock_fs = MockFileSystem::new();
        state.input_path = true;
        state.input_path_string = Some("test_path".to_string());
        state.input_path_name = Some(String::new());
        state.focused_input_box = FocusedInputBox::InputBox1;

        let key = create_key_event(KeyCode::Enter);
        let result = start_kh(&mut state, key, &mock_fs);

        assert_eq!(result, AppAction::Continue);
        assert_eq!(state.focused_input_box, FocusedInputBox::InputBox2);
    }

    #[test]
    fn test_input_mode_enter_in_input_box2_submits() {
        // Note: This test verifies that Enter in InputBox2 calls submit_path.
        // The actual filesystem interactions are tested with mocks in 
        // states/tests/start_tests.rs. Here we just verify the input handler
        // correctly triggers the submission, without creating real directories.
        
        let mut state = create_test_start_state();
        let mock_fs = MockFileSystem::new();
        state.input_path = true;
        state.input_path_string = Some("/nonexistent/test_path/".to_string());
        state.input_path_name = Some("test_name".to_string());
        state.focused_input_box = FocusedInputBox::InputBox2;

        let key = create_key_event(KeyCode::Enter);
        let result = start_kh(&mut state, key, &mock_fs);

        // Mock filesystem returns ok for operations in submit_path_with_fs in default state
        // (If not set otherwise by constructors)
        assert_eq!(result, AppAction::CreateMapFile(PathBuf::from("/nonexistent/test_path/test_name.json")));
        assert!(state.needs_clear_and_redraw);
        // Should have an error message (either DirCreate or DirFind)
        assert!(state.display_err_msg.is_none());
    }

    #[test]
    fn test_unknown_key_returns_continue() {
        let mut state = create_test_start_state();
        let mock_fs = MockFileSystem::new();
        
        let key = create_key_event(KeyCode::Char('x')); // Random key not handled
        let result = start_kh(&mut state, key, &mock_fs);
        
        assert_eq!(result, AppAction::Continue);
    }

    #[test]
    fn test_input_mode_with_none_strings() {
        let mut state = create_test_start_state();
        let mock_fs = MockFileSystem::new();
        state.input_path = true;
        state.input_path_string = None; // This shouldn't happen in practice
        state.input_path_name = None;   // but we test defensive behavior
        state.focused_input_box = FocusedInputBox::InputBox1;

        let key = create_key_event(KeyCode::Char('t'));
        let result = start_kh(&mut state, key, &mock_fs);

        // Should still return Continue and not panic
        assert_eq!(result, AppAction::Continue);
    }

    #[test]
    fn test_recent_paths_with_none_values() {
        let mut state = create_test_start_state();
        let mock_fs = MockFileSystem::new();
        // Set recent paths with None values
        state.recent_paths = Ok(RecentPaths {
            recent_path_1: None,
            recent_path_2: None,
            recent_path_3: None,
        });
        state.selected_button = SelectedStartButton::Recent1;

        let key = create_key_event(KeyCode::Enter);
        let result = start_kh(&mut state, key, &mock_fs);

        // Should return Continue since the path is None
        assert_eq!(result, AppAction::Continue);
    }

    #[test]
    fn test_enter_on_recent_paths_nonexistent_file() {
        let mut state = create_test_start_state();
        let mock_fs = MockFileSystem::new();
        // Keep the default test paths (which don't exist)
        
        // Test Recent1 with non-existent file
        state.selected_button = SelectedStartButton::Recent1;
        let key = create_key_event(KeyCode::Enter);
        let result = start_kh(&mut state, key, &mock_fs);
        
        // Should return Continue since the file doesn't exist
        assert_eq!(result, AppAction::Continue);
        // Should have set an error message
        assert_eq!(state.display_err_msg, Some(crate::states::start::ErrMsg::FileRead));
    }
}