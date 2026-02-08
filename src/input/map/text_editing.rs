use crate::states::{MapState, map::NotesState};

pub fn cursor_pos_beginning(notes_state: &mut NotesState) {
    notes_state.set_cursor_pos(0);
}

// FIXME: consideration for non-ASCII
/// Panics if no note is selected.
pub fn move_cursor_left(notes_state: &mut NotesState) {
    let _ = notes_state.expect_selected_note();
    let cursor_pos = notes_state.cursor_pos();

    notes_state.set_cursor_pos(cursor_pos.saturating_sub(1));
}

// FIXME: consideration for non-ASCII
/// Panics if no note is selected.
pub fn move_cursor_right(notes_state: &mut NotesState) {
    let note = notes_state.expect_selected_note();
    let cursor_pos = notes_state.cursor_pos();

    notes_state.set_cursor_pos((cursor_pos + 1).min(note.content.len()));
}

/// Panics if no note is selected.
pub fn backspace_char(map_state: &mut MapState) {
    map_state.persistence.mark_dirty();

    let cursor_pos = map_state.notes_state.cursor_pos();

    if cursor_pos > 0 {
        let note = map_state.notes_state.expect_selected_note_mut();
        let mut chars: Vec<char> = note.content.chars().collect();
        
        // Backspace deletes the character *before* the cursor
        chars.remove(cursor_pos - 1);
        note.content = chars.into_iter().collect();
        
        map_state.notes_state.set_cursor_pos(cursor_pos - 1);
    }
}

/// Panics if no note is selected.
pub fn insert_char(map_state: &mut MapState, c: char) {
    // Mark dirty so changes are saved or prompt user before exiting
    map_state.persistence.mark_dirty();

    let cursor_pos = map_state.notes_state.cursor_pos();
    let note = map_state.notes_state.expect_selected_note_mut();

    // Use char_indices to handle multi-byte UTF-8 characters correctly
    if let Some((byte_pos, _)) = note.content.char_indices().nth(cursor_pos) {
        note.content.insert(byte_pos, c);
    } else {
        note.content.push(c);
    }

    move_cursor_right(&mut map_state.notes_state);
}

/// Panics if no note is selected.
pub fn move_cursor_up(notes_state: &mut NotesState) {
    let note = notes_state.expect_selected_note();
    let cursor_pos = notes_state.cursor_pos();

    let mut current_line_start = 0;
    let mut previous_line_start = 0;

    for line in note.content.lines() {
        if current_line_start + line.chars().count() >= cursor_pos {
            break;
        }
        previous_line_start = current_line_start;
        current_line_start += line.chars().count() + 1; // +1 for newline
    }

    if current_line_start == 0 { return } // Already on first line

    let index_in_the_current_line = cursor_pos - current_line_start;
    let previous_line_length = current_line_start - previous_line_start - 1;

    // Preserve horizontal position when moving up, or snap to line end if shorter
    if previous_line_length > index_in_the_current_line {
        notes_state.set_cursor_pos(previous_line_start + index_in_the_current_line);
    } else {
        notes_state.set_cursor_pos(previous_line_start + previous_line_length);
    }
}

/// Panics if no note is selected.
pub fn move_cursor_down(notes_state: &mut NotesState) {
    let note = notes_state.expect_selected_note();
    let cursor_pos = notes_state.cursor_pos();

    let mut current_line_start = 0;
    let mut next_line_start = 0;

    for line in note.content.lines() {
        if next_line_start + line.chars().count() >= cursor_pos {
            current_line_start = next_line_start;
            next_line_start += line.chars().count() + 1; // +1 for newline
            break;
        }
        current_line_start = next_line_start;
        next_line_start += line.chars().count() + 1;
    }

    if next_line_start > note.content.len() { return } // Already on last line

    let index_in_the_current_line = cursor_pos - current_line_start;

    let remaining_content = &note.content[next_line_start..];
    let next_line_length = match remaining_content.find('\n') {
        Some(newline_pos) => newline_pos,
        None => remaining_content.len(), // Last line has no trailing newline
    };

    // Preserve horizontal position when moving down, or snap to line end if shorter
    if next_line_length > index_in_the_current_line {
        notes_state.set_cursor_pos(next_line_start + index_in_the_current_line);
    } else {
        notes_state.set_cursor_pos(next_line_start + next_line_length);
    }
}
