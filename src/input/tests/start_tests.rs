//! Tests for start screen input handling

#[cfg(test)]
mod tests {
    use crate::input::{start::start_kh, AppAction};
    use crate::states::{
        start::{FocusedInputBox, SelectedStartButton, StartState, RecentPaths},
    };
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
        let result = start_kh(&mut state, key);
        assert_eq!(result, AppAction::Quit);
    }

    #[test]
    fn test_navigation_with_k_and_up() {
        let mut state = create_test_start_state();
        state.selected_button = SelectedStartButton::Recent1;

        // Test 'k' key
        let key = create_key_event(KeyCode::Char('k'));
        start_kh(&mut state, key);
        assert_eq!(state.selected_button, SelectedStartButton::CreateSelect);

        // Test Up arrow
        state.selected_button = SelectedStartButton::Recent2;
        let key = create_key_event(KeyCode::Up);
        start_kh(&mut state, key);
        assert_eq!(state.selected_button, SelectedStartButton::Recent1);
    }

    #[test]
    fn test_navigation_with_j_and_down() {
        let mut state = create_test_start_state();
        
        // Test 'j' key
        let key = create_key_event(KeyCode::Char('j'));
        start_kh(&mut state, key);
        assert_eq!(state.selected_button, SelectedStartButton::Recent1);

        // Test Down arrow
        let key = create_key_event(KeyCode::Down);
        start_kh(&mut state, key);
        assert_eq!(state.selected_button, SelectedStartButton::Recent2);
    }

    #[test]
    fn test_navigation_boundaries() {
        let mut state = create_test_start_state();
        
        // Test that we can't go up from CreateSelect
        assert_eq!(state.selected_button, SelectedStartButton::CreateSelect);
        let key = create_key_event(KeyCode::Char('k'));
        start_kh(&mut state, key);
        assert_eq!(state.selected_button, SelectedStartButton::CreateSelect);

        // Test that we can't go down from Recent3
        state.selected_button = SelectedStartButton::Recent3;
        let key = create_key_event(KeyCode::Char('j'));
        start_kh(&mut state, key);
        assert_eq!(state.selected_button, SelectedStartButton::Recent3);
    }

    #[test]
    fn test_enter_on_create_select_opens_input_mode() {
        let mut state = create_test_start_state();
        state.selected_button = SelectedStartButton::CreateSelect;
        
        let key = create_key_event(KeyCode::Enter);
        let result = start_kh(&mut state, key);
        
        assert_eq!(result, AppAction::Continue);
        assert!(state.input_path);
        assert!(state.input_path_string.is_some());
        assert!(state.input_path_name.is_some());
        assert_eq!(state.focused_input_box, FocusedInputBox::InputBox1);
        assert!(state.display_err_msg.is_none());
    }

    #[test]
    fn test_enter_on_recent_paths_loads_file() {
        use std::fs::File;
        use tempfile::tempdir;
        
        // Create temporary directory and files for testing
        let temp_dir = tempdir().unwrap();
        let temp_path1 = temp_dir.path().join("path1.json");
        let temp_path2 = temp_dir.path().join("path2.json");
        let temp_path3 = temp_dir.path().join("path3.json");
        
        // Create the temporary files
        File::create(&temp_path1).unwrap();
        File::create(&temp_path2).unwrap();
        File::create(&temp_path3).unwrap();
        
        let mut state = create_test_start_state();
        // Override with existing temporary file paths
        state.recent_paths = Ok(RecentPaths {
            recent_path_1: Some(temp_path1.clone()),
            recent_path_2: Some(temp_path2.clone()),
            recent_path_3: Some(temp_path3.clone()),
        });
        
        // Test Recent1
        state.selected_button = SelectedStartButton::Recent1;
        let key = create_key_event(KeyCode::Enter);
        let result = start_kh(&mut state, key);
        if let AppAction::LoadMapFile(path) = result {
            assert_eq!(path, temp_path1);
        } else {
            panic!("Expected LoadMapFile action, got: {:?}", result);
        }

        // Test Recent2
        state.selected_button = SelectedStartButton::Recent2;
        let result = start_kh(&mut state, key);
        if let AppAction::LoadMapFile(path) = result {
            assert_eq!(path, temp_path2);
        } else {
            panic!("Expected LoadMapFile action, got: {:?}", result);
        }

        // Test Recent3
        state.selected_button = SelectedStartButton::Recent3;
        let result = start_kh(&mut state, key);
        if let AppAction::LoadMapFile(path) = result {
            assert_eq!(path, temp_path3);
        } else {
            panic!("Expected LoadMapFile action, got: {:?}", result);
        }
    }

    #[test]
    fn test_input_mode_escape_exits() {
        let mut state = create_test_start_state();
        state.input_path = true;
        state.input_path_string = Some("test_path".to_string());
        state.input_path_name = Some("test_name".to_string());
        state.focused_input_box = FocusedInputBox::InputBox2;

        let key = create_key_event(KeyCode::Esc);
        let result = start_kh(&mut state, key);

        assert_eq!(result, AppAction::Continue);
        assert!(!state.input_path);
        assert_eq!(state.focused_input_box, FocusedInputBox::InputBox1);
        assert!(state.input_path_string.is_none());
        assert!(state.input_path_name.is_none());
    }

    #[test]
    fn test_input_mode_char_input_in_input_box1() {
        let mut state = create_test_start_state();
        state.input_path = true;
        state.input_path_string = Some(String::new());
        state.input_path_name = Some(String::new());
        state.focused_input_box = FocusedInputBox::InputBox1;

        let key = create_key_event(KeyCode::Char('t'));
        let result = start_kh(&mut state, key);

        assert_eq!(result, AppAction::Continue);
        assert_eq!(state.input_path_string, Some("t".to_string()));
    }

    #[test]
    fn test_input_mode_char_input_in_input_box2() {
        let mut state = create_test_start_state();
        state.input_path = true;
        state.input_path_string = Some("path".to_string());
        state.input_path_name = Some(String::new());
        state.focused_input_box = FocusedInputBox::InputBox2;

        let key = create_key_event(KeyCode::Char('n'));
        let result = start_kh(&mut state, key);

        assert_eq!(result, AppAction::Continue);
        assert_eq!(state.input_path_name, Some("n".to_string()));
    }

    #[test]
    fn test_input_mode_char_input_length_limit() {
        let mut state = create_test_start_state();
        state.input_path = true;
        state.input_path_string = Some("a".repeat(46)); // Already at max length
        state.input_path_name = Some(String::new());
        state.focused_input_box = FocusedInputBox::InputBox1;

        let key = create_key_event(KeyCode::Char('x'));
        let result = start_kh(&mut state, key);

        assert_eq!(result, AppAction::Continue);
        // Should still be 46 characters, no new character added
        assert_eq!(state.input_path_string.as_ref().unwrap().len(), 46);
        assert_eq!(state.input_path_string, Some("a".repeat(46)));
    }

    #[test]
    fn test_input_mode_backspace_in_input_box1() {
        let mut state = create_test_start_state();
        state.input_path = true;
        state.input_path_string = Some("test".to_string());
        state.input_path_name = Some(String::new());
        state.focused_input_box = FocusedInputBox::InputBox1;

        let key = create_key_event(KeyCode::Backspace);
        let result = start_kh(&mut state, key);

        assert_eq!(result, AppAction::Continue);
        assert_eq!(state.input_path_string, Some("tes".to_string()));
    }

    #[test]
    fn test_input_mode_backspace_in_input_box2() {
        let mut state = create_test_start_state();
        state.input_path = true;
        state.input_path_string = Some("path".to_string());
        state.input_path_name = Some("name".to_string());
        state.focused_input_box = FocusedInputBox::InputBox2;

        let key = create_key_event(KeyCode::Backspace);
        let result = start_kh(&mut state, key);

        assert_eq!(result, AppAction::Continue);
        assert_eq!(state.input_path_name, Some("nam".to_string()));
    }

    #[test]
    fn test_input_mode_backspace_on_empty_string() {
        let mut state = create_test_start_state();
        state.input_path = true;
        state.input_path_string = Some(String::new());
        state.input_path_name = Some(String::new());
        state.focused_input_box = FocusedInputBox::InputBox1;

        let key = create_key_event(KeyCode::Backspace);
        let result = start_kh(&mut state, key);

        assert_eq!(result, AppAction::Continue);
        assert_eq!(state.input_path_string, Some(String::new()));
    }

    #[test]
    fn test_input_mode_enter_switches_focus() {
        let mut state = create_test_start_state();
        state.input_path = true;
        state.input_path_string = Some("test_path".to_string());
        state.input_path_name = Some(String::new());
        state.focused_input_box = FocusedInputBox::InputBox1;

        let key = create_key_event(KeyCode::Enter);
        let result = start_kh(&mut state, key);

        assert_eq!(result, AppAction::Continue);
        assert_eq!(state.focused_input_box, FocusedInputBox::InputBox2);
    }

    #[test]
    fn test_input_mode_enter_in_input_box2_submits() {
        let mut state = create_test_start_state();
        state.input_path = true;
        state.input_path_string = Some("test_path".to_string());
        state.input_path_name = Some("test_name".to_string());
        state.focused_input_box = FocusedInputBox::InputBox2;

        let key = create_key_event(KeyCode::Enter);
        let result = start_kh(&mut state, key);

        // The submit_path function will be called and can return:
        // - AppAction::Continue (if there's an error like home dir not found)
        // - AppAction::LoadMapFile (if file exists)  
        // - AppAction::CreateMapFile (if file doesn't exist)
        // We just verify that the function was called by checking that
        // clear_and_redraw was called (needs_clear_and_redraw should be true)
        assert!(state.needs_clear_and_redraw);
        
        // The result can be any valid AppAction
        match result {
            AppAction::Continue | AppAction::LoadMapFile(_) | AppAction::CreateMapFile(_) => {
                // These are all valid outcomes
            }
            _ => panic!("Unexpected AppAction returned: {:?}", result),
        }
    }

    #[test]
    fn test_unknown_key_returns_continue() {
        let mut state = create_test_start_state();
        
        let key = create_key_event(KeyCode::Char('x')); // Random key not handled
        let result = start_kh(&mut state, key);
        
        assert_eq!(result, AppAction::Continue);
    }

    #[test]
    fn test_input_mode_with_none_strings() {
        let mut state = create_test_start_state();
        state.input_path = true;
        state.input_path_string = None; // This shouldn't happen in practice
        state.input_path_name = None;   // but we test defensive behavior
        state.focused_input_box = FocusedInputBox::InputBox1;

        let key = create_key_event(KeyCode::Char('t'));
        let result = start_kh(&mut state, key);

        // Should still return Continue and not panic
        assert_eq!(result, AppAction::Continue);
    }

    #[test]
    fn test_recent_paths_with_none_values() {
        let mut state = create_test_start_state();
        // Set recent paths with None values
        state.recent_paths = Ok(RecentPaths {
            recent_path_1: None,
            recent_path_2: None,
            recent_path_3: None,
        });
        state.selected_button = SelectedStartButton::Recent1;

        let key = create_key_event(KeyCode::Enter);
        let result = start_kh(&mut state, key);

        // Should return Continue since the path is None
        assert_eq!(result, AppAction::Continue);
    }

    #[test]
    fn test_enter_on_recent_paths_nonexistent_file() {
        let mut state = create_test_start_state();
        // Keep the default test paths (which don't exist)
        
        // Test Recent1 with non-existent file
        state.selected_button = SelectedStartButton::Recent1;
        let key = create_key_event(KeyCode::Enter);
        let result = start_kh(&mut state, key);
        
        // Should return Continue since the file doesn't exist
        assert_eq!(result, AppAction::Continue);
        // Should have set an error message
        assert_eq!(state.display_err_msg, Some(crate::states::start::ErrMsg::FileRead));
    }
}