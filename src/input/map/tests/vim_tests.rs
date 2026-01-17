use std::path::PathBuf;
use ratatui::style::Color;

use crate::{
    input::map::vim::{
        switch_to_modal_normal_mode, 
        switch_to_modal_insert_mode,
        jump_forward_a_word,
        jump_back_a_word,
    },
    states::{MapState, map::{Mode, ModalEditMode, Note}},
};

fn create_test_map_state() -> MapState {
    let mut map_state = MapState::new(PathBuf::from("/test/path"));
    map_state.settings.edit_modal = false;
    map_state.viewport.screen_width = 100;
    map_state.viewport.screen_height = 50;
    map_state
}

// ============================================================================
// Tests for switch_to_modal_normal_mode
// ============================================================================

#[test]
fn test_switch_to_modal_normal_mode_from_insert() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Insert));

    switch_to_modal_normal_mode(&mut map_state);

    assert_eq!(map_state.current_mode, Mode::Edit(Some(ModalEditMode::Normal)));
}

#[test]
fn test_switch_to_modal_normal_mode_from_normal_mode() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;

    switch_to_modal_normal_mode(&mut map_state);

    assert_eq!(map_state.current_mode, Mode::Edit(Some(ModalEditMode::Normal)));
}

#[test]
fn test_switch_to_modal_normal_mode_from_visual() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Visual;

    switch_to_modal_normal_mode(&mut map_state);

    assert_eq!(map_state.current_mode, Mode::Edit(Some(ModalEditMode::Normal)));
}

#[test]
fn test_switch_to_modal_normal_mode_already_in_modal_normal() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));

    switch_to_modal_normal_mode(&mut map_state);

    // Should remain in modal normal mode
    assert_eq!(map_state.current_mode, Mode::Edit(Some(ModalEditMode::Normal)));
}

// ============================================================================
// Tests for switch_to_modal_insert_mode
// ============================================================================

#[test]
fn test_switch_to_modal_insert_mode_from_normal() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));

    switch_to_modal_insert_mode(&mut map_state);

    assert_eq!(map_state.current_mode, Mode::Edit(Some(ModalEditMode::Insert)));
}

#[test]
fn test_switch_to_modal_insert_mode_from_normal_mode() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Normal;

    switch_to_modal_insert_mode(&mut map_state);

    assert_eq!(map_state.current_mode, Mode::Edit(Some(ModalEditMode::Insert)));
}

#[test]
fn test_switch_to_modal_insert_mode_from_visual() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Visual;

    switch_to_modal_insert_mode(&mut map_state);

    assert_eq!(map_state.current_mode, Mode::Edit(Some(ModalEditMode::Insert)));
}

#[test]
fn test_switch_to_modal_insert_mode_already_in_modal_insert() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Insert));

    switch_to_modal_insert_mode(&mut map_state);

    // Should remain in modal insert mode
    assert_eq!(map_state.current_mode, Mode::Edit(Some(ModalEditMode::Insert)));
}

// ============================================================================
// Tests for jump_forward_a_word
// ============================================================================

#[test]
fn test_jump_forward_a_word_no_selected_note() {
    // Shouldn't happen

    let mut map_state = create_test_map_state();
    map_state.notes_state.selected_note = None;
    map_state.notes_state.cursor_pos = 0;

    jump_forward_a_word(&mut map_state);

    // Should not crash and cursor should remain at 0
    assert_eq!(map_state.notes_state.cursor_pos, 0);
}

#[test]
fn test_jump_forward_a_word_empty_content() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from(""), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 0;

    jump_forward_a_word(&mut map_state);

    // Should not crash with empty content
    assert_eq!(map_state.notes_state.cursor_pos, 0);
}

#[test]
fn test_jump_forward_a_word_simple_two_words() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("hello world"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 0;

    jump_forward_a_word(&mut map_state);

    // Should jump from 'h' (position 0) to 'w' (position 6)
    assert_eq!(map_state.notes_state.cursor_pos, 6);
}

