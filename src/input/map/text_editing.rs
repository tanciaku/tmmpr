use crate::states::MapState;


pub fn backspace_char(map_state: &mut MapState, selected_note: usize) {
    // Mark dirty so changes are saved or prompt user before exiting
    map_state.persistence.mark_dirty();

    if let Some(note) = map_state.notes_state.notes.get_mut(&selected_note) {
        if map_state.notes_state.cursor_pos > 0 {
            let mut chars: Vec<char> = note.content.chars().collect();
            
            // Backspace deletes the character *before* the cursor
            chars.remove(map_state.notes_state.cursor_pos - 1);
            map_state.notes_state.cursor_pos -= 1;

            note.content = chars.into_iter().collect();
        }
    }
}

pub fn remove_char(map_state: &mut MapState, selected_note: usize) {
    // Mark dirty so changes are saved or prompt user before exiting
    map_state.persistence.mark_dirty();

    if let Some(note) = map_state.notes_state.notes.get_mut(&selected_note) {
        if !note.content.is_empty() {
            let mut chars: Vec<char> = note.content.chars().collect();
            
            if map_state.notes_state.cursor_pos < chars.len() {
                chars.remove(map_state.notes_state.cursor_pos);

                // Adjust cursor position to stay in bounds after deletion
                if map_state.notes_state.cursor_pos >= chars.len() && !chars.is_empty() {
                    map_state.notes_state.cursor_pos = chars.len() - 1;
                } else if chars.is_empty() {
                    map_state.notes_state.cursor_pos = 0;
                }
            }

            note.content = chars.into_iter().collect();
        }
    }
}

pub fn insert_char(map_state: &mut MapState, selected_note: usize, c: char) {
    // Mark dirty so changes are saved or prompt user before exiting
    map_state.persistence.mark_dirty();

    if let Some(note) = map_state.notes_state.notes.get_mut(&selected_note) {
        // Use char_indices to handle multi-byte UTF-8 characters correctly
        if let Some((byte_pos, _)) = note.content.char_indices().nth(map_state.notes_state.cursor_pos) {
            note.content.insert(byte_pos, c);
        } else {
            note.content.push(c);
        }

        map_state.notes_state.cursor_pos += 1;
    }
}

pub fn move_cursor_up(map_state: &mut MapState) {
    if let Some(selected_note) = &map_state.notes_state.selected_note {
        if let Some(note) = map_state.notes_state.notes.get(selected_note) {
            let mut current_line_start = 0;
            let mut previous_line_start = 0;

            for line in note.content.lines() {
                if current_line_start + line.chars().count() >= map_state.notes_state.cursor_pos {
                    break;
                }
                previous_line_start = current_line_start;
                current_line_start += line.chars().count() + 1; // +1 for newline
            }

            if current_line_start == 0 { return } // Already on first line

            let index_in_the_current_line = map_state.notes_state.cursor_pos - current_line_start;
            let previous_line_length = current_line_start - previous_line_start - 1;

            // Preserve horizontal position when moving up, or snap to line end if shorter
            if previous_line_length > index_in_the_current_line {
                map_state.notes_state.cursor_pos = previous_line_start + index_in_the_current_line;
            } else {
                map_state.notes_state.cursor_pos = previous_line_start + previous_line_length;
            }
        }
    }
}

pub fn move_cursor_down(map_state: &mut MapState) {
    if let Some(selected_note) = &map_state.notes_state.selected_note {
        if let Some(note) = map_state.notes_state.notes.get(selected_note) {
            let mut current_line_start = 0;
            let mut next_line_start = 0;

            for line in note.content.lines() {
                if next_line_start + line.chars().count() >= map_state.notes_state.cursor_pos {
                    current_line_start = next_line_start;
                    next_line_start += line.chars().count() + 1; // +1 for newline
                    break;
                }
                current_line_start = next_line_start;
                next_line_start += line.chars().count() + 1;
            }

            if next_line_start > note.content.len() { return } // Already on last line

            let index_in_the_current_line = map_state.notes_state.cursor_pos - current_line_start;

            let remaining_content = &note.content[next_line_start..];
            let next_line_length = match remaining_content.find('\n') {
                Some(newline_pos) => newline_pos,
                None => remaining_content.len(), // Last line has no trailing newline
            };

            // Preserve horizontal position when moving down, or snap to line end if shorter
            if next_line_length > index_in_the_current_line {
                map_state.notes_state.cursor_pos = next_line_start + index_in_the_current_line;
            } else {
                map_state.notes_state.cursor_pos = next_line_start + next_line_length;
            }
        }
    }
}
