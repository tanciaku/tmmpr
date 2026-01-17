use std::path::PathBuf;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;

use crate::{
    input::{map::delete::map_delete_kh, AppAction},
    states::{MapState, map::{Mode, Note, Connection, Side}},
};

fn create_test_map_state() -> MapState {
    let mut map_state = MapState::new(PathBuf::from("/test/path"));
    map_state.settings.edit_modal = false;
    map_state.viewport.screen_width = 100;
    map_state.viewport.screen_height = 50;
    map_state.persistence.mark_clean(); // Start with can_exit as true to test it gets set to false
    map_state
}

fn create_key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn test_delete_kh_no_selected_note() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Delete;
    map_state.notes_state.selected_note = None;

    let result = map_delete_kh(&mut map_state, create_key_event(KeyCode::Char('d')));

    // Should not change anything when no note is selected
    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.current_mode, Mode::Delete);
    assert_eq!(map_state.notes_state.selected_note, None);
    assert_eq!(map_state.persistence.can_exit, true); // Should remain unchanged
}

#[test]
fn test_delete_kh_escape_switches_to_visual() {
    let mut map_state = create_test_map_state();
    map_state.current_mode = Mode::Delete;
    map_state.notes_state.selected_note = Some(0);

    let result = map_delete_kh(&mut map_state, create_key_event(KeyCode::Esc));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.current_mode, Mode::Visual);
    assert_eq!(map_state.notes_state.selected_note, Some(0)); // Should remain selected
}

#[test]
fn test_delete_single_note() {
    let mut map_state = create_test_map_state();
    
    // Add a single note
    let note = Note::new(50, 25, String::from("Test Note"), true, Color::White);
    map_state.notes_state.notes.insert(0, note);
    map_state.notes_state.render_order.push(0);
    map_state.notes_state.selected_note = Some(0);
    map_state.current_mode = Mode::Delete;

    let result = map_delete_kh(&mut map_state, create_key_event(KeyCode::Char('d')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.persistence.can_exit, false); // Should be set to false
    assert!(map_state.notes_state.notes.is_empty()); // Note should be removed
    assert!(map_state.notes_state.render_order.is_empty()); // Render order should be empty
    assert_eq!(map_state.notes_state.selected_note, None); // No selected note
    assert_eq!(map_state.current_mode, Mode::Normal); // Should switch to Normal mode
}

#[test]
fn test_delete_note_with_multiple_notes() {
    let mut map_state = create_test_map_state();
    
    // Add multiple notes
    map_state.notes_state.notes.insert(0, Note::new(10, 10, String::from("Note 0"), false, Color::White));
    map_state.notes_state.notes.insert(1, Note::new(50, 25, String::from("Note 1"), true, Color::White));
    map_state.notes_state.notes.insert(2, Note::new(80, 40, String::from("Note 2"), false, Color::White));
    
    map_state.notes_state.render_order = vec![0, 1, 2];
    map_state.notes_state.selected_note = Some(1); // Select middle note
    map_state.current_mode = Mode::Delete;

    let result = map_delete_kh(&mut map_state, create_key_event(KeyCode::Char('d')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.persistence.can_exit, false);
    
    // Check that only note 1 was removed
    assert_eq!(map_state.notes_state.notes.len(), 2);
    assert!(map_state.notes_state.notes.contains_key(&0));
    assert!(!map_state.notes_state.notes.contains_key(&1)); // This one should be deleted
    assert!(map_state.notes_state.notes.contains_key(&2));
    
    // Check render order
    assert_eq!(map_state.notes_state.render_order, vec![0, 2]);
    
    assert_eq!(map_state.notes_state.selected_note, None);
    assert_eq!(map_state.current_mode, Mode::Normal);
}

#[test]
fn test_delete_note_with_multiple_connections() {
    let mut map_state = create_test_map_state();
    
    // Add three notes
    map_state.notes_state.notes.insert(0, Note::new(10, 10, String::from("Note 0"), false, Color::White));
    map_state.notes_state.notes.insert(1, Note::new(50, 25, String::from("Note 1"), true, Color::White));
    map_state.notes_state.notes.insert(2, Note::new(80, 40, String::from("Note 2"), false, Color::White));
    
    map_state.notes_state.render_order = vec![0, 1, 2];
    map_state.notes_state.selected_note = Some(1); // Select middle note to delete
    
    // Add connections: 0->1 and 1->2
    let connection1 = Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: Some(1),
        to_side: Some(Side::Left),
        color: Color::White,
    };
    
    let connection2 = Connection {
        from_id: 1,
        from_side: Side::Right,
        to_id: Some(2),
        to_side: Some(Side::Left),
        color: Color::Green,
    };
    
    // Add an unrelated connection: 0->2 (should be preserved)
    let connection3 = Connection {
        from_id: 0,
        from_side: Side::Bottom,
        to_id: Some(2),
        to_side: Some(Side::Top),
        color: Color::Blue,
    };
    
    map_state.connections_state.connections.extend_from_slice(&[connection1, connection2, connection3]);
    
    // Set up connection index
    map_state.connections_state.connection_index.entry(0).or_default().extend_from_slice(&[connection1, connection3]);
    map_state.connections_state.connection_index.entry(1).or_default().extend_from_slice(&[connection1, connection2]);
    map_state.connections_state.connection_index.entry(2).or_default().extend_from_slice(&[connection2, connection3]);
    
    map_state.current_mode = Mode::Delete;

    let result = map_delete_kh(&mut map_state, create_key_event(KeyCode::Char('d')));

    assert_eq!(result, AppAction::Continue);
    
    // Note 1 should be deleted
    assert_eq!(map_state.notes_state.notes.len(), 2);
    assert!(map_state.notes_state.notes.contains_key(&0));
    assert!(!map_state.notes_state.notes.contains_key(&1));
    assert!(map_state.notes_state.notes.contains_key(&2));
    
    // Only connection3 (0->2) should remain
    assert_eq!(map_state.connections_state.connections.len(), 1);
    assert_eq!(map_state.connections_state.connections[0], connection3);
    
    // Connection index should be cleaned up
    // Note 0 should only have connection3
    if let Some(connections) = map_state.connections_state.connection_index.get(&0) {
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0], connection3);
    }
    
    // Note 2 should only have connection3
    if let Some(connections) = map_state.connections_state.connection_index.get(&2) {
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0], connection3);
    }
    
    // Note 1 should not exist in connection index
    assert!(!map_state.connections_state.connection_index.contains_key(&1));
    
    assert_eq!(map_state.notes_state.selected_note, None);
    assert_eq!(map_state.current_mode, Mode::Normal);
}

