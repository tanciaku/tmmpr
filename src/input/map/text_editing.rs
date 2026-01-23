use crate::states::MapState;


pub fn backspace_char(map_state: &mut MapState, selected_note: usize) {
    // Edited note's contents - need to save or discard changes before exiting.
    map_state.persistence.mark_dirty();

    if let Some(note) = map_state.notes_state.notes.get_mut(&selected_note) {
        // We can only backspace if the cursor is not at the very beginning of the text.
        if map_state.notes_state.cursor_pos > 0 {
            
            let mut chars: Vec<char> = note.content.chars().collect();
            
            // To delete the character *before* the cursor, we must remove the character
            // at the index `cursor_pos - 1`.
            chars.remove(map_state.notes_state.cursor_pos - 1);

            // After removing the character, we move the cursor's position back by one.
            map_state.notes_state.cursor_pos -= 1;

            // Update the note's text content
            note.content = chars.into_iter().collect();
        }
    }
}

pub fn remove_char(map_state: &mut MapState, selected_note: usize) {
    // Edited note's contents - need to save or discard changes before exiting.
    map_state.persistence.mark_dirty();

    if let Some(note) = map_state.notes_state.notes.get_mut(&selected_note) {
        if !note.content.is_empty() {
            let mut chars: Vec<char> = note.content.chars().collect();
            
            // Check if cursor is within bounds
            if map_state.notes_state.cursor_pos < chars.len() {
                // Delete the character at the cursor
                chars.remove(map_state.notes_state.cursor_pos);

                // If cursor is now past the end, move it back
                if map_state.notes_state.cursor_pos >= chars.len() && !chars.is_empty() {
                    map_state.notes_state.cursor_pos = chars.len() - 1;
                } else if chars.is_empty() {
                    map_state.notes_state.cursor_pos = 0;
                }
            }

            // Update the note's text content
            note.content = chars.into_iter().collect();
        }
    }
}

pub fn insert_char(map_state: &mut MapState, selected_note: usize, c: char) {
    // Edited note's contents - need to save or discard changes before exiting.
    map_state.persistence.mark_dirty();

    if let Some(note) = map_state.notes_state.notes.get_mut(&selected_note) {
        // Insert the typed character at the cursor's current position.
        if let Some((byte_pos, _)) = note.content.char_indices().nth(map_state.notes_state.cursor_pos) {
            note.content.insert(byte_pos, c);
        } else {
            note.content.push(c);
        }

        // Move the cursor forward one position.
        map_state.notes_state.cursor_pos += 1;
    }
}

pub fn move_cursor_up(map_state: &mut MapState) {
    if let Some(selected_note) = &map_state.notes_state.selected_note {
        if let Some(note) = map_state.notes_state.notes.get(selected_note) {
            // --- 1. Find the start of the current and previous lines ---

            // `current_line_start` will hold the starting index of the line the cursor is on.
            let mut current_line_start = 0;
            // `previous_line_start` will hold the starting index of the line *above* the cursor.
            let mut previous_line_start = 0;

            // Iterate through the lines of the note to find the cursor's position.
            for line in note.content.lines() {
                // Check if the end of the current line is past the cursor's position.
                // If it is, we've found the line the cursor is on.
                if current_line_start + line.chars().count() >= map_state.notes_state.cursor_pos {
                    break;
                }

                // If we haven't found the cursor's line yet, we update our variables.
                // The current line's start becomes the new 'previous' line start.
                previous_line_start = current_line_start;
                // We update the current line's start to the beginning of the *next* line,
                // accounting for the current line's length plus the newline character.
                current_line_start += line.chars().count() + 1;
            }

            // --- 2. Handle the edge case of being on the first line ---

            // If `current_line_start` is still 0, it means the loop broke on the first
            // line. We can't move up, so we exit early.
            if current_line_start == 0 { return }

            // --- 3. Calculate the new cursor position ---

            // Determine the cursor's horizontal position (column) within its current line.
            let index_in_the_current_line = map_state.notes_state.cursor_pos - current_line_start;

            // Calculate the character length of the previous line.
            let previous_line_length = current_line_start - previous_line_start - 1;

            // --- 4. Set the new cursor position, snapping if necessary ---

            // Check if the previous line is long enough to place the cursor at the same column.
            if previous_line_length > index_in_the_current_line {
                // If it is, the new position is the start of the previous line plus the column offset.
                map_state.notes_state.cursor_pos = previous_line_start + index_in_the_current_line;
            } else {
                // If the previous line is shorter, "snap" the cursor to the end of that line.
                map_state.notes_state.cursor_pos = previous_line_start + previous_line_length;
            }
        }
    }
}

pub fn move_cursor_down(map_state: &mut MapState) {
    if let Some(selected_note) = &map_state.notes_state.selected_note {
        if let Some(note) = map_state.notes_state.notes.get(selected_note) {
            // --- 1. Find the start of the current and next lines ---
            let mut current_line_start = 0;
            let mut next_line_start = 0;

            // Iterate through the lines to find the cursor's current line and the start of the next.
            for line in note.content.lines() {
                // The `if` condition checks if the cursor is on the current line being processed.
                // We use `next_line_start` for the check because it holds the starting index
                // of the line we are currently evaluating in the loop.
                if next_line_start + line.chars().count() >= map_state.notes_state.cursor_pos {
                    // Once we find the correct line, we perform one final update.
                    // `current_line_start` gets the correct value for the cursor's actual line.
                    current_line_start = next_line_start;
                    // `next_line_start` is pushed forward to the start of the *following* line.
                    next_line_start += line.chars().count() + 1;
                    // We've found what we need, so we exit the loop.
                    break;
                }

                // If the cursor isn't on this line, we update the variables for the next iteration.
                current_line_start = next_line_start;
                next_line_start += line.chars().count() + 1;
            }

            // --- 2. Handle the edge case of being on the last line ---

            // If the calculated `next_line_start` is beyond the total length of the note,
            // it means there is no next line to move to, so we exit early.
            if next_line_start > note.content.len() { return }

            // --- 3. Calculate the new cursor position ---

            // Determine the cursor's horizontal position (column) within its current line.
            let index_in_the_current_line = map_state.notes_state.cursor_pos - current_line_start;

            // To find the length of the next line, we first create a slice of the note
            // content starting from the beginning of the next line.
            let remaining_content = &note.content[next_line_start..];

            // We then search for a newline character within that remaining slice.
            let next_line_length = match remaining_content.find('\n') {
                // If a newline is found, its index within the slice is the length of the next line.
                Some(newline_pos) => newline_pos,
                // If no newline is found, it's the last line, so its length is the length of the entire remaining slice.
                None => remaining_content.len(),
            };

            // --- 4. Set the new cursor position, snapping if necessary ---

            // Check if the next line is long enough to place the cursor at the same column.
            if next_line_length > index_in_the_current_line {
                // If it is, the new position is the start of the next line plus the column offset.
                map_state.notes_state.cursor_pos = next_line_start + index_in_the_current_line;
            } else {
                // If the next line is shorter, "snap" the cursor to the end of that line.
                map_state.notes_state.cursor_pos = next_line_start + next_line_length;
            }
        }
    }
}
