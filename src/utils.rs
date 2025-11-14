use crate::app::{Note, Side};

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
    
    // Calculate available space for the connection path.
    let available_space_x = end.x - start.x;
    let available_space_y = end.y - start.y;

    // Determine where the end point is in relation to the start point
    // (Polarity of the available space determines placement)
    // (Space must be greater than 1, since that's just the next cell (2))
    let h_placement = match available_space_x {
        2.. => HPlacement::Right,
        ..=-2 => HPlacement::Left,
        _ => HPlacement::Level,
    };
    let v_placement = match available_space_y {
        2.. => VPlacement::Below,
        ..=-2 => VPlacement::Above,
        _ => VPlacement::Level,
    };

    let mut points = vec![];

    match (start_side, end_side) {
        (Side::Right, Side::Left) => {
            match (h_placement, v_placement) {
                (HPlacement::Right, VPlacement::Below) => {
                    let midway_point_x = start.x + (available_space_x/2);
                    // is this path correct?
                    points = vec![
                        Point { x: start.x, y: start.y },
                        Point { x: midway_point_x, y: start.y, },
                        Point { x: midway_point_x, y: end.y, },
                        Point { x: end.x, y: end.y}
                    ];
                }
                _ => {}
            }
        }
        _ => {}
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