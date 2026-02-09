use std::path::PathBuf;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;

use crate::{
    input::{AppAction, map::edit::map_edit_kh},
    states::{MapState, map::{ModalEditMode, Mode}}, utils::test_utils::MockFileSystem,
};

fn create_test_map_state() -> MapState {
    let mock_fs = MockFileSystem::new();
    let mut map_state = MapState::new_with_fs(PathBuf::from("/test/path"), &mock_fs);
    map_state.settings.edit_modal = false;
    map_state.viewport.screen_width = 100;
    map_state.viewport.screen_height = 50;
    map_state.persistence.mark_clean(); // Start with can_exit as true to test it gets set to false
    map_state
}

fn create_key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn test_non_modal_escape_to_normal() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(None);
    map_state.notes_state.set_cursor_pos(5);

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Esc), None);

    assert_eq!(result, AppAction::Continue);
    assert!(map_state.notes_state.selected_note_id().is_none());
    assert_eq!(map_state.current_mode, Mode::Normal);
    assert_eq!(map_state.notes_state.cursor_pos(), 0); // Should reset cursor position
}

#[test]
fn test_modal_insert_escape_to_modal_normal() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Insert));
    map_state.notes_state.set_cursor_pos(5);

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Esc), Some(ModalEditMode::Insert));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.current_mode, Mode::Edit(Some(ModalEditMode::Normal)));
    assert_eq!(map_state.notes_state.cursor_pos(), 4); // Should move cursor back by 1
}

#[test]
fn test_modal_normal_escape_to_normal() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));
    map_state.notes_state.set_cursor_pos(5);

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Esc), Some(ModalEditMode::Normal));

    assert_eq!(result, AppAction::Continue);
    assert!(map_state.notes_state.selected_note_id().is_none());
    assert_eq!(map_state.current_mode, Mode::Normal);
    assert_eq!(map_state.notes_state.cursor_pos(), 0); // Should reset cursor position    
}

#[test]
fn test_insert_char_non_modal() {
    let mut map_state = create_test_map_state();
    
    // Add a note with existing content
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(None);
    map_state.notes_state.set_cursor_pos(2); // Position between 'e' and 'l'

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('X')), None);

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.persistence.has_unsaved_changes, true); // Should be set to true for modifications
    assert_eq!(map_state.notes_state.cursor_pos(), 3); // Should advance cursor
    
    if let Some(note) = map_state.notes_state.notes().get(&0) {
        assert_eq!(note.content, "HeXllo");
    }
}

#[test]
fn test_insert_enter_character() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(None);
    map_state.notes_state.set_cursor_pos(2);

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Enter), None);

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.persistence.has_unsaved_changes, true); // Should be set to true
    assert_eq!(map_state.notes_state.cursor_pos(), 3); // Should advance cursor
    
    if let Some(note) = map_state.notes_state.notes().get(&0) {
        assert_eq!(note.content, "He\nllo");
    }
}

#[test]
fn test_backspace_char() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(None);
    map_state.notes_state.set_cursor_pos(3); // Position after 'l'

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Backspace), None);

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.persistence.has_unsaved_changes, true); // Should be set to true
    assert_eq!(map_state.notes_state.cursor_pos(), 2); // Should move cursor back
    
    if let Some(note) = map_state.notes_state.notes().get(&0) {
        assert_eq!(note.content, "Helo");
    }
}

#[test]
fn test_backspace_at_beginning() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(None);
    map_state.notes_state.set_cursor_pos(0); // At the beginning

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Backspace), None);

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.persistence.has_unsaved_changes, true); // Should still be set to true
    assert_eq!(map_state.notes_state.cursor_pos(), 0); // Should stay at beginning
    
    if let Some(note) = map_state.notes_state.notes().get(&0) {
        assert_eq!(note.content, "Hello"); // Should be unchanged
    }
}

#[test]
fn test_cursor_left_movement() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(None);
    map_state.notes_state.set_cursor_pos(3);

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Left), None);

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.cursor_pos(), 2);
}

#[test]
fn test_cursor_left_at_beginning() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(None);
    map_state.notes_state.set_cursor_pos(0);

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Left), None);

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.cursor_pos(), 0); // Should stay at 0
}

#[test]
fn test_cursor_right_movement() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(None);
    map_state.notes_state.set_cursor_pos(2);

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Right), None);

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.cursor_pos(), 3);
}

