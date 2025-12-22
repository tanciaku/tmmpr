use crate::{
    app::{App, Screen},
    states::{
        MapState, map::{MapData, Note, Notification, Side}, start::ErrMsg
    },
};
use ratatui::style::Color;
use std::{fs, path::Path};

pub fn calculate_path(
    start_note: &Note,
    start_side: Side,
    end_note: &Note,
    end_side: Side,
) -> Vec<Point> {
    // Get start and end points for the path
    let start_tuple = start_note.get_connection_point(start_side);
    let end_tuple = end_note.get_connection_point(end_side);

    // Convert them to Point type for easier usage
    let start = Point { x: start_tuple.0 as isize, y: start_tuple.1 as isize, };
    let end = Point { x: end_tuple.0 as isize, y: end_tuple.1 as isize, };
    // Get the offset points (to clear note boundaries, makes it look appropriate)
    // NOTE: offset points are not used for all cases
    let start_off = get_offset_point(start, start_side);
    let end_off = get_offset_point(end, end_side);
    
    // Calculate available space for the connection path.
    let available_space_x = end.x - start.x;
    let available_space_y = end.y - start.y;

    // Determine where the end point is in relation to the start point
    // (Polarity of the available space determines placement)
    // Space for "Level" area must be x2 the offset amount in both directions
    let h_placement = match available_space_x {
        4.. => HPlacement::Right,
        ..=-4 => HPlacement::Left,
        _ => HPlacement::Level,
    };
    let v_placement = match available_space_y {
        4.. => VPlacement::Below,
        ..=-4 => VPlacement::Above,
        _ => VPlacement::Level,
    };

    let mut points = vec![];

    match (start_side, end_side) {
        // Right to _
        (Side::Right, Side::Right) => {
            match (h_placement, v_placement) {
                // Right
                (HPlacement::Right, VPlacement::Below) |
                (HPlacement::Right, VPlacement::Above) | 
                (HPlacement::Right, VPlacement::Level) |
                // Bottom
                (HPlacement::Level, VPlacement::Below) |
                (HPlacement::Left, VPlacement::Below) |
                // Top
                (HPlacement::Level, VPlacement::Above) |
                (HPlacement::Left, VPlacement::Above) |
                // Left
                (HPlacement::Left, VPlacement::Level) => {
                    points = reverse_c_shape(start, start_off, end, end_off);
                }
                _ => {}
            }
        }
        (Side::Right, Side::Left) => {
            match (h_placement, v_placement) {
                // Right
                (HPlacement::Right, VPlacement::Below) | 
                (HPlacement::Right, VPlacement::Above) | 
                (HPlacement::Right, VPlacement::Level) => {
                    points = s_shapes(start, start_off, end, end_off, available_space_x);
                }
                // Bottom
                (HPlacement::Level, VPlacement::Below) |
                (HPlacement::Left, VPlacement::Below) |
                // Left
                (HPlacement::Left, VPlacement::Level) |
                // Top
                (HPlacement::Level, VPlacement::Above) |
                (HPlacement::Left, VPlacement::Above) => {
                    points = sideways_s_shapes_y(start, start_off, end, end_off, available_space_y);
                }
                _ => {}
            }
        }
        (Side::Right, Side::Top) => {
            match (h_placement, v_placement) {
                // Right
                (HPlacement::Right, VPlacement::Below) => {
                    points = corner_shapes_1(start, start_off, end, end_off);
                }
                (HPlacement::Right, VPlacement::Above) | 
                (HPlacement::Right, VPlacement::Level) => {
                    points = sideways_s_shapes_x(start, start_off, end, end_off, available_space_x);
                }
                // Bottom
                (HPlacement::Level, VPlacement::Below) |
                (HPlacement::Left, VPlacement::Below) => {
                    points = sideways_s_shapes_y(start, start_off, end, end_off, available_space_y);
                }
                // Top
                (HPlacement::Level, VPlacement::Above) |
                (HPlacement::Left, VPlacement::Above) |
                // Left
                (HPlacement::Left, VPlacement::Level) => {
                    points = corner_shapes_2(start, start_off, end, end_off);
                }
                _ => {}
            }
        }
        (Side::Right, Side::Bottom) => {
            match (h_placement, v_placement) {
                // Right
                (HPlacement::Right, VPlacement::Below) |
                (HPlacement::Right, VPlacement::Level) => {
                    points = s_shapes(start, start_off, end, end_off, available_space_x);
                }
                (HPlacement::Right, VPlacement::Above) => {
                    points = corner_shapes_1(start, start_off, end, end_off);
                }
                // Bottom
                (HPlacement::Level, VPlacement::Below) |
                (HPlacement::Left, VPlacement::Below) |
                // Top
                (HPlacement::Level, VPlacement::Above) |
                (HPlacement::Left, VPlacement::Above) |
                // Left
                (HPlacement::Left, VPlacement::Level) => {
                    points = corner_shapes_2(start, start_off, end, end_off);
                }
                _ => {}
            }
        }
        // Left to _
        (Side::Left, Side::Right) => {
            match (h_placement, v_placement) {
                // Right
                (HPlacement::Right, VPlacement::Below) |
                (HPlacement::Right, VPlacement::Above) | 
                (HPlacement::Right, VPlacement::Level) |
                // Bottom
                (HPlacement::Level, VPlacement::Below) |
                // Top
                (HPlacement::Level, VPlacement::Above) => {
                    points = sideways_s_shapes_y(start, start_off, end, end_off, available_space_y);
                }
                // Bottom
                (HPlacement::Left, VPlacement::Below) |
                // Top
                (HPlacement::Left, VPlacement::Above) |
                // Left
                (HPlacement::Left, VPlacement::Level) => {
                    points = sideways_s_shapes_x(start, start_off, end, end_off, available_space_x);
                }
                _ => {}
            }
        }
        (Side::Left, Side::Left) => {
            match (h_placement, v_placement) {
                // Right
                (HPlacement::Right, VPlacement::Below) |
                (HPlacement::Right, VPlacement::Above) | 
                (HPlacement::Right, VPlacement::Level) |
                // Bottom
                (HPlacement::Level, VPlacement::Below) |
                (HPlacement::Left, VPlacement::Below) |
                // Top
                (HPlacement::Level, VPlacement::Above) |
                (HPlacement::Left, VPlacement::Above) |
                // Left
                (HPlacement::Left, VPlacement::Level) => {
                    points = c_shape(start, start_off, end, end_off);
                }
                _ => {}
            }
        }
        (Side::Left, Side::Top) => {
            match (h_placement, v_placement) {
                // Right
                (HPlacement::Right, VPlacement::Below) |
                // Bottom
                (HPlacement::Level, VPlacement::Below) => {
                    points = sideways_s_shapes_y(start, start_off, end, end_off, available_space_y);
                }
                // Right
                (HPlacement::Right, VPlacement::Above) | 
                (HPlacement::Right, VPlacement::Level) => {
                    points = c_shape(start, start_off, end, end_off);
                }
                // Bottom
                (HPlacement::Left, VPlacement::Below) => {
                    points = corner_shapes_1(start, start_off, end, end_off);
                }
                // Top
                (HPlacement::Level, VPlacement::Above) |
                (HPlacement::Left, VPlacement::Above) |
                // Left
                (HPlacement::Left, VPlacement::Level) => {
                    points = corner_shapes_2(start, start_off, end, end_off);
                }
                _ => {}
            }
        }
        (Side::Left, Side::Bottom) => {
            match (h_placement, v_placement) {
                // Right
                (HPlacement::Right, VPlacement::Below) |
                (HPlacement::Right, VPlacement::Above) | 
                (HPlacement::Right, VPlacement::Level) |
                // Bottom
                (HPlacement::Level, VPlacement::Below) |
                (HPlacement::Left, VPlacement::Below) |
                // Top
                (HPlacement::Level, VPlacement::Above) |
                // Left
                (HPlacement::Left, VPlacement::Level) => {
                    points = corner_shapes_2(start, start_off, end, end_off);
                }
                // Top
                (HPlacement::Left, VPlacement::Above) => {
                    points = corner_shapes_1(start, start_off, end, end_off);
                }                    
                _ => {}
            }
        }
        // Top to _
        (Side::Top, Side::Right) => {
            match (h_placement, v_placement) {
                // Right
                (HPlacement::Right, VPlacement::Below) |
                (HPlacement::Right, VPlacement::Above) | 
                (HPlacement::Right, VPlacement::Level) |
                // Bottom
                (HPlacement::Level, VPlacement::Below) |
                (HPlacement::Left, VPlacement::Below) |
                // Top
                (HPlacement::Level, VPlacement::Above) |
                // Left
                (HPlacement::Left, VPlacement::Level) => {
                    points = corner_shapes_1(start, start_off, end, end_off);
                }
                // Top
                (HPlacement::Left, VPlacement::Above) => {
                    points = corner_shapes_2(start, start_off, end, end_off);
                }
                _ => {}
            }
        }
        (Side::Top, Side::Left) => {
            match (h_placement, v_placement) {
                // Right
                (HPlacement::Right, VPlacement::Below) |
                (HPlacement::Right, VPlacement::Level) |
                // Bottom
                (HPlacement::Level, VPlacement::Below) |
                (HPlacement::Left, VPlacement::Below) |
                // Top
                (HPlacement::Level, VPlacement::Above) |
                (HPlacement::Left, VPlacement::Above) |
                // Left
                (HPlacement::Left, VPlacement::Level) => {
                    points = corner_shapes_1(start, start_off, end, end_off);
                }
                // Right
                (HPlacement::Right, VPlacement::Above) => {
                    points = corner_shapes_2(start, start_off, end, end_off);
                }
                _ => {}
            }
        }
        (Side::Top, Side::Top) => {
            match (h_placement, v_placement) {
                // Right
                (HPlacement::Right, VPlacement::Below) |
                (HPlacement::Right, VPlacement::Level) |
                (HPlacement::Right, VPlacement::Above) |
                // Bottom
                (HPlacement::Level, VPlacement::Below) |
                (HPlacement::Left, VPlacement::Below) |
                // Top
                (HPlacement::Level, VPlacement::Above) |
                (HPlacement::Left, VPlacement::Above) |
                // Left
                (HPlacement::Left, VPlacement::Level) => {
                    points = upside_down_u_shapes(start, start_off, end, end_off);
                }
                _ => {}
            }
        }
        (Side::Top, Side::Bottom) => {
            match (h_placement, v_placement) {
                // Right
                (HPlacement::Right, VPlacement::Below) |
                (HPlacement::Right, VPlacement::Level) |
                // Bottom
                (HPlacement::Level, VPlacement::Below) |
                (HPlacement::Left, VPlacement::Below) |
                // Left
                (HPlacement::Left, VPlacement::Level) => {
                    points = s_shapes(start, start_off, end, end_off, available_space_x);
                }
                // Right
                (HPlacement::Right, VPlacement::Above) |
                // Top
                (HPlacement::Level, VPlacement::Above) |
                (HPlacement::Left, VPlacement::Above) => {
                    points = corner_shapes_1(start, start_off, end, end_off);
                }
                _ => {}
            }
        }
        // Bottom to _
        (Side::Bottom, Side::Right) => {
            match (h_placement, v_placement) {
                // Right
                (HPlacement::Right, VPlacement::Below) |
                // Bottom
                (HPlacement::Level, VPlacement::Below) => {
                    points = sideways_s_shapes_y(start, start_off, end, end_off, available_space_y);
                }
                // Right
                (HPlacement::Right, VPlacement::Above) | 
                (HPlacement::Right, VPlacement::Level) |
                // Top
                (HPlacement::Level, VPlacement::Above) |
                (HPlacement::Left, VPlacement::Above) |
                // Left
                (HPlacement::Left, VPlacement::Level) => {
                    points = corner_shapes_1(start, start_off, end, end_off);
                }
                // Bottom
                (HPlacement::Left, VPlacement::Below) => {
                    points = corner_shapes_2(start, start_off, end, end_off);
                }
                _ => {}
            }
        }
        (Side::Bottom, Side::Left) => {
            match (h_placement, v_placement) {
                // Right
                (HPlacement::Right, VPlacement::Below) => {
                    points = corner_shapes_2(start, start_off, end, end_off);
                }
                (HPlacement::Right, VPlacement::Above) | 
                (HPlacement::Right, VPlacement::Level) |
                // Bottom
                (HPlacement::Level, VPlacement::Below) |
                (HPlacement::Left, VPlacement::Below) |
                // Top
                (HPlacement::Level, VPlacement::Above) |
                (HPlacement::Left, VPlacement::Above) |
                // Left
                (HPlacement::Left, VPlacement::Level) => {
                    points = corner_shapes_1(start, start_off, end, end_off);
                }
                _ => {}
            }
        }
        (Side::Bottom, Side::Top) => {
            match (h_placement, v_placement) {
                // Right
                (HPlacement::Right, VPlacement::Below) |
                // Bottom
                (HPlacement::Level, VPlacement::Below) |
                (HPlacement::Left, VPlacement::Below) => {
                    points = sideways_s_shapes_y(start, start_off, end, end_off, available_space_y);
                }
                // Right
                (HPlacement::Right, VPlacement::Above) | 
                (HPlacement::Right, VPlacement::Level) |
                // Top
                (HPlacement::Level, VPlacement::Above) |
                (HPlacement::Left, VPlacement::Above) |
                // Left
                (HPlacement::Left, VPlacement::Level) => {
                    points = sideways_s_shapes_x(start, start_off, end, end_off, available_space_x);
                }
                _ => {}
            }
        }
        (Side::Bottom, Side::Bottom) => {
            match (h_placement, v_placement) {
                // Right
                (HPlacement::Right, VPlacement::Below) |
                (HPlacement::Right, VPlacement::Above) | 
                (HPlacement::Right, VPlacement::Level) |
                // Bottom
                (HPlacement::Level, VPlacement::Below) |
                (HPlacement::Left, VPlacement::Below) |
                // Top
                (HPlacement::Level, VPlacement::Above) |
                (HPlacement::Left, VPlacement::Above) |
                // Left
                (HPlacement::Left, VPlacement::Level) => {
                    points = u_shapes(start, start_off, end, end_off);
                }
                _ => {}
            }
        }
    }

    points
}

