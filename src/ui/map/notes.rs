
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

/// Renders notes with proper clipping, scrolling, and z-ordering.
///
/// Notes are drawn in render order (z-index), with viewport culling and partial
/// visibility handling. Connection points are drawn after each note to prevent
/// visual layering issues.
pub fn render_notes(frame: &mut Frame, map_state: &mut MapState) {
    for &note_id in &map_state.notes_state.render_order {
        if let Some(note) = map_state.notes_state.notes.get(&note_id) {
            let (mut note_width, mut note_height) = note.get_dimensions();

            // FIXME:
            // Enforce minimum size for readability
            if note_width < 20 { note_width = 20; }
            if note_height < 4 { note_height = 4; }
            // Extra column needed to prevent cursor from overlapping border
            note_width += 1;

            // Convert canvas coordinates to screen space (can be negative if off-screen)
            let (p_x, p_y) = map_state.viewport.to_screen_coords(note.x as isize, note.y as isize);
            let note_rect = SignedRect {
                x: p_x,
                y: p_y,
                width: note_width as isize,
                height: note_height as isize,
            };

            let frame_rect = SignedRect {
                x: 0,
                y: 0,
                width: frame.area().width as isize,
                height: frame.area().height as isize,
            };

            // Skip notes completely outside the viewport
            if let Some(visible_part) = note_rect.intersection(&frame_rect) {
                // Coordinates are guaranteed non-negative after clipping to frame
                let note_area = Rect::new(
                    visible_part.x as u16,
                    visible_part.y as u16,
                    visible_part.width as u16,
                    visible_part.height as u16,
                );

                // Scroll text content to match clipped portion
                let horizontal_scroll = (visible_part.x - note_rect.x) as u16;
                let vertical_scroll = (visible_part.y - note_rect.y) as u16;

                // Show borders only for sides that are fully visible (not clipped)
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

                let block = Block::default()
                    .borders(borders)
                    .border_style(border_color)
                    .border_type(border_type);

                let text_widget = Paragraph::new(note.content.as_str())
                    .scroll((vertical_scroll, horizontal_scroll))
                    .block(block);

                // Clear before drawing to prevent artifacts from notes beneath this one
                frame.render_widget(Clear, note_area);
                frame.render_widget(text_widget, note_area);

                if let Some(selected_note) = &map_state.notes_state.selected_note {
                    if matches!(map_state.current_mode, Mode::Edit(_)) && note_id == *selected_note {
                        let text_before_cursor = &note.content[..map_state.notes_state.cursor_pos];

                        let cursor_y_relative = text_before_cursor.matches('\n').count();

                        let cursor_x_relative = match text_before_cursor.rfind('\n') {
                            Some(c) => {
                                text_before_cursor[c+1..map_state.notes_state.cursor_pos].chars().count()
                            }
                            None => {
                                text_before_cursor[0..map_state.notes_state.cursor_pos].chars().count()
                            }
                        };

                        // Only show cursor if it's within the scrolled visible area
                        // (note_area.height - 2) accounts for top and bottom borders
                        if cursor_y_relative >= vertical_scroll as usize 
                           && cursor_y_relative <= (note_area.height - 2) as usize {

                            let final_cursor_x = note_area.x as usize
                                + 1
                                + cursor_x_relative
                                - horizontal_scroll as usize;

                            let final_cursor_y = note_area.y as usize
                                + 1
                                + cursor_y_relative
                                - vertical_scroll as usize;

                            frame.set_cursor_position(Position::new(final_cursor_x as u16, final_cursor_y as u16));
                        }
                    }
                }

                // Draw connection characters after each note to prevent them covering
                // notes with higher z-index. Only done for visible notes as an optimization.
                if let Some(connection_vec) = map_state.connections_state.connection_index.get(&note_id) {
                    // NOTE: Multiple connections to the same side will redraw the character,
                    // but this has negligible performance impact
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

    // Highlight connection endpoints while user is creating a new connection
    if let Some(connection) = &map_state.connections_state.focused_connection {
    
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
