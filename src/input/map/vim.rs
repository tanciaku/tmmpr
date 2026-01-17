use std::io::stdout;

use crossterm::{cursor::SetCursorStyle, execute};

use crate::states::{MapState, map::{ModalEditMode, Mode}};


pub fn switch_to_modal_normal_mode(map_state: &mut MapState) {
    // Set a block cursor
    let _ = execute!(stdout(), SetCursorStyle::SteadyBlock);

    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Normal));
}

pub fn switch_to_modal_insert_mode(map_state: &mut MapState) {
    // Set a line cursor
    let _ = execute!(stdout(), SetCursorStyle::SteadyBar);

    map_state.current_mode = Mode::Edit(Some(ModalEditMode::Insert));
}

/// Jumps the cursor forward to the beginning of the next word (vim 'w' key behavior).
/// 
/// This function implements a simplified version of vim's 'w' command:
/// - Finds the next space or newline character after the cursor
/// - Jumps to the first character of the next word (skipping multiple consecutive spaces)
/// - If no space/newline is found, jumps to the end of the text
/// - Includes bounds checking to prevent cursor from going out of range
///
/// # Behavior Examples:
/// - "hello world" → cursor jumps from 'h' to 'w'  
/// - "hello   world" → cursor jumps from 'h' to 'w' (skips multiple spaces)
/// - "hello\nworld" → cursor jumps from 'h' to 'w' (crosses line boundaries)
pub fn jump_forward_a_word(map_state: &mut MapState) {
    if let Some(selected_note) = &map_state.notes_state.selected_note {
        if let Some(note) = map_state.notes_state.notes.get(selected_note) {
            // Only proceed if note's text content isn't empty to avoid index errors
            if !note.content.is_empty() {
                
                // Find the first space character after the current cursor position
                let mut space_pos = note.content[map_state.notes_state.cursor_pos..].find(' ');

                // Handle multiple consecutive spaces by finding the first non-space character
                if let Some(pos) = space_pos {
                    // Search from the found space position for the first non-space character
                    if let Some(new_pos) = note.content[map_state.notes_state.cursor_pos + pos..].find(|c| {c != ' '}) {
                        // Update space_pos to point one position before the target character
                        // (the -1 allows us to use +1 in the match arms consistently for both single and multiple spaces)
                        space_pos = Some(pos + new_pos - 1);
                    }
                }

                // Find the first newline character after the current cursor position
                let newline_pos = note.content[map_state.notes_state.cursor_pos..].find('\n');

                // Determine target position based on which delimiter comes first
                let target_pos = match (space_pos, newline_pos) {
                    (Some(s), Some(n)) => map_state.notes_state.cursor_pos + s.min(n) + 1, // Choose the closest delimiter
                    (Some(s), None) => map_state.notes_state.cursor_pos + s + 1, // Only space found
                    (None, Some(n)) => map_state.notes_state.cursor_pos + n + 1, // Only newline found
                    (None, None) => note.content.len() - 1, // No delimiters found, jump to end
                };

                // Apply bounds checking to ensure cursor stays within valid text range
                if target_pos > note.content.len() - 1 {
                    map_state.notes_state.cursor_pos = note.content.len() - 1; // Go to last character
                } else {
                    map_state.notes_state.cursor_pos = target_pos;
                }
            }
        }
    }
}

/// Jumps the cursor backward to the beginning of the previous word (vim 'b' key behavior).
/// 
/// This function implements a simplified version of vim's 'b' command:
/// - If cursor is in the middle of a word, jumps to the beginning of that word
/// - If cursor is at the beginning of a word, jumps to the beginning of the previous word
/// - Handles multiple consecutive whitespace characters by skipping over them
/// - Works across line boundaries (treats newlines as word delimiters)
/// - If no previous word exists, jumps to the very beginning of the text
///
/// # Behavior Examples:
/// - "hello world" (cursor on 'r') → jumps to 'w'
/// - "hello world" (cursor on 'w') → jumps to 'h'  
/// - "hello   world" → skips multiple spaces correctly
/// - "hello\nworld" → works across line boundaries
pub fn jump_back_a_word(map_state: &mut MapState) {
    if let Some(selected_note) = &map_state.notes_state.selected_note {
        if let Some(note) = map_state.notes_state.notes.get(selected_note) {
            // Early return if text is empty or cursor is already at the beginning
            if note.content.is_empty() || map_state.notes_state.cursor_pos == 0 {
                return;
            }

            // Determine if we're at the beginning of a word by checking the previous character
            let at_word_beginning = if map_state.notes_state.cursor_pos > 0 {
                note.content.chars()
                    .nth(map_state.notes_state.cursor_pos.saturating_sub(1))
                    .map(|c| c == ' ' || c == '\n')
                    .unwrap_or(false)
            } else {
                true // At position 0, considered at beginning
            };

            let target_pos = if at_word_beginning {
                // Skip whitespace, then find the beginning of the previous word
                find_previous_word_start(note, map_state.notes_state.cursor_pos)
            } else {
                // Find the beginning of the current word
                find_current_word_start(note, map_state.notes_state.cursor_pos)
            };

            map_state.notes_state.cursor_pos = target_pos;
        }
    }
}

/// Helper function to find the start of the current word when cursor is in the middle of it.
fn find_current_word_start(note: &crate::states::map::Note, cursor_pos: usize) -> usize {
    // Search backward from cursor position for the nearest delimiter
    let text_before_cursor = &note.content[..cursor_pos];
    
    // Find the last occurrence of a delimiter (space or newline)
    let last_delimiter_pos = text_before_cursor
        .rfind(|c: char| c == ' ' || c == '\n');
    
    match last_delimiter_pos {
        Some(pos) => pos + 1, // Move to the character after the delimiter
        None => 0,            // No delimiter found, go to the beginning
    }
}

/// Helper function to find the start of the previous word when cursor is at word beginning.
fn find_previous_word_start(note: &crate::states::map::Note, cursor_pos: usize) -> usize {
    if cursor_pos == 0 {
        return 0;
    }

    let chars: Vec<char> = note.content.chars().collect();
    let mut pos = cursor_pos.saturating_sub(1);

    // Phase 1: Skip backward over whitespace characters
    while pos > 0 && (chars[pos] == ' ' || chars[pos] == '\n') {
        pos = pos.saturating_sub(1);
    }

    // If we've reached the beginning and it's still whitespace, we're done
    if pos == 0 && (chars[0] == ' ' || chars[0] == '\n') {
        return 0;
    }

    // Phase 2: Continue backward through the previous word until we find its beginning
    while pos > 0 {
        let prev_char = chars[pos.saturating_sub(1)];
        if prev_char == ' ' || prev_char == '\n' {
            break; // Found the beginning of the word
        }
        pos = pos.saturating_sub(1);
    }

    pos
}