#[test]
fn test_jump_forward_a_word_multiple_spaces() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("hello   world"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 0;

    jump_forward_a_word(&mut map_state);

    // Should jump from 'h' to 'w', skipping multiple spaces
    assert_eq!(map_state.notes_state.cursor_pos, 8);
}

#[test]
fn test_jump_forward_a_word_with_newline() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("hello\nworld"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 0;

    jump_forward_a_word(&mut map_state);

    // Should jump from 'h' to 'w', crossing the newline
    assert_eq!(map_state.notes_state.cursor_pos, 6);
}

#[test]
fn test_jump_forward_a_word_no_more_words() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("hello"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 0;

    jump_forward_a_word(&mut map_state);

    // Should jump to the end (last character position)
    // "hello" has length 5, so last valid position is 4
    assert_eq!(map_state.notes_state.cursor_pos, 4);
}

#[test]
fn test_jump_forward_a_word_already_at_end() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("hello world"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 10; // At 'd', the last character

    jump_forward_a_word(&mut map_state);

    // Should stay at the end since there are no more words
    assert_eq!(map_state.notes_state.cursor_pos, 10);
}

#[test]
fn test_jump_forward_a_word_multiple_jumps() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("the quick brown fox"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 0;

    jump_forward_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 4); // 'q' in "quick"

    jump_forward_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 10); // 'b' in "brown"

    jump_forward_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 16); // 'f' in "fox"

    jump_forward_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 18); // Last character 'x'
}

#[test]
fn test_jump_forward_a_word_space_before_newline() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("hello \nworld"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 0;

    jump_forward_a_word(&mut map_state);

    // Should jump to position 6 (the space comes first at pos 5, +1 = 6 which is the newline)
    // The function finds the nearest delimiter and jumps past it
    assert_eq!(map_state.notes_state.cursor_pos, 6);
}

#[test]
fn test_jump_forward_a_word_from_middle_of_word() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("hello world"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 2; // At 'l' in "hello"

    jump_forward_a_word(&mut map_state);

    // Should still jump to 'w' in "world"
    assert_eq!(map_state.notes_state.cursor_pos, 6);
}

#[test]
fn test_jump_forward_a_word_single_character_word() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("a b c"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 0;

    jump_forward_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 2); // 'b'

    jump_forward_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 4); // 'c'
}

#[test]
fn test_jump_forward_a_word_multiline_text() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("first line\nsecond line\nthird"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 0;

    jump_forward_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 6); // 'l' in "line"

    jump_forward_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 11); // 's' in "second"
}

// ============================================================================
// Tests for jump_back_a_word
// ============================================================================

#[test]
fn test_jump_back_a_word_no_selected_note() {
    // Shouldn't happen

    let mut map_state = create_test_map_state();
    map_state.notes_state.selected_note = None;
    map_state.notes_state.cursor_pos = 5;

    jump_back_a_word(&mut map_state);

    // Should not crash and cursor should remain at 5
    assert_eq!(map_state.notes_state.cursor_pos, 5);
}

#[test]
fn test_jump_back_a_word_empty_content() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from(""), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 0;

    jump_back_a_word(&mut map_state);

    // Should not crash with empty content
    assert_eq!(map_state.notes_state.cursor_pos, 0);
}

#[test]
fn test_jump_back_a_word_already_at_beginning() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("hello world"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 0;

    jump_back_a_word(&mut map_state);

    // Should stay at the beginning
    assert_eq!(map_state.notes_state.cursor_pos, 0);
}

#[test]
fn test_jump_back_a_word_simple_two_words() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("hello world"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 6; // At 'w' in "world"

    jump_back_a_word(&mut map_state);

    // Should jump back to 'h' in "hello"
    assert_eq!(map_state.notes_state.cursor_pos, 0);
}

#[test]
fn test_jump_back_a_word_from_middle_of_word() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("hello world"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 8; // At 'r' in "world"

    jump_back_a_word(&mut map_state);

    // Should jump to the beginning of "world"
    assert_eq!(map_state.notes_state.cursor_pos, 6);
}

