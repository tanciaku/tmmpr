use std::cmp::Reverse;
use ratatui::style::Color;

use crate::states::{MapState, map::Side};


/// Go forward a page in the help screen
pub fn help_next_page(map_state: &mut MapState) {
    if let Some(current_page) = &mut map_state.help_screen {
        map_state.help_screen = Some(
            match current_page {
                1 => 2,
                2 => 3,
                3 => 4,
                4 => 5,
                5 => 1,
                _ => unreachable!(),
        });
    }
}

/// Go back a page in the help screen
pub fn help_previous_page(map_state: &mut MapState) {
    if let Some(current_page) = &mut map_state.help_screen {
        map_state.help_screen = Some(
            match current_page {
                1 => 5,
                2 => 1,
                3 => 2,
                4 => 3,
                5 => 4,
                _ => unreachable!(),
        });
    }
}

pub fn move_viewport(map_state: &mut MapState, axis: &str, amount: isize) {
    match axis {
        "x" => {
            if amount > 0 {
                map_state.viewport.view_pos.x += amount as usize;
            } else {
                map_state.viewport.view_pos.x = map_state.viewport.view_pos.x.saturating_sub(amount.abs() as usize);
            }
        }
        "y" => {
            if amount > 0 {
                map_state.viewport.view_pos.y += amount as usize;
            } else {
                map_state.viewport.view_pos.y = map_state.viewport.view_pos.y.saturating_sub(amount.abs() as usize);
            }
        }
        _ => {}
    }
    
    map_state.persistence.mark_dirty();
}

pub fn move_note(map_state: &mut MapState, axis: &str, amount: isize) {    
    if let Some(selected_note) = map_state.notes_state.selected_note {
        // Get note dimensions for:
        // When a note moves beyond the screen edge, automatically adjust the viewport to keep it visible.
        let (mut note_width, mut note_height) = if let Some(note) = map_state.notes_state.notes.get_mut(&selected_note) { 
                note.get_dimensions()
        } else {
            unreachable!()
        };
        // Enforce a minimum size for readability.
        if note_width < 20 { note_width = 20; }
        if note_height < 4 { note_height = 4; }
        // Add space for cursor
        note_width+=1;

        if let Some(note) = map_state.notes_state.notes.get_mut(&selected_note) {
            match axis {
                "x" => {
                    if amount > 0 {
                        // First, update the note's x-coordinate.
                        note.x += amount as usize;
                        // Check if the right edge of the note is past the right edge of the screen.
                        if note.x + note_width as usize > map_state.viewport.view_pos.x + map_state.viewport.screen_width {
                            // If it is, move the viewport right to keep the note in view.
                            map_state.viewport.view_pos.x += amount as usize;
                        }
                    } else {
                        // First, update the note's x-coordinate.
                        note.x = note.x.saturating_sub(amount.abs() as usize);
                        // Then, check if the left edge of the note is now to the left of the viewport's edge.
                        if note.x < map_state.viewport.view_pos.x {
                            // If it is, move the viewport left to keep the note in view.
                            map_state.viewport.view_pos.x = map_state.viewport.view_pos.x.saturating_sub(amount.abs() as usize);
                        }
                    }
                }
                "y" => {
                    if amount > 0 {
                        // Update the note's y-coordinate.
                        note.y += amount as usize; 
                        // Check if the bottom edge of the note is below the visible screen area.
                        // We subtract 3 from the screen height to account for the bottom info bar.
                        if note.y as isize + note_height as isize > map_state.viewport.view_pos.y as isize + map_state.viewport.screen_height as isize - 3 {
                            // If it is, move the viewport down to keep the note in view.
                            map_state.viewport.view_pos.y += amount as usize;
                        }
                    } else {
                        // Update the note's y-coordinate.
                        note.y = note.y.saturating_sub(amount.abs() as usize);
                        // Then, check if the top edge of the note is now above the top edge of the viewport.
                        if note.y < map_state.viewport.view_pos.y {
                            // If it is, move the viewport down to up the note in view.
                            map_state.viewport.view_pos.y = map_state.viewport.view_pos.y.saturating_sub(amount.abs() as usize);
                        }
                    }
                }
                _ => {}
            }
            
            map_state.persistence.mark_dirty();
        }
    }
}

