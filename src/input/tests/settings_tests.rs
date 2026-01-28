use std::path::PathBuf;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, KeyEventKind};
use tempfile::TempDir;

use crate::{
    app::Screen,
    input::{AppAction, settings::settings_kh},
    states::{
        SettingsState, map::Side, settings::{
            BackupsErr, BackupsInterval, DiscardExitTo, RuntimeBackupsInterval, SelectedToggle, Settings, SettingsType
        }, start::ErrMsg
    },
    utils::test_utils::{MockFileSystem, TempFileSystem},
};

// Helper function to create a key event
fn create_key_event(key_code: KeyCode) -> KeyEvent {
    KeyEvent {
        code: key_code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::NONE,
    }
}

// Helper function to create a settings state with default settings
fn create_default_settings_state() -> SettingsState {
    let mock_fs = MockFileSystem::new();
    let mut state = SettingsState::new_with_fs(PathBuf::from("/test/map.json"), &mock_fs);
    state.settings = SettingsType::Default(Settings::new(), None);
    state.can_exit = true;
    state
}

// Helper function to create a settings state with error
fn create_error_settings_state() -> SettingsState {
    let mock_fs = MockFileSystem::new();
    let mut state = SettingsState::new_with_fs(PathBuf::from("/test/map.json"), &mock_fs);
    state.settings = SettingsType::Default(Settings::new(), Some(ErrMsg::FileRead));
    state
}

#[test]
fn test_settings_error_state_q_key() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_error_settings_state();
    let key_event = create_key_event(KeyCode::Char('q'));
    
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    match result {
        AppAction::Switch(Screen::Start(_)) => assert!(true),
        _ => panic!("Expected switch to start screen"),
    }
}

#[test]
fn test_settings_error_state_o_key() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_error_settings_state();
    let key_event = create_key_event(KeyCode::Char('o'));
    
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    match result {
        AppAction::LoadMapFile(path) => {
            assert_eq!(path, PathBuf::from("/test/map.json"));
        }
        _ => panic!("Expected load map file action"),
    }
}

#[test]
fn test_settings_error_state_other_keys() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_error_settings_state();
    let key_event = create_key_event(KeyCode::Char('b'));
    
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    // Should continue for any other key
    assert_eq!(result, AppAction::Continue);
}

#[test]
fn test_confirm_discard_menu_esc() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.confirm_discard_menu = Some(DiscardExitTo::StartScreen);
    state.needs_clear_and_redraw = false;
    
    let key_event = create_key_event(KeyCode::Esc);
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert!(state.confirm_discard_menu.is_none());
    assert!(state.needs_clear_and_redraw);
}

#[test]
fn test_confirm_discard_menu_q_to_start_screen() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.confirm_discard_menu = Some(DiscardExitTo::StartScreen);
    
    let key_event = create_key_event(KeyCode::Char('q'));
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    match result {
        AppAction::Switch(Screen::Start(_)) => assert!(true),
        _ => panic!("Expected switch to start screen"),
    }
}

#[test]
fn test_confirm_discard_menu_q_to_map_screen() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.confirm_discard_menu = Some(DiscardExitTo::MapScreen);
    
    let key_event = create_key_event(KeyCode::Char('q'));
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    match result {
        AppAction::LoadMapFile(path) => {
            assert_eq!(path, PathBuf::from("/test/map.json"));
        }
        _ => panic!("Expected load map file action"),
    }
}

#[test]
fn test_confirm_discard_menu_other_keys() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.confirm_discard_menu = Some(DiscardExitTo::StartScreen);
    state.needs_clear_and_redraw = false;
    
    let key_event = create_key_event(KeyCode::Char('s'));
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert!(state.needs_clear_and_redraw);
}

#[test]
fn test_context_page_help_key() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.context_page = true;
    state.needs_clear_and_redraw = false;
    
    let key_event = create_key_event(KeyCode::Char('?'));
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert!(!state.context_page);
    assert!(state.needs_clear_and_redraw);
}

#[test]
fn test_context_page_f1_key() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.context_page = true;
    state.needs_clear_and_redraw = false;
    
    let key_event = create_key_event(KeyCode::F(1));
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert!(!state.context_page);
    assert!(state.needs_clear_and_redraw);
}