#[test]
fn test_delete_note_as_connection_target() {
    let mut map_state = create_test_map_state();
    
    // Add two notes
    map_state.notes_state.notes.insert(0, Note::new(10, 10, String::from("Note 0"), true, Color::White));
    map_state.notes_state.notes.insert(1, Note::new(50, 25, String::from("Note 1"), false, Color::White));
    
    map_state.notes_state.render_order = vec![0, 1];
    map_state.notes_state.selected_note = Some(0); // Select note that is the target
    
    // Add a connection where 0 is the target: 1->0
    let connection = Connection {
        from_id: 1,
        from_side: Side::Right,
        to_id: Some(0),
        to_side: Some(Side::Left),
        color: Color::White,
    };
    
    map_state.connections_state.connections.push(connection);
    
    // Add to connection index
    map_state.connections_state.connection_index.entry(0).or_default().push(connection);
    map_state.connections_state.connection_index.entry(1).or_default().push(connection);
    
    map_state.current_mode = Mode::Delete;

    let result = map_delete_kh(&mut map_state, create_key_event(KeyCode::Char('d')));

    assert_eq!(result, AppAction::Continue);
    
    // Note 0 should be deleted
    assert_eq!(map_state.notes_state.notes.len(), 1);
    assert!(!map_state.notes_state.notes.contains_key(&0));
    assert!(map_state.notes_state.notes.contains_key(&1));
    
    // Connection should be removed
    assert!(map_state.connections_state.connections.is_empty());
    
    // Connection index should be cleaned up
    assert!(!map_state.connections_state.connection_index.contains_key(&0));
    if let Some(connections) = map_state.connections_state.connection_index.get(&1) {
        assert!(connections.is_empty());
    }
    
    assert_eq!(map_state.notes_state.selected_note, None);
    assert_eq!(map_state.current_mode, Mode::Normal);
}

#[test]
fn test_delete_note_as_connection_source() {
    let mut map_state = create_test_map_state();
    
    // Add two notes
    map_state.notes_state.notes.insert(0, Note::new(10, 10, String::from("Note 0"), true, Color::White));
    map_state.notes_state.notes.insert(1, Note::new(50, 25, String::from("Note 1"), false, Color::White));
    
    map_state.notes_state.render_order = vec![0, 1];
    map_state.notes_state.selected_note = Some(0); // Select note that is the source
    
    // Add a connection where 0 is the source: 0->1
    let connection = Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: Some(1),
        to_side: Some(Side::Left),
        color: Color::White,
    };
    
    map_state.connections_state.connections.push(connection);
    
    // Add to connection index
    map_state.connections_state.connection_index.entry(0).or_default().push(connection);
    map_state.connections_state.connection_index.entry(1).or_default().push(connection);
    
    map_state.current_mode = Mode::Delete;

    let result = map_delete_kh(&mut map_state, create_key_event(KeyCode::Char('d')));

    assert_eq!(result, AppAction::Continue);
    
    // Note 0 should be deleted
    assert_eq!(map_state.notes_state.notes.len(), 1);
    assert!(!map_state.notes_state.notes.contains_key(&0));
    assert!(map_state.notes_state.notes.contains_key(&1));
    
    // Connection should be removed
    assert!(map_state.connections_state.connections.is_empty());
    
    // Connection index should be cleaned up
    assert!(!map_state.connections_state.connection_index.contains_key(&0));
    if let Some(connections) = map_state.connections_state.connection_index.get(&1) {
        assert!(connections.is_empty());
    }
    
    assert_eq!(map_state.notes_state.selected_note, None);
    assert_eq!(map_state.current_mode, Mode::Normal);
}