#[derive(Clone, Copy)]
pub struct Point {
    pub x: isize,
    pub y: isize,
}

// Where the end point is, in relation to the start point
pub enum HPlacement {
    Right,
    Left,
    Level,
}

pub enum VPlacement {
    Above,
    Below,
    Level,
}

/// Get an offset point in relation to the side.
/// Used to clear the boundaries of notes for both start and end points.
/// NOTE: offset points are not used for all cases
fn get_offset_point(p: Point, side: Side) -> Point {
    let offset = 2;
    let p_off = match side {
        Side::Right => Point { x: p.x + offset, y: p.y },
        Side::Left => Point { x: p.x - offset, y: p.y },
        Side::Top => Point { x: p.x, y: p.y - offset },
        Side::Bottom => Point { x: p.x, y: p.y + offset },
    };
    p_off
}

/// Helper function for displaying the color currently set for the selected note/connection
pub fn get_color_name_in_string(color: Color) -> String {
    String::from(match color {
        Color::Red => "Red",
        Color::Green => "Green",
        Color::Yellow => "Yellow",
        Color::Blue => "Blue",
        Color::Magenta => "Magenta",
        Color::Cyan => "Cyan",
        Color::White => "White",
        Color::Black => "Black",
        _ => "",
    })
}

pub fn get_color_from_string(color_name_str: &str) -> Color {
    match color_name_str {
        "Red" => Color::Red,
        "Green" => Color::Green,
        "Yellow" => Color::Yellow,
        "Blue" => Color::Blue,
        "Magenta" => Color::Magenta,
        "Cyan" => Color::Cyan,
        "White" => Color::White,
        "Black" => Color::Black,
        _ => Color::White,
    }
}

