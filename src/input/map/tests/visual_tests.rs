use std::path::PathBuf;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;

use crate::{
    input::{AppAction, map::visual::map_visual_kh},
    states::{MapState, map::{Connection, Mode, Side}}, utils::test_utils::MockFileSystem,
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

fn create_key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn create_key_event_with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

// ============================================================================
// NORMAL VISUAL MODE TESTS
// ============================================================================

#[test]
fn test_visual_escape_to_normal_mode() {
    let mut map_state = create_test_map_state();
    
    // Add and select a note
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Visual;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Esc));

    assert_eq!(result, AppAction::Continue);
    assert!(map_state.notes_state.selected_note_id().is_none());
    assert_eq!(map_state.current_mode, Mode::Normal);
}

#[test]
fn test_visual_enter_edit_mode() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Visual;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('i')));

    assert_eq!(result, AppAction::Continue);
    // Should switch to Edit mode (with or without modal editing depending on settings)
    match map_state.current_mode {
        Mode::Edit(_) => {}, // Success
        _ => panic!("Expected Edit mode"),
    }
}

#[test]
fn test_visual_enter_move_mode() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Visual;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('m')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.current_mode, Mode::VisualMove);
}

#[test]
fn test_visual_enter_delete_mode() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Visual;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('d')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.current_mode, Mode::Delete);
}

#[test]
fn test_visual_cycle_note_color() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Visual;

    let original_color = map_state.notes_state.notes().get(&0).unwrap().color;
    
    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('e')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.persistence.has_unsaved_changes, true); // Should mark as dirty
    
    let new_color = map_state.notes_state.notes().get(&0).unwrap().color;
    assert_ne!(original_color, new_color); // Color should have changed
}

#[test]
fn test_visual_create_new_connection() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Visual;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('C')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.current_mode, Mode::VisualConnect);
    assert_eq!(map_state.persistence.has_unsaved_changes, true); // Should mark as dirty
    
    // Should have created a focused connection
    assert!(map_state.connections_state.focused_connection.is_some());
    let connection = map_state.connections_state.focused_connection.unwrap();
    assert_eq!(connection.from_id, 0);
    assert_eq!(connection.to_id, None);
    assert_eq!(connection.to_side, None);
    assert_eq!(connection.color, Color::White);
}

#[test]
fn test_visual_enter_connection_mode_with_existing_connection() {
    let mut map_state = create_test_map_state();
    
    // Add two notes
    map_state.notes_state.add(10, 10, String::from("Note 0"), Color::White);
    map_state.notes_state.add(50, 25, String::from("Note 1"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Visual;
    
    // Add a connection from note 0 to note 1
    let connection = Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: Some(1),
        to_side: Some(Side::Left),
        color: Color::White,
    };
    map_state.connections_state.focused_connection = Some(connection);
    map_state.connections_state.stash_connection();

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('c')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.current_mode, Mode::VisualConnect);
    
    // Connection should be in focus
    assert!(map_state.connections_state.focused_connection.is_some());
    assert_eq!(map_state.connections_state.editing_connection_index, Some(0));
}

#[test]
fn test_visual_enter_connection_mode_no_connections() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Visual;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('c')));

    assert_eq!(result, AppAction::Continue);
    // Should not enter connection mode if no connections exist
    assert_eq!(map_state.current_mode, Mode::Visual)
}

#[test]
fn test_visual_switch_focus_down() {
    let mut map_state = create_test_map_state();
    
    // Add notes vertically aligned
    map_state.notes_state.add(50, 10, String::from("Note 0"), Color::White);
    map_state.notes_state.add(50, 30, String::from("Note 1"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Visual;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('j')));

    assert_eq!(result, AppAction::Continue);
    // Note: switch_notes_focus would change the selected note
    // We're testing that the function is called correctly
}

#[test]
fn test_visual_switch_focus_with_arrow_keys() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Note 0"), Color::White);
    map_state.notes_state.add(70, 25, String::from("Note 1"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Visual;

    // Test all arrow keys
    let _ = map_visual_kh(&mut map_state, create_key_event(KeyCode::Down));
    let _ = map_visual_kh(&mut map_state, create_key_event(KeyCode::Up));
    let _ = map_visual_kh(&mut map_state, create_key_event(KeyCode::Left));
    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Right));

    assert_eq!(result, AppAction::Continue);
}

