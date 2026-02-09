use ratatui::style::Color;

use crate::{
    states::map::{Note, Side},
    utils::geometry::{Point, calculate_path, get_offset_point},
};

// Helper functions for creating test data
fn create_test_note(x: usize, y: usize, content: &str) -> Note {
    Note::new(x, y, content.to_string(), Color::White)
}

// --- Tests for get_offset_point ---

#[test]
fn test_get_offset_point_right() {
    let point = Point { x: 10, y: 20 };
    let offset_point = get_offset_point(point, Side::Right);

    assert_eq!(offset_point.x, 12); // x + 2
    assert_eq!(offset_point.y, 20); // y unchanged
}

#[test]
fn test_get_offset_point_left() {
    let point = Point { x: 10, y: 20 };
    let offset_point = get_offset_point(point, Side::Left);
    
    assert_eq!(offset_point.x, 8); // x - 2
    assert_eq!(offset_point.y, 20); // y unchanged
}

#[test]
fn test_get_offset_point_top() {
    let point = Point { x: 10, y: 20 };
    let offset_point = get_offset_point(point, Side::Top);
    
    assert_eq!(offset_point.x, 10); // x unchanged
    assert_eq!(offset_point.y, 18); // y - 2
}

#[test]
fn test_get_offset_point_bottom() {
    let point = Point { x: 10, y: 20 };
    let offset_point = get_offset_point(point, Side::Bottom);
    
    assert_eq!(offset_point.x, 10); // x unchanged
    assert_eq!(offset_point.y, 22); // y + 2
}

#[test]
fn test_get_offset_point_negative_coordinates() {
    let point = Point { x: -10, y: -20 };
    
    let right = get_offset_point(point, Side::Right);
    assert_eq!(right.x, -8);
    assert_eq!(right.y, -20);
    
    let left = get_offset_point(point, Side::Left);
    assert_eq!(left.x, -12);
    assert_eq!(left.y, -20);
    
    let top = get_offset_point(point, Side::Top);
    assert_eq!(top.x, -10);
    assert_eq!(top.y, -22);
    
    let bottom = get_offset_point(point, Side::Bottom);
    assert_eq!(bottom.x, -10);
    assert_eq!(bottom.y, -18);
}

// --- Tests for path continuity ---

#[test]
fn test_path_continuity_all_combinations() {
    // Test that all side combinations produce continuous paths
    let start_note = create_test_note(10, 10, "Start");
    let end_note = create_test_note(60, 40, "End");
    
    let sides = [Side::Right, Side::Left, Side::Top, Side::Bottom];
    
    for start_side in &sides {
        for end_side in &sides {
            let path = calculate_path(&start_note, *start_side, &end_note, *end_side);
            
            // Path must have at least 2 points
            assert!(path.len() >= 2, 
                "Path from {:?} to {:?} should have at least 2 points", 
                start_side, end_side);
            
            // Verify continuity: each consecutive pair must share x or y coordinate
            for i in 0..path.len() - 1 {
                let curr = &path[i];
                let next = &path[i + 1];
                
                let same_x = curr.x == next.x;
                let same_y = curr.y == next.y;
                
                assert!(same_x || same_y, 
                    "Path from {:?} to {:?} is not continuous at segment {}->{}: ({}, {}) to ({}, {})", 
                    start_side, end_side, i, i+1, curr.x, curr.y, next.x, next.y);
            }
        }
    }
}

// --- Tests for path correctness (specific shapes) ---

