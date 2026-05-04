//! Internal geometry utilities for the graph module.
//!
//! This module is not part of the public API.

use crate::graph::{Node, NodeLayout, Side};

/// A rectangle representation that uses signed integers (`isize`) for its coordinates.
///
/// This is crucial for performing screen-space calculations where coordinates can
/// temporarily become negative (e.g., a note is partially off-screen to the left)
/// before being clipped to the viewport boundaries.
// TODO: restore pub(crate) later
pub struct SignedRect {
    // TODO: restore pub(crate) later
    pub x: isize,
    // TODO: restore pub(crate) later
    pub y: isize,
    // TODO: restore pub(crate) later
    pub width: isize,
    // TODO: restore pub(crate) later
    pub height: isize,
}

impl SignedRect {
    /// Core clipping logic for determining the visible portion of a rectangle (typically
    /// a note) within the viewport bounds.
    // TODO: restore pub(crate) later
    pub fn intersection(&self, view: &SignedRect) -> Option<SignedRect> {
        if self.x >= view.x + view.width
            || self.x + self.width <= view.x
            || self.y >= view.y + view.height
            || self.y + self.height <= view.y
        {
            return None;
        } else {
            let x_start = self.x.max(view.x);
            let x_end = (self.x + self.width).min(view.x + view.width);
            let x_width = x_end - x_start;

            let y_start = self.y.max(view.y);
            let y_end = (self.y + self.height).min(view.y + view.height);
            let y_height = y_end - y_start;

            Some(SignedRect {
                x: x_start,
                y: y_start,
                width: x_width,
                height: y_height,
            })
        }
    }
}

/// A 2D point in the coordinate space.
///
/// Uses signed integers where X increases rightward and Y increases downward,
/// following standard coordinate conventions.
// TODO: restore pub(crate) later
#[derive(Clone, Copy)]
pub struct Point {
    // TODO: restore pub(crate) later
    pub x: isize,
    // TODO: restore pub(crate) later
    pub y: isize,
}

/// Where the end point is, in relation to the start point horizontally
// TODO: restore pub(crate) later
pub enum HPlacement {
    Right,
    Left,
    Level,
}

/// Where the end point is, in relation to the start point vertically
// TODO: restore pub(crate) later
pub enum VPlacement {
    Above,
    Below,
    Level,
}

/// Computes an ordered sequence of points forming a routed connection line between two nodes.
///
/// Selects a path shape (S, C, U, corner, etc.) based on the relative positions of the nodes
/// and the sides being connected, with 2-unit offsets for visual clearance from node boundaries.
// TODO: restore pub(crate) later
pub fn calculate_path<T: NodeLayout>(
    start_node: &Node<T>,
    start_side: Side,
    end_node: &Node<T>,
    end_side: Side,
) -> Vec<Point> {
    let start_tuple = start_node.connection_point(start_side);
    let end_tuple = end_node.connection_point(end_side);

    let start = Point {
        x: start_tuple.0 as isize,
        y: start_tuple.1 as isize,
    };
    let end = Point {
        x: end_tuple.0 as isize,
        y: end_tuple.1 as isize,
    };

    // Offset points extend 2 units away from note edges for visual clearance
    let start_off = get_offset_point(start, start_side);
    let end_off = get_offset_point(end, end_side);

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

// TODO: restore pub(crate) later
pub fn get_offset_point(p: Point, side: Side) -> Point {
    let offset = 2;
    let p_off = match side {
        Side::Right => Point {
            x: p.x + offset,
            y: p.y,
        },
        Side::Left => Point {
            x: p.x - offset,
            y: p.y,
        },
        Side::Top => Point {
            x: p.x,
            y: p.y - offset,
        },
        Side::Bottom => Point {
            x: p.x,
            y: p.y + offset,
        },
    };
    p_off
}

fn c_shape(start: Point, start_off: Point, end: Point, end_off: Point) -> Vec<Point> {
    let furthest_point_x = start_off.x.min(end_off.x); // furthest point to the left
    vec![
        start,
        start_off,
        Point {
            x: furthest_point_x,
            y: start_off.y,
        },
        Point {
            x: furthest_point_x,
            y: end_off.y,
        },
        end_off,
        end,
    ]
}

fn reverse_c_shape(start: Point, start_off: Point, end: Point, end_off: Point) -> Vec<Point> {
    let furthest_point_x = start_off.x.max(end_off.x); // furthest point to the right
    vec![
        start,
        start_off,
        Point {
            x: furthest_point_x,
            y: start_off.y,
        },
        Point {
            x: furthest_point_x,
            y: end_off.y,
        },
        end_off,
        end,
    ]
}

// s shape, reverse s shape
fn s_shapes(
    start: Point,
    start_off: Point,
    end: Point,
    end_off: Point,
    available_space_x: isize,
) -> Vec<Point> {
    let midway_point_x = start.x + (available_space_x / 2);
    vec![
        start,
        start_off,
        Point {
            x: midway_point_x,
            y: start_off.y,
        },
        Point {
            x: midway_point_x,
            y: end_off.y,
        },
        end_off,
        end,
    ]
}

fn sideways_s_shapes_y(
    start: Point,
    start_off: Point,
    end: Point,
    end_off: Point,
    available_space_y: isize,
) -> Vec<Point> {
    let midway_point_y = start.y + (available_space_y / 2);
    vec![
        start,
        start_off,
        Point {
            x: start_off.x,
            y: midway_point_y,
        },
        Point {
            x: end_off.x,
            y: midway_point_y,
        },
        end_off,
        end,
    ]
}

fn sideways_s_shapes_x(
    start: Point,
    start_off: Point,
    end: Point,
    end_off: Point,
    available_space_x: isize,
) -> Vec<Point> {
    let midway_point_x = start.x + (available_space_x / 2);
    vec![
        start,
        start_off,
        Point {
            x: midway_point_x,
            y: start_off.y,
        },
        Point {
            x: midway_point_x,
            y: end_off.y,
        },
        end_off,
        end,
    ]
}

/// midpoint:  end_off.x, start_off.y
fn corner_shapes_1(start: Point, start_off: Point, end: Point, end_off: Point) -> Vec<Point> {
    vec![
        start,
        start_off,
        Point {
            x: end_off.x,
            y: start_off.y,
        },
        end_off,
        end,
    ]
}

/// midpoint:  start_off.x, end_off.y
fn corner_shapes_2(start: Point, start_off: Point, end: Point, end_off: Point) -> Vec<Point> {
    vec![
        start,
        start_off,
        Point {
            x: start_off.x,
            y: end_off.y,
        },
        end_off,
        end,
    ]
}

fn upside_down_u_shapes(start: Point, start_off: Point, end: Point, end_off: Point) -> Vec<Point> {
    let highest_y = start_off.y.min(end_off.y);
    vec![
        start,
        start_off,
        Point {
            x: start_off.x,
            y: highest_y,
        },
        Point {
            x: end_off.x,
            y: highest_y,
        },
        end_off,
        end,
    ]
}

fn u_shapes(start: Point, start_off: Point, end: Point, end_off: Point) -> Vec<Point> {
    let lowest_y = start_off.y.max(end_off.y);
    vec![
        start,
        start_off,
        Point {
            x: start_off.x,
            y: lowest_y,
        },
        Point {
            x: end_off.x,
            y: lowest_y,
        },
        end_off,
        end,
    ]
}