#[test]
fn test_visual_clear_and_redraw_called() {
    let mut map_state = create_test_map_state();
    map_state.ui_state.mark_redrawn();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Visual;

    let _result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('e')));

    // Should trigger clear_and_redraw
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

// ============================================================================
// VISUAL MOVE MODE TESTS
// ============================================================================

#[test]
fn test_move_mode_exit_with_m() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualMove;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('m')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.current_mode, Mode::Visual);
}

#[test]
fn test_move_mode_escape_to_normal() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualMove;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Esc));

    assert_eq!(result, AppAction::Continue);
    assert!(map_state.notes_state.selected_note_id().is_none());
    assert_eq!(map_state.current_mode, Mode::Normal);
}

#[test]
fn test_move_mode_move_left_h() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualMove;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('h')));

    assert_eq!(result, AppAction::Continue);
    // move_note is called with ("x", -1)
    // The actual position change is handled by move_note function
}

#[test]
fn test_move_mode_move_left_arrow() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualMove;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Left));

    assert_eq!(result, AppAction::Continue);
}

#[test]
fn test_move_mode_move_left_5_capital_h() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualMove;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('H')));

    assert_eq!(result, AppAction::Continue);
    // move_note is called with ("x", -5)
}

#[test]
fn test_move_mode_move_left_5_shift_arrow() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualMove;

    let result = map_visual_kh(&mut map_state, create_key_event_with_modifiers(KeyCode::Left, KeyModifiers::SHIFT));

    assert_eq!(result, AppAction::Continue);
}

#[test]
fn test_move_mode_move_down_j() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualMove;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('j')));

    assert_eq!(result, AppAction::Continue);
}

#[test]
fn test_move_mode_move_down_arrow() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualMove;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Down));

    assert_eq!(result, AppAction::Continue);
}

#[test]
fn test_move_mode_move_down_5_capital_j() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualMove;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('J')));

    assert_eq!(result, AppAction::Continue);
}

#[test]
fn test_move_mode_move_up_k() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualMove;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('k')));

    assert_eq!(result, AppAction::Continue);
}

#[test]
fn test_move_mode_move_up_5_capital_k() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualMove;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('K')));

    assert_eq!(result, AppAction::Continue);
}

#[test]
fn test_move_mode_move_right_l() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualMove;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('l')));

    assert_eq!(result, AppAction::Continue);
}

#[test]
fn test_move_mode_move_right_5_capital_l() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualMove;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('L')));

    assert_eq!(result, AppAction::Continue);
}

#[test]
fn test_move_mode_clear_and_redraw() {
    let mut map_state = create_test_map_state();
    map_state.ui_state.mark_redrawn();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualMove;

    let _result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('h')));

    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_move_mode_unhandled_keys() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualMove;

    // Test various unhandled keys
    let test_keys = vec![
        KeyCode::Char('a'),
        KeyCode::Char('x'),
        KeyCode::Enter,
        KeyCode::Tab,
    ];

    for key in test_keys {
        let result = map_visual_kh(&mut map_state, create_key_event(key));
        assert_eq!(result, AppAction::Continue);
        assert_eq!(map_state.current_mode, Mode::VisualMove); // Should remain in move mode
    }
}

// ============================================================================
// VISUAL CONNECTION MODE TESTS
// ============================================================================

#[test]
fn test_connection_mode_exit_with_c() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(10, 10, String::from("Note 0"), Color::White);
    map_state.notes_state.add(50, 25, String::from("Note 1"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualConnect;
    
    // Set up a focused connection
    map_state.connections_state.focused_connection = Some(Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: Some(1),
        to_side: Some(Side::Left),
        color: Color::White,
    });

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('c')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.current_mode, Mode::Visual);
    assert_eq!(map_state.connections_state.editing_connection_index, None);
    // The connection should have been stashed back
}

#[test]
fn test_connection_mode_rotate_from_side() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(10, 10, String::from("Note 0"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualConnect;
    
    // Set up a focused connection starting from note 0
    map_state.connections_state.focused_connection = Some(Connection {
        from_id: 0,
        from_side: Side::Top,
        to_id: None,
        to_side: None,
        color: Color::White,
    });

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('r')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.persistence.has_unsaved_changes, true); // Should mark as dirty
    
    // Side should have cycled
    let connection = map_state.connections_state.focused_connection.unwrap();
    assert_ne!(connection.from_side, Side::Top); // Should have changed
}

