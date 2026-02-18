use crate::states::{MapState, map::NotesState};

pub fn cursor_pos_beginning(notes_state: &mut NotesState) {
    notes_state.set_cursor_pos(0);
}

/// Panics if no note is selected.
pub fn move_cursor_left(notes_state: &mut NotesState) {
    let note = notes_state.expect_selected_note();
    let cursor_pos = notes_state.cursor_pos();

    if cursor_pos > 0 {
        // Find the byte index of the previous character
        let new_pos = note.content[..cursor_pos]
            .char_indices()
            .next_back()
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        
        notes_state.set_cursor_pos(new_pos);
    }
}

/// Panics if no note is selected.
pub fn move_cursor_right(notes_state: &mut NotesState) {
    let note = notes_state.expect_selected_note();
    let cursor_pos = notes_state.cursor_pos();

    let new_pos = note.content[cursor_pos..]
        .char_indices()
        .nth(1)
        .map(|(idx, _)| cursor_pos + idx)
        .unwrap_or(note.content.len());

    notes_state.set_cursor_pos(new_pos);
}

/// Panics if no note is selected.
pub fn backspace_char(map_state: &mut MapState) {
    map_state.persistence.mark_dirty();

    let cursor_pos = map_state.notes_state.cursor_pos();

    if cursor_pos > 0 {
        let note = map_state.notes_state.expect_selected_note_mut();

        // Find the byte start of the character immediately before the cursor
        let prev_char_byte_pos = note.content[..cursor_pos]
            .char_indices()
            .next_back()
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        // Remove the bytes of that character
        note.content.drain(prev_char_byte_pos..cursor_pos);

        map_state.notes_state.set_cursor_pos(prev_char_byte_pos);
    }
}

/// Panics if no note is selected.
pub fn insert_char(map_state: &mut MapState, c: char) {
    // Mark dirty so changes are saved or prompt user before exiting
    map_state.persistence.mark_dirty();

    let cursor_pos = map_state.notes_state.cursor_pos();
    let note = map_state.notes_state.expect_selected_note_mut();

    // cursor_pos is a byte index on a char boundary, so insert directly
    note.content.insert(cursor_pos, c);

    move_cursor_right(&mut map_state.notes_state);
}

/// Panics if no note is selected.
pub fn move_cursor_up(notes_state: &mut NotesState) {
    let note = notes_state.expect_selected_note();
    let cursor_pos = notes_state.cursor_pos();

    let mut current_line_start = 0;
    let mut previous_line_start = 0;

    for line in note.content.lines() {
        // Use byte length: '\n' is always 1 byte so +1 is correct
        if current_line_start + line.len() >= cursor_pos {
            break;
        }
        previous_line_start = current_line_start;
        current_line_start += line.len() + 1; // +1 for '\n'
    }

    if current_line_start == 0 { return } // Already on first line

    // Char count of current line prefix gives the visual column
    let col = note.content[current_line_start..cursor_pos].chars().count();

    let previous_line = &note.content[previous_line_start..current_line_start - 1];

    // Find the byte position in the previous line at the target column,
    // clamping to the line end if it's shorter
    let new_pos = previous_line
        .char_indices()
        .nth(col)
        .map(|(idx, _)| previous_line_start + idx)
        .unwrap_or(previous_line_start + previous_line.len());

    notes_state.set_cursor_pos(new_pos);
}

/// Panics if no note is selected.
pub fn move_cursor_down(notes_state: &mut NotesState) {
    let note = notes_state.expect_selected_note();
    let cursor_pos = notes_state.cursor_pos();

    let mut current_line_start = 0;
    let mut next_line_start = 0;

    for line in note.content.lines() {
        // Use byte length: '\n' is always 1 byte so +1 is correct
        if next_line_start + line.len() >= cursor_pos {
            current_line_start = next_line_start;
            next_line_start += line.len() + 1; // +1 for '\n'
            break;
        }
        current_line_start = next_line_start;
        next_line_start += line.len() + 1;
    }

    if next_line_start > note.content.len() { return } // Already on last line

    // Char count of current line prefix gives the visual column
    let col = note.content[current_line_start..cursor_pos].chars().count();

    let next_line = &note.content[next_line_start..];
    let next_line = match next_line.find('\n') {
        Some(newline_pos) => &next_line[..newline_pos],
        None => next_line, // Last line has no trailing newline
    };

    // Find the byte position in the next line at the target column,
    // clamping to the line end if it's shorter
    let new_pos = next_line
        .char_indices()
        .nth(col)
        .map(|(idx, _)| next_line_start + idx)
        .unwrap_or(next_line_start + next_line.len());

    notes_state.set_cursor_pos(new_pos);
}
