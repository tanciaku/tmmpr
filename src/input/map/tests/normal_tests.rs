use std::path::PathBuf;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;

use crate::{
    app::Screen, input::{AppAction, map::normal::map_normal_kh}, states::{MapState, map::{DiscardMenuType, Mode, Note}}, utils::test_utils::MockFileSystem
};

fn create_test_map_state() -> MapState {
    let mock_fs = MockFileSystem::new();
    let mut map_state = MapState::new_with_fs(PathBuf::from("/test/path"), &mock_fs);
    map_state.settings.edit_modal = false;
    map_state.viewport.screen_width = 100;
    map_state.viewport.screen_height = 50;
    map_state.persistence.mark_clean();
    map_state
}

fn create_key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn create_key_event_with_mods(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

// ==================== HELP SCREEN TESTS ====================

#[test]
fn test_toggle_help_screen_with_f1() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.ui_state.hide_help();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::F(1)));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.ui_state.help_screen, Some(1));
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_toggle_help_screen_with_question_mark() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.ui_state.hide_help();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('?')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.ui_state.help_screen, Some(1));
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_close_help_screen_with_f1() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.ui_state.show_help(1);

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::F(1)));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.ui_state.help_screen, None);
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_close_help_screen_with_question_mark() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.ui_state.show_help(1);

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('?')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.ui_state.help_screen, None);
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_close_help_screen_with_escape() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.ui_state.show_help(3);

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Esc));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.ui_state.help_screen, None);
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_help_next_page_with_right_arrow() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.ui_state.show_help(1);

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Right));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.ui_state.help_screen, Some(2));
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_help_next_page_with_l() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.ui_state.show_help(2);

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('l')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.ui_state.help_screen, Some(3));
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_help_next_page_with_tab() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.ui_state.show_help(4);

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Tab));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.ui_state.help_screen, Some(5));
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_help_next_page_wraps_to_first() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.ui_state.show_help(5);

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Right));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.ui_state.help_screen, Some(1));
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_help_previous_page_with_left_arrow() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.ui_state.show_help(2);

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Left));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.ui_state.help_screen, Some(1));
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_help_previous_page_with_h() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.ui_state.show_help(3);

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('h')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.ui_state.help_screen, Some(2));
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_help_previous_page_wraps_to_last() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.ui_state.show_help(1);

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Left));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.ui_state.help_screen, Some(5));
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_help_screen_blocks_other_input() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.ui_state.show_help(1);
    map_state.viewport.view_pos.x = 10;
    map_state.viewport.view_pos.y = 10;

    // Try to move viewport - should be blocked by help screen
    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('j')));

    assert_eq!(result, AppAction::Continue);
    // Viewport should not have moved
    assert_eq!(map_state.viewport.view_pos.x, 10);
    assert_eq!(map_state.viewport.view_pos.y, 10);
    // Still on help screen
    assert_eq!(map_state.ui_state.help_screen, Some(1));
}

// ==================== DISCARD MENU TESTS ====================

#[test]
fn test_discard_menu_cancel_with_escape() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.ui_state.show_discard_menu(DiscardMenuType::Start);

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Esc));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.ui_state.confirm_discard_menu, None);
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_discard_menu_confirm_to_start_screen() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.ui_state.show_discard_menu(DiscardMenuType::Start);

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('q')));

    match result {
        AppAction::Switch(Screen::Start(_)) => {
            // Success
        }
        _ => panic!("Expected AppAction::Switch to Start screen"),
    }
}

#[test]
fn test_discard_menu_confirm_to_settings_screen() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.persistence.file_write_path = PathBuf::from("/test/map.json");
    map_state.ui_state.show_discard_menu(DiscardMenuType::Settings);

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('q')));

    match result {
        AppAction::Switch(Screen::Settings(_)) => {
            // Success
        }
        _ => panic!("Expected AppAction::Switch to Settings screen"),
    }
}

#[test]
fn test_discard_menu_blocks_other_input() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.ui_state.show_discard_menu(DiscardMenuType::Start);
    map_state.viewport.view_pos.x = 10;
    map_state.viewport.view_pos.y = 10;

    // Try to move viewport - should be blocked
    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('j')));

    assert_eq!(result, AppAction::Continue);
    // Viewport should not have moved
    assert_eq!(map_state.viewport.view_pos.x, 10);
    assert_eq!(map_state.viewport.view_pos.y, 10);
    // Still showing discard menu
    assert_eq!(map_state.ui_state.confirm_discard_menu, Some(DiscardMenuType::Start));
}

// ==================== EXIT/QUIT TESTS ====================

#[test]
fn test_quit_when_can_exit_is_true() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('q')));

    match result {
        AppAction::Switch(Screen::Start(_)) => {
            // Success
        }
        _ => panic!("Expected AppAction::Switch to Start screen"),
    }
}

#[test]
fn test_quit_when_can_exit_is_false_shows_discard_menu() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.persistence.mark_dirty();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('q')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.ui_state.confirm_discard_menu, Some(DiscardMenuType::Start));
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

// ==================== SAVE TESTS ====================