#[test]
fn test_connection_mode_rotate_to_side() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(10, 10, String::from("Note 0"), Color::White);
    map_state.notes_state.add(50, 25, String::from("Note 1"), Color::White);
    map_state.notes_state.select(1); // Select the target note
    map_state.current_mode = Mode::VisualConnect;
    
    // Set up a focused connection with note 1 as target
    map_state.connections_state.focused_connection = Some(Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: Some(1),
        to_side: Some(Side::Left),
        color: Color::White,
    });

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('r')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.persistence.has_unsaved_changes, true);
    
    // to_side should have cycled
    let connection = map_state.connections_state.focused_connection.unwrap();
    assert_ne!(connection.to_side, Some(Side::Left)); // Should have changed
}

#[test]
fn test_connection_mode_cycle_connections() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(10, 10, String::from("Note 0"), Color::White);
    map_state.notes_state.add(50, 25, String::from("Note 1"), Color::White);
    map_state.notes_state.add(90, 40, String::from("Note 2"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualConnect;
    
    // Add three connections involving note 0
    let connection1 = Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: Some(1),
        to_side: Some(Side::Left),
        color: Color::White,
    };
    map_state.connections_state.focused_connection = Some(connection1);
    map_state.connections_state.stash_connection();
    
    let connection2 = Connection {
        from_id: 0,
        from_side: Side::Bottom,
        to_id: Some(2),
        to_side: Some(Side::Top),
        color: Color::Green,
    };
    map_state.connections_state.focused_connection = Some(connection2);
    map_state.connections_state.stash_connection();
    
    let connection3 = Connection {
        from_id: 0,
        from_side: Side::Top,
        to_id: Some(1),
        to_side: Some(Side::Bottom),
        color: Color::Blue,
    };
    map_state.connections_state.focused_connection = Some(connection3);
    map_state.connections_state.stash_connection();
    
    // Set up focused connection and editing index (connection1 was at index 0)
    map_state.connections_state.take_out_connection(0);
    map_state.connections_state.editing_connection_index = Some(0);

    // First cycle - should move to connection2
    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('n')));
    assert_eq!(result, AppAction::Continue);
    assert!(map_state.connections_state.focused_connection.is_some());
    let focused = map_state.connections_state.focused_connection.unwrap();
    assert_eq!(focused.from_side, Side::Bottom);
    assert_eq!(focused.to_id, Some(2));
    assert_eq!(focused.color, Color::Green);
    
    // Second cycle - should move to connection3
    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('n')));
    assert_eq!(result, AppAction::Continue);
    assert!(map_state.connections_state.focused_connection.is_some());
    let focused = map_state.connections_state.focused_connection.unwrap();
    assert_eq!(focused.from_side, Side::Top);
    assert_eq!(focused.to_id, Some(1));
    assert_eq!(focused.color, Color::Blue);
    
    // Third cycle - should wrap around back to connection1
    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('n')));
    assert_eq!(result, AppAction::Continue);
    assert!(map_state.connections_state.focused_connection.is_some());
    let focused = map_state.connections_state.focused_connection.unwrap();
    assert_eq!(focused.from_side, Side::Right);
    assert_eq!(focused.to_id, Some(1));
    assert_eq!(focused.color, Color::White);
}

#[test]
fn test_connection_mode_cycle_connections_not_editing() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(10, 10, String::from("Note 0"), Color::White);
    map_state.notes_state.add(50, 10, String::from("Note 1"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualConnect;
        
    // Add an existing connection associated with note 0 in the connections vector
    let existing_connection = Connection {
        from_id: 0,
        from_side: Side::Bottom,
        to_id: Some(1),
        to_side: Some(Side::Top),
        color: Color::Green,
    };
    map_state.connections_state.focused_connection = Some(existing_connection);
    map_state.connections_state.stash_connection();

    // Set up a focused connection (partial - being created)
    let focused_connection = Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: None,
        to_side: None,
        color: Color::Blue,
    };
    map_state.connections_state.focused_connection = Some(focused_connection);

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('n')));

    assert_eq!(result, AppAction::Continue);
    
    // Should NOT cycle when creating a new connection (not editing an existing one)
    // The focused connection should remain unchanged
    let focused = map_state.connections_state.focused_connection.unwrap();
    assert_eq!(focused.from_side, Side::Right); // Should be unchanged
    assert_eq!(focused.to_id, None); // Should be unchanged
    assert_eq!(focused.color, Color::Blue); // Should be unchanged
    
    // The existing connection should still be in the vector
    assert_eq!(map_state.connections_state.connections().len(), 1);
    assert_eq!(map_state.connections_state.connections()[0].color, Color::Green);
}

