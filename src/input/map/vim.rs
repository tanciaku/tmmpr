use std::io::stdout;

use crossterm::{cursor::SetCursorStyle, execute};
use unicode_segmentation::UnicodeSegmentation;

use crate::states::{
    MapState,
    map::{Mode, NotesState},
};

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

    let last_grapheme_pos = note
        .data
        .content
        .grapheme_indices(true)
        .last()
        .map(|(idx, _)| idx)
        .unwrap_or(0);

    notes_state.set_cursor_pos(last_grapheme_pos);
}

/// Panics if no note is selected.
pub fn move_cursor_right_norm(notes_state: &mut NotesState) {
    let note = notes_state.expect_selected_note();
    let cursor_pos = notes_state.cursor_pos();

    let last_grapheme_pos = note
        .data
        .content
        .grapheme_indices(true)
        .last()
        .map(|(idx, _)| idx)
        .unwrap_or(0);

    let new_pos = note.data.content[cursor_pos..]
        .grapheme_indices(true)
        .nth(1)
        .map(|(idx, _)| cursor_pos + idx)
        .unwrap_or(cursor_pos);

    // Stop before last grapheme to match vim's normal mode behavior
    notes_state.set_cursor_pos(new_pos.min(last_grapheme_pos));
}

/// Panics if no note is selected.
pub fn append(map_state: &mut MapState) {
    let cursor_pos = map_state.notes_state.cursor_pos();
    let note = map_state.notes_state.expect_selected_note();

    // Advance by the byte length of the grapheme cluster at cursor_pos so the
    // new position is always on a valid char boundary, even for multi-byte clusters.
    let new_pos = note.data.content[cursor_pos..]
        .grapheme_indices(true)
        .nth(1)
        .map(|(idx, _)| cursor_pos + idx)
        .unwrap_or(note.data.content.len());

    map_state.notes_state.set_cursor_pos(new_pos);
    switch_to_modal_insert_mode(map_state);
}

/// Jumps cursor forward to the beginning of the next word (vim `w` behavior).
/// Treats spaces and newlines as word delimiters; crosses line boundaries.
///
/// If no next word exists the cursor stays at the last grapheme.
///
/// # Panics
/// If no note is selected.
pub fn jump_forward_a_word(notes_state: &mut NotesState) {
    let note = notes_state.expect_selected_note();
    let cursor_pos = notes_state.cursor_pos();

    if note.data.content.is_empty() {
        return;
    }

    // `idx` from grapheme_indices is relative to the tail slice, not the full string.
    let tail = &note.data.content[cursor_pos..];
    let mut graphemes = tail.grapheme_indices(true);

    // Phase 1: skip non-whitespace. Always advance at least one grapheme to make progress.
    let mut advanced = false;
    for (idx, g) in graphemes.by_ref() {
        let is_ws = g == " " || g == "\n";
        if is_ws && advanced {
            // The iterator already consumed this whitespace grapheme,
            // use its byte offset directly to start phase 2.
            let ws_start = cursor_pos + idx;

            // Phase 2: skip whitespace to the next word start.
            let after_ws = note.data.content[ws_start..]
                .grapheme_indices(true)
                .find(|(_, g)| *g != " " && *g != "\n")
                .map(|(i, _)| ws_start + i);

            if let Some(target) = after_ws {
                notes_state.set_cursor_pos(target);
            }
            // If no non-ws follows, stay put (already at last word).
            return;
        }
        advanced = true;
    }

    // No next word found; clamp to last grapheme.
    let last = note
        .data
        .content
        .grapheme_indices(true)
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0);
    notes_state.set_cursor_pos(last);
}

/// Jumps cursor backward to the beginning of the previous word (vim `b` behavior).
///
/// If the cursor is mid-word, lands on the start of the current word.
/// If already at a word boundary, skips back to the start of the previous word.
/// Treats spaces and newlines as delimiters; skips consecutive whitespace.
///
/// # Panics
/// If no note is selected.
pub fn jump_back_a_word(notes_state: &mut NotesState) {
    let note = notes_state.expect_selected_note();
    let cursor_pos = notes_state.cursor_pos();

    if note.data.content.is_empty() || cursor_pos == 0 {
        return;
    }

    // cursor_pos is a byte index; read the char immediately before it.
    let at_word_beginning = note.data.content[..cursor_pos]
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

fn find_current_word_start(note: &crate::states::map::Note, cursor_pos: usize) -> usize {
    let text_before_cursor = &note.data.content[..cursor_pos];
    let last_delimiter_pos = text_before_cursor.rfind(|c: char| c == ' ' || c == '\n');

    match last_delimiter_pos {
        Some(pos) => pos + 1,
        None => 0,
    }
}

/// Finds the start of the previous word when the cursor is at a word boundary.
///
/// All positions are byte indices into `note.data.content`.
fn find_previous_word_start(note: &crate::states::map::Note, cursor_pos: usize) -> usize {
    if cursor_pos == 0 {
        return 0;
    }

    // Skip backward over whitespace to find where the previous word ends.
    let text_before = &note.data.content[..cursor_pos];
    let word_end = text_before
        .trim_end_matches(|c: char| c == ' ' || c == '\n')
        .len();

    if word_end == 0 {
        return 0;
    }

    match note.data.content[..word_end].rfind(|c: char| c == ' ' || c == '\n') {
        // +1 skips the delimiter itself (space/newline are always single-byte in UTF-8).
        Some(pos) => pos + 1,
        None => 0,
    }
}

/// Panics if no note is selected.
pub fn remove_char(map_state: &mut MapState) {
    map_state.persistence.mark_dirty();

    let cursor_pos = map_state.notes_state.cursor_pos();
    let note = map_state.notes_state.expect_selected_note_mut();

    if note.data.content.is_empty() || cursor_pos >= note.data.content.len() {
        return;
    }

    // A grapheme cluster can span multiple bytes, so find its true end byte.
    let grapheme_end = note.data.content[cursor_pos..]
        .grapheme_indices(true)
        .nth(1)
        .map(|(idx, _)| cursor_pos + idx)
        .unwrap_or(note.data.content.len());

    note.data.content.drain(cursor_pos..grapheme_end);

    // In normal mode the cursor must not sit past the last grapheme cluster.
    let new_cursor_pos = if note.data.content.is_empty() {
        0
    } else {
        let last_grapheme_start = note
            .data
            .content
            .grapheme_indices(true)
            .last()
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        cursor_pos.min(last_grapheme_start)
    };

    map_state.notes_state.set_cursor_pos(new_cursor_pos);
}
