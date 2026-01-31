use std::path::PathBuf;
use ratatui::style::Color;

use crate::{
    input::map::helpers::{cycle_color, cycle_side, help_next_page, help_previous_page, move_note, move_viewport, switch_notes_focus},
    states::{MapState, map::{Connection, Mode, Note, Side}}, utils::test_utils::MockFileSystem,
};

fn create_test_map_state() -> MapState {
    let mock_fs = MockFileSystem::new();
    let mut map_state = MapState::new_with_fs(PathBuf::from("/test/path"), &mock_fs);
    map_state.settings.edit_modal = false;
    map_state.viewport.screen_width = 100;
    map_state.viewport.screen_height = 50;
    map_state.persistence.mark_clean();
    map_state
}

#[test]
fn test_help_next_page_cycles_forward() {
    let mut map_state = create_test_map_state();
    
    // Test cycling through all pages
    map_state.ui_state.show_help(1);
    help_next_page(&mut map_state);
    assert_eq!(map_state.ui_state.help_screen, Some(2));
    
    help_next_page(&mut map_state);
    assert_eq!(map_state.ui_state.help_screen, Some(3));
    
    help_next_page(&mut map_state);
    assert_eq!(map_state.ui_state.help_screen, Some(4));
    
    help_next_page(&mut map_state);
    assert_eq!(map_state.ui_state.help_screen, Some(5));
    
    // Should wrap back to 1
    help_next_page(&mut map_state);
    assert_eq!(map_state.ui_state.help_screen, Some(1));
}

#[test]
fn test_help_next_page_no_help_screen() {
    let mut map_state = create_test_map_state();
    map_state.ui_state.hide_help();
    
    help_next_page(&mut map_state);
    
    // Should remain None when help_screen is not active
    assert_eq!(map_state.ui_state.help_screen, None);
}

#[test]
fn test_help_previous_page_cycles_backward() {
    let mut map_state = create_test_map_state();
    
    // Test cycling backward through all pages
    map_state.ui_state.show_help(1);
    help_previous_page(&mut map_state);
    assert_eq!(map_state.ui_state.help_screen, Some(5));
    
    help_previous_page(&mut map_state);
    assert_eq!(map_state.ui_state.help_screen, Some(4));
    
    help_previous_page(&mut map_state);
    assert_eq!(map_state.ui_state.help_screen, Some(3));
    
    help_previous_page(&mut map_state);
    assert_eq!(map_state.ui_state.help_screen, Some(2));
    
    help_previous_page(&mut map_state);
    assert_eq!(map_state.ui_state.help_screen, Some(1));
}

#[test]
fn test_help_previous_page_no_help_screen() {
    let mut map_state = create_test_map_state();
    map_state.ui_state.hide_help();
    
    help_previous_page(&mut map_state);
    
    // Should remain None when help_screen is not active
    assert_eq!(map_state.ui_state.help_screen, None);
}

#[test]
fn test_move_viewport_x_positive() {
    let mut map_state = create_test_map_state();
    map_state.viewport.view_pos.x = 10;
    map_state.viewport.view_pos.y = 20;
    
    move_viewport(&mut map_state, "x", 5);
    
    assert_eq!(map_state.viewport.view_pos.x, 15);
    assert_eq!(map_state.viewport.view_pos.y, 20); // Should remain unchanged
    assert_eq!(map_state.persistence.has_unsaved_changes, true); // Should be set to true
}

#[test]
fn test_move_viewport_x_negative() {
    let mut map_state = create_test_map_state();
    map_state.viewport.view_pos.x = 10;
    map_state.viewport.view_pos.y = 20;
    
    move_viewport(&mut map_state, "x", -5);
    
    assert_eq!(map_state.viewport.view_pos.x, 5);
    assert_eq!(map_state.viewport.view_pos.y, 20); // Should remain unchanged
    assert_eq!(map_state.persistence.has_unsaved_changes, true);
}

#[test]
fn test_move_viewport_x_negative_saturating() {
    let mut map_state = create_test_map_state();
    map_state.viewport.view_pos.x = 3;
    map_state.viewport.view_pos.y = 20;
    
    move_viewport(&mut map_state, "x", -10);
    
    assert_eq!(map_state.viewport.view_pos.x, 0); // Should saturate at 0
    assert_eq!(map_state.viewport.view_pos.y, 20);
    assert_eq!(map_state.persistence.has_unsaved_changes, true);
}

