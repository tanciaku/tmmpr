use std::io::stdout;

use crossterm::{cursor::SetCursorStyle, execute};

use crate::states::{MapState, map::{Mode, NotesState}};


pub fn switch_to_modal_normal_mode(map_state: &mut MapState) {
    let _ = execute!(stdout(), SetCursorStyle::SteadyBlock);
    map_state.mode = Mode::EditNormal;
}

pub fn switch_to_modal_insert_mode(map_state: &mut MapState) {
    let _ = execute!(stdout(), SetCursorStyle::SteadyBar);
    map_state.mode = Mode::EditInsert;
}

/// Panics if no note is selected.
pub fn cursor_pos_end(notes_state: &mut NotesState) {
    let note = notes_state.expect_selected_note();

    let last_char_pos = note.content
        .char_indices()
        .last()
        .map(|(idx, _)| idx)
        .unwrap_or(0);

    notes_state.set_cursor_pos(last_char_pos);
}

/// Panics if no note is selected.
pub fn move_cursor_right_norm(notes_state: &mut NotesState) {
    let note = notes_state.expect_selected_note();
    let cursor_pos = notes_state.cursor_pos();

    let last_char_pos = note.content
        .char_indices()
        .last()
        .map(|(idx, _)| idx)
        .unwrap_or(0);

    let new_pos = note.content[cursor_pos..]
        .char_indices()
        .nth(1)
        .map(|(idx, _)| cursor_pos + idx)
        .unwrap_or(cursor_pos);

    // Stop before last char to match vim's normal mode behavior
    notes_state.set_cursor_pos(new_pos.min(last_char_pos));
}

/// Panics if no note is selected.
pub fn append(map_state: &mut MapState) {
    let cursor_pos = map_state.notes_state.cursor_pos();
    let note = map_state.notes_state.expect_selected_note();

    // Advance by the byte length of the character at cursor_pos so the new
    // position is always on a valid char boundary, even for multi-byte chars.
    let new_pos = note.content[cursor_pos..]
        .char_indices()
        .nth(1)
        .map(|(idx, _)| cursor_pos + idx)
        .unwrap_or(note.content.len());

    map_state.notes_state.set_cursor_pos(new_pos);
    switch_to_modal_insert_mode(map_state);
}

/// Jumps cursor forward to the beginning of the next word (vim 'w' behavior).
/// 
/// Treats spaces and newlines as word delimiters. Handles consecutive whitespace
/// and bounds checking.
/// 
/// # Examples
/// - `"hello world"` → jumps from 'h' to 'w'  
/// - `"hello   world"` → skips multiple spaces
/// - `"hello\nworld"` → crosses line boundaries
///
/// # Panics
/// If no note is selected.
pub fn jump_forward_a_word(notes_state: &mut NotesState) {
    let note = notes_state.expect_selected_note();
    let cursor_pos = notes_state.cursor_pos();

    if note.content.is_empty() {
        return;
    }
        
    let mut space_pos = note.content[cursor_pos..].find(' ');

    // Skip consecutive spaces to find start of next word
    if let Some(pos) = space_pos {
        if let Some(new_pos) = note.content[cursor_pos + pos..].find(|c| {c != ' '}) {
            // Adjust by -1 to allow consistent +1 offset in match arms below
            space_pos = Some(pos + new_pos - 1);
        }
    }

    let newline_pos = note.content[cursor_pos..].find('\n');

    let target_pos = match (space_pos, newline_pos) {
        (Some(s), Some(n)) => cursor_pos + s.min(n) + 1,
        (Some(s), None) => cursor_pos + s + 1,
        (None, Some(n)) => cursor_pos + n + 1,
        (None, None) => note.content.len() - 1,
    };

    notes_state.set_cursor_pos(target_pos.min(note.content.len() - 1));
}

/// Jumps cursor backward to the beginning of the previous word (vim 'b' behavior).
/// 
/// If cursor is mid-word, jumps to the start of current word. If at word start,
/// jumps to the start of previous word. Treats spaces and newlines as delimiters.
/// 
/// # Examples
/// - `"hello world"` (cursor on 'r') → jumps to 'w'
/// - `"hello world"` (cursor on 'w') → jumps to 'h'  
/// - `"hello   world"` → skips consecutive spaces
///
/// # Panics
/// If no note is selected.
pub fn jump_back_a_word(notes_state: &mut NotesState) {
    let note = notes_state.expect_selected_note();
    let cursor_pos = notes_state.cursor_pos();
    
    if note.content.is_empty() || cursor_pos == 0 {
        return;
    }

    // Check if the character immediately before the cursor is whitespace.
    // cursor_pos is a byte index, so slice to it and read the last char.
    let at_word_beginning = note.content[..cursor_pos]
        .chars()
        .next_back()
        .map(|c| c == ' ' || c == '\n')
        .unwrap_or(true);

    let target_pos = if at_word_beginning {
        find_previous_word_start(note, cursor_pos)
    } else {
        find_current_word_start(note, cursor_pos)
    };

    notes_state.set_cursor_pos(target_pos);
}

/// Finds the start of the current word when cursor is mid-word.
fn find_current_word_start(note: &crate::states::map::Note, cursor_pos: usize) -> usize {
    let text_before_cursor = &note.content[..cursor_pos];
    let last_delimiter_pos = text_before_cursor.rfind(|c: char| c == ' ' || c == '\n');
    
    match last_delimiter_pos {
        Some(pos) => pos + 1,
        None => 0,
    }
}

/// Finds the start of the previous word when cursor is at word beginning.
///
/// Two-phase algorithm: skip backward over whitespace, then find word start.
/// All positions are byte indices.
fn find_previous_word_start(note: &crate::states::map::Note, cursor_pos: usize) -> usize {
    if cursor_pos == 0 {
        return 0;
    }

    // Phase 1: skip backward over whitespace to find the end of the previous word.
    let text_before = &note.content[..cursor_pos];
    let word_end = text_before.trim_end_matches(|c: char| c == ' ' || c == '\n').len();

    if word_end == 0 {
        return 0; // Only whitespace before cursor
    }

    // Phase 2: find the last delimiter before the word to locate its start.
    match note.content[..word_end].rfind(|c: char| c == ' ' || c == '\n') {
        Some(pos) => pos + 1, // +1 skips the delimiter byte (space/newline are single-byte)
        None => 0,
    }
}

/// Panics if no note is selected.
pub fn remove_char(map_state: &mut MapState) {
    map_state.persistence.mark_dirty();

    let cursor_pos = map_state.notes_state.cursor_pos();
    let note = map_state.notes_state.expect_selected_note_mut();

    if note.content.is_empty() || cursor_pos >= note.content.len() {
        return;
    }

    // Find the byte end of the character at cursor_pos (cursor_pos is a byte index).
    let char_end = note.content[cursor_pos..]
        .char_indices()
        .nth(1)
        .map(|(idx, _)| cursor_pos + idx)
        .unwrap_or(note.content.len());

    note.content.drain(cursor_pos..char_end);

    // In normal mode the cursor must not sit past the last character.
    let new_cursor_pos = if note.content.is_empty() {
        0
    } else {
        let last_char_start = note.content
            .char_indices()
            .last()
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        cursor_pos.min(last_char_start)
    };

    map_state.notes_state.set_cursor_pos(new_cursor_pos);
}
