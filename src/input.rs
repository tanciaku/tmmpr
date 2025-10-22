//! This module handles terminal events, focusing on keyboard input
//! to control the application's state and behavior.

use crate::app::{App, Mode};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

/// Reads the terminal events.
pub fn handle_events(app: &mut App) -> Result<()> {
    // Poll for an event with a timeout of 50ms. This is the main "tick" rate.
    if event::poll(std::time::Duration::from_millis(50))? {
        // Read the event and dispatch to the appropriate handler.
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => on_key_event(app, key), // Handle keyboard input
            Event::Mouse(_) => {}
            // Redraw the UI if terminal window resized
            Event::Resize(_, _) => { app.needs_clear_and_redraw = true; }
            _ => {}
        }
    }
    Ok(())
}

/// Handles all keyboard input events and updates the application state accordingly.
///
/// This function is the central hub for all user commands. Its behavior is
/// determined by the application's current `Mode`.
fn on_key_event(app: &mut App, key: KeyEvent) {
    match app.current_mode {
        // Normal mode is for navigation and high-level commands.
        Mode::Normal => {
            match key.code {
                // --- Application Commands ---
                KeyCode::Char('q') => app.quit(),

                // --- Viewport Navigation ---
                // Move left
                KeyCode::Char('h') => app.view_pos.x = app.view_pos.x.saturating_sub(1),
                KeyCode::Char('H') => app.view_pos.x = app.view_pos.x.saturating_sub(5),
                // Move down
                KeyCode::Char('j') => app.view_pos.y += 1,
                KeyCode::Char('J') => app.view_pos.y += 5,
                // Move up
                KeyCode::Char('k') => app.view_pos.y = app.view_pos.y.saturating_sub(1),
                KeyCode::Char('K') => app.view_pos.y = app.view_pos.y.saturating_sub(5),
                // Move right 
                KeyCode::Char('l') => app.view_pos.x += 1,
                KeyCode::Char('L') => app.view_pos.x += 5,

                // --- Note Manipulation ---
                // Add note
                KeyCode::Char('a') => app.add_note(),
                // Select note (first selects closest to the center of the screen)
                KeyCode::Char('v') => {
                    app.select_note();
                    app.current_mode = Mode::Visual;
                }

                _ => {}
            }
            // Any action in Normal mode triggers a redraw.
            app.clear_and_redraw();
        }

        // Visual mode for selections.
        Mode::Visual => {
            match key.code {
                // Switch back to Normal Mode
                KeyCode::Esc => {
                    app.current_mode = Mode::Normal;
                    if let Some(note) = app.notes.get_mut(&app.selected_note) {
                        note.selected = false;
                    }
                }
                KeyCode::Char('i') => app.current_mode = Mode::Insert,
                _ => {}
            }
            // Any action in Visual mode triggers a redraw.
            app.clear_and_redraw(); 
        }

        // Insert mode is for editing the content of a note.
        Mode::Insert => {
            match key.code {
                // Switch back to Normal Mode
                KeyCode::Esc => {
                    app.current_mode = Mode::Normal;
                    if let Some(note) = app.notes.get_mut(&app.selected_note) {
                        note.selected = false;
                        // Reset cursor position for the next time entering Insert mode.
                        app.cursor_pos = 0;
                    }
                }

                // --- Text Editing ---
                KeyCode::Char(c) => {
                    if let Some(note) = app.notes.get_mut(&app.selected_note) {
                        // Insert the typed character at the cursor's current position.
                        note.content.insert(app.cursor_pos, c);
                        // Move the cursor forward one position.
                        app.cursor_pos += 1;
                    }
                }
                KeyCode::Enter => {
                    if let Some(note) = app.notes.get_mut(&app.selected_note) {
                        // Insert a newline character at the cursor's position.
                        note.content.insert(app.cursor_pos, '\n');
                        // Move the cursor forward one position.
                        app.cursor_pos += 1;
                    }
                }
                KeyCode::Backspace => {
                    if let Some(note) = app.notes.get_mut(&app.selected_note) {
                        // We can only backspace if the cursor is not at the very beginning of the text.
                        if app.cursor_pos > 0 {
                            // To delete the character *before* the cursor, we must remove the character
                            // at the index `cursor_pos - 1`.
                            note.content.remove(app.cursor_pos - 1);
                            // After removing the character, we move the cursor's position back by one.
                            app.cursor_pos -= 1;
                        }
                    }
                }
                KeyCode::Left => {
                    if app.cursor_pos > 0 { 
                        app.cursor_pos -= 1 
                    }
                }
                KeyCode::Right => {
                    if let Some(note) = app.notes.get(&app.selected_note) {
                        if app.cursor_pos < note.content.len() {
                            app.cursor_pos += 1;
                        }
                    }
                }
                KeyCode::Up => move_cursor_up(app), 
                KeyCode::Down => move_cursor_down(app),
                _ => {}
            }
            // Any action in Insert mode triggers a redraw.
            app.clear_and_redraw();
        }
    }
}

fn move_cursor_up(app: &mut App) {
    if let Some(note) = app.notes.get(&app.selected_note) {
        // --- 1. Find the start of the current and previous lines ---

        // `current_line_start` will hold the starting index of the line the cursor is on.
        let mut current_line_start = 0;
        // `previous_line_start` will hold the starting index of the line *above* the cursor.
        let mut previous_line_start = 0;

        // Iterate through the lines of the note to find the cursor's position.
        for line in note.content.lines() {
            // Check if the end of the current line is past the cursor's position.
            // If it is, we've found the line the cursor is on.
            if current_line_start + line.chars().count() >= app.cursor_pos {
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
        let index_in_the_current_line = app.cursor_pos - current_line_start;

        // Calculate the character length of the previous line.
        let previous_line_length = current_line_start - previous_line_start - 1;

        // --- 4. Set the new cursor position, snapping if necessary ---
        
        // Check if the previous line is long enough to place the cursor at the same column.
        if previous_line_length > index_in_the_current_line {
            // If it is, the new position is the start of the previous line plus the column offset.
            app.cursor_pos = previous_line_start + index_in_the_current_line;
        } else {
            // If the previous line is shorter, "snap" the cursor to the end of that line.
            app.cursor_pos = previous_line_start + previous_line_length;
        }
    }
}

fn move_cursor_down(app: &mut App) {
    if let Some(note) = app.notes.get(&app.selected_note) {
        // --- 1. Find the start of the current and next lines ---
        let mut current_line_start = 0;
        let mut next_line_start = 0;

        // Iterate through the lines to find the cursor's current line and the start of the next.
        for line in note.content.lines() {
            // The `if` condition checks if the cursor is on the current line being processed.
            // We use `next_line_start` for the check because it holds the starting index
            // of the line we are currently evaluating in the loop.
            if next_line_start + line.chars().count() >= app.cursor_pos {
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
        let index_in_the_current_line = app.cursor_pos - current_line_start;

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
            app.cursor_pos = next_line_start + index_in_the_current_line;
        } else {
            // If the next line is shorter, "snap" the cursor to the end of that line.
            app.cursor_pos = next_line_start + next_line_length;
        }
    }
}