#[test]
fn test_context_page_other_keys() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.context_page = true;
    state.needs_clear_and_redraw = false;
    
    let key_event = create_key_event(KeyCode::Char('a'));
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert!(state.context_page); // Should still be true
    assert!(state.needs_clear_and_redraw);
}

#[test]
fn test_input_prompt_esc() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.input_prompt = true;
    state.settings.settings_mut().backups_path = Some(String::from("test_path"));
    state.settings.settings_mut().backups_interval = Some(BackupsInterval::Daily);
    state.settings.settings_mut().runtime_backups_interval = Some(RuntimeBackupsInterval::Hourly);
    state.input_prompt_err = Some(BackupsErr::DirCreate);
    state.needs_clear_and_redraw = false;
    
    let key_event = create_key_event(KeyCode::Esc);
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert!(!state.input_prompt);
    assert!(state.settings.settings().backups_path.is_none());
    assert!(state.settings.settings().backups_interval.is_none());
    assert!(state.settings.settings().runtime_backups_interval.is_none());
    assert!(state.input_prompt_err.is_none());
    assert!(state.needs_clear_and_redraw);
}

#[test]
fn test_input_prompt_char_typing() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.input_prompt = true;
    state.settings.settings_mut().backups_path = Some(String::from("test"));
    state.needs_clear_and_redraw = false;
    
    let key_event = create_key_event(KeyCode::Char('a'));
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert_eq!(state.settings.settings().backups_path.as_ref().unwrap(), "testa");
    assert!(state.needs_clear_and_redraw);
}

#[test]
fn test_input_prompt_char_typing_max_length() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.input_prompt = true;
    // Create a string that's exactly 46 characters long
    let long_string = "a".repeat(46);
    state.settings.settings_mut().backups_path = Some(long_string.clone());
    
    let key_event = create_key_event(KeyCode::Char('x'));
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    // Should not have added the character due to length limit
    assert_eq!(state.settings.settings().backups_path.as_ref().unwrap(), &long_string);
}

#[test]
fn test_input_prompt_backspace() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.input_prompt = true;
    state.settings.settings_mut().backups_path = Some(String::from("test"));
    state.needs_clear_and_redraw = false;
    
    let key_event = create_key_event(KeyCode::Backspace);
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert_eq!(state.settings.settings().backups_path.as_ref().unwrap(), "tes");
    assert!(state.needs_clear_and_redraw);
}

#[test]
fn test_input_prompt_backspace_empty() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.input_prompt = true;
    state.settings.settings_mut().backups_path = Some(String::new());
    
    let key_event = create_key_event(KeyCode::Backspace);
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    // Should remain empty
    assert_eq!(state.settings.settings().backups_path.as_ref().unwrap(), "");
}

#[test]
fn test_input_prompt_enter() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.input_prompt = true;
    state.settings.settings_mut().backups_path = Some(String::from("/tmp"));
    state.needs_clear_and_redraw = false;
    
    let key_event = create_key_event(KeyCode::Enter);
    let _result = settings_kh(&mut state, key_event, &mock_fs);
    
    // Note: submit_path will likely fail in tests due to filesystem operations
    // but we can verify the function was called by checking the state changes
    assert!(state.needs_clear_and_redraw);
}

#[test]
fn test_normal_mode_q_can_exit() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.can_exit = true;
    
    let key_event = create_key_event(KeyCode::Char('q'));
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    match result {
        AppAction::Switch(Screen::Start(_)) => assert!(true),
        _ => panic!("Expected switch to start screen"),
    }
}

#[test]
fn test_normal_mode_q_cannot_exit() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.can_exit = false;
    
    let key_event = create_key_event(KeyCode::Char('q'));
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert_eq!(state.confirm_discard_menu, Some(DiscardExitTo::StartScreen));
}

#[test]
fn test_normal_mode_o_can_exit() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.can_exit = true;
    
    let key_event = create_key_event(KeyCode::Char('o'));
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    match result {
        AppAction::LoadMapFile(path) => {
            assert_eq!(path, PathBuf::from("/test/map.json"));
        }
        _ => panic!("Expected load map file action"),
    }
}