// --- Path shapes ---

fn c_shape(start: Point, start_off: Point, end: Point, end_off: Point) -> Vec<Point> { 
    let furthest_point_x = start_off.x.min(end_off.x); // furthest point to the left
    vec![
        start,
        start_off,
        Point { x: furthest_point_x, y: start_off.y },
        Point { x: furthest_point_x, y: end_off.y },
        end_off,
        end,
    ]
}

fn reverse_c_shape(start: Point, start_off: Point, end: Point, end_off: Point) -> Vec<Point> { 
    let furthest_point_x = start_off.x.max(end_off.x); // furthest point to the right
    vec![
        start,
        start_off,
        Point { x: furthest_point_x, y: start_off.y },
        Point { x: furthest_point_x, y: end_off.y },
        end_off,
        end
    ]
}

// s shape, reverse s shape
fn s_shapes(start: Point, start_off: Point, end: Point, end_off: Point, available_space_x: isize) -> Vec<Point> {
    let midway_point_x = start.x + (available_space_x/2);
    vec![
        start,
        start_off,
        Point { x: midway_point_x, y: start_off.y },
        Point { x: midway_point_x, y: end_off.y },
        end_off,
        end
    ]
}

fn sideways_s_shapes_y(start: Point, start_off: Point, end: Point, end_off: Point, available_space_y: isize) -> Vec<Point> {
    let midway_point_y = start.y + (available_space_y/2);
    vec![
        start,
        start_off,
        Point { x: start_off.x, y: midway_point_y },
        Point { x: end_off.x, y: midway_point_y },
        end_off,
        end
    ]
}