#[test]
fn test_save_map_file() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.persistence.file_write_path = PathBuf::from("/test/my_map.json");

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('s')));

    match result {
        AppAction::SaveMapFile(path) => {
            assert_eq!(path, PathBuf::from("/test/my_map.json"));
        }
        _ => panic!("Expected AppAction::SaveMapFile"),
    }
}

// ==================== SETTINGS TESTS ====================

#[test]
fn test_open_settings_when_can_exit_is_true() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.persistence.mark_clean();
    map_state.persistence.file_write_path = PathBuf::from("/test/map.json");

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('o')));

    match result {
        AppAction::Switch(Screen::Settings(_)) => {
            // Success
        }
        _ => panic!("Expected AppAction::Switch to Settings screen"),
    }
}

#[test]
fn test_open_settings_when_can_exit_is_false_shows_discard_menu() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.persistence.mark_dirty();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('o')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.ui_state.confirm_discard_menu, Some(DiscardMenuType::Settings));
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

// ==================== VIEWPORT NAVIGATION TESTS ====================

#[test]
fn test_move_viewport_left_with_h() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.x = 10;
    map_state.viewport.view_pos.y = 10;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('h')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.x, 9);
    assert_eq!(map_state.viewport.view_pos.y, 10);
    assert_eq!(map_state.persistence.can_exit, false);
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_move_viewport_left_with_arrow() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.x = 10;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Left));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.x, 9);
    assert_eq!(map_state.persistence.can_exit, false);
}

#[test]
fn test_move_viewport_left_saturates_at_zero() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.x = 0;

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('h')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.x, 0); // Should not underflow
}

#[test]
fn test_move_viewport_left_by_5_with_shift_h() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.x = 20;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('H')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.x, 15);
    assert_eq!(map_state.persistence.can_exit, false);
}

#[test]
fn test_move_viewport_left_by_5_with_shift_arrow() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.x = 20;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event_with_mods(KeyCode::Left, KeyModifiers::SHIFT));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.x, 15);
    assert_eq!(map_state.persistence.can_exit, false);
}

#[test]
fn test_move_viewport_down_with_j() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.x = 10;
    map_state.viewport.view_pos.y = 10;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('j')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.x, 10);
    assert_eq!(map_state.viewport.view_pos.y, 11);
    assert_eq!(map_state.persistence.can_exit, false);
}

#[test]
fn test_move_viewport_down_with_arrow() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.y = 10;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Down));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.y, 11);
    assert_eq!(map_state.persistence.can_exit, false);
}

#[test]
fn test_move_viewport_down_by_5_with_shift_j() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.y = 20;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('J')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.y, 25);
    assert_eq!(map_state.persistence.can_exit, false);
}

#[test]
fn test_move_viewport_down_by_5_with_shift_arrow() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.y = 20;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event_with_mods(KeyCode::Down, KeyModifiers::SHIFT));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.y, 25);
    assert_eq!(map_state.persistence.can_exit, false);
}

#[test]
fn test_move_viewport_up_with_k() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.x = 10;
    map_state.viewport.view_pos.y = 10;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('k')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.x, 10);
    assert_eq!(map_state.viewport.view_pos.y, 9);
    assert_eq!(map_state.persistence.can_exit, false);
}

#[test]
fn test_move_viewport_up_with_arrow() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.y = 10;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Up));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.y, 9);
    assert_eq!(map_state.persistence.can_exit, false);
}

#[test]
fn test_move_viewport_up_saturates_at_zero() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.y = 0;

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('k')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.y, 0); // Should not underflow
}

#[test]
fn test_move_viewport_up_by_5_with_shift_k() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.y = 20;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('K')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.y, 15);
    assert_eq!(map_state.persistence.can_exit, false);
}

#[test]
fn test_move_viewport_up_by_5_with_shift_arrow() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.y = 20;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event_with_mods(KeyCode::Up, KeyModifiers::SHIFT));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.y, 15);
    assert_eq!(map_state.persistence.can_exit, false);
}

#[test]
fn test_move_viewport_right_with_l() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.x = 10;
    map_state.viewport.view_pos.y = 10;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('l')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.x, 11);
    assert_eq!(map_state.viewport.view_pos.y, 10);
    assert_eq!(map_state.persistence.can_exit, false);
}

#[test]
fn test_move_viewport_right_with_arrow() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.x = 10;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Right));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.x, 11);
    assert_eq!(map_state.persistence.can_exit, false);
}

#[test]
fn test_move_viewport_right_by_5_with_shift_l() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.x = 20;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('L')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.x, 25);
    assert_eq!(map_state.persistence.can_exit, false);
}

#[test]
fn test_move_viewport_right_by_5_with_shift_arrow() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.x = 20;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event_with_mods(KeyCode::Right, KeyModifiers::SHIFT));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.viewport.view_pos.x, 25);
    assert_eq!(map_state.persistence.can_exit, false);
}

// ==================== NOTE MANIPULATION TESTS ====================