#[test]
fn test_normal_mode_o_cannot_exit() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.can_exit = false;
    
    let key_event = create_key_event(KeyCode::Char('o'));
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert_eq!(state.confirm_discard_menu, Some(DiscardExitTo::MapScreen));
}

#[test]
fn test_normal_mode_help_key() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    
    let key_event = create_key_event(KeyCode::Char('?'));
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert!(state.context_page);
}

#[test]
fn test_normal_mode_f1_key() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    
    let key_event = create_key_event(KeyCode::F(1));
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert!(state.context_page);
}

#[test]
fn test_normal_mode_save_key() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_path_buf(); 
    let temp_fs = TempFileSystem { home_path: temp_path.clone() };

    let mut state = create_default_settings_state();
    
    let key_event = create_key_event(KeyCode::Char('s'));
    let _result = settings_kh(&mut state, key_event, &temp_fs);
}

#[test]
fn test_normal_mode_navigation_down() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.selected_toggle = SelectedToggle::Toggle1;
    state.needs_clear_and_redraw = false;
    
    let key_event = create_key_event(KeyCode::Char('j'));
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle2);
    assert!(state.needs_clear_and_redraw);
}

#[test]
fn test_normal_mode_navigation_down_arrow() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.selected_toggle = SelectedToggle::Toggle1;
    
    let key_event = create_key_event(KeyCode::Down);
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle2);
}

#[test]
fn test_normal_mode_navigation_up() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.selected_toggle = SelectedToggle::Toggle2;
    state.needs_clear_and_redraw = false;
    
    let key_event = create_key_event(KeyCode::Char('k'));
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle1);
    assert!(state.needs_clear_and_redraw);
}

#[test]
fn test_normal_mode_navigation_up_arrow() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.selected_toggle = SelectedToggle::Toggle2;
    
    let key_event = create_key_event(KeyCode::Up);
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert_eq!(state.selected_toggle, SelectedToggle::Toggle1);
}

#[test]
fn test_normal_mode_enter_toggle1() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.selected_toggle = SelectedToggle::Toggle1;
    state.can_exit = true;
    let initial_interval = state.settings.settings().save_interval;
    
    let key_event = create_key_event(KeyCode::Enter);
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert!(!state.can_exit);
    // Should have cycled the save interval
    assert_ne!(state.settings.settings().save_interval, initial_interval);
}

#[test]
fn test_normal_mode_enter_toggle2() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.selected_toggle = SelectedToggle::Toggle2;
    state.can_exit = true;
    
    let key_event = create_key_event(KeyCode::Enter);
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert!(!state.can_exit);
    assert!(state.input_prompt);
    assert!(state.settings.settings().backups_path.is_some());
}

#[test]
fn test_normal_mode_enter_toggle4() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.selected_toggle = SelectedToggle::Toggle4;
    state.can_exit = true;
    let initial_side = state.settings.settings().default_start_side;
    
    let key_event = create_key_event(KeyCode::Enter);
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert!(!state.can_exit);
    // Should have cycled the default start side
    assert_ne!(state.settings.settings().default_start_side, initial_side);
}

#[test]
fn test_normal_mode_enter_toggle5() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.selected_toggle = SelectedToggle::Toggle5;
    state.can_exit = true;
    let initial_side = state.settings.settings().default_end_side;
    
    let key_event = create_key_event(KeyCode::Enter);
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert!(!state.can_exit);
    // Should have cycled the default end side
    assert_ne!(state.settings.settings().default_end_side, initial_side);
}

#[test]
fn test_normal_mode_enter_toggle6() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.selected_toggle = SelectedToggle::Toggle6;
    state.can_exit = true;
    let initial_modal = state.settings.settings().edit_modal;
    
    let key_event = create_key_event(KeyCode::Enter);
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert!(!state.can_exit);
    // Should have toggled edit_modal
    assert_eq!(state.settings.settings().edit_modal, !initial_modal);
}

