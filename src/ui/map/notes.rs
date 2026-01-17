
use ratatui::{
    Frame,
    layout::Position,
    prelude::Rect,
    style::Color,
    widgets::{Block, Borders, Clear, Paragraph, BorderType},
};

use crate::{
    states::{
        MapState,
        map::{Mode, SignedRect},
    }, ui::draw_connecting_character,
};

/// Renders the main canvas where notes are displayed.
///
/// This function iterates through all notes and performs a series of calculations
/// to determine if, where, and how each note should be rendered.
pub fn render_notes(frame: &mut Frame, map_state: &mut MapState) {
    // -- Render the notes in render order --
    for &note_id in &map_state.notes_state.render_order {
        if let Some(note) = map_state.notes_state.notes.get(&note_id) {
            // --- 1. Get Note Dimensions ---
            let (mut note_width, mut note_height) = note.get_dimensions();
            // Enforce a minimum size for readability.
            if note_width < 20 { note_width = 20; }
            if note_height < 4 { note_height = 4; } 
            // Add space for cursor
            note_width+=1;

            // --- 2. Translate to Screen Coordinates ---
            // Convert the note's absolute canvas coordinates into screen-relative coordinates.
            // This can result in negative values if the note is partially off-screen.
            let (p_x, p_y) = map_state.viewport.to_screen_coords(note.x as isize, note.y as isize);
            let note_rect = SignedRect {
                x: p_x,
                y: p_y,
                width: note_width as isize,
                height: note_height as isize,
            };

            // The frame itself is a rectangle starting at (0,0) in screen space.
            let frame_rect = SignedRect {
                x: 0,
                y: 0,
                width: frame.area().width as isize,
                height: frame.area().height as isize,
            };

            // --- 3. Clipping ---
            // Calculate the intersection between the note and the frame. If there's no
            // overlap, `intersection_result` will be `None`, and drawing the note will be skipped.
            let intersection_result = note_rect.intersection(&frame_rect);
            if let Some(visible_part) = intersection_result {
                // Convert the clipped, screen-space rectangle back to a `ratatui::Rect`.
                // The coordinates are guaranteed to be non-negative at this point.
                let note_area = Rect::new(
                    visible_part.x as u16,
                    visible_part.y as u16,
                    visible_part.width as u16,
                    visible_part.height as u16,
                );

                // --- 4. Calculate Text Scrolling ---
                // If the note was clipped on the top or left, we need to scroll the text content
                // to show the correct portion.
                let horizontal_scroll = (visible_part.x - note_rect.x) as u16;
                let vertical_scroll = (visible_part.y - note_rect.y) as u16;

                // --- 5. Determine Dynamic Borders ---
                // Show only the borders for the sides of the note that are not clipped.
                let mut borders = Borders::NONE;
                if note_rect.x == visible_part.x {
                    borders |= Borders::LEFT;
                }
                if note_rect.x + note_rect.width == visible_part.x + visible_part.width {
                    borders |= Borders::RIGHT;
                }
                if note_rect.y == visible_part.y {
                    borders |= Borders::TOP;
                }
                if note_rect.y + note_rect.height == visible_part.y + visible_part.height {
                    borders |= Borders::BOTTOM;
                }

                // --- 6. Determine border color  ---
                // (based on selection and mode)
                let border_color = if note.selected {
                    match map_state.current_mode {
                        Mode::Normal => Color::White,
                        Mode::Visual => Color::Yellow,
                        Mode::Edit(_) => Color::Blue,
                        Mode::Delete => Color::Red,
                    }
                } else {
                    note.color
                };

                // Determine border type
                let border_type = if note.selected {
                    match map_state.current_mode {
                        Mode::Normal => BorderType::Plain,
                        Mode::Visual => BorderType::Thick,
                        Mode::Edit(_) => BorderType::Double,
                        Mode::Delete => BorderType::Rounded,
                    }
                } else {
                    BorderType::Plain
                };

                // Create a block for the note with borders.
                let block = Block::default()
                    .borders(borders)
                    .border_style(border_color)
                    .border_type(border_type);

                // --- 7. Create the widget itself ---
                let text_widget = Paragraph::new(note.content.as_str())
                    .scroll((vertical_scroll, horizontal_scroll))
                    .block(block);

                // --- 8. Render the Widget(s) ---
                // With a stable rendering order, we can now clear any content 
                // from notes rendered beneath the note to be drawn and then draw it.
                frame.render_widget(Clear, note_area);
                frame.render_widget(text_widget, note_area);

                // -- 9. Render the cursor if in Edit Mode on the selected note ---
                // This logic only runs if the app is in Edit Mode AND the note currently being
                // drawn is the one that's actively selected.
                if let Some(selected_note) = &map_state.notes_state.selected_note {
                    if matches!(map_state.current_mode, Mode::Edit(_)) && note_id == *selected_note {

                        // To calculate the cursor's position, we first need a slice of the text
                        // from the beginning of the note's content up to the cursor's byte index.
                        let text_before_cursor = &note.content[..map_state.notes_state.cursor_pos];

                        // --- Calculate cursor's position RELATIVE to the text inside the note ---

                        // The Y position (row) is the number of newline characters before the cursor.
                        let cursor_y_relative = text_before_cursor.matches('\n').count();

                        // The X position (column) is the number of characters since the last newline.
                        let cursor_x_relative = match text_before_cursor.rfind('\n') {
                            // If a newline is found, the X position is the number of characters
                            // between that newline and the cursor. `c+1` to skip the newline itself.
                            Some(c) => {
                                text_before_cursor[c+1..map_state.notes_state.cursor_pos].chars().count()
                            }
                            // If no newline is found, we're on the first line. The X position is
                            // simply the total number of characters before the cursor.
                            None => { 
                                text_before_cursor[0..map_state.notes_state.cursor_pos].chars().count()
                            }
                        };

                        // --- Check if the calculated position is VISIBLE on screen ---
                        // The cursor is only visible if its calculated row is within the scrolled view.
                        // `cursor_y_relative` must be at or after the `vertical_scroll` offset.
                        // It must also be within the visible height of the note area.
                        // `note_area.height - 2` accounts for the top and bottom borders.
                        if cursor_y_relative >= vertical_scroll as usize 
                           && cursor_y_relative <= (note_area.height - 2) as usize {

                            // --- Translate relative coordinates to absolute screen coordinates ---
                            let final_cursor_x = note_area.x as usize // Start at the note's visible edge
                                + 1                                   // Add 1 for the left border
                                + cursor_x_relative                   // Add the cursor's column in the text
                                - horizontal_scroll as usize;         // Subtract any horizontal text scrolling

                            let final_cursor_y = note_area.y as usize // Start at the note's visible edge
                                + 1                                   // Add 1 for the top border
                                + cursor_y_relative                   // Add the cursor's row in the text
                                - vertical_scroll as usize;           // Subtract any vertical text scrolling

                            // Finally, place the cursor at the calculated position in the frame.
                            frame.set_cursor_position(Position::new(final_cursor_x as u16, final_cursor_y as u16));
                        }
                    }
                }

                // -- 10. Render this note's connecting characters --
                // To fix the visual bug where connection characters were drawn over the notes "above",
                // this logic now runs *after* each note is drawn. It looks up the note's id in the
                // connection_index hash map and loops through the connections associated with that id.
                // Draws the appropriate character (`┬`, `┴`, etc.) on top of the border.
                // This entire block is inside the `if let` for visible notes as a key
                // optimization, avoiding any of this work for off-screen notes.

                // Get the connections associated with the note's id
                if let Some(connection_vec) = map_state.connection_index.get(&note_id) {
                    // Loop through the connections in the connection vector that are 
                    // associated with the note's id
                    // NOTE: if there are multiple connections to the same side - it draws
                    //        the connecting character that many times (not a performance issue whatsoever tho)
                    for connection in connection_vec {
                        if note_id == connection.from_id {
                            draw_connecting_character(note, connection.from_side, false, border_color, frame, map_state);
                        } else {
                            draw_connecting_character(note, connection.to_side.unwrap(), false, border_color, frame, map_state);
                        }
                    }
                }
            }
        }
    }

    // Render the start/end point for the "in progress" connection, if any
    if let Some(connection) = &map_state.focused_connection {
    
        if let Some(start_note) = map_state.notes_state.notes.get(&connection.from_id){
            draw_connecting_character(start_note, connection.from_side, true, Color::Yellow, frame, map_state);

            if let Some(end_note_id) = connection.to_id {
                if let Some(end_note) = map_state.notes_state.notes.get(&end_note_id) {
                    draw_connecting_character(end_note, connection.to_side.unwrap(), true, Color::Yellow, frame, map_state);
                }
            }
        }
    }
}