#[test]
fn test_cursor_right_at_end() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(None);
    map_state.notes_state.set_cursor_pos(5); // At the end

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Right), None);

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.cursor_pos(), 5); // Should stay at end
}

#[test]
fn test_modal_normal_insert_mode_switch() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('i')), Some(ModalEditMode::Normal));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.current_mode, Mode::Edit(Some(ModalEditMode::Insert)));
}

#[test]
fn test_modal_normal_hjkl_movement() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello\nWorld"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));
    map_state.notes_state.set_cursor_pos(7); // Position at 'o' in "World"

    // Test 'h' (left)
    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('h')), Some(ModalEditMode::Normal));
    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.cursor_pos(), 6);

    // Test 'l' (right)
    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('l')), Some(ModalEditMode::Normal));
    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.cursor_pos(), 7);

    // Test 'k' (up) - this will call move_cursor_up function
    map_state.notes_state.set_cursor_pos(7); // Reset to 'o' in "World"
    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('k')), Some(ModalEditMode::Normal));
    assert_eq!(result, AppAction::Continue);
    // Cursor should move to equivalent position in "Hello"
    assert_eq!(map_state.notes_state.cursor_pos(), 1);

    // Test 'j' (down) - this will call move_cursor_down function
    map_state.notes_state.set_cursor_pos(2); // Position in "Hello"
    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('j')), Some(ModalEditMode::Normal));
    assert_eq!(result, AppAction::Continue);
    // Cursor should move to equivalent position in "World"
    assert_eq!(map_state.notes_state.cursor_pos(), 8)
}

#[test]
fn test_modal_normal_h_at_beginning() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));
    map_state.notes_state.set_cursor_pos(0);

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('h')), Some(ModalEditMode::Normal));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.cursor_pos(), 0); // Should stay at 0
}

#[test]
fn test_modal_normal_l_at_end() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));
    map_state.notes_state.set_cursor_pos(4); // At the last character (note content.len() - 1)

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('l')), Some(ModalEditMode::Normal));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.cursor_pos(), 4); // Should stay at last character
}

#[test]
fn test_modal_normal_g_beginning() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello World"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));
    map_state.notes_state.set_cursor_pos(5);

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('g')), Some(ModalEditMode::Normal));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.cursor_pos(), 0);
}

#[test]
fn test_modal_normal_g_end() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello World"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));
    map_state.notes_state.set_cursor_pos(3);

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('G')), Some(ModalEditMode::Normal));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.cursor_pos(), 10); // Last character position (content.len() - 1)
}

#[test]
fn test_modal_normal_append_mode() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));
    map_state.notes_state.set_cursor_pos(2);

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('a')), Some(ModalEditMode::Normal));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.cursor_pos(), 3); // Should advance cursor by 1
    assert_eq!(map_state.current_mode, Mode::Edit(Some(ModalEditMode::Insert)));
}

#[test]
fn test_modal_normal_append_at_end() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));
    map_state.notes_state.set_cursor_pos(4); // At the last character

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('a')), Some(ModalEditMode::Normal));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.cursor_pos(), 5); // Should move to after the last character
    assert_eq!(map_state.current_mode, Mode::Edit(Some(ModalEditMode::Insert)));
}

#[test]
fn test_modal_normal_remove_char() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));
    map_state.notes_state.set_cursor_pos(2); // Position at 'l'

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('x')), Some(ModalEditMode::Normal));

    assert_eq!(result, AppAction::Continue);
    
    if let Some(note) = map_state.notes_state.notes().get(&0) {
        assert_eq!(note.content, "Helo"); // Should remove the 'l' at position 2
    }
}

#[test]
fn test_modal_normal_word_jump_forward() {
    let mut map_state = create_test_map_state();
    
    // Add a note with multiple words
    map_state.notes_state.add(50, 25, String::from("Hello World Test"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));
    map_state.notes_state.set_cursor_pos(0); // At the beginning

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('w')), Some(ModalEditMode::Normal));

    assert_eq!(result, AppAction::Continue);
    // Cursor should jump to the beginning of "World" (position 6)
    assert_eq!(map_state.notes_state.cursor_pos(), 6);
}