#[test]
fn test_move_viewport_y_positive() {
    let mut map_state = create_test_map_state();
    map_state.viewport.view_pos.x = 10;
    map_state.viewport.view_pos.y = 20;
    
    move_viewport(&mut map_state, "y", 7);
    
    assert_eq!(map_state.viewport.view_pos.x, 10); // Should remain unchanged
    assert_eq!(map_state.viewport.view_pos.y, 27);
    assert_eq!(map_state.persistence.has_unsaved_changes, true);
}

#[test]
fn test_move_viewport_y_negative() {
    let mut map_state = create_test_map_state();
    map_state.viewport.view_pos.x = 10;
    map_state.viewport.view_pos.y = 20;
    
    move_viewport(&mut map_state, "y", -8);
    
    assert_eq!(map_state.viewport.view_pos.x, 10); // Should remain unchanged
    assert_eq!(map_state.viewport.view_pos.y, 12);
    assert_eq!(map_state.persistence.has_unsaved_changes, true);
}

#[test]
fn test_move_viewport_y_negative_saturating() {
    let mut map_state = create_test_map_state();
    map_state.viewport.view_pos.x = 10;
    map_state.viewport.view_pos.y = 5;
    
    move_viewport(&mut map_state, "y", -10);
    
    assert_eq!(map_state.viewport.view_pos.x, 10);
    assert_eq!(map_state.viewport.view_pos.y, 0); // Should saturate at 0
    assert_eq!(map_state.persistence.has_unsaved_changes, true);
}

#[test]
fn test_move_viewport_invalid_axis() {
    let mut map_state = create_test_map_state();
    map_state.viewport.view_pos.x = 10;
    map_state.viewport.view_pos.y = 20;
    
    move_viewport(&mut map_state, "z", 5);
    
    // Should not change anything for invalid axis
    assert_eq!(map_state.viewport.view_pos.x, 10);
    assert_eq!(map_state.viewport.view_pos.y, 20);
    assert_eq!(map_state.persistence.has_unsaved_changes, true); // Still should set to true
}

#[test]
fn test_move_note_no_selected_note() {
    let mut map_state = create_test_map_state();
    map_state.notes_state.selected_note = None;
    map_state.viewport.view_pos.x = 10;
    map_state.viewport.view_pos.y = 10;
    
    move_note(&mut map_state, "x", 5);
    
    // Should not change anything when no note is selected
    assert_eq!(map_state.viewport.view_pos.x, 10);
    assert_eq!(map_state.viewport.view_pos.y, 10);
    assert_eq!(map_state.persistence.has_unsaved_changes, false); // Should remain unchanged
}

