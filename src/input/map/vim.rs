use std::io::stdout;

use crossterm::{cursor::SetCursorStyle, execute};

use crate::states::{MapState, map::{Mode, NotesState}};


pub fn switch_to_modal_normal_mode(map_state: &mut MapState) {
    let _ = execute!(stdout(), SetCursorStyle::SteadyBlock);
    map_state.current_mode = Mode::EditNormal;
}

pub fn switch_to_modal_insert_mode(map_state: &mut MapState) {
    let _ = execute!(stdout(), SetCursorStyle::SteadyBar);
    map_state.current_mode = Mode::EditInsert;
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
    let note = map_state.notes_state.expect_selected_note(); 
    let cursor_pos = map_state.notes_state.cursor_pos();

    map_state.notes_state.set_cursor_pos((cursor_pos + 1).min(note.content.len()));
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

    // Check if previous character is whitespace to determine strategy
    let at_word_beginning = if cursor_pos > 0 {
        note.content.chars()
            .nth(cursor_pos.saturating_sub(1))
            .map(|c| c == ' ' || c == '\n')
            .unwrap_or(false)
    } else {
        true
    };

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
fn find_previous_word_start(note: &crate::states::map::Note, cursor_pos: usize) -> usize {
    if cursor_pos == 0 {
        return 0;
    }

    let chars: Vec<char> = note.content.chars().collect();
    let mut pos = cursor_pos.saturating_sub(1);

    // Skip backward over whitespace
    while pos > 0 && (chars[pos] == ' ' || chars[pos] == '\n') {
        pos = pos.saturating_sub(1);
    }

    if pos == 0 && (chars[0] == ' ' || chars[0] == '\n') {
        return 0;
    }

    // Continue backward through word until delimiter found
    while pos > 0 {
        let prev_char = chars[pos.saturating_sub(1)];
        if prev_char == ' ' || prev_char == '\n' {
            break;
        }
        pos = pos.saturating_sub(1);
    }

    pos
}

/// Panics if no note is selected.
pub fn remove_char(map_state: &mut MapState) {
    map_state.persistence.mark_dirty();

    let cursor_pos = map_state.notes_state.cursor_pos();
    let note = map_state.notes_state.expect_selected_note_mut();

    if note.content.is_empty() || cursor_pos >= note.content.chars().count() {
        return;
    }

    let mut chars: Vec<char> = note.content.chars().collect(); 
    chars.remove(cursor_pos);

    let new_cursor_pos = if chars.is_empty() {
        0
    } else {
        cursor_pos.min(chars.len() - 1)
    };

    note.content = chars.into_iter().collect();
    map_state.notes_state.set_cursor_pos(new_cursor_pos);
}
