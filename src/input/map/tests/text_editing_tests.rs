use std::path::PathBuf;
use ratatui::style::Color;

use crate::{
    input::map::text_editing::{backspace_char, remove_char, insert_char, move_cursor_up, move_cursor_down},
    states::{MapState, map::Note},
};

fn create_test_map_state() -> MapState {
    let mut map_state = MapState::new(PathBuf::from("/test/path"));
    map_state.settings.edit_modal = false;
    map_state.viewport.screen_width = 100;
    map_state.viewport.screen_height = 50;
    map_state.can_exit = true;
    map_state
}

// ============================================================================
// Tests for backspace_char
// ============================================================================

#[test]
fn test_backspace_at_beginning() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 0; // Cursor at beginning
    
    backspace_char(&mut map_state, 0);
    
    // Should not change anything when cursor is at position 0
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Hello");
    assert_eq!(map_state.notes_state.cursor_pos, 0);
}

#[test]
fn test_backspace_in_middle() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 3; // Cursor after "Hel"
    
    backspace_char(&mut map_state, 0);
    
    // Should remove the 'l' before the cursor
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Helo");
    assert_eq!(map_state.notes_state.cursor_pos, 2);
}

#[test]
fn test_backspace_at_end() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 5; // Cursor at end
    
    backspace_char(&mut map_state, 0);
    
    // Should remove the last character
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Hell");
    assert_eq!(map_state.notes_state.cursor_pos, 4);
}

#[test]
fn test_backspace_single_character() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("A"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 1;
    
    backspace_char(&mut map_state, 0);
    
    // Should result in empty string
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "");
    assert_eq!(map_state.notes_state.cursor_pos, 0);
}

#[test]
fn test_backspace_with_newlines() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello\nWorld"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 6; // Cursor at start of "World"
    
    backspace_char(&mut map_state, 0);
    
    // Should remove the newline character
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "HelloWorld");
    assert_eq!(map_state.notes_state.cursor_pos, 5);
}

#[test]
fn test_backspace_unicode() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello 世界"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 7; // After "Hello 世"
    
    backspace_char(&mut map_state, 0);
    
    // Should remove the Chinese character '世'
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Hello 界");
    assert_eq!(map_state.notes_state.cursor_pos, 6);
}

#[test]
fn test_backspace_nonexistent_note() {
    let mut map_state = create_test_map_state();
    map_state.notes_state.cursor_pos = 5;
    
    // Try to backspace on note that doesn't exist
    backspace_char(&mut map_state, 999);
    
    // Should not panic, just do nothing
    assert_eq!(map_state.notes_state.cursor_pos, 5);
}

#[test]
fn test_backspace_multiple_times() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 5;
    
    backspace_char(&mut map_state, 0);
    backspace_char(&mut map_state, 0);
    backspace_char(&mut map_state, 0);
    
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "He");
    assert_eq!(map_state.notes_state.cursor_pos, 2);
}

// ============================================================================
// Tests for remove_char
// ============================================================================

#[test]
fn test_remove_at_end() {
    // Cannot happen in Normal Mode, cannot move cursor past last character
    // Last pos here would be 4

    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 5; // Cursor at end
    
    remove_char(&mut map_state, 0);
    
    // Should not remove anything when cursor is at end
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Hello");
    assert_eq!(map_state.notes_state.cursor_pos, 5);
}

#[test]
fn test_remove_in_middle() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 2; // Cursor at 'l'
    
    remove_char(&mut map_state, 0);
    
    // Should remove the 'l' at the cursor
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Helo");
    assert_eq!(map_state.notes_state.cursor_pos, 2); // Cursor stays in place
}

#[test]
fn test_remove_at_beginning() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 0; // Cursor at beginning
    
    remove_char(&mut map_state, 0);
    
    // Should remove the first character
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "ello");
    assert_eq!(map_state.notes_state.cursor_pos, 0);
}

#[test]
fn test_remove_empty_content() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from(""), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 0;
    
    remove_char(&mut map_state, 0);
    
    // Should do nothing on empty content
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "");
    assert_eq!(map_state.notes_state.cursor_pos, 0);
}

#[test]
fn test_remove_single_character() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("A"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 0;
    
    remove_char(&mut map_state, 0);
    
    // Should result in empty string
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "");
    assert_eq!(map_state.notes_state.cursor_pos, 0);
}

#[test]
fn test_remove_with_newlines() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello\nWorld"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 5; // Cursor at newline
    
    remove_char(&mut map_state, 0);
    
    // Should remove the newline character
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "HelloWorld");
    assert_eq!(map_state.notes_state.cursor_pos, 5);
}