#[test]
fn test_s_shape_horizontal_right_to_left() {
    // Right to Left with enough horizontal space should create an s-shape
    let start_note = create_test_note(10, 20, "A");
    let end_note = create_test_note(70, 20, "B");
    
    let path = calculate_path(&start_note, Side::Right, &end_note, Side::Left);
    
    // S-shape should have 6 points: start, start_off, mid_top, mid_bottom, end_off, end
    assert_eq!(path.len(), 6, "S-shape should have 6 points");
    
    let start_conn = start_note.get_connection_point(Side::Right);
    let end_conn = end_note.get_connection_point(Side::Left);
    
    // Verify start and end
    assert_eq!(path[0].x, start_conn.0 as isize);
    assert_eq!(path[0].y, start_conn.1 as isize);
    assert_eq!(path[5].x, end_conn.0 as isize);
    assert_eq!(path[5].y, end_conn.1 as isize);
    
    // Verify offset points (2 units from start/end)
    assert_eq!(path[1].x, path[0].x + 2);
    assert_eq!(path[1].y, path[0].y);
    assert_eq!(path[4].x, path[5].x - 2);
    assert_eq!(path[4].y, path[5].y);
    
    // Verify middle vertical segments share x coordinate (halfway between)
    let expected_mid_x = path[0].x + ((end_conn.0 as isize - start_conn.0 as isize) / 2);
    assert_eq!(path[2].x, expected_mid_x);
    assert_eq!(path[3].x, expected_mid_x);
    
    // Middle segments should connect the offset y positions
    assert_eq!(path[2].y, path[1].y);
    assert_eq!(path[3].y, path[4].y);
}

#[test]
fn test_c_shape_left_to_left() {
    // Left to Left should create a c-shape wrapping around to the left
    let start_note = create_test_note(50, 15, "A");
    let end_note = create_test_note(30, 35, "B");
    
    let path = calculate_path(&start_note, Side::Left, &end_note, Side::Left);
    
    // C-shape should have 6 points
    assert_eq!(path.len(), 6, "C-shape should have 6 points");
    
    let start_conn = start_note.get_connection_point(Side::Left);
    let end_conn = end_note.get_connection_point(Side::Left);
    
    // Verify start and end
    assert_eq!(path[0].x, start_conn.0 as isize);
    assert_eq!(path[5].x, end_conn.0 as isize);
    
    // Verify it goes left (offset should be x - 2)
    assert_eq!(path[1].x, path[0].x - 2);
    assert_eq!(path[4].x, path[5].x - 2);
    
    // The furthest x point should be to the left of both notes
    let furthest_x = path[2].x;
    assert!(furthest_x <= path[1].x.min(path[4].x), 
        "C-shape should extend left of both offset points");
    
    // Verify the shape: horizontal -> vertical -> horizontal pattern
    assert_eq!(path[1].y, path[2].y); // horizontal segment
    assert_eq!(path[2].x, path[3].x); // vertical segment
    assert_eq!(path[3].y, path[4].y); // horizontal segment
}

#[test]
fn test_reverse_c_shape_right_to_right() {
    // Right to Right should create a reverse c-shape wrapping around to the right
    let start_note = create_test_note(10, 15, "A");
    let end_note = create_test_note(30, 35, "B");
    
    let path = calculate_path(&start_note, Side::Right, &end_note, Side::Right);
    
    // Reverse C-shape should have 6 points
    assert_eq!(path.len(), 6, "Reverse C-shape should have 6 points");
    
    let start_conn = start_note.get_connection_point(Side::Right);
    let end_conn = end_note.get_connection_point(Side::Right);
    
    // Verify start and end
    assert_eq!(path[0].x, start_conn.0 as isize);
    assert_eq!(path[5].x, end_conn.0 as isize);
    
    // Verify it goes right (offset should be x + 2)
    assert_eq!(path[1].x, path[0].x + 2);
    assert_eq!(path[4].x, path[5].x + 2);
    
    // The furthest x point should be to the right of both notes
    let furthest_x = path[2].x;
    assert!(furthest_x >= path[1].x.max(path[4].x), 
        "Reverse C-shape should extend right of both offset points");
}

#[test]
fn test_corner_shape_right_to_top() {
    // Right to Top with appropriate placement should create a corner shape
    // Start above, end to the right and below -> uses corner_shapes_1
    let start_note = create_test_note(10, 10, "A");
    let end_note = create_test_note(50, 30, "B");
    
    let path = calculate_path(&start_note, Side::Right, &end_note, Side::Top);
    
    // Corner shape should have 5 points: start, start_off, corner, end_off, end
    assert_eq!(path.len(), 5, "Corner shape should have 5 points");
    
    let start_conn = start_note.get_connection_point(Side::Right);
    let end_conn = end_note.get_connection_point(Side::Top);
    
    // Verify start and end
    assert_eq!(path[0].x, start_conn.0 as isize);
    assert_eq!(path[0].y, start_conn.1 as isize);
    assert_eq!(path[4].x, end_conn.0 as isize);
    assert_eq!(path[4].y, end_conn.1 as isize);
    
    // Verify offset points
    assert_eq!(path[1].x, path[0].x + 2);
    assert_eq!(path[3].y, path[4].y - 2);
    
    // The corner point should be at end_off.x, start_off.y (corner_shapes_1)
    assert_eq!(path[2].x, path[3].x);
    assert_eq!(path[2].y, path[1].y);
}