#[test]
fn test_modal_normal_word_jump_backward() {
    let mut map_state = create_test_map_state();
    
    // Add a note with multiple words
    map_state.notes_state.add(50, 25, String::from("Hello World Test"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));
    map_state.notes_state.set_cursor_pos(12); // Position at 'T' in "Test"

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('b')), Some(ModalEditMode::Normal));

    assert_eq!(result, AppAction::Continue);
    // Cursor should jump to the beginning of "World" (position 6)
    assert_eq!(map_state.notes_state.cursor_pos(), 6);
}

#[test]
fn test_always_triggers_clear_and_redraw() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(None);
    map_state.ui_state.mark_redrawn();

    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('a')), None);

    assert_eq!(result, AppAction::Continue);
    assert!(map_state.ui_state.needs_clear_and_redraw); // Should be set to true by clear_and_redraw()
}

#[test]
fn test_always_returns_continue() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(None);

    // Test various keys - all should return Continue
    let test_keys = vec![
        KeyCode::Char('a'),
        KeyCode::Enter,
        KeyCode::Backspace,
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Esc,
    ];

    for key in test_keys {
        let result = map_edit_kh(&mut map_state, create_key_event(key), None);
        assert_eq!(result, AppAction::Continue);
    }
}

#[test]
fn test_unhandled_keys_ignored() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(None);
    map_state.notes_state.set_cursor_pos(2);

    // Test unhandled keys
    let test_keys = vec![
        KeyCode::Tab,
        KeyCode::Delete,
        KeyCode::PageUp,
        KeyCode::PageDown,
        KeyCode::Home,
        KeyCode::End,
    ];

    for key in test_keys {
        let original_cursor = map_state.notes_state.cursor_pos();
        let original_content = map_state.notes_state.notes().get(&0).unwrap().content.clone();
        
        let result = map_edit_kh(&mut map_state, create_key_event(key), None);
        
        assert_eq!(result, AppAction::Continue);
        assert_eq!(map_state.notes_state.cursor_pos(), original_cursor); // Should not change cursor
        
        if let Some(note) = map_state.notes_state.notes().get(&0) {
            assert_eq!(note.content, original_content); // Should not change content
        }
    }
}

#[test]
fn test_modal_normal_unhandled_keys_ignored() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.add(50, 25, String::from("Hello"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));
    map_state.notes_state.set_cursor_pos(2);

    // Test unhandled keys in modal normal mode
    let test_keys = vec![
        KeyCode::Char('z'),
        KeyCode::Char('c'),
        KeyCode::Char('d'),
        KeyCode::Char('y'),
        KeyCode::Tab,
        KeyCode::Delete,
    ];

    for key in test_keys {
        let original_cursor = map_state.notes_state.cursor_pos();
        let original_content = map_state.notes_state.notes().get(&0).unwrap().content.clone();
        let original_mode = map_state.current_mode;
        
        let result = map_edit_kh(&mut map_state, create_key_event(key), Some(ModalEditMode::Normal));
        
        assert_eq!(result, AppAction::Continue);
        assert_eq!(map_state.notes_state.cursor_pos(), original_cursor); // Should not change cursor
        assert_eq!(map_state.current_mode, original_mode); // Should not change mode
        
        if let Some(note) = map_state.notes_state.notes().get(&0) {
            assert_eq!(note.content, original_content); // Should not change content
        }
    }
}

#[test]
fn test_empty_note_content_handling() {
    let mut map_state = create_test_map_state();
    
    // Add a note with empty content
    map_state.notes_state.add(50, 25, String::new(), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(None);
    map_state.notes_state.set_cursor_pos(0);

    // Test inserting character into empty note
    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('H')), None);

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.cursor_pos(), 1);
    
    if let Some(note) = map_state.notes_state.notes().get(&0) {
        assert_eq!(note.content, "H");
    }
}

#[test]
fn test_cursor_bounds_edge_cases() {
    let mut map_state = create_test_map_state();
    
    // Add a note with single character
    map_state.notes_state.add(50, 25, String::from("A"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));
    map_state.notes_state.set_cursor_pos(0); // At the only character

    // Test 'G' (go to end) on single character
    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('G')), Some(ModalEditMode::Normal));
    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.cursor_pos(), 0); // Should stay at 0 for single character (content.len() - 1 = 0)

    // Test 'l' (right) at the only character position
    let result = map_edit_kh(&mut map_state, create_key_event(KeyCode::Char('l')), Some(ModalEditMode::Normal));
    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.cursor_pos(), 0); // Should stay at 0
}