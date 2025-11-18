//! This module handles terminal events, focusing on keyboard input
//! to control the application's state and behavior.

use crate::app::{App, Mode};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::cmp::Reverse;

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
                    // Don't enter Visual Mode on the off-chance that there are no notes
                    if let None = app.notes.get(&app.selected_note) { return }

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
            // If Move State for Visual Mode is enabled 
            if app.visual_move {
                // Get the currently selected note.
                if let Some(note) = app.notes.get_mut(&app.selected_note) {

                    // Get note dimensions
                    let (mut note_width, mut note_height) = note.get_dimensions();
                    // Enforce a minimum size for readability.
                    if note_width < 20 { note_width = 20; }
                    if note_height < 4 { note_height = 4; }
                    // Add space for cursor
                    note_width+=1;
                    
                    match key.code {
                        // Switch back to Visual Mode Normal State
                        KeyCode::Char('m') => app.visual_move = false,

                        // Switch back to Normal Mode
                        KeyCode::Esc => {
                            app.current_mode = Mode::Normal;
                                note.selected = false;
                                app.visual_move = false;
                        }

                        // --- Moving the note ---

                        // Move the note up
                        KeyCode::Char('k') => {
                            // First, update the note's y-coordinate.
                            note.y = note.y.saturating_sub(1);
                            // Then, check if the top edge of the note is now above the top edge of the viewport.
                            if note.y < app.view_pos.y {
                                // If it is, move the viewport up by the same amount to "follow" the note.
                                app.view_pos.y -= 1;
                            }
                        }
                        KeyCode::Char('K') => {
                            note.y = note.y.saturating_sub(5);
                            if note.y < app.view_pos.y {
                                app.view_pos.y = app.view_pos.y.saturating_sub(5);
                            }
                        }

                        // Move the note down
                        KeyCode::Char('j') => {
                            // First, update the note's y-coordinate.
                            note.y += 1; 
                            // Check if the bottom edge of the note is below the visible screen area.
                            // We subtract 2 from the screen height to account for the bottom info bar.
                            if note.y as isize + note_height as isize > app.view_pos.y as isize + app.screen_height as isize - 2 {
                                // If it is, move the viewport down to keep the note in view.
                                app.view_pos.y += 1;
                            }
                        }
                        KeyCode::Char('J') => {
                            note.y += 5;
                            if note.y as isize + note_height as isize > app.view_pos.y as isize + app.screen_height as isize - 2 {
                                app.view_pos.y += 5;
                            }
                        }

                        // Move the note left
                        KeyCode::Char('h') => {
                            // First, update the note's x-coordinate.
                            note.x = note.x.saturating_sub(1);
                            // Then, check if the left edge of the note is now to the left of the viewport's edge.
                            if note.x < app.view_pos.x {
                                // If it is, move the viewport left to keep it in view.
                                app.view_pos.x -= 1;
                            }
                        }
                        KeyCode::Char('H') => {
                            note.x = note.x.saturating_sub(5);
                            if note.x < app.view_pos.x {
                                app.view_pos.x = app.view_pos.x.saturating_sub(5);
                            }
                        }

                        // Move the note right 
                        KeyCode::Char('l') => {
                            // First, update the note's x-coordinate.
                            note.x += 1;
                            // Check if the right edge of the note is past the right edge of the screen.
                            if note.x + note_width as usize > app.view_pos.x + app.screen_width {
                                // If it is, move the viewport right to keep up.
                                app.view_pos.x += 1;
                            }
                        }
                        KeyCode::Char('L') => {
                            note.x += 5;
                            if note.x + note_width as usize > app.view_pos.x + app.screen_width {
                                app.view_pos.x += 5;
                            }
                        }

                        _ => {}
                    }

                    // Trigger a redraw and stop there
                    app.clear_and_redraw(); 
                    return
                }
            }

            // If Visual Mode is in Normal State
            match key.code {
                // Switch back to Normal Mode
                KeyCode::Esc => {
                    if let Some(note) = app.notes.get_mut(&app.selected_note) {
                        app.current_mode = Mode::Normal;
                        note.selected = false;
                    }
                }
                // Switch to Insert mode
                KeyCode::Char('i') => app.current_mode = Mode::Insert,

                // Switch to Move State for the Visual Mode
                KeyCode::Char('m') => app.visual_move = true,

                // Switch to Delete Mode
                KeyCode::Char('d') => app.current_mode = Mode::Delete,

                // -- Switching focus between notes --

                // Switch focus to the closest note below the currently selected one
                KeyCode::Char('j') => { switch_notes_focus(app, 'j'); }
                // Above
                KeyCode::Char('k') => { switch_notes_focus(app, 'k'); }
                // Left
                KeyCode::Char('h') => { switch_notes_focus(app, 'h'); }
                // Right
                KeyCode::Char('l') => { switch_notes_focus(app, 'l'); }

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
    
        Mode::Delete => {
            match key.code {
                // Switch back to Visual Mode
                KeyCode::Esc => {
                    app.current_mode = Mode::Visual;
                }
                KeyCode::Char('d') => {
                    app.notes.remove(&app.selected_note);

                    // After deleting, update selected_note to a valid ID to prevent
                    // the application from retaining a stale reference. We'll pick
                    // the note with the highest ID as a predictable default.
                    // If no notes are left, it will default to 0.
                    app.selected_note = app.notes.keys().copied().max().unwrap_or(0);

                    app.current_mode = Mode::Normal;
                }
                _ => {}
            }
            
            app.clear_and_redraw();
        }
    }
}