fn sideways_s_shapes_x(start: Point, start_off: Point, end: Point, end_off: Point, available_space_x: isize) -> Vec<Point> {
    let midway_point_x = start.x + (available_space_x/2);
    vec![
        start,
        start_off,
        Point { x: midway_point_x, y: start_off.y },
        Point { x: midway_point_x, y: end_off.y },
        end_off,
        end
    ]
}

/// midpoint:  end_off.x, start_off.y
fn corner_shapes_1(start: Point, start_off: Point, end: Point, end_off: Point) -> Vec<Point> {
    vec![
        start,
        start_off,
        Point { x: end_off.x, y: start_off.y },
        end_off,
        end
    ]
}

// ┐ ┘
/// midpoint:  start_off.x, end_off.y
fn corner_shapes_2(start: Point, start_off: Point, end: Point, end_off: Point) -> Vec<Point> {
    vec![
        start,
        start_off,
        Point { x: start_off.x, y: end_off.y },
        end_off,
        end
    ]
}

fn upside_down_u_shapes(start: Point, start_off: Point, end: Point, end_off: Point) -> Vec<Point> {
    let highest_y = start_off.y.min(end_off.y);
    vec![
        start,
        start_off,
        Point { x: start_off.x, y: highest_y },
        Point { x: end_off.x, y: highest_y },
        end_off,
        end
    ]
}