#[test]
fn test_remove_unicode() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello 世界"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 6; // At '世'
    
    remove_char(&mut map_state, 0);
    
    // Should remove the Chinese character '世'
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Hello 界");
    assert_eq!(map_state.notes_state.cursor_pos, 6);
}

#[test]
fn test_remove_cursor_adjustment() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("AB"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 1; // At 'B'
    
    remove_char(&mut map_state, 0);
    
    // Should remove 'B' and adjust cursor
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "A");
    assert_eq!(map_state.notes_state.cursor_pos, 0); // Cursor moved back
}

#[test]
fn test_remove_nonexistent_note() {
    // Shouldn't be possible

    let mut map_state = create_test_map_state();
    map_state.notes_state.cursor_pos = 5;
    
    // Try to remove on note that doesn't exist
    remove_char(&mut map_state, 999);
    
    // Should not panic, just do nothing
    assert_eq!(map_state.notes_state.cursor_pos, 5);
}

#[test]
fn test_remove_multiple_times() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 0;
    
    remove_char(&mut map_state, 0); // Remove 'H'
    remove_char(&mut map_state, 0); // Remove 'e'
    remove_char(&mut map_state, 0); // Remove 'l'
    
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "lo");
    assert_eq!(map_state.notes_state.cursor_pos, 0);
}

// ============================================================================
// Tests for insert_char
// ============================================================================

#[test]
fn test_insert_at_beginning() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 0;
    
    insert_char(&mut map_state, 0, 'X');
    
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "XHello");
    assert_eq!(map_state.notes_state.cursor_pos, 1);
    assert_eq!(map_state.can_exit, false); // Should set can_exit to false
}

#[test]
fn test_insert_in_middle() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 2; // After "He"
    
    insert_char(&mut map_state, 0, 'X');
    
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "HeXllo");
    assert_eq!(map_state.notes_state.cursor_pos, 3);
}

#[test]
fn test_insert_at_end() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 5;
    
    insert_char(&mut map_state, 0, '!');
    
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Hello!");
    assert_eq!(map_state.notes_state.cursor_pos, 6);
}

#[test]
fn test_insert_into_empty() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from(""), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 0;
    
    insert_char(&mut map_state, 0, 'A');
    
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "A");
    assert_eq!(map_state.notes_state.cursor_pos, 1);
}

#[test]
fn test_insert_newline() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 5;
    
    insert_char(&mut map_state, 0, '\n');
    
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Hello\n");
    assert_eq!(map_state.notes_state.cursor_pos, 6);
}

#[test]
fn test_insert_unicode() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello "), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 6;
    
    insert_char(&mut map_state, 0, '世');
    
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Hello 世");
    assert_eq!(map_state.notes_state.cursor_pos, 7);
}

#[test]
fn test_insert_special_characters() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Test"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 4;
    
    insert_char(&mut map_state, 0, ' ');
    insert_char(&mut map_state, 0, '!');
    insert_char(&mut map_state, 0, '@');
    insert_char(&mut map_state, 0, '#');
    
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Test !@#");
    assert_eq!(map_state.notes_state.cursor_pos, 8);
}

#[test]
fn test_insert_nonexistent_note() {
    // Shouldn't happen

    let mut map_state = create_test_map_state();
    map_state.notes_state.cursor_pos = 0;
    
    // Try to insert on note that doesn't exist
    insert_char(&mut map_state, 999, 'X');
    
    // Should not panic, just do nothing
    assert_eq!(map_state.notes_state.cursor_pos, 0);
    assert_eq!(map_state.can_exit, false); // Still sets can_exit to false
}

#[test]
fn test_insert_can_exit_flag() {
    let mut map_state = create_test_map_state();
    map_state.can_exit = true; // Start with true
    
    let note = Note::new(10, 10, String::from("Test"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 0;
    
    insert_char(&mut map_state, 0, 'X');
    
    // Should always set can_exit to false on edit
    assert_eq!(map_state.can_exit, false);
}

#[test]
fn test_insert_multiple_chars() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from(""), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 0;
    
    let word = "Hello";
    for ch in word.chars() {
        insert_char(&mut map_state, 0, ch);
    }
    
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Hello");
    assert_eq!(map_state.notes_state.cursor_pos, 5);
}

// ============================================================================
// Tests for move_cursor_up
// ============================================================================

#[test]
fn test_move_up_on_first_line() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello\nWorld"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 3; // On first line
    
    move_cursor_up(&mut map_state);
    
    // Should not move when already on first line
    assert_eq!(map_state.notes_state.cursor_pos, 3);
}