#[test]
fn test_u_shape_bottom_to_bottom() {
    // Bottom to Bottom should create a u-shape going below both notes
    let start_note = create_test_note(10, 10, "A");
    let end_note = create_test_note(50, 15, "B");
    
    let path = calculate_path(&start_note, Side::Bottom, &end_note, Side::Bottom);
    
    // U-shape should have 6 points
    assert_eq!(path.len(), 6, "U-shape should have 6 points");
    
    let start_conn = start_note.get_connection_point(Side::Bottom);
    let end_conn = end_note.get_connection_point(Side::Bottom);
    
    // Verify start and end
    assert_eq!(path[0].x, start_conn.0 as isize);
    assert_eq!(path[5].x, end_conn.0 as isize);
    
    // Verify it goes down (offset should be y + 2)
    assert_eq!(path[1].y, path[0].y + 2);
    assert_eq!(path[4].y, path[5].y + 2);
    
    // The lowest y point should be below both offset points
    let lowest_y = path[2].y;
    assert!(lowest_y >= path[1].y.max(path[4].y), 
        "U-shape should extend to the 'lowest' of offset points");
    
    // Verify the shape: vertical -> horizontal -> vertical pattern
    assert_eq!(path[1].x, path[2].x); // vertical segment
    assert_eq!(path[2].y, path[3].y); // horizontal segment
    assert_eq!(path[3].x, path[4].x); // vertical segment
}

#[test]
fn test_upside_down_u_shape_top_to_top() {
    // Top to Top should create an upside-down u-shape going above both notes
    let start_note = create_test_note(10, 30, "A");
    let end_note = create_test_note(50, 25, "B");
    
    let path = calculate_path(&start_note, Side::Top, &end_note, Side::Top);
    
    // Upside-down U-shape should have 6 points
    assert_eq!(path.len(), 6, "Upside-down U-shape should have 6 points");
    
    let start_conn = start_note.get_connection_point(Side::Top);
    let end_conn = end_note.get_connection_point(Side::Top);
    
    // Verify start and end
    assert_eq!(path[0].x, start_conn.0 as isize);
    assert_eq!(path[5].x, end_conn.0 as isize);
    
    // Verify it goes up (offset should be y - 2)
    assert_eq!(path[1].y, path[0].y - 2);
    assert_eq!(path[4].y, path[5].y - 2);
    
    // The highest y point (lowest value) should be the highest (lowest value) offset point
    let highest_y = path[2].y;
    assert!(highest_y <= path[1].y.min(path[4].y), 
        "Upside-down U-shape should extend to the highest (lowest value) offset point");
}

#[test]
fn test_sideways_s_shape_vertical() {
    // When horizontal space is limited but vertical space is good, should use sideways s-shape
    let start_note = create_test_note(20, 10, "A");
    let end_note = create_test_note(22, 40, "B"); // Close horizontally, far vertically
    
    let path = calculate_path(&start_note, Side::Right, &end_note, Side::Left);
    
    // Should have 6 points for sideways s-shape
    assert_eq!(path.len(), 6);
    
    let start_conn = start_note.get_connection_point(Side::Right);
    let end_conn = end_note.get_connection_point(Side::Left);
    
    // Verify start and end
    assert_eq!(path[0].y, start_conn.1 as isize);
    assert_eq!(path[5].y, end_conn.1 as isize);
    
    // Verify the middle horizontal segment uses the midpoint in y
    let expected_mid_y = path[0].y + ((end_conn.1 as isize - start_conn.1 as isize) / 2);
    assert_eq!(path[2].y, expected_mid_y);
    assert_eq!(path[3].y, expected_mid_y);
}
