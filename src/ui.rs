//! This module is responsible for all rendering logic of the application.
//! It takes the application state (`App`) and a `ratatui` frame, and draws the UI.

use crate::app::{App, Mode, SignedRect, Note, Side};
use crate::utils::{calculate_path, Point};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Position},
    prelude::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Padding, Paragraph, BorderType},
    Frame
};

const IN_PROGRESS_CHARSET: [&str; 6] = ["━", "┃", "┏", "┓", "┗", "┛"];
const NORMAL_CHARSET: [&str; 6] = ["─", "│", "┌", "┐", "└", "┘"];

const PLAIN_JUNCTIONS: [&str; 4] = ["┴", "┬", "┤", "├"];
const THICK_JUNCTIONS: [&str; 4] = ["┻", "┳", "┫", "┣"];
const DOUBLE_JUNCTIONS: [&str; 4] = ["╩", "╦", "╣", "╠"];

/// The main rendering entry point.
///
/// This function orchestrates the entire rendering process for a single frame.
/// It updates the app state with the current screen dimensions and then calls
/// the specific rendering functions for different parts of the UI.
pub fn render(frame: &mut Frame, app: &mut App) {
    // Clear the frame before drawing anything new.
    frame.render_widget(Clear, frame.area());

    // Update the app state with the current terminal size. This is crucial for
    // calculations that depend on screen dimensions, like centering new notes.
    app.screen_width = frame.area().width as usize;
    app.screen_height = frame.area().height as usize;

    // Render the main UI components.
    render_connections(frame, app);
    render_map(frame, app); // Notes will be drawn over connections (if any)
    render_bar(frame, app); // The bar will be drawn over everything
}

/// Renders the bottom information bar.
///
/// This bar displays debugging information and the current application state,
/// such as viewport position, mode, and selected note.
fn render_bar(frame: &mut Frame, app: &App) {

    // Get the total available screen area.
    let size = frame.area();

    // Determine the display text and color for the current application mode.
    let (mode_text, mode_text_color) = match &app.current_mode {
        Mode::Normal => (String::from("[ NORMAL ]"), Style::new().fg(Color::White)),
        Mode::Visual => {
            if app.visual_move {
                (String::from("[ VISUAL (MOVE) ]"), Style::new().fg(Color::Yellow))
            } else if app.visual_connection {
                (String::from("[ VISUAL (CONNECTION) ]"), Style::new().fg(Color::Yellow))
            } else {
                (String::from("[ VISUAL ]"), Style::new().fg(Color::Yellow))
            }
        }
        Mode::Insert => (String::from("[ INSERT ]"), Style::new().fg(Color::Blue)),
        Mode::Delete => (String::from("Delete the selected note [d]            Go back to Visual Mode [ESC]"), Style::new().fg(Color::Red)),
    };

    // --- Left-Aligned Widget: Mode Display ---
    // Create a Paragraph for the mode, styling it with the color determined above.
    // It's aligned to the left and given some padding.
    let mode_display = Paragraph::new(format!("\n{}", mode_text))
        .style(mode_text_color)
        .alignment(Alignment::Left)
        .block(Block::default().padding(Padding::new(2, 0, 0, 0)));

    // --- Right-Aligned Widget: View Position ---
    // Create a separate Paragraph to show the viewport's x/y coordinates.
    // This is aligned to the right, with padding on the right side.
    let view_position_display = Paragraph::new(format!(
        "\nView: {},{}",
        app.view_pos.x,
        app.view_pos.y,
    ))
    .alignment(Alignment::Right)
    .block(Block::default().padding(Padding::new(0, 2, 0, 0)));
    
    // Define the rectangular area for the entire bottom bar.
    let bottom_rect = Rect {
        x: size.x,
        y: size.height - 2, // Position it in the last two rows of the terminal.
        width: size.width,
        height: 2,
    };

    // --- Layout Management ---
    // Split the `bottom_rect` into two equal horizontal chunks.
    // This prevents the styling of one widget from affecting the other.
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left side gets 50% of the width.
            Constraint::Percentage(50), // Right side gets the other 50%.
        ])
        .split(bottom_rect);
    
    // Assign the created chunks to variables for clarity.
    let left_bar = chunks[0];
    let right_bar = chunks[1];

    // --- Rendering ---
    // Finally, render the widgets to the frame.
    frame.render_widget(Clear, bottom_rect); // First, clear the entire bar area.
    frame.render_widget(mode_display, left_bar); // Render the mode display to the left chunk.
    frame.render_widget(view_position_display, right_bar); // Render the position to the right chunk.
}