fn u_shapes(start: Point, start_off: Point, end: Point, end_off: Point) -> Vec<Point> {
    let lowest_y = start_off.y.max(end_off.y);
    vec![
        start,
        start_off,
        Point { x: start_off.x, y: lowest_y },
        Point { x: end_off.x, y: lowest_y },
        end_off,
        end
    ]
}

/// Writes the relevant data to a file
pub fn write_json_data<T>(path: &Path, data: &T) -> Result<(), Box<dyn std::error::Error>>
where
    T: serde::Serialize,
{
    let json_string = serde_json::to_string_pretty(data)?;
    fs::write(path, json_string)?;
    Ok(())
}

/// Reads the relevant data from a file
pub fn read_json_data<T>(path: &Path) -> Result<T, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    let json_string = fs::read_to_string(path)?;
    let data: T = serde_json::from_str(&json_string)?;
    Ok(data)
}

/// Creates a new map file.
/// 
/// Handles file write error by displaying appropriate error message to the user.
pub fn create_map_file(app: &mut App, path: &Path) {
    // Create a new Map State for creating a new map file
    let map_state = MapState::new(path.to_path_buf()); // Only clone when storing
    // Take the default values from that to write to the file
    let map_data = MapData {
        view_pos: map_state.view_pos,
        next_note_id: map_state.next_note_id,
        notes: map_state.notes,
        connections: map_state.connections,
        connection_index: map_state.connection_index,
    };

    // Attempt to write that data to the file
    if let Err(_) = write_json_data(path, &map_data) {
        // Display an error
        if let Screen::Start(start_state) = &mut app.screen {
            start_state.handle_submit_error(ErrMsg::FileWrite);
        }
        return // Stop here without switching screens.
    }

    // Adding the path to the newly created map file to recent_paths
    if let Screen::Start(start_state) = &mut app.screen { // guaranteed
        // If recent_paths functionality available
        if let Ok(recent_paths) = &mut start_state.recent_paths {
            // Add the file path to recent_paths (if not already there)
            if !recent_paths.contains_path(path) {
                recent_paths.add(path.to_path_buf());

                // Save the recent_paths file
                recent_paths.save();
            }
        }
    }

    // If successful in the previous step -
    // switch to the Map Screen, with the newly created Map State
    app.screen = Screen::Map(MapState::new(path.to_path_buf())); // Only clone when storing
}