#[test]
fn test_move_up_to_same_column() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello\nWorld"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 8; // On 'r' in "World"
    
    move_cursor_up(&mut map_state);
    
    // Should move to same column on previous line (position 2 - 'l' in "Hello")
    assert_eq!(map_state.notes_state.cursor_pos, 2);
}

#[test]
fn test_move_up_snap_to_shorter_line() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hi\nLonger line"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 10; // On 'r' in "Longer"
    
    move_cursor_up(&mut map_state);
    
    // Should snap to end of shorter previous line (position 2 - end of "Hi")
    assert_eq!(map_state.notes_state.cursor_pos, 2);
}

#[test]
fn test_move_up_from_third_line() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("First\nSecond\nThird"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 14; // On 'h' in "Third"
    
    move_cursor_up(&mut map_state);
    
    // Should move to second line, position 7 ('e' in "Second")
    assert_eq!(map_state.notes_state.cursor_pos, 7);
}

#[test]
fn test_move_up_at_line_start() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello\nWorld"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 6; // At start of "World"
    
    move_cursor_up(&mut map_state);
    
    // Should move to start of previous line
    assert_eq!(map_state.notes_state.cursor_pos, 0);
}

#[test]
fn test_move_up_at_line_end() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello\nWorld"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 11; // At end of "World"
    
    move_cursor_up(&mut map_state);
    
    // Should move to end of previous line
    assert_eq!(map_state.notes_state.cursor_pos, 5);
}

#[test]
fn test_move_up_no_selected_note() {
    // Shouldn't happen

    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello\nWorld"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = None; // No selected note
    map_state.notes_state.cursor_pos = 8;
    
    move_cursor_up(&mut map_state);
    
    // Should do nothing when no note is selected
    assert_eq!(map_state.notes_state.cursor_pos, 8);
}

#[test]
fn test_move_up_empty_lines() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello\n\nWorld"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 7; // At start of "World"
    
    move_cursor_up(&mut map_state);
    
    // Should move to the empty line (position 6)
    assert_eq!(map_state.notes_state.cursor_pos, 6);
}

#[test]
fn test_move_up_with_multiple_empty_lines() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("A\n\n\nB"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 4; // At 'B'
    
    move_cursor_up(&mut map_state);
    
    // Should move to previous empty line
    assert_eq!(map_state.notes_state.cursor_pos, 3);
}

#[test]
fn test_move_up_single_line() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 3;
    
    move_cursor_up(&mut map_state);
    
    // Should not move on single line
    assert_eq!(map_state.notes_state.cursor_pos, 3);
}

#[test]
fn test_move_up_varying_line_lengths() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Short\nThis is a longer line\nMed"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 30; // On 'd' in "Med"
    
    move_cursor_up(&mut map_state);
    
    // Should move to same column on longer line (position 8 - 'i' in "This")
    assert_eq!(map_state.notes_state.cursor_pos, 8);
}

// ============================================================================
// Tests for move_cursor_down
// ============================================================================

#[test]
fn test_move_down_on_last_line() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello\nWorld"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 8; // On second line
    
    move_cursor_down(&mut map_state);
    
    // Should not move when already on last line
    assert_eq!(map_state.notes_state.cursor_pos, 8);
}

#[test]
fn test_move_down_to_same_column() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello\nWorld"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 2; // On 'l' in "Hello"
    
    move_cursor_down(&mut map_state);
    
    // Should move to same column on next line (position 8 - 'r' in "World")
    assert_eq!(map_state.notes_state.cursor_pos, 8);
}

#[test]
fn test_move_down_snap_to_shorter_line() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Longer line\nHi"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 7; // On 'l' in "line"
    
    move_cursor_down(&mut map_state);
    
    // Should snap to end of shorter next line (position 14 - end of "Hi")
    assert_eq!(map_state.notes_state.cursor_pos, 14);
}

#[test]
fn test_move_down_from_first_line() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("First\nSecond\nThird"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 2; // On 'r' in "First"
    
    move_cursor_down(&mut map_state);
    
    // Should move to second line, position 8 ('c' in "Second")
    assert_eq!(map_state.notes_state.cursor_pos, 8);
}

#[test]
fn test_move_down_at_line_start() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello\nWorld"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 0; // At start of "Hello"
    
    move_cursor_down(&mut map_state);
    
    // Should move to start of next line
    assert_eq!(map_state.notes_state.cursor_pos, 6);
}

#[test]
fn test_move_down_at_line_end() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello\nWorld"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 5; // At end of "Hello"
    
    move_cursor_down(&mut map_state);
    
    // Should move to end of next line
    assert_eq!(map_state.notes_state.cursor_pos, 11);
}