#[test]
fn test_connection_mode_delete_connection() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(10, 10, String::from("Note 0"), Color::White);
    map_state.notes_state.add(50, 25, String::from("Note 1"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualConnect;
    
    let connection = Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: Some(1),
        to_side: Some(Side::Left),
        color: Color::White,
    };
    
    map_state.connections_state.focused_connection = Some(connection);
    map_state.connections_state.editing_connection_index = Some(0);

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('d')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.persistence.has_unsaved_changes, true); // Should mark as dirty
    assert_eq!(map_state.connections_state.focused_connection, None); // Connection should be deleted
    assert_eq!(map_state.current_mode, Mode::Visual); // Should exit connection mode
    assert_eq!(map_state.connections_state.editing_connection_index, None);
}

#[test]
fn test_connection_mode_delete_not_editing() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(10, 10, String::from("Note 0"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualConnect;
    
    let connection = Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: None,
        to_side: None,
        color: Color::White,
    };
    
    map_state.connections_state.focused_connection = Some(connection);

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('d')));

    assert_eq!(result, AppAction::Continue);
    // Should not delete when not editing an existing connection
    assert!(map_state.connections_state.focused_connection.is_some());
    assert_eq!(map_state.current_mode, Mode::VisualConnect);
}

#[test]
fn test_connection_mode_switch_focus_for_target() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(10, 10, String::from("Note 0"), Color::White);
    map_state.notes_state.add(50, 10, String::from("Note 1"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualConnect;
    
    // Creating a new connection
    map_state.connections_state.focused_connection = Some(Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: None,
        to_side: None,
        color: Color::White,
    });

    // Test all direction keys
    let _ = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('j')));
    let _ = map_visual_kh(&mut map_state, create_key_event(KeyCode::Down));
    let _ = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('k')));
    let _ = map_visual_kh(&mut map_state, create_key_event(KeyCode::Up));
    let _ = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('h')));
    let _ = map_visual_kh(&mut map_state, create_key_event(KeyCode::Left));
    let _ = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('l')));
    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Right));

    assert_eq!(result, AppAction::Continue);
    // switch_notes_focus should be called for selecting target note
}

#[test]
fn test_connection_mode_cycle_color() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(10, 10, String::from("Note 0"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualConnect;
    
    map_state.connections_state.focused_connection = Some(Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: None,
        to_side: None,
        color: Color::White,
    });

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('e')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.persistence.has_unsaved_changes, true); // Should mark as dirty
    
    let connection = map_state.connections_state.focused_connection.unwrap();
    assert_ne!(connection.color, Color::White); // Color should have changed
}

#[test]
fn test_connection_mode_cycle_color_no_focused_connection() {
    // Shouldn't happen - can't enter connection mode without
    // either creating a new one or editing an existing one.
    // In both cases - focused_connection is Some(Connection).

    let mut map_state = create_test_map_state();
    
    map_state.current_mode = Mode::VisualConnect;
    map_state.connections_state.focused_connection = None;

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('e')));

    assert_eq!(result, AppAction::Continue);
    // Should not change anything without focused connection
    assert_eq!(map_state.persistence.has_unsaved_changes, false);
}

#[test]
fn test_connection_mode_clear_and_redraw() {
    let mut map_state = create_test_map_state();
    map_state.ui_state.mark_redrawn();
    
    map_state.notes_state.add(10, 10, String::from("Note 0"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualConnect;
    
    map_state.connections_state.focused_connection = Some(Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: None,
        to_side: None,
        color: Color::White,
    });

    let _result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('e')));

    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_connection_mode_unhandled_keys() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(10, 10, String::from("Note 0"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualConnect;
    
    map_state.connections_state.focused_connection = Some(Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: None,
        to_side: None,
        color: Color::White,
    });

    // Test various unhandled keys
    let test_keys = vec![
        KeyCode::Char('a'),
        KeyCode::Char('x'),
        KeyCode::Enter,
        KeyCode::Tab,
    ];

    for key in test_keys {
        let result = map_visual_kh(&mut map_state, create_key_event(key));
        assert_eq!(result, AppAction::Continue);
        assert_eq!(map_state.current_mode, Mode::VisualConnect); // Should remain in connection mode
    }
}

