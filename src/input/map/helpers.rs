use std::cmp::Reverse;
use ratatui::style::Color;

use crate::states::{MapState, map::Side};


/// Cycles to the next help page (1→2→3→4→5→1)
pub fn help_next_page(map_state: &mut MapState) {
    if let Some(current_page) = &mut map_state.ui_state.help_screen {
        map_state.ui_state.help_screen = Some(
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

/// Cycles to the previous help page (1→5→4→3→2→1)
pub fn help_previous_page(map_state: &mut MapState) {
    if let Some(current_page) = &mut map_state.ui_state.help_screen {
        map_state.ui_state.help_screen = Some(
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

/// Moves the viewport by a specified amount along the x or y axis.
/// Uses saturating subtraction to prevent underflow when moving in negative direction.
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

/// Moves the selected note and automatically pans the viewport to keep it visible.
/// 
/// Viewport follows the note when it would move beyond screen edges, creating a
/// smooth panning effect. Uses saturating subtraction to prevent coordinate underflow.
pub fn move_note(map_state: &mut MapState, axis: &str, amount: isize) {    
    if let Some(selected_note) = map_state.notes_state.selected_note {
        let (mut note_width, mut note_height) = if let Some(note) = map_state.notes_state.notes.get_mut(&selected_note) { 
                note.get_dimensions()
        } else {
            unreachable!()
        };
        // Enforce minimum size for readability
        if note_width < 20 { note_width = 20; }
        if note_height < 4 { note_height = 4; }
        note_width += 1; // Reserve space for cursor

        if let Some(note) = map_state.notes_state.notes.get_mut(&selected_note) {
            match axis {
                "x" => {
                    if amount > 0 {
                        note.x += amount as usize;
                        if note.x + note_width as usize > map_state.viewport.view_pos.x + map_state.viewport.screen_width {
                            map_state.viewport.view_pos.x += amount as usize;
                        }
                    } else {
                        note.x = note.x.saturating_sub(amount.abs() as usize);
                        if note.x < map_state.viewport.view_pos.x {
                            map_state.viewport.view_pos.x = map_state.viewport.view_pos.x.saturating_sub(amount.abs() as usize);
                        }
                    }
                }
                "y" => {
                    if amount > 0 {
                        note.y += amount as usize;
                        // Account for bottom info bar (3 lines) when checking visibility
                        if note.y as isize + note_height as isize > map_state.viewport.view_pos.y as isize + map_state.viewport.screen_height as isize - 3 {
                            map_state.viewport.view_pos.y += amount as usize;
                        }
                    } else {
                        note.y = note.y.saturating_sub(amount.abs() as usize);
                        if note.y < map_state.viewport.view_pos.y {
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

/// Switches focus to an adjacent note using vim-style directional navigation (h/j/k/l).
/// 
/// Uses a "cone of selection" algorithm: a note is only a candidate if the primary axis
/// distance exceeds the secondary axis distance. For example, moving right ('l') only
/// selects notes that are more to the right than they are up or down. This creates
/// intuitive, predictable navigation behavior.
/// 
/// When multiple candidates exist, selects the closest one along the primary axis,
/// with secondary axis used as a tiebreaker. Centers viewport on the newly selected note.
pub fn switch_notes_focus(map_state: &mut MapState, key: &str) {
    if let Some(selected_note) = map_state.notes_state.selected_note {
        // Copy coordinates to avoid borrowing conflicts during iteration
        let (selected_note_x, selected_note_y) = if let Some(note) = map_state.notes_state.notes.get(&selected_note) {
            (note.x, note.y)
        } else {
            return;
        };

        let candidate_ids: Vec<usize> = map_state.notes_state.notes.iter()
            .filter(|(id, note)| {
                let dx = (note.x as isize - selected_note_x as isize).abs();
                let dy = (note.y as isize - selected_note_y as isize).abs();

                // Cone of selection: primary axis distance must exceed secondary axis distance
                let is_in_direction = match key {
                    "j" | "Down" => note.y > selected_note_y && dy > dx,
                    "k" | "Up" => note.y < selected_note_y && dy > dx,
                    "l" | "Right" => note.x > selected_note_x && dx > dy,
                    "h" | "Left" => note.x < selected_note_x && dx > dy,
                    _ => false,
                };
            
                is_in_direction && **id != selected_note
            })
            .map(|(id, _)| *id)
            .collect();

        // Find closest note by primary axis, use secondary axis as tiebreaker
        let closest_note_id_option = match key {
            "j" | "Down" => {
                candidate_ids.iter().min_by_key(|&&id| {
                    let note = &map_state.notes_state.notes[&id];
                    let x_dist = (note.x as isize - selected_note_x as isize).abs() as usize;
                    (note.y, x_dist)
                })
            }
            "k" | "Up" => {
                candidate_ids.iter().max_by_key(|&&id| {
                    let note = &map_state.notes_state.notes[&id];
                    let x_dist = (note.x as isize - selected_note_x as isize).abs() as usize;
                    (note.y, Reverse(x_dist))
                })
            }
            "l" | "Right" => {
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
    
        if let Some(&id) = closest_note_id_option { 
            if let Some(note) = map_state.notes_state.notes.get_mut(&selected_note) {
                note.selected = false;
            }

            map_state.notes_state.selected_note = Some(id);

            // Move selected note to back of render order so it renders on top
            if let Some(pos) = map_state.notes_state.render_order.iter().position(|&x| x == id) {
                let item = map_state.notes_state.render_order.remove(pos);
                map_state.notes_state.render_order.push(item);
            }

            if let Some(note) = map_state.notes_state.notes.get_mut(&id) {
                note.selected = true;
            }

            if let Some(note) = map_state.notes_state.notes.get(&id) {
                map_state.viewport.view_pos.x = note.x.saturating_sub(map_state.viewport.screen_width/2);
                map_state.viewport.view_pos.y = note.y.saturating_sub(map_state.viewport.screen_height/2);
            }

            // Update connection endpoint if in visual connection mode
            if map_state.visual_mode.visual_connection {
                if let Some(focused_connection) = map_state.connections_state.focused_connection.as_mut() {
                    // Prevent self-connections
                    if id == focused_connection.from_id {
                        focused_connection.to_id = None;
                        focused_connection.to_side = None;
                    } else {
                        focused_connection.to_id = Some(id);
                        focused_connection.to_side = Some(map_state.settings.default_end_side);
                    }

                    map_state.persistence.mark_dirty();
                }
            }
        }
    }
}

/// Cycles through connection sides clockwise: Right → Bottom → Left → Top → Right
pub fn cycle_side(side: Side) -> Side {
    match side {
        Side::Right => Side::Bottom,
        Side::Bottom => Side::Left,
        Side::Left => Side::Top,
        Side::Top => Side::Right,
    }
}

/// Cycles through available colors. Returns white for unrecognized colors.
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