#[test]
fn test_delete_kh_other_keys_ignored() {
    let mut map_state = create_test_map_state();
    
    // Add a note
    map_state.notes_state.notes.insert(0, Note::new(50, 25, String::from("Test Note"), true, Color::White));
    map_state.notes_state.render_order.push(0);
    map_state.notes_state.selected_note = Some(0);
    map_state.current_mode = Mode::Delete;

    // Test various other keys
    let test_keys = vec![
        KeyCode::Char('a'),
        KeyCode::Char('x'),
        KeyCode::Enter,
        KeyCode::Backspace,
        KeyCode::Delete,
        KeyCode::Tab,
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Left,
        KeyCode::Right,
    ];

    for key in test_keys {
        let result = map_delete_kh(&mut map_state, create_key_event(key));
        
        // Should not change anything for unhandled keys
        assert_eq!(result, AppAction::Continue);
        assert_eq!(map_state.current_mode, Mode::Delete);
        assert_eq!(map_state.notes_state.selected_note, Some(0));
        assert_eq!(map_state.notes_state.notes.len(), 1); // Note should still be there
        assert_eq!(map_state.persistence.can_exit, true); // Should remain unchanged
    }
}

#[test]
fn test_delete_note_render_order_edge_cases() {
    // This shouldn't happen

    let mut map_state = create_test_map_state();
    
    // Add notes with the selected note at different positions in render order
    map_state.notes_state.notes.insert(0, Note::new(10, 10, String::from("Note 0"), false, Color::White));
    map_state.notes_state.notes.insert(1, Note::new(50, 25, String::from("Note 1"), false, Color::White));
    map_state.notes_state.notes.insert(2, Note::new(80, 40, String::from("Note 2"), true, Color::White));
    
    // Test deleting first note in render order
    map_state.notes_state.render_order = vec![2, 1, 0]; // Note 2 is first
    map_state.notes_state.selected_note = Some(2);
    map_state.current_mode = Mode::Delete;

    let result = map_delete_kh(&mut map_state, create_key_event(KeyCode::Char('d')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.render_order, vec![1, 0]); // Note 2 removed from front
    assert_eq!(map_state.notes_state.notes.len(), 2);
    assert!(!map_state.notes_state.notes.contains_key(&2));
}

#[test]
fn test_delete_note_last_in_render_order() {
    // Normal behavior

    let mut map_state = create_test_map_state();
    
    // Add notes
    map_state.notes_state.notes.insert(0, Note::new(10, 10, String::from("Note 0"), false, Color::White));
    map_state.notes_state.notes.insert(1, Note::new(50, 25, String::from("Note 1"), false, Color::White));
    map_state.notes_state.notes.insert(2, Note::new(80, 40, String::from("Note 2"), true, Color::White));
    
    // Test deleting last note in render order
    map_state.notes_state.render_order = vec![0, 1, 2]; // Note 2 is last
    map_state.notes_state.selected_note = Some(2);
    map_state.current_mode = Mode::Delete;

    let result = map_delete_kh(&mut map_state, create_key_event(KeyCode::Char('d')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.notes_state.render_order, vec![0, 1]); // Note 2 removed from end
    assert_eq!(map_state.notes_state.notes.len(), 2);
    assert!(!map_state.notes_state.notes.contains_key(&2));
}

#[test]
fn test_delete_note_clears_and_redraws() {
    let mut map_state = create_test_map_state();
    map_state.needs_clear_and_redraw = false; // Set to false initially
    
    // Add a note
    map_state.notes_state.notes.insert(0, Note::new(50, 25, String::from("Test Note"), true, Color::White));
    map_state.notes_state.render_order.push(0);
    map_state.notes_state.selected_note = Some(0);
    map_state.current_mode = Mode::Delete;

    let _result = map_delete_kh(&mut map_state, create_key_event(KeyCode::Char('d')));

    // Should call clear_and_redraw() which sets needs_clear_and_redraw to true
    assert_eq!(map_state.needs_clear_and_redraw, true);
}

#[test]
fn test_escape_clears_and_redraws() {
    let mut map_state = create_test_map_state();
    map_state.needs_clear_and_redraw = false; // Set to false initially
    map_state.notes_state.selected_note = Some(0);
    map_state.current_mode = Mode::Delete;

    let _result = map_delete_kh(&mut map_state, create_key_event(KeyCode::Esc));

    // Should call clear_and_redraw() which sets needs_clear_and_redraw to true
    assert_eq!(map_state.needs_clear_and_redraw, true);
}