use std::io::stdout;

use crossterm::{cursor::SetCursorStyle, execute};

use crate::states::{MapState, map::{ModalEditMode, Mode}};


pub fn switch_to_modal_normal_mode(map_state: &mut MapState) {
    let _ = execute!(stdout(), SetCursorStyle::SteadyBlock);
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));
}

pub fn switch_to_modal_insert_mode(map_state: &mut MapState) {
    let _ = execute!(stdout(), SetCursorStyle::SteadyBar);
    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Insert));
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
pub fn jump_forward_a_word(map_state: &mut MapState) {
    if let Some(selected_note) = &map_state.notes_state.selected_note {
        if let Some(note) = map_state.notes_state.notes.get(selected_note) {
            if note.content.is_empty() {
                return;
            }
                
            let mut space_pos = note.content[map_state.notes_state.cursor_pos..].find(' ');

            // Skip consecutive spaces to find start of next word
            if let Some(pos) = space_pos {
                if let Some(new_pos) = note.content[map_state.notes_state.cursor_pos + pos..].find(|c| {c != ' '}) {
                    // Adjust by -1 to allow consistent +1 offset in match arms below
                    space_pos = Some(pos + new_pos - 1);
                }
            }

            let newline_pos = note.content[map_state.notes_state.cursor_pos..].find('\n');

            let target_pos = match (space_pos, newline_pos) {
                (Some(s), Some(n)) => map_state.notes_state.cursor_pos + s.min(n) + 1,
                (Some(s), None) => map_state.notes_state.cursor_pos + s + 1,
                (None, Some(n)) => map_state.notes_state.cursor_pos + n + 1,
                (None, None) => note.content.len() - 1,
            };

            map_state.notes_state.cursor_pos = target_pos.min(note.content.len() - 1);
        }
    }
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
pub fn jump_back_a_word(map_state: &mut MapState) {
    if let Some(selected_note) = &map_state.notes_state.selected_note {
        if let Some(note) = map_state.notes_state.notes.get(selected_note) {
            if note.content.is_empty() || map_state.notes_state.cursor_pos == 0 {
                return;
            }

            // Check if previous character is whitespace to determine strategy
            let at_word_beginning = if map_state.notes_state.cursor_pos > 0 {
                note.content.chars()
                    .nth(map_state.notes_state.cursor_pos.saturating_sub(1))
                    .map(|c| c == ' ' || c == '\n')
                    .unwrap_or(false)
            } else {
                true
            };

            let target_pos = if at_word_beginning {
                find_previous_word_start(note, map_state.notes_state.cursor_pos)
            } else {
                find_current_word_start(note, map_state.notes_state.cursor_pos)
            };

            map_state.notes_state.cursor_pos = target_pos;
        }
    }
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