fn switch_notes_focus(app: &mut App, key: char) {
    // --- 1. Get the starting position ---
    // Safely get the coordinates of the currently selected note.
    // We copy the `x` and `y` values into local variables so we are
    // no longer borrowing `app.notes`, which allows us to borrow it again later.
    let (selected_note_x, selected_note_y) = if let Some(note) = app.notes.get(&app.selected_note) {
        (note.x, note.y)
    } else {
        // If there's no selected note for some reason, we can't proceed.
        return;
    };

    // --- 2. Find all candidate notes ---
    // Use an iterator chain to declaratively find all valid notes to jump to.
    let candidate_ids: Vec<usize> = app.notes.iter()
        .filter(|(id, note)| {
            let dx = (note.x as isize - selected_note_x as isize).abs();
            let dy = (note.y as isize - selected_note_y as isize).abs();

            // This logic defines a "cone of selection" to find intuitive candidates.
            // A note is a valid candidate only if it's both in the correct direction
            // AND the distance along the primary axis of movement is greater than the
            // distance on the secondary axis. For example, when moving right ('l'),
            // a note is only a candidate if it is truly "more to the right" than it is
            // "up" or "down".

            // First, determine if the note is a valid candidate based on the direction.
            let is_in_direction = match key {
                // For vertical movement ('j'/'k'), the vertical distance must be greater.
                'j' => note.y > selected_note_y && dy > dx,
                'k' => note.y < selected_note_y && dy > dx,
                // For horizontal movement ('l'/'h'), the horizontal distance must be greater.
                'l' => note.x > selected_note_x && dx > dy,
                'h' => note.x < selected_note_x && dx > dy,
                // If an invalid character is passed, no notes will be candidates.
                _ => false,
            };
        
            // The final condition is: is it in the right direction AND not the note we started on?
            is_in_direction && **id != app.selected_note
        })
        // Only need the IDs.
        .map(|(id, _)| *id)
        .collect();

    // --- 3. Find the single best candidate ---
    let closest_note_id_option = match key {
        'j' => {
            // Find the closest note below
            candidate_ids.iter().min_by_key(|&&id| {
                let note = &app.notes[&id];
                // Calculate horizontal distance.
                let x_dist = (note.x as isize - selected_note_x as isize).abs() as usize;
            
                // The key is a tuple: `(vertical_position, horizontal_distance)`.
                // It will compare tuples element by element, find the note with the
                // minimum `y` value. If there's a tie, it will use `x_dist` to find the winner.
                (note.y, x_dist)
            })
        }
        'k' => {
            // Find the closest note above
            candidate_ids.iter().max_by_key(|&&id| {
                let note = &app.notes[&id];
                let x_dist = (note.x as isize - selected_note_x as isize).abs() as usize;
            
                (note.y, Reverse(x_dist))
            })
        }
        'l' => {
            // Find the closest note to the right
            candidate_ids.iter().min_by_key(|&&id| {
                let note = &app.notes[&id];
                let y_dist = (note.y as isize - selected_note_y as isize).abs() as usize;
                
                (note.x, y_dist)
            })
        }
        'h' => {
            candidate_ids.iter().max_by_key(|&&id| {
                let note = &app.notes[&id];
                let y_dist = (note.y as isize - selected_note_y as isize).abs() as usize;

                (note.x, Reverse(y_dist))
            })
        }
        _ => None,
    };
    

    // --- 4. Perform the selection switch and view update ---
    // This block only runs if `closest_note_id_option` is `Some`, meaning a note was found.
    if let Some(&id) = closest_note_id_option { 
        // First, deselect the old note. This mutable borrow is short-lived.
        if let Some(note) = app.notes.get_mut(&app.selected_note) {
            note.selected = false;
        }

        // Then, update the application's state to the new ID.
        app.selected_note = id;

        // Finally, select the new note. This is another, separate mutable borrow.
        if let Some(note) = app.notes.get_mut(&app.selected_note) {
            note.selected = true;
        }

        // As a final step, center the viewport on the newly selected note.
        if let Some(note) = app.notes.get(&app.selected_note) {
            app.view_pos.x = note.x.saturating_sub(app.screen_width/2);
            app.view_pos.y = note.y.saturating_sub(app.screen_height/2);
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