#[test]
fn test_move_note_x_positive_simple() {
    let mut map_state = create_test_map_state();
    
    // Add a note far from screen edges
    let note = Note::new(40, 20, String::from("Test"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.viewport.view_pos.x = 0;
    map_state.viewport.view_pos.y = 0;
    
    move_note(&mut map_state, "x", 5);
    
    // Note should move
    assert_eq!(map_state.notes_state.notes[&0].x, 45);
    assert_eq!(map_state.notes_state.notes[&0].y, 20); // Should remain unchanged
    // Viewport shouldn't move since note is still in view
    assert_eq!(map_state.viewport.view_pos.x, 0);
    assert_eq!(map_state.viewport.view_pos.y, 0);
    assert_eq!(map_state.persistence.has_unsaved_changes, true);
}

#[test]
fn test_move_note_x_positive_viewport_adjustment() {
    let mut map_state = create_test_map_state();
    
    // Add a note near the right edge of the screen
    let note = Note::new(95, 20, String::from("Test"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.viewport.view_pos.x = 0;
    map_state.viewport.view_pos.y = 0;
    
    move_note(&mut map_state, "x", 5);
    
    // Note should move
    assert_eq!(map_state.notes_state.notes[&0].x, 100);
    // Viewport should also move to keep note in view
    // Note width + cursor space = minimum 21, so note right edge is at 121
    // Screen width is 100, so viewport needs to move
    assert_eq!(map_state.viewport.view_pos.x, 5);
    assert_eq!(map_state.persistence.has_unsaved_changes, true);
}

#[test]
fn test_move_note_x_negative_simple() {
    let mut map_state = create_test_map_state();
    
    // Add a note far from screen edges
    let note = Note::new(40, 20, String::from("Test"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.viewport.view_pos.x = 0;
    map_state.viewport.view_pos.y = 0;
    
    move_note(&mut map_state, "x", -5);
    
    // Note should move
    assert_eq!(map_state.notes_state.notes[&0].x, 35);
    assert_eq!(map_state.notes_state.notes[&0].y, 20);
    // Viewport shouldn't move since note is still in view
    assert_eq!(map_state.viewport.view_pos.x, 0);
    assert_eq!(map_state.viewport.view_pos.y, 0);
    assert_eq!(map_state.persistence.has_unsaved_changes, true);
}

#[test]
fn test_move_note_x_negative_viewport_adjustment() {
    let mut map_state = create_test_map_state();
    
    // Add a note and set viewport so note is at left edge
    let note = Note::new(5, 20, String::from("Test"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.viewport.view_pos.x = 5;
    map_state.viewport.view_pos.y = 0;
    
    move_note(&mut map_state, "x", -3);
    
    // Note should move
    assert_eq!(map_state.notes_state.notes[&0].x, 2);
    // Viewport should move to keep note in view
    assert_eq!(map_state.viewport.view_pos.x, 2);
    assert_eq!(map_state.persistence.has_unsaved_changes, true);
}

#[test]
fn test_move_note_y_positive_simple() {
    let mut map_state = create_test_map_state();
    
    // Add a note far from screen edges
    let note = Note::new(40, 20, String::from("Test"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.viewport.view_pos.x = 0;
    map_state.viewport.view_pos.y = 0;
    
    move_note(&mut map_state, "y", 5);
    
    // Note should move
    assert_eq!(map_state.notes_state.notes[&0].x, 40);
    assert_eq!(map_state.notes_state.notes[&0].y, 25);
    // Viewport shouldn't move since note is still in view
    assert_eq!(map_state.viewport.view_pos.x, 0);
    assert_eq!(map_state.viewport.view_pos.y, 0);
    assert_eq!(map_state.persistence.has_unsaved_changes, true);
}

#[test]
fn test_move_note_y_positive_viewport_adjustment() {
    let mut map_state = create_test_map_state();
    
    // Add a note near the bottom of the screen
    // Screen height is 50, minus 3 for info bar = 47 effective height
    let note = Note::new(40, 45, String::from("Test"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.viewport.view_pos.x = 0;
    map_state.viewport.view_pos.y = 0;
    
    move_note(&mut map_state, "y", 5);
    
    // Note should move
    assert_eq!(map_state.notes_state.notes[&0].y, 50);
    // Viewport should move to keep note in view
    // Note height is minimum 4, so bottom at y=54, viewport limit is 47, so viewport moves
    assert_eq!(map_state.viewport.view_pos.y, 5);
    assert_eq!(map_state.persistence.has_unsaved_changes, true);
}

#[test]
fn test_move_note_y_negative_simple() {
    let mut map_state = create_test_map_state();
    
    // Add a note far from screen edges
    let note = Note::new(40, 20, String::from("Test"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.viewport.view_pos.x = 0;
    map_state.viewport.view_pos.y = 0;
    
    move_note(&mut map_state, "y", -5);
    
    // Note should move
    assert_eq!(map_state.notes_state.notes[&0].x, 40);
    assert_eq!(map_state.notes_state.notes[&0].y, 15);
    // Viewport shouldn't move since note is still in view
    assert_eq!(map_state.viewport.view_pos.x, 0);
    assert_eq!(map_state.viewport.view_pos.y, 0);
    assert_eq!(map_state.persistence.has_unsaved_changes, true);
}

#[test]
fn test_move_note_y_negative_viewport_adjustment() {
    let mut map_state = create_test_map_state();
    
    // Add a note and set viewport so note is at top edge
    let note = Note::new(40, 5, String::from("Test"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    map_state.viewport.view_pos.x = 0;
    map_state.viewport.view_pos.y = 5;
    
    move_note(&mut map_state, "y", -3);
    
    // Note should move
    assert_eq!(map_state.notes_state.notes[&0].y, 2);
    // Viewport should move to keep note in view
    assert_eq!(map_state.viewport.view_pos.y, 2);
    assert_eq!(map_state.persistence.has_unsaved_changes, true);
}

#[test]
fn test_move_note_invalid_axis() {
    let mut map_state = create_test_map_state();
    
    let note = Note::new(40, 20, String::from("Test"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.selected_note = Some(0);
    
    move_note(&mut map_state, "z", 5);
    
    // Note position should remain unchanged for invalid axis
    assert_eq!(map_state.notes_state.notes[&0].x, 40);
    assert_eq!(map_state.notes_state.notes[&0].y, 20);
    assert_eq!(map_state.persistence.has_unsaved_changes, true); // Still should set true
}

#[test]
fn test_switch_notes_focus_no_selected_note() {
    let mut map_state = create_test_map_state();
    map_state.notes_state.selected_note = None;
    
    // Add some notes
    map_state.notes_state.notes.insert(0, Note::new(10, 10, String::from("Note 0"), false, Color::White));
    map_state.notes_state.notes.insert(1, Note::new(50, 25, String::from("Note 1"), false, Color::White));
    
    switch_notes_focus(&mut map_state, "j");
    
    // Should not change anything when no note is selected
    assert_eq!(map_state.notes_state.selected_note, None);
    assert!(!map_state.notes_state.notes[&0].selected);
    assert!(!map_state.notes_state.notes[&1].selected);
}

#[test]
fn test_switch_notes_focus_down_j() {
    let mut map_state = create_test_map_state();
    
    // Add notes with vertical alignment
    map_state.notes_state.notes.insert(0, Note::new(50, 10, String::from("Top"), true, Color::White));
    map_state.notes_state.notes.insert(1, Note::new(50, 30, String::from("Bottom"), false, Color::White)); // Further down
    map_state.notes_state.render_order = vec![1, 0];
    map_state.notes_state.selected_note = Some(0);
    
    switch_notes_focus(&mut map_state, "j");
    
    // Should switch to note 1
    assert_eq!(map_state.notes_state.selected_note, Some(1));
    assert!(!map_state.notes_state.notes[&0].selected);
    assert!(map_state.notes_state.notes[&1].selected);
    // Render order should be updated (selected note moved to back)
    assert_eq!(map_state.notes_state.render_order, vec![0, 1]);
}

#[test]
fn test_switch_notes_focus_up_k() {
    let mut map_state = create_test_map_state();
    
    // Add notes with vertical alignment
    map_state.notes_state.notes.insert(0, Note::new(50, 10, String::from("Top"), false, Color::White));
    map_state.notes_state.notes.insert(1, Note::new(50, 30, String::from("Bottom"), true, Color::White));
    map_state.notes_state.render_order = vec![0, 1];
    map_state.notes_state.selected_note = Some(1);
    
    switch_notes_focus(&mut map_state, "k");
    
    // Should switch to note 0
    assert_eq!(map_state.notes_state.selected_note, Some(0));
    assert!(map_state.notes_state.notes[&0].selected);
    assert!(!map_state.notes_state.notes[&1].selected);
    // Render order should be updated
    assert_eq!(map_state.notes_state.render_order, vec![1, 0]);
}

#[test]
fn test_switch_notes_focus_right_l() {
    let mut map_state = create_test_map_state();
    
    // Add notes with horizontal alignment
    map_state.notes_state.notes.insert(0, Note::new(10, 25, String::from("Left"), true, Color::White));
    map_state.notes_state.notes.insert(1, Note::new(80, 25, String::from("Right"), false, Color::White)); // Further right
    map_state.notes_state.render_order = vec![1, 0];
    map_state.notes_state.selected_note = Some(0);
    
    switch_notes_focus(&mut map_state, "l");
    
    // Should switch to note 1
    assert_eq!(map_state.notes_state.selected_note, Some(1));
    assert!(!map_state.notes_state.notes[&0].selected);
    assert!(map_state.notes_state.notes[&1].selected);
    assert_eq!(map_state.notes_state.render_order, vec![0, 1]);
}

#[test]
fn test_switch_notes_focus_left_h() {
    let mut map_state = create_test_map_state();
    
    // Add notes with horizontal alignment
    map_state.notes_state.notes.insert(0, Note::new(10, 25, String::from("Left"), false, Color::White));
    map_state.notes_state.notes.insert(1, Note::new(80, 25, String::from("Right"), true, Color::White));
    map_state.notes_state.render_order = vec![0, 1];
    map_state.notes_state.selected_note = Some(1);
    
    switch_notes_focus(&mut map_state, "h");
    
    // Should switch to note 0
    assert_eq!(map_state.notes_state.selected_note, Some(0));
    assert!(map_state.notes_state.notes[&0].selected);
    assert!(!map_state.notes_state.notes[&1].selected);
    assert_eq!(map_state.notes_state.render_order, vec![1, 0]);
}

#[test]
fn test_switch_notes_focus_no_valid_candidate() {
    let mut map_state = create_test_map_state();
    
    // Add only one note
    map_state.notes_state.notes.insert(0, Note::new(50, 25, String::from("Only"), true, Color::White));
    map_state.notes_state.selected_note = Some(0);
    
    switch_notes_focus(&mut map_state, "j");
    
    // Should remain on the same note since no valid candidate
    assert_eq!(map_state.notes_state.selected_note, Some(0));
    assert!(map_state.notes_state.notes[&0].selected);
}

#[test]
fn test_switch_notes_focus_cone_selection() {
    let mut map_state = create_test_map_state();
    
    // Add notes in a pattern where cone selection matters
    map_state.notes_state.notes.insert(0, Note::new(50, 25, String::from("Center"), true, Color::White)); // Selected
    map_state.notes_state.notes.insert(1, Note::new(60, 30, String::from("Diagonal"), false, Color::White)); // Diagonal (dx=10, dy=5, dx>dy)
    map_state.notes_state.notes.insert(2, Note::new(70, 26, String::from("Right"), false, Color::White)); // More to the right (dx=20, dy=1, dx>dy)
    map_state.notes_state.selected_note = Some(0);
    
    switch_notes_focus(&mut map_state, "l"); // Move right
    
    // Should choose note 1 because it has the smallest x-coordinate among valid candidates (algorithm uses min_by_key on (x, y_dist))
    assert_eq!(map_state.notes_state.selected_note, Some(1));
    assert!(!map_state.notes_state.notes[&0].selected);
    assert!(map_state.notes_state.notes[&1].selected);
}

#[test]
fn test_switch_notes_focus_with_visual_connection() {
    let mut map_state = create_test_map_state();
    
    // Set up visual connection mode
    map_state.current_mode = Mode::VisualConnectAdd;
    let connection = Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: None,
        to_side: None,
        color: Color::White,
    };
    map_state.connections_state.focused_connection = Some(connection);
    
    // Add notes
    map_state.notes_state.notes.insert(0, Note::new(10, 25, String::from("Start"), true, Color::White));
    map_state.notes_state.notes.insert(1, Note::new(80, 25, String::from("End"), false, Color::White));
    map_state.notes_state.selected_note = Some(0);
    
    switch_notes_focus(&mut map_state, "l");
    
    // Should update the focused connection
    assert_eq!(map_state.notes_state.selected_note, Some(1));
    if let Some(focused_conn) = &map_state.connections_state.focused_connection {
        assert_eq!(focused_conn.to_id, Some(1));
        assert_eq!(focused_conn.to_side, Some(map_state.settings.default_end_side));
    }
    assert_eq!(map_state.persistence.has_unsaved_changes, true);
}

#[test]
fn test_switch_notes_focus_with_visual_connection_same_note() {
    let mut map_state = create_test_map_state();
    
    // Set up visual connection mode
    map_state.current_mode = Mode::VisualConnectAdd;
    let connection = Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: None,
        to_side: None,
        color: Color::White,
    };
    map_state.connections_state.focused_connection = Some(connection);
    
    // Add a note
    map_state.notes_state.notes.insert(0, Note::new(50, 25, String::from("Only"), true, Color::White));
    map_state.notes_state.selected_note = Some(0);
    
    // Add a note to the right and jump to it
    map_state.notes_state.notes.insert(1, Note::new(70, 25, String::from("Too close"), false, Color::White)); 
    switch_notes_focus(&mut map_state, "l");

    // Check that fields changed to target note id
    assert_eq!(map_state.notes_state.selected_note, Some(1));
    if let Some(focused_conn) = &map_state.connections_state.focused_connection {
        assert_eq!(focused_conn.to_id, Some(1));
        assert_eq!(focused_conn.to_side, Some(map_state.settings.default_end_side));
    }

    // Jump back to the connection source (from_id) note
    switch_notes_focus(&mut map_state, "h");
    
    // Should reset to_id and to_side
    // This is to prevent being able to make connection from source note to source note
    assert_eq!(map_state.notes_state.selected_note, Some(0));
    if let Some(focused_conn) = &map_state.connections_state.focused_connection {
        assert_eq!(focused_conn.to_id, None);
        assert_eq!(focused_conn.to_side, None);
    }
}

#[test]
fn test_switch_notes_focus_invalid_key() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.notes.insert(0, Note::new(50, 25, String::from("Test"), true, Color::White));
    map_state.notes_state.selected_note = Some(0);
    
    switch_notes_focus(&mut map_state, "x");
    
    // Should remain unchanged for invalid key
    assert_eq!(map_state.notes_state.selected_note, Some(0));
    assert!(map_state.notes_state.notes[&0].selected);
}

#[test]
fn test_switch_notes_focus_arrow_keys() {
    let mut map_state = create_test_map_state();
    
    // Add notes for arrow key testing
    map_state.notes_state.notes.insert(0, Note::new(50, 25, String::from("Center"), true, Color::White));
    map_state.notes_state.notes.insert(1, Note::new(50, 40, String::from("Down"), false, Color::White));
    map_state.notes_state.notes.insert(2, Note::new(50, 10, String::from("Up"), false, Color::White));
    map_state.notes_state.notes.insert(3, Note::new(80, 25, String::from("Right"), false, Color::White));
    map_state.notes_state.notes.insert(4, Note::new(20, 25, String::from("Left"), false, Color::White));
    map_state.notes_state.render_order = vec![1, 2, 3, 4, 0];
    map_state.notes_state.selected_note = Some(0);
    
    // Test Down arrow
    switch_notes_focus(&mut map_state, "Down");
    assert_eq!(map_state.notes_state.selected_note, Some(1));
    
    // Reset and test Up arrow
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.notes.get_mut(&0).unwrap().selected = true;
    map_state.notes_state.notes.get_mut(&1).unwrap().selected = false;
    switch_notes_focus(&mut map_state, "Up");
    assert_eq!(map_state.notes_state.selected_note, Some(2));
    
    // Reset and test Right arrow
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.notes.get_mut(&0).unwrap().selected = true;
    map_state.notes_state.notes.get_mut(&2).unwrap().selected = false;
    switch_notes_focus(&mut map_state, "Right");
    assert_eq!(map_state.notes_state.selected_note, Some(3));
    
    // Reset and test Left arrow
    map_state.notes_state.selected_note = Some(0);
    map_state.notes_state.notes.get_mut(&0).unwrap().selected = true;
    map_state.notes_state.notes.get_mut(&3).unwrap().selected = false;
    switch_notes_focus(&mut map_state, "Left");
    assert_eq!(map_state.notes_state.selected_note, Some(4));
}

#[test]
fn test_cycle_side() {
    assert_eq!(cycle_side(Side::Right), Side::Bottom);
    assert_eq!(cycle_side(Side::Bottom), Side::Left);
    assert_eq!(cycle_side(Side::Left), Side::Top);
    assert_eq!(cycle_side(Side::Top), Side::Right);
}

#[test]
fn test_cycle_color() {
    assert_eq!(cycle_color(Color::Red), Color::Green);
    assert_eq!(cycle_color(Color::Green), Color::Yellow);
    assert_eq!(cycle_color(Color::Yellow), Color::Blue);
    assert_eq!(cycle_color(Color::Blue), Color::Magenta);
    assert_eq!(cycle_color(Color::Magenta), Color::Cyan);
    assert_eq!(cycle_color(Color::Cyan), Color::White);
    assert_eq!(cycle_color(Color::White), Color::Black);
    assert_eq!(cycle_color(Color::Black), Color::Red);
    
    // Test non-standard colors default to White
    assert_eq!(cycle_color(Color::Gray), Color::White);
    assert_eq!(cycle_color(Color::DarkGray), Color::White);
}