#[test]
fn test_move_down_no_selected_note() {
    // Shouldn't happen

    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello\nWorld"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = None; // No selected note
    map_state.notes_state.cursor_pos = 2;
    
    move_cursor_down(&mut map_state);
    
    // Should do nothing when no note is selected
    assert_eq!(map_state.notes_state.cursor_pos, 2);
}

#[test]
fn test_move_down_empty_lines() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello\n\nWorld"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 3; // On 'l' in "Hello"
    
    move_cursor_down(&mut map_state);
    
    // Should move to the empty line (position 6)
    assert_eq!(map_state.notes_state.cursor_pos, 6);
}

#[test]
fn test_move_down_with_multiple_empty_lines() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("A\n\n\nB"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 0; // At 'A'
    
    move_cursor_down(&mut map_state);
    
    // Should move to next empty line
    assert_eq!(map_state.notes_state.cursor_pos, 2);
}

#[test]
fn test_move_down_single_line() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 3;
    
    move_cursor_down(&mut map_state);
    
    // Should not move on single line
    assert_eq!(map_state.notes_state.cursor_pos, 3);
}

#[test]
fn test_move_down_varying_line_lengths() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Med\nThis is a longer line\nShort"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 2; // On 'd' in "Med"
    
    move_cursor_down(&mut map_state);
    
    // Should move to same column on longer line (position 6 - 'i' in "This")
    assert_eq!(map_state.notes_state.cursor_pos, 6);
}

#[test]
fn test_move_down_to_last_line() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("First\nSecond\nThird"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 8; // On 'c' in "Second"
    
    move_cursor_down(&mut map_state);
    
    // Should move to third line (position 15 - 'i' in "Third")
    assert_eq!(map_state.notes_state.cursor_pos, 15);
}

// ============================================================================
// Integration tests - combining operations
// ============================================================================

#[test]
fn test_edit_workflow_insert_and_backspace() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from(""), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.cursor_pos = 0;
    
    // Type "Hello"
    insert_char(&mut map_state, 0, 'H');
    insert_char(&mut map_state, 0, 'e');
    insert_char(&mut map_state, 0, 'l');
    insert_char(&mut map_state, 0, 'l');
    insert_char(&mut map_state, 0, 'o');
    
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Hello");
    assert_eq!(map_state.notes_state.cursor_pos, 5);
    
    // Backspace twice
    backspace_char(&mut map_state, 0);
    backspace_char(&mut map_state, 0);
    
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Hel");
    assert_eq!(map_state.notes_state.cursor_pos, 3);
}

#[test]
fn test_edit_workflow_navigation_and_editing() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Line 1\nLine 2\nLine 3"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 0;
    
    // Move down to line 2
    move_cursor_down(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 7); // Start of "Line 2"
    
    // Insert an 'X'
    insert_char(&mut map_state, 0, 'X');
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Line 1\nXLine 2\nLine 3");
    assert_eq!(map_state.notes_state.cursor_pos, 8); // Cursor moved after insert
    
    // Backspace to remove the 'X'
    backspace_char(&mut map_state, 0);
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Line 1\nLine 2\nLine 3");
}

#[test]
fn test_cursor_movement_up_and_down() {
    let mut map_state = create_test_map_state();
    
    // String layout: "AAA\nBBBBB\nCC"
    // Line 0: "AAA" (indices 0,1,2)
    // Line 1: "BBBBB" (indices 4,5,6,7,8)
    // Line 2: "CC" (indices 10,11)
    let note = Note::new(10, 10, String::from("AAA\nBBBBB\nCC"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 6; // On third 'B' in middle line (column 2)
    
    // Move down (should snap to end of shorter line)
    move_cursor_down(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 12); // End of "CC"
    
    // Move up (column 2 in "BBBBB")
    move_cursor_up(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 6); // Back to third 'B' in "BBBBB"
    
    // Move up again (column 2 in "AAA")
    move_cursor_up(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 2); // Third 'A' in "AAA"
}

#[test]
fn test_multiline_editing() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(10, 10, String::from("Hello"), false, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 5;
    
    // Add newline and more text
    insert_char(&mut map_state, 0, '\n');
    insert_char(&mut map_state, 0, 'W');
    insert_char(&mut map_state, 0, 'o');
    insert_char(&mut map_state, 0, 'r');
    insert_char(&mut map_state, 0, 'l');
    insert_char(&mut map_state, 0, 'd');
    
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Hello\nWorld");
    
    // Navigate back up and edit
    move_cursor_up(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 5); // End of "Hello"
    
    insert_char(&mut map_state, 0, '!');
    assert_eq!(map_state.notes_state.notes.get(&0).unwrap().content, "Hello!\nWorld");
}