#[test]
fn test_jump_back_a_word_from_end_of_word() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("hello world"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 10; // At 'd' in "world"

    jump_back_a_word(&mut map_state);

    // Should jump to the beginning of "world"
    assert_eq!(map_state.notes_state.cursor_pos, 6);
}

#[test]
fn test_jump_back_a_word_multiple_spaces() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("hello   world"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 8; // At 'w' in "world"

    jump_back_a_word(&mut map_state);

    // Should skip multiple spaces and jump to 'h' in "hello"
    assert_eq!(map_state.notes_state.cursor_pos, 0);
}

#[test]
fn test_jump_back_a_word_with_newline() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("hello\nworld"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 6; // At 'w' in "world"

    jump_back_a_word(&mut map_state);

    // Should jump back to 'h' in "hello", crossing the newline
    assert_eq!(map_state.notes_state.cursor_pos, 0);
}

#[test]
fn test_jump_back_a_word_multiple_jumps() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("the quick brown fox"), true, Color::White);

    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 18; // At 'x' in "fox"

    jump_back_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 16); // 'f' in "fox"

    jump_back_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 10); // 'b' in "brown"

    jump_back_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 4); // 'q' in "quick"

    jump_back_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 0); // 't' in "the"
}

#[test]
fn test_jump_back_a_word_from_space_between_words() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("hello world"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 5; // At space between "hello" and "world"

    jump_back_a_word(&mut map_state);

    // Should jump to the beginning of "hello"
    assert_eq!(map_state.notes_state.cursor_pos, 0);
}

#[test]
fn test_jump_back_a_word_single_character_words() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("a b c"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 4; // At 'c'

    jump_back_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 2); // 'b'

    jump_back_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 0); // 'a'
}

#[test]
fn test_jump_back_a_word_multiline_text() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("first line\nsecond line\nthird"), true, Color::White);

    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 27; // At 'd' in "third"

    jump_back_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 23); // 't' in "third"

    jump_back_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 18); // 'l' in "line" (second)

    jump_back_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 11); // 's' in "second"
}

#[test]
fn test_jump_back_a_word_whitespace_at_beginning() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("  hello world"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 8; // At 'w' in "world"

    jump_back_a_word(&mut map_state);

    // Should jump to 'h' in "hello"
    assert_eq!(map_state.notes_state.cursor_pos, 2);
}

#[test]
fn test_jump_back_a_word_only_whitespace_before() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("   word"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 3; // At 'w'

    jump_back_a_word(&mut map_state);

    // Should jump to the beginning (all whitespace before)
    assert_eq!(map_state.notes_state.cursor_pos, 0);
}

#[test]
fn test_jump_back_a_word_mixed_whitespace() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("hello \n world"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 8; // At 'w' in "world" (position 8)

    jump_back_a_word(&mut map_state);

    // Should skip both space and newline, jump to 'h' in "hello"
    assert_eq!(map_state.notes_state.cursor_pos, 0);
}

// ============================================================================
// Integration tests: combining forward and backward jumps
// ============================================================================

#[test]
fn test_jump_forward_then_back() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("hello world test"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 0;

    jump_forward_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 6); // 'w' in "world"

    jump_back_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 0); // Back to 'h' in "hello"
}

#[test]
fn test_jump_back_then_forward() {
    let mut map_state = create_test_map_state();
    let note = Note::new(10, 10, String::from("hello world test"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 12; // 't' in "test"

    jump_back_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 6); // 'w' in "world"

    jump_forward_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 12); // Back to 't' in "test"
}

#[test]
fn test_navigation_through_complex_text() {
    let mut map_state = create_test_map_state();
    let note = Note::new(
        10, 
        10, 
        String::from("This is a\ntest of word\nnavigation functionality"), 
        true, 
        Color::White
    );
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.cursor_pos = 0;

    // Navigate forward through the text
    jump_forward_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 5); // 'i' in "is"

    jump_forward_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 8); // 'a'

    jump_forward_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 10); // 't' in "test"

    // Navigate backward
    jump_back_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 8); // 'a'

    jump_back_a_word(&mut map_state);
    assert_eq!(map_state.notes_state.cursor_pos, 5); // 'i' in "is"
}
