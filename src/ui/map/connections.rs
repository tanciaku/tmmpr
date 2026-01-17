
use ratatui::{
    Frame,
    style::Color,
};

use crate::{
    states::{
        MapState,
        map::{Mode, Note, Side},
    },
    ui::{DOUBLE_JUNCTIONS, IN_PROGRESS_CHARSET, NORMAL_CHARSET, PLAIN_JUNCTIONS, SegDir, THICK_JUNCTIONS},
    utils::{Point, calculate_path}
};


pub fn render_connections(frame: &mut Frame, map_state: &mut MapState) {

    for connection in &map_state.connections {
        if let Some(start_note) = map_state.notes.get(&connection.from_id){
            if let Some(end_note_id) = connection.to_id {
                if let Some(end_note) = map_state.notes.get(&end_note_id) {
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
                        let (p_x, p_y) = map_state.viewport.to_screen_coords(point.x, point.y);
                        p_x >= 0 && p_x < frame.area().width as isize && p_y >= 0 && p_y < frame.area().height as isize
                    });

                    // If no points in the path are within the visible screen area, skip
                    // the expensive drawing logic and move to the next connection.
                    if !is_visible {
                        continue
                    }

                    draw_connection(path, false, connection.color, frame, map_state);
                }
            }
        }
    }
        
    // Render the "in progress" connection, if any
    if let Some(focused_connection) = &map_state.focused_connection {

        if let Some(start_note) = map_state.notes.get(&focused_connection.from_id){

            if let Some(end_note_id) = focused_connection.to_id {
                if let Some(end_note) = map_state.notes.get(&end_note_id) {
                    let path = calculate_path(
                        start_note, 
                        focused_connection.from_side, 
                        end_note, 
                        focused_connection.to_side.unwrap(), // unwrap here, since if there is an
                                                             // end note - there is an end side
                    );

                    draw_connection(path, true, Color::Yellow, frame, map_state);
                }
            }
        }
    }
}

// `in_progess` is a bool argument for whether it is the "in progress" (of making/editing)
// connection being drawn.
pub fn draw_connection(path: Vec<Point>, in_progress: bool, color: Color, frame: &mut Frame, map_state: &MapState) {
    let connection_charset = if in_progress {
        &IN_PROGRESS_CHARSET
    } else {
        &NORMAL_CHARSET
    };

    // Draw the horizontal and vertical line segments that make
    // up a connection (.windows(2) - 2 points that make up a line)
    for points in path.windows(2) {
        // Translate first point absolute coordinates to screen coordinates
        let (p1_x, p1_y) = map_state.viewport.to_screen_coords(points[0].x, points[0].y);

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
        let (p_x, p_y) = map_state.viewport.to_screen_coords(points[1].x, points[1].y);

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
pub fn draw_connecting_character(note: &Note, side: Side, is_editing: bool, color: Color, frame: &mut Frame, map_state: &MapState) {
    // Set of connection characters for the selected note (depends on the current_mode)
    let connection_charset = if note.selected || is_editing {
        match map_state.current_mode {
            Mode::Visual => &THICK_JUNCTIONS,
            Mode::Edit(_) => &DOUBLE_JUNCTIONS,
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
    let (p_x, p_y) = map_state.viewport.to_screen_coords(p.0 as isize, p.1 as isize);

    if p_x >= 0 && p_x < frame.area().width as isize && p_y >= 0 && p_y < frame.area().height as isize {
        if let Some(cell) = frame.buffer_mut().cell_mut((p_x as u16, p_y as u16)) {
            cell.set_symbol(connection_point_character)
                .set_fg(color);
        }
    }
}