#[test]
fn test_add_note() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.x = 100;
    map_state.viewport.view_pos.y = 50;
    map_state.viewport.screen_width = 80;
    map_state.viewport.screen_height = 40;
    map_state.persistence.mark_clean();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('a')));

    assert_eq!(result, AppAction::Continue);
    
    // Check that a note was added
    assert_eq!(map_state.notes_state.notes.len(), 1);
    assert!(map_state.notes_state.notes.contains_key(&0));
    
    // Check note is at center of screen
    let note = &map_state.notes_state.notes[&0];
    assert_eq!(note.x, 100 + 80/2); // view_pos.x + screen_width/2 = 140
    assert_eq!(note.y, 50 + 40/2);  // view_pos.y + screen_height/2 = 70
    assert_eq!(note.content, "");
    assert_eq!(note.selected, true);
    assert_eq!(note.color, Color::White);
    
    // Check render order
    assert_eq!(map_state.notes_state.render_order, vec![0]);
    
    // Check selected note
    assert_eq!(map_state.notes_state.selected_note, Some(0));
    
    // Check mode switched to Edit
    assert!(matches!(map_state.current_mode, Mode::Edit(_)));
    
    // Check can_exit is false
    assert_eq!(map_state.persistence.can_exit, false);
    
    // Check next_note_id incremented
    assert_eq!(map_state.notes_state.next_note_id, 1);
}

#[test]
fn test_add_multiple_notes() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;

    // Add first note
    let _result1 = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('a')));
    map_state.current_mode = Mode::Normal; // Reset mode
    
    // Add second note
    let _result2 = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('a')));
    map_state.current_mode = Mode::Normal; // Reset mode
    
    // Add third note
    let _result3 = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('a')));

    assert_eq!(map_state.notes_state.notes.len(), 3);
    assert!(map_state.notes_state.notes.contains_key(&0));
    assert!(map_state.notes_state.notes.contains_key(&1));
    assert!(map_state.notes_state.notes.contains_key(&2));
    assert_eq!(map_state.notes_state.render_order, vec![0, 1, 2]);
    assert_eq!(map_state.notes_state.selected_note, Some(2)); // Last added note is selected
    assert_eq!(map_state.notes_state.next_note_id, 3);
}

#[test]
fn test_select_note_with_no_notes() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('v')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.selected_note, None);
    assert_eq!(map_state.current_mode, Mode::Normal);
}

#[test]
fn test_select_note_with_single_note() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    
    // Add a note
    map_state.notes_state.notes.insert(0, Note::new(50, 25, String::from("Test"), false, Color::White));
    map_state.notes_state.render_order.push(0);
    map_state.viewport.view_pos.x = 0;
    map_state.viewport.view_pos.y = 0;

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('v')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.selected_note, Some(0));
    assert_eq!(map_state.current_mode, Mode::Visual);
    assert_eq!(map_state.notes_state.render_order, vec![0]); // Should be moved to back
}

#[test]
fn test_select_closest_note_to_center() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.viewport.view_pos.x = 0;
    map_state.viewport.view_pos.y = 0;
    map_state.viewport.screen_width = 100;
    map_state.viewport.screen_height = 50;
    
    // Screen center is at (50, 25)
    // Add three notes at different distances from center
    map_state.notes_state.notes.insert(0, Note::new(10, 10, String::from("Far"), false, Color::White));     // Distance: 40 + 15 = 55
    map_state.notes_state.notes.insert(1, Note::new(45, 20, String::from("Close"), false, Color::White));   // Distance: 5 + 5 = 10
    map_state.notes_state.notes.insert(2, Note::new(80, 40, String::from("Medium"), false, Color::White));  // Distance: 30 + 15 = 45
    map_state.notes_state.render_order = vec![0, 1, 2];

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('v')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.selected_note, Some(1)); // Note 1 is closest
    assert_eq!(map_state.current_mode, Mode::Visual);
    assert_eq!(map_state.notes_state.render_order, vec![0, 2, 1]); // Note 1 moved to back
}

#[test]
fn test_select_note_updates_render_order() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    
    // Add multiple notes
    map_state.notes_state.notes.insert(0, Note::new(10, 10, String::from("Note 0"), false, Color::White));
    map_state.notes_state.notes.insert(1, Note::new(50, 25, String::from("Note 1"), false, Color::White));
    map_state.notes_state.notes.insert(2, Note::new(80, 40, String::from("Note 2"), false, Color::White));
    map_state.notes_state.render_order = vec![0, 1, 2];
    
    // Set viewport so note 0 is closest to center
    map_state.viewport.view_pos.x = 0;
    map_state.viewport.view_pos.y = 0;
    map_state.viewport.screen_width = 20;
    map_state.viewport.screen_height = 20;

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('v')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.selected_note, Some(0));
    // Note 0 should be moved to the back of render order
    assert_eq!(map_state.notes_state.render_order, vec![1, 2, 0]);
}

// ==================== MISC TESTS ====================

#[test]
fn test_unhandled_keys_still_trigger_redraw() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;
    map_state.ui_state.mark_redrawn();

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('z')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_normal_mode_continues() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;

    let result = map_normal_kh(&mut map_state, create_key_event(KeyCode::Char('h')));

    assert_eq!(result, AppAction::Continue);
}