#[test]
fn test_normal_mode_tab_toggle2_with_backups() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.selected_toggle = SelectedToggle::Toggle2;
    state.can_exit = true;
    state.settings.settings_mut().backups_interval = Some(BackupsInterval::Daily);
    
    let key_event = create_key_event(KeyCode::Tab);
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert!(!state.can_exit);
    // Should have cycled backup interval
    assert_eq!(state.settings.settings().backups_interval, Some(BackupsInterval::Every3Days));
}

#[test]
fn test_normal_mode_tab_toggle2_without_backups() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.selected_toggle = SelectedToggle::Toggle2;
    state.can_exit = true;
    state.settings.settings_mut().backups_interval = None;
    
    let key_event = create_key_event(KeyCode::Tab);
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    // No changes made - so can exit without prompt.
    assert!(state.can_exit);
}

#[test]
fn test_normal_mode_tab_toggle3_with_runtime_backups() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.selected_toggle = SelectedToggle::Toggle3;
    state.can_exit = true;
    state.settings.settings_mut().runtime_backups_interval = Some(RuntimeBackupsInterval::Hourly);
    
    let key_event = create_key_event(KeyCode::Tab);
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert!(!state.can_exit);
    // Should have cycled runtime backup interval
    assert_eq!(state.settings.settings().runtime_backups_interval, Some(RuntimeBackupsInterval::Every2Hours));
}

#[test]
fn test_normal_mode_tab_other_toggle() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.selected_toggle = SelectedToggle::Toggle1;
    state.can_exit = true;
    
    let key_event = create_key_event(KeyCode::Tab);
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    // No changes made - so can exit without prompt.
    assert!(state.can_exit);
}

#[test]
fn test_normal_mode_other_key() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.needs_clear_and_redraw = false;
    
    let key_event = create_key_event(KeyCode::Char('x'));
    let result = settings_kh(&mut state, key_event, &mock_fs);
    
    assert_eq!(result, AppAction::Continue);
    assert!(state.needs_clear_and_redraw);
}

#[test]
fn test_settings_kh_always_sets_needs_redraw() {
    let mock_fs = MockFileSystem::new();
    let mut state = create_default_settings_state();
    state.needs_clear_and_redraw = false;
    
    // Test various keys to ensure they all set needs_clear_and_redraw
    let test_keys = vec![
        KeyCode::Char('j'), KeyCode::Char('k'), KeyCode::Down, KeyCode::Up,
        KeyCode::Enter, KeyCode::Tab, KeyCode::Char('?'), KeyCode::F(1),
        KeyCode::Char('z'), // Unknown key
    ];
    
    for key_code in test_keys {
        state.needs_clear_and_redraw = false;
        let key_event = create_key_event(key_code);
        let _result = settings_kh(&mut state, key_event, &mock_fs);
        
        assert!(state.needs_clear_and_redraw, "Key {:?} should set needs_clear_and_redraw", key_code);
    }
}

#[test]
fn test_cycle_save_intervals() {
    let mut settings = Settings::new();
    assert_eq!(settings.save_interval, Some(20));
    
    settings.cycle_save_intervals();
    assert_eq!(settings.save_interval, Some(30));
    
    settings.cycle_save_intervals();
    assert_eq!(settings.save_interval, Some(60));
    
    settings.cycle_save_intervals();
    assert_eq!(settings.save_interval, None);
    
    settings.cycle_save_intervals();
    assert_eq!(settings.save_interval, Some(10));
}

#[test]
fn test_cycle_default_sides() {
    let mut settings = Settings::new();
    assert_eq!(settings.default_start_side, Side::Right);
    assert_eq!(settings.default_end_side, Side::Right);
    
    settings.cycle_default_sides(true);  // start side
    assert_eq!(settings.default_start_side, Side::Bottom);
    
    settings.cycle_default_sides(false); // end side
    assert_eq!(settings.default_end_side, Side::Bottom);
    
    settings.cycle_default_sides(true);
    assert_eq!(settings.default_start_side, Side::Left);
    
    settings.cycle_default_sides(true);
    assert_eq!(settings.default_start_side, Side::Top);
    
    settings.cycle_default_sides(true);
    assert_eq!(settings.default_start_side, Side::Right); // Back to start
}