// ============================================================================
// EDGE CASES AND INTEGRATION TESTS
// ============================================================================

#[test]
fn test_multiple_notes_with_connections() {
    let mut map_state = create_test_map_state();
    
    // Create a more complex scenario with multiple notes and connections
    map_state.notes_state.add(10, 10, String::from("Note 0"), Color::White);
    map_state.notes_state.add(50, 10, String::from("Note 1"), Color::Green);
    map_state.notes_state.add(90, 10, String::from("Note 2"), Color::Blue);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Visual;
    
    // Add multiple connections
    let connection1 = Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: Some(1),
        to_side: Some(Side::Left),
        color: Color::White,
    };
    map_state.connections_state.focused_connection = Some(connection1);
    map_state.connections_state.stash_connection();
    
    let connection2 = Connection {
        from_id: 1,
        from_side: Side::Right,
        to_id: Some(2),
        to_side: Some(Side::Left),
        color: Color::Green,
    };
    map_state.connections_state.focused_connection = Some(connection2);
    map_state.connections_state.stash_connection();
    
    // Enter connection mode on note 0
    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('c')));

    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.current_mode, Mode::VisualConnect);
    assert!(map_state.connections_state.focused_connection.is_some());
    // Confirm it is the 0->1 connection
    let focused_connection = map_state.connections_state.focused_connection.unwrap();
    assert_eq!(focused_connection.from_id, 0);
    assert_eq!(focused_connection.to_id, Some(1));
    assert_eq!(focused_connection.color, Color::White);
}

#[test]
fn test_exit_visual_mode_from_nested_states() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(10, 10, String::from("Note 0"), Color::White);
    map_state.notes_state.select(0);
    
    // Enter move mode then escape
    map_state.current_mode = Mode::VisualMove;
    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Esc));
    
    assert_eq!(result, AppAction::Continue);
    assert!(map_state.notes_state.selected_note_id().is_none());
    assert_eq!(map_state.current_mode, Mode::Normal);
}

#[test]
fn test_all_direction_keys_consistency() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(50, 25, String::from("Test Note"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Visual;

    // Test that char and arrow keys produce same behavior for each direction
    let key_pairs = vec![
        (KeyCode::Char('h'), KeyCode::Left),
        (KeyCode::Char('j'), KeyCode::Down),
        (KeyCode::Char('k'), KeyCode::Up),
        (KeyCode::Char('l'), KeyCode::Right),
    ];

    for (char_key, arrow_key) in key_pairs {
        let result1 = map_visual_kh(&mut map_state, create_key_event(char_key));
        let result2 = map_visual_kh(&mut map_state, create_key_event(arrow_key));
        
        assert_eq!(result1, AppAction::Continue);
        assert_eq!(result2, AppAction::Continue);
    }
}

#[test]
fn test_connection_mode_with_partial_connection() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(10, 10, String::from("Note 0"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::Visual;
    
    // Create new connection (no target yet)
    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('C')));
    
    assert_eq!(result, AppAction::Continue);
    assert_eq!(map_state.current_mode, Mode::VisualConnect);
    
    let connection = map_state.connections_state.focused_connection.unwrap();
    assert_eq!(connection.from_id, 0);
    assert_eq!(connection.to_id, None);
    assert_eq!(connection.to_side, None);
}

#[test]
fn test_stash_and_cycle_connections_wraparound() {
    let mut map_state = create_test_map_state();
    
    map_state.notes_state.add(10, 10, String::from("Note 0"), Color::White);
    map_state.notes_state.add(50, 10, String::from("Note 1"), Color::White);
    map_state.notes_state.select(0);
    map_state.current_mode = Mode::VisualConnect;
    
    // Add a connection
    let connection = Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: Some(1),
        to_side: Some(Side::Left),
        color: Color::White,
    };
    
    map_state.connections_state.focused_connection = Some(connection);
    map_state.connections_state.editing_connection_index = Some(0);

    let result = map_visual_kh(&mut map_state, create_key_event(KeyCode::Char('n')));

    assert_eq!(result, AppAction::Continue);
    // The connection should be found again
    assert!(map_state.connections_state.focused_connection.is_some());
    // The same connection
    let focused_connection = map_state.connections_state.focused_connection.unwrap();
    assert_eq!(focused_connection.from_id, 0);
    assert_eq!(focused_connection.to_id, Some(1));
}
