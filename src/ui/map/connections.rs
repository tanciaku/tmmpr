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

    for connection in map_state.connections_state.connections() {
        if let Some(start_note) = map_state.notes_state.notes().get(&connection.from_id){
            if let Some(end_note_id) = connection.to_id {
                if let Some(end_note) = map_state.notes_state.notes().get(&end_note_id) {
                    let path = calculate_path(
                        start_note, 
                        connection.from_side, 
                        end_note, 
                        connection.to_side.unwrap(), // Safe: to_side guaranteed present when to_id is Some
                    );

                    // Optimization: skip off-screen connections to avoid expensive per-cell iteration.
                    // `.any()` short-circuits on first visible point.
                    let is_visible = path.iter().any(|point| {
                        let (p_x, p_y) = map_state.viewport.to_screen_coords(point.x, point.y);
                        p_x >= 0 && p_x < frame.area().width as isize && p_y >= 0 && p_y < frame.area().height as isize
                    });

                    if !is_visible {
                        continue
                    }

                    draw_connection(path, false, connection.color, frame, map_state);
                }
            }
        }
    }
        
    // Render focused connection being created/edited
    if let Some(focused_connection) = &map_state.connections_state.focused_connection {

        if let Some(start_note) = map_state.notes_state.notes().get(&focused_connection.from_id){

            if let Some(end_note_id) = focused_connection.to_id {
                if let Some(end_note) = map_state.notes_state.notes().get(&end_note_id) {
                    let path = calculate_path(
                        start_note, 
                        focused_connection.from_side, 
                        end_note, 
                        focused_connection.to_side.unwrap(), // Safe: to_side guaranteed present when to_id is Some
                    );

                    draw_connection(path, true, Color::Yellow, frame, map_state);
                }
            }
        }
    }
}

/// Draws a connection path on the screen.
/// `in_progress`: if true, uses special charset to indicate connection being created/edited
pub fn draw_connection(path: Vec<Point>, in_progress: bool, color: Color, frame: &mut Frame, map_state: &MapState) {
    let connection_charset = if in_progress {
        &IN_PROGRESS_CHARSET
    } else {
        &NORMAL_CHARSET
    };

    // Draw horizontal and vertical line segments (path split into pairs of points)
    for points in path.windows(2) {
        let (p1_x, p1_y) = map_state.viewport.to_screen_coords(points[0].x, points[0].y);
        
        if points[0].x != points[1].x {
            let x_diff = (points[1].x - points[0].x).abs();
            let mut x_coor: isize;
            
            for offset in 0..x_diff {
                if points[1].x > points[0].x {
                    x_coor = p1_x + offset;
                } else {
                    x_coor = p1_x - offset;
                }

                if x_coor >= 0 && x_coor < frame.area().width as isize && p1_y >= 0 && p1_y < frame.area().height as isize {
                    if let Some(cell) = frame.buffer_mut().cell_mut((x_coor as u16, p1_y as u16)) {
                        cell.set_symbol(connection_charset[0])
                            .set_fg(color);
                    }
                }
            }
        } else {
            let y_diff = (points[1].y - points[0].y).abs();
            let mut y_coor: isize;

            for offset in 0..y_diff {
                if points[1].y > points[0].y {
                    y_coor = p1_y + offset;
                } else {
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

    // Calculate segment directions for each pair of points.
    // Used to determine which corner character (┌ ┐ └ ┘) to draw at path bends.
    let mut segment_directions: Vec<SegDir> = vec![];

    for points in path.windows(2) {
        if points[0].x != points[1].x {
            if points[1].x > points[0].x {
                segment_directions.push(SegDir::Right);
            } else {
                segment_directions.push(SegDir::Left);
            }
        } else if points[0].y != points[1].y {
            if points[1].y > points[0].y {
                segment_directions.push(SegDir::Down);
            } else {
                segment_directions.push(SegDir::Up);
            } 
        } else {
            // Points are identical; continue in same direction to avoid corner artifacts
            if let Some(last_direction) = segment_directions.last() {
                segment_directions.push(*last_direction); 
            }
        }
    }

    // Draw corner characters at path bends (skip first and last points)
    for (i, points) in path.windows(3).enumerate() {
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

/// Draws the connection point character at the specified side of a note.
/// `is_editing`: true when drawing connection being created/edited

// use connection from and to id's for the argument

pub fn draw_connecting_character(note: &Note, note_id: usize, side: Side, is_editing: bool, color: Color, frame: &mut Frame, map_state: &MapState) {
    // Visual style varies based on mode: thick for Visual, double for Edit, plain otherwise
    let connection_charset = match map_state.notes_state.selected_note_id() {
        Some(selected_note_id) if selected_note_id == note_id || is_editing => {
            match map_state.current_mode {
                Mode::Normal => unreachable!("Bug: cannot be in Normal Mode with a selected note"),
                Mode::Visual | Mode::VisualMove | Mode::VisualConnect => &THICK_JUNCTIONS,
                Mode::Edit(_) => &DOUBLE_JUNCTIONS,
                Mode::Delete => &PLAIN_JUNCTIONS,
            }
        }
        _ => &PLAIN_JUNCTIONS,
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