/// Saves map data to a file.
/// 
/// Handles file write error by displaying appropriate error message to the user.
/// 
/// 
pub fn save_map_file(app: &mut App, path: &Path, show_save_notification: bool) {
    if let Screen::Map(map_state) = &app.screen {
        // Get the relevant values from the current Map State
        let map_data = MapData {
            view_pos: map_state.view_pos.clone(), // necessary
            next_note_id: map_state.next_note_id,
            notes: map_state.notes.clone(), // necessary
            connections: map_state.connections.clone(), // necessary
            connection_index: map_state.connection_index.clone(), // necessary
        };

        // Attempt to write map data to the file
        match write_json_data(path, &map_data) {
            Ok(_) => {
                // Show successfully saved the map file message and redraw
                if let Screen::Map(map_state) = &mut app.screen {
                    // Can exit the app - now that have successfully saved the map file.
                    map_state.can_exit = true;

                    if show_save_notification {
                        map_state.show_notification = Some(Notification::SaveSuccess);
                        map_state.needs_clear_and_redraw = true;
                    }
                }
            }
            Err(_) => {
                // Show failed saving the map file message and redraw
                if let Screen::Map(map_state) = &mut app.screen {
                    if show_save_notification {
                        map_state.show_notification = Some(Notification::SaveFail);
                        map_state.needs_clear_and_redraw = true;
                    }
                }
            }
        }            
    }
}

/// Loads map data from a file and transitions the application to the Map screen.
/// 
/// This function is exclusively called from the Start screen when the user wants to 
/// open an existing map file. It reads the map data from the specified file path,
/// populates a new MapState with the loaded data, and transitions the app to the Map screen.
/// 
/// If the file cannot be read or contains invalid data, the function will show
/// an error message via the Start screen's error handling and prevent screen transition.
pub fn load_map_file(app: &mut App, path: &Path) {
    // Initialize a default MapState that will be populated with loaded data.
    // This ensures we have valid defaults for any fields not present in the file.
    let mut map_state = MapState::new(path.to_path_buf()); // Only clone when storing

    match read_json_data::<MapData>(path) {
        Ok(map_data) => {
            // Successfully loaded data from file - now populate the MapState
            // with the saved values, overriding the defaults
            map_state.view_pos = map_data.view_pos;
            map_state.next_note_id = map_data.next_note_id;
            map_state.notes = map_data.notes;
            map_state.connections = map_data.connections;
            map_state.connection_index = map_data.connection_index;
        }
        Err(_) => {
            // Failed to read or parse the map file - show error to user
            // and stay on the Start screen/Input menu to allow them to try again
            //
            // If an error occurs when using the "recent paths" functionality (in the start screen)
            // handle_submit_error will also unnecessarily reset the input fields, even though
            // the user is not in the input menu (affects nothing).
            if let Screen::Start(start_state) = &mut app.screen {
                start_state.handle_submit_error(ErrMsg::FileRead);
            }
            return; // Early return prevents screen transition
        }
    }

    // Adding the path to the map file to recent_paths
    if let Screen::Start(start_state) = &mut app.screen { // guaranteed
        // If recent_paths functionality available
        if let Ok(recent_paths) = &mut start_state.recent_paths {
            // Add the file path to recent_paths (if not already there)
            if !recent_paths.contains_path(path) {
                recent_paths.add(path.to_path_buf());

                // Save the recent_paths file
                recent_paths.save();
            }
        }
    }

    // File loaded successfully - transition to the Map screen with the 
    // populated MapState containing the loaded map data
    app.screen = Screen::Map(map_state);
}