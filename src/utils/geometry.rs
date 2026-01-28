use crate::{
    states::{
        map::{Note, Side}
    },
};


/// A 2D point in the coordinate space.
///
/// Uses signed integers where X increases rightward and Y increases downward,
/// following standard coordinate conventions.
///
/// This type is `Copy`, making it efficient for geometry calculations.
///
/// # Fields
///
/// * `x` - Horizontal position (positive = right, negative = left)
/// * `y` - Vertical position (positive = down, negative = up)
///
/// # Examples
///
/// ```
/// # use tmmpr::utils::geometry::Point;
/// let origin = Point { x: 0, y: 0 };
/// let bottom_right = Point { x: 100, y: 50 };
/// ```
#[derive(Clone, Copy)]
pub struct Point {
    pub x: isize,
    pub y: isize,
}

/// Where the end point is, in relation to the start point horizontally
pub enum HPlacement {
    Right,
    Left,
    Level,
}

/// Where the end point is, in relation to the start point vertically
pub enum VPlacement {
    Above,
    Below,
    Level,
}

/// Calculates a visual connection path between two notes.
///
/// This function computes a series of points that form an aesthetically pleasing
/// connection line between two notes, taking into account their relative positions
/// and which sides of each note are being connected. The algorithm automatically
/// determines the appropriate path shape (C-shape, S-shape, U-shape, corner, etc.)
/// based on the spatial relationship between the notes.
///
/// # Parameters
///
/// * `start_note` - A reference to the note where the connection begins
/// * `start_side` - The side of the start note where the connection originates
///   (Right, Left, Top, or Bottom)
/// * `end_note` - A reference to the note where the connection terminates
/// * `end_side` - The side of the end note where the connection ends
///   (Right, Left, Top, or Bottom)
///
/// # Returns
///
/// Returns a `Vec<Point>` containing the ordered sequence of points that define
/// the connection path. The first point is at the edge of the start note, and the
/// last point is at the edge of the end note. Intermediate points define the curves
/// and bends of the connection line.
///
/// An empty vector is returned if no valid path can be computed (though this should
/// be rare in normal operation).
///
/// # Algorithm
///
/// The function operates in several stages:
///
/// 1. **Extract connection points**: Gets the actual coordinate points on each note's
///    edge where the connection should attach, based on the specified sides.
///
/// 2. **Calculate offset points**: Computes points slightly away from each note
///    (by an offset of 2 units) to ensure the connection line clears the note
///    boundaries and looks visually appropriate.
///
/// 3. **Determine spatial relationship**: Analyzes the available space between notes
///    in both X and Y dimensions to determine the relative placement:
///    - Horizontal: Right (4+ units), Left (-4+ units), or Level (within ±3 units)
///    - Vertical: Below (4+ units), Above (-4+ units), or Level (within ±3 units)
///
/// 4. **Select path shape**: Based on the combination of:
///    - Start side (4 options)
///    - End side (4 options)
///    - Horizontal placement (3 options)
///    - Vertical placement (3 options)
///    
///    The function selects one of several path shape generators:
///    - `c_shape` / `reverse_c_shape`: C-shaped paths, useful when connecting to
///      the same side or wrapping around notes
///    - `s_shapes` / `sideways_s_shapes_x` / `sideways_s_shapes_y`: S-shaped paths
///      for connections that need to change both horizontal and vertical direction
///    - `corner_shapes_1` / `corner_shapes_2`: Simple L-shaped corner connections
///    - `u_shapes` / `upside_down_u_shapes`: U-shaped paths for connecting notes
///      stacked vertically with same-side connections
///
/// 5. **Generate path points**: The selected shape function generates the specific
///    sequence of points that form the connection path.
///
/// # Coordinate System
///
/// The function uses `isize` coordinates where:
/// - X increases to the right
/// - Y increases downward
///
/// # Examples
///
/// ```rust,ignore
/// use tmmpr::graph::{Note, Side, calculate_path};
///
/// // Create two notes
/// let start_note = Note::new(10, 10, "Start".to_string());
/// let end_note = Note::new(50, 20, "End".to_string());
///
/// // Calculate path from right side of start to left side of end
/// let path = calculate_path(&start_note, Side::Right, &end_note, Side::Left);
///
/// // The path vector contains all points to draw the connection
/// for point in path {
///     // Draw line segment to this point
///     println!("Point: ({}, {})", point.x, point.y);
/// }
/// ```
///
/// # Performance
///
/// This function performs simple arithmetic operations and allocates a small vector
/// (typically 4-6 points). It is designed to be called frequently during rendering
/// without significant performance impact.
///
/// # Visual Quality
///
/// The function prioritizes visual aesthetics:
/// - Lines clear note boundaries by using offset points
/// - The "Level" threshold of ±3 units provides a tolerance zone to avoid
///   excessive path complexity when notes are nearly aligned
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


/// Get an offset point in relation to the side.
/// Used to clear the boundaries of notes for both start and end points.
/// NOTE: offset points are not used for all cases
pub fn get_offset_point(p: Point, side: Side) -> Point {
    let offset = 2;
    let p_off = match side {
        Side::Right => Point { x: p.x + offset, y: p.y },
        Side::Left => Point { x: p.x - offset, y: p.y },
        Side::Top => Point { x: p.x, y: p.y - offset },
        Side::Bottom => Point { x: p.x, y: p.y + offset },
    };
    p_off
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