/// Renders the main canvas where notes are displayed.
///
/// This function iterates through all notes and performs a series of calculations
/// to determine if, where, and how each note should be rendered.
fn render_map(frame: &mut Frame, app: &App) {
    // To ensure consistent rendering and prevent flickering, we must draw notes
    // in a stable order. A HashMap's iterator is not guaranteed to be stable,
    // so we collect the notes into a Vec and sort them by their ID (the key).
    // This acts as a stable z-index, where higher IDs are drawn on top.
    let mut sorted_notes: Vec<_> = app.notes.iter().collect();
    sorted_notes.sort_by_key(|(id, _note)| **id);

    // -- Render the notes --
    for (id, note) in sorted_notes {
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
        let note_rect = SignedRect {
            x: note.x as isize - app.view_pos.x as isize,
            y: note.y as isize - app.view_pos.y as isize,
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
                match app.current_mode {
                    Mode::Normal => Color::White,
                    Mode::Visual => Color::Yellow,
                    Mode::Insert => Color::Blue,
                    Mode::Delete => Color::Red,
                }
            } else {
                Color::White
            };

            // Determine border type
            let border_type = if note.selected {
                match app.current_mode {
                    Mode::Normal => BorderType::Plain,
                    Mode::Visual => BorderType::Thick,
                    Mode::Insert => BorderType::Double,
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

            // -- 9. Render the cursor if in Insert Mode on the selected note ---
            // This logic only runs if the app is in Insert mode AND the note currently being
            // drawn is the one that's actively selected.
            if matches!(app.current_mode, Mode::Insert) && *id == app.selected_note {

                // To calculate the cursor's position, we first need a slice of the text
                // from the beginning of the note's content up to the cursor's byte index.
                let text_before_cursor = &note.content[..app.cursor_pos];

                // --- Calculate cursor's position RELATIVE to the text inside the note ---

                // The Y position (row) is the number of newline characters before the cursor.
                let cursor_y_relative = text_before_cursor.matches('\n').count();

                // The X position (column) is the number of characters since the last newline.
                let cursor_x_relative = match text_before_cursor.rfind('\n') {
                    // If a newline is found, the X position is the number of characters
                    // between that newline and the cursor. `c+1` to skip the newline itself.
                    Some(c) => {
                        text_before_cursor[c+1..app.cursor_pos].chars().count()
                    }
                    // If no newline is found, we're on the first line. The X position is
                    // simply the total number of characters before the cursor.
                    None => { 
                        text_before_cursor[0..app.cursor_pos].chars().count()
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
            
            // -- 10. Render this note's connecting characters --
            // To fix the visual bug where connection characters were drawn over the notes "above",
            // this logic now runs *after* each note is drawn. It iterates through all
            // connections and checks if the current note is a part of any of them. If it
            // is, it draws the appropriate character (`┬`, `┴`, etc.) on top of the border.
            // This entire block is inside the `if let` for visible notes as a key
            // optimization, avoiding any of this work for off-screen notes.
            //
            // NOTE: This approach of iterating through all connections for every visible
            // note is not the most performant solution, but it is acceptable for now.
            // The work inside the loop is minimal (mostly cheap `if` checks), so the
            // performance impact should be negligible for a reasonable number of notes.
            // A truly optimized solution would require a more complex data structure for
            // connections (like a HashMap for O(1) lookups). This can be revisited if
            // it ever becomes a noticeable bottleneck in the future.

            // Draw the connecting characters. (A note can have multiple connecting characters)
            for connection in &app.connections {
                if let Some(start_note) = app.notes.get(&connection.from_id){
                    // Check if the current note is the *starting* point of the connection.
                    // If it is, draw the character on the "from" side.
                    if *id == connection.from_id {
                        draw_connecting_character(start_note, connection.from_side, false, border_color, frame, app);
                    }

                    // Then, check if the current note is the *ending* point of the connection.
                    // If it is, draw the character on the "to" side.
                    if let Some(end_note_id) = connection.to_id {
                        if let Some(end_note) = app.notes.get(&end_note_id) {
                            if *id == end_note_id {
                                draw_connecting_character(end_note, connection.to_side.unwrap(), false, border_color, frame, app);
                            }
                        }
                    }
                }
            }
        }
    }

    // Render the start/end point for the "in progress" connection, if any
    if let Some(connection) = &app.focused_connection {
    
        if let Some(start_note) = app.notes.get(&connection.from_id){
            draw_connecting_character(start_note, connection.from_side, true, Color::Yellow, frame, app);

            if let Some(end_note_id) = connection.to_id {
                if let Some(end_note) = app.notes.get(&end_note_id) {
                    draw_connecting_character(end_note, connection.to_side.unwrap(), true, Color::Yellow, frame, app);
                }
            }
        }
    }
}

fn render_connections(frame: &mut Frame, app: &App) {

    for connection in &app.connections {
        if let Some(start_note) = app.notes.get(&connection.from_id){
            if let Some(end_note_id) = connection.to_id {
                if let Some(end_note) = app.notes.get(&end_note_id) {
                    let path = calculate_path(
                        start_note, 
                        connection.from_side, 
                        end_note, 
                        connection.to_side.unwrap(), // unwrap here, since if there is an
                                                     // end note - there is an end side
                    );

                    // For optimization, quickly check if the connection is visible before attempting
                    // to draw it. This avoids iterating over every cell of connections that are
                    // completely off-screen.
                    // The `.any()` iterator is efficient, stopping as soon as the first visible
                    // point is found.
                    let is_visible = path.iter().any(|point| {
                        let p_x = point.x - app.view_pos.x as isize;
                        let p_y = point.y - app.view_pos.y as isize;
                        p_x >= 0 && p_x < frame.area().width as isize && p_y >= 0 && p_y < frame.area().height as isize
                    });

                    // If no points in the path are within the visible screen area, skip
                    // the expensive drawing logic and move to the next connection.
                    if !is_visible {
                        continue
                    }

                    draw_connection(path, false, Color::White, frame, app);
                }
            }
        }
    }
        
    // Render the "in progress" connection, if any
    if let Some(focused_connection) = &app.focused_connection {

        if let Some(start_note) = app.notes.get(&focused_connection.from_id){

            if let Some(end_note_id) = focused_connection.to_id {
                if let Some(end_note) = app.notes.get(&end_note_id) {
                    let path = calculate_path(
                        start_note, 
                        focused_connection.from_side, 
                        end_note, 
                        focused_connection.to_side.unwrap(), // unwrap here, since if there is an
                                                             // end note - there is an end side
                    );

                    draw_connection(path, true, Color::Yellow, frame, app);
                }
            }
        }
    }
}

// * bool argument for whether it is the th
fn draw_connection(path: Vec<Point>, in_progress: bool, color: Color, frame: &mut Frame, app: &App) {
    let connection_charset = if in_progress {
        &IN_PROGRESS_CHARSET
    } else {
        &NORMAL_CHARSET
    };

    // Draw the horizontal and vertical line segments that make
    // up a connection (.windows(2) - 2 points that make up a line)
    for points in path.windows(2) {
        // Translate first point absolute coordinates to screen coordinates
        let p1_x = points[0].x - app.view_pos.x as isize;
        let p1_y = points[0].y - app.view_pos.y as isize;

        // -- Determine line characters and draw them --
        
        // If the difference is in x coordinates - draw horizontal segment characters
        if points[0].x != points[1].x {
            let x_diff = (points[1].x - points[0].x).abs(); // difference in point 1 to point 2
            let mut x_coor: isize;
            
            for offset in 0..x_diff {
                if points[1].x > points[0].x { // +difference (going right)
                    x_coor = p1_x + offset;
                } else { // -difference (going left)
                    x_coor = p1_x - offset;
                }

                if x_coor >= 0 && x_coor < frame.area().width as isize && p1_y >= 0 && p1_y < frame.area().height as isize {
                    if let Some(cell) = frame.buffer_mut().cell_mut((x_coor as u16, p1_y as u16)) {
                        cell.set_symbol(connection_charset[0])
                            .set_fg(color);
                    }
                }
            }
        } else { // If the difference is in y coordinates - draw vertical segment characters                        
            let y_diff = (points[1].y - points[0].y).abs(); // difference in point 1 to point 2
            let mut y_coor: isize;

            for offset in 0..y_diff {
                if points[1].y > points[0].y { // +difference (going down)
                    y_coor = p1_y + offset;
                } else { // -difference (going up)
                    y_coor = p1_y - offset;
                }
                
                if y_coor >= 0 && y_coor < frame.area().height as isize && p1_x >= 0 && p1_x < frame.area().width as isize {
                    if let Some(cell) = frame.buffer_mut().cell_mut((p1_x as u16, y_coor as u16)) {
                        cell.set_symbol(connection_charset[1])
                            .set_fg(color);
                    }
                }
            }
        }
    }

    // -- Determine segment directions --
    // (p1->p2, p2->p3, ...)
    // Used later to determine corner characters (┌ ┐ └ ┘)
    let mut segment_directions: Vec<SegDir> = vec![];

    for points in path.windows(2) {
        if points[0].x != points[1].x { // horizontal difference
            if points[1].x > points[0].x { // +difference (going right)
                segment_directions.push(SegDir::Right);
            } else { // -difference (going left)
                segment_directions.push(SegDir::Left);
            }
        } else if points[0].y != points[1].y { // vertical difference
            if points[1].y > points[0].y { // +difference (going down)
                segment_directions.push(SegDir::Down);
            } else { // -difference (going up)
                segment_directions.push(SegDir::Up);
            } 
        } else { // no difference, difference on the same axis
            if let Some(last_direction) = segment_directions.last() {
                segment_directions.push(*last_direction); 
            }
        }
    }

    // -- Draw the corner characters for segments --
    for (i, points) in path.windows(3).enumerate() {
        // Translate points absolute coordinates to screen coordinates
        // points[1] - to draw every 2nd point, so all besides the first and last [1, 0, 0, 0, 1]
        let p_x = points[1].x - app.view_pos.x as isize;
        let p_y = points[1].y - app.view_pos.y as isize;

        let incoming = segment_directions[i];
        let outgoing = segment_directions[i + 1];
        
        let corner_character = match (incoming, outgoing) {
            // ┌
            (SegDir::Left, SegDir::Down) => { connection_charset[2] }
            (SegDir::Up, SegDir::Right) => { connection_charset[2] }
            // ┐
            (SegDir::Right, SegDir::Down) => { connection_charset[3] }
            (SegDir::Up, SegDir::Left) => { connection_charset[3] }
            // └
            (SegDir::Down, SegDir::Right) => { connection_charset[4] }
            (SegDir::Left, SegDir::Up) => { connection_charset[4] }
            // ┘
            (SegDir::Down, SegDir::Left) => { connection_charset[5] }
            (SegDir::Right, SegDir::Up) => { connection_charset[5] }
            // ─
            (SegDir::Left, SegDir::Left) => { connection_charset[0] }
            (SegDir::Left, SegDir::Right) => { connection_charset[0] }
            (SegDir::Right, SegDir::Right) => { connection_charset[0] }
            (SegDir::Right, SegDir::Left) => { connection_charset[0] }
            // │
            (SegDir::Up, SegDir::Up) => { connection_charset[1] }
            (SegDir::Up, SegDir::Down) => { connection_charset[1] }
            (SegDir::Down, SegDir::Down) => { connection_charset[1] }
            (SegDir::Down, SegDir::Up) => { connection_charset[1] }
        };

        if p_x >= 0 && p_x < frame.area().width as isize && p_y >= 0 && p_y < frame.area().height as isize {
            if let Some(cell) = frame.buffer_mut().cell_mut((p_x as u16, p_y as u16)) {
                cell.set_symbol(corner_character)
                    .set_fg(color);
            }
        }
    }
}

// `is_editing` argument is to determine whether the function is called from the
// block that is responosible for drawing the "in progress" connection (being made or edited)
fn draw_connecting_character(note: &Note, side: Side, is_editing: bool, color: Color, frame: &mut Frame, app: &App) {
    // Set of connection characters for the selected note (depends on the current_mode)
    let connection_charset = if note.selected || is_editing {
        match app.current_mode {
            Mode::Visual => &THICK_JUNCTIONS,
            Mode::Insert => &DOUBLE_JUNCTIONS,
            // For Normal and Delete, we use the plain set
            _ => &PLAIN_JUNCTIONS,
        }
    } else { // Default set of connection characters (if note or the connection is not selected)
        &PLAIN_JUNCTIONS
    };

    let connection_point_character = match side {
        Side::Top => connection_charset[0],
        Side::Bottom => connection_charset[1],
        Side::Left => connection_charset[2],
        Side::Right => connection_charset[3],
    };

    let p = note.get_connection_point(side);
    let p_x = p.0 as isize - app.view_pos.x as isize; // connection start point relative x
    let p_y = p.1 as isize - app.view_pos.y as isize; // connection start point relative y

    if p_x >= 0 && p_x < frame.area().width as isize && p_y >= 0 && p_y < frame.area().height as isize {
        if let Some(cell) = frame.buffer_mut().cell_mut((p_x as u16, p_y as u16)) {
            cell.set_symbol(connection_point_character)
                .set_fg(color);
        }
    }
}

// Which direction the segment is going
// Used to determine which corner character to draw
#[derive(Copy, Clone)]
enum SegDir {
    Right, // Horizontal going Right
    Left, // Horizontal going Left
    Up, // Vertical going Up
    Down, // Vertical going Down
}