pub fn switch_notes_focus(map_state: &mut MapState, key: &str) {
    if let Some(selected_note) = map_state.notes_state.selected_note {
        // --- 1. Get the starting position ---
        // Safely get the coordinates of the currently selected note.
        // We copy the `x` and `y` values into local variables so we are
        // no longer borrowing `app.notes`, which allows us to borrow it again later.
        let (selected_note_x, selected_note_y) = if let Some(note) = map_state.notes_state.notes.get(&selected_note) {
            (note.x, note.y)
        } else {
            // If there's no selected note for some reason, we can't proceed.
            return;
        };

        // --- 2. Find all candidate notes ---
        // Use an iterator chain to declaratively find all valid notes to jump to.
        let candidate_ids: Vec<usize> = map_state.notes_state.notes.iter()
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
                    "j" | "Down" => note.y > selected_note_y && dy > dx,
                    "k" | "Up" => note.y < selected_note_y && dy > dx,
                    // For horizontal movement ('l'/'h'), the horizontal distance must be greater.
                    "l" | "Right" => note.x > selected_note_x && dx > dy,
                    "h" | "Left" => note.x < selected_note_x && dx > dy,
                    // If an invalid character is passed, no notes will be candidates.
                    _ => false,
                };
            
                // The final condition is: is it in the right direction AND not the note we started on?
                is_in_direction && **id != selected_note
            })
            // Only need the IDs.
            .map(|(id, _)| *id)
            .collect();

        // --- 3. Find the single best candidate ---
        let closest_note_id_option = match key {
            "j" | "Down" => {
                // Find the closest note below
                candidate_ids.iter().min_by_key(|&&id| {
                    let note = &map_state.notes_state.notes[&id];
                    // Calculate horizontal distance.
                    let x_dist = (note.x as isize - selected_note_x as isize).abs() as usize;
                
                    // The key is a tuple: `(vertical_position, horizontal_distance)`.
                    // It will compare tuples element by element, find the note with the
                    // minimum `y` value. If there's a tie, it will use `x_dist` to find the winner.
                    (note.y, x_dist)
                })
            }
            "k" | "Up" => {
                // Find the closest note above
                candidate_ids.iter().max_by_key(|&&id| {
                    let note = &map_state.notes_state.notes[&id];
                    let x_dist = (note.x as isize - selected_note_x as isize).abs() as usize;
                
                    (note.y, Reverse(x_dist))
                })
            }
            "l" | "Right" => {
                // Find the closest note to the right
                candidate_ids.iter().min_by_key(|&&id| {
                    let note = &map_state.notes_state.notes[&id];
                    let y_dist = (note.y as isize - selected_note_y as isize).abs() as usize;

                    (note.x, y_dist)
                })
            }
            "h" | "Left" => {
                candidate_ids.iter().max_by_key(|&&id| {
                    let note = &map_state.notes_state.notes[&id];
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
            if let Some(note) = map_state.notes_state.notes.get_mut(&selected_note) {
                note.selected = false;
            }

            // Then, update the application's state to the new ID.
            map_state.notes_state.selected_note = Some(id);

            // Update the render order
            // (put the just selected note's id to the back of the render_order vector -
            //      so it renders it over every other note "below")
            if let Some(pos) = map_state.notes_state.render_order.iter().position(|&x| x == id) {
                let item = map_state.notes_state.render_order.remove(pos);  // Remove from current position
                map_state.notes_state.render_order.push(item);              // Add to back
            }

            // Finally, select the new note. This is another, separate mutable borrow.
            if let Some(note) = map_state.notes_state.notes.get_mut(&id) {
                note.selected = true;
            }

            // As a final step, center the viewport on the newly selected note.
            if let Some(note) = map_state.notes_state.notes.get(&id) {
                map_state.viewport.view_pos.x = note.x.saturating_sub(map_state.viewport.screen_width/2);
                map_state.viewport.view_pos.y = note.y.saturating_sub(map_state.viewport.screen_height/2);
            }

            // If in the middle of creating a connection:
            if map_state.visual_mode.visual_connection {
                if let Some(focused_connection) = map_state.connections_state.focused_connection.as_mut() {
                    // only create a connection on note other than the start note
                    // (otherwise could have a connection going from start note to itself)
                    if id == focused_connection.from_id {
                        // if tried to make a connection (jumped to) from the start note
                        // to itself - just set the "to" fields to None (the default)
                        focused_connection.to_id = None;
                        focused_connection.to_side = None;
                    } else {
                        // update the `to_id` of "in-progress" connection to point to the newly found note.
                        focused_connection.to_id = Some(id); // id of the note that just jumped to
                        focused_connection.to_side = Some(map_state.settings.default_end_side); // default side
                    }

                    map_state.persistence.mark_dirty();
                }
            }
        }
    }
}

pub fn cycle_side(side: Side) -> Side {
    match side {
        Side::Right => Side::Bottom,
        Side::Bottom => Side::Left,
        Side::Left => Side::Top,
        Side::Top => Side::Right,
    }
}

pub fn cycle_color(color: Color) -> Color {
    match color {
        Color::Red => Color::Green,
        Color::Green => Color::Yellow,
        Color::Yellow => Color::Blue,
        Color::Blue => Color::Magenta,
        Color::Magenta => Color::Cyan,
        Color::Cyan => Color::White,
        Color::White => Color::Black,
        Color::Black => Color::Red,
        _ => Color::White,
    }
}