use std::{collections::HashMap, path::PathBuf, time::{Duration, Instant}};
use ratatui::style::Color;

use crate::{states::{MapState, map::{Connection, ModalEditMode, Mode, Note, Side}, start::ErrMsg}, utils::test_utils::MockFileSystem};

fn create_map_state_using_mock_filesystem(path: PathBuf) -> MapState {
    let mock_fs = MockFileSystem::new();
    MapState::new_with_fs(path, &mock_fs)
}

fn create_test_map_state(view_pos_x: usize, view_pos_y: usize, width: usize, height: usize) -> MapState {
    // Create a new MapState and set some simple test values
    let mut map_state = create_map_state_using_mock_filesystem(PathBuf::from("/test/path"));
    map_state.settings.edit_modal = false;

    // Set fields for testing
    map_state.viewport.view_pos.x = view_pos_x;
    map_state.viewport.view_pos.y = view_pos_y;
    map_state.viewport.screen_width = width;
    map_state.viewport.screen_height = height;

    map_state
}

#[test]
fn test_add_note() {
    let mut map_state = create_test_map_state(0, 0, 100, 50);

    map_state.add_note();

    assert_eq!(map_state.persistence.can_exit, false);
    assert_eq!(map_state.notes_state.notes, HashMap::from([(0, Note::new(50, 25, String::from(""), true, Color::White))]));
    assert_eq!(map_state.notes_state.render_order, vec![0]);
    assert_eq!(map_state.notes_state.selected_note, Some(0));
    assert_eq!(map_state.current_mode, Mode::Edit(None));
    assert_eq!(map_state.notes_state.next_note_id, 1);
}

#[test]
fn test_add_several_notes() {
    let mut map_state = create_test_map_state(0, 0, 100, 50);

    map_state.add_note();
    map_state.add_note();
    map_state.add_note();

    assert_eq!(map_state.persistence.can_exit, false);
    assert_eq!(map_state.notes_state.notes, HashMap::from([
        (0, Note::new(50, 25, String::from(""), true, Color::White)),
        (1, Note::new(50, 25, String::from(""), true, Color::White)),
        (2, Note::new(50, 25, String::from(""), true, Color::White)),
    ]));
    assert_eq!(map_state.notes_state.render_order, vec![0, 1, 2]);
    assert_eq!(map_state.notes_state.selected_note, Some(2));
    assert_eq!(map_state.current_mode, Mode::Edit(None));
    assert_eq!(map_state.notes_state.next_note_id, 3);
}

#[test]
fn test_add_note_diff_viewpos() {
    let mut map_state = create_test_map_state(20, 70, 250, 100);

    map_state.add_note();

    assert_eq!(map_state.persistence.can_exit, false);
    assert_eq!(map_state.notes_state.notes, HashMap::from([(0, Note::new(145, 120, String::from(""), true, Color::White))]));
    assert_eq!(map_state.notes_state.render_order, vec![0]);
    assert_eq!(map_state.notes_state.selected_note, Some(0));
    assert_eq!(map_state.current_mode, Mode::Edit(None));
    assert_eq!(map_state.notes_state.next_note_id, 1);
}

#[test]
fn test_new() {
    let path = PathBuf::from("/test/path");
    let map_state = create_map_state_using_mock_filesystem(path.clone());

    // Test initial values
    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
    assert_eq!(map_state.current_mode, Mode::Normal);
    assert_eq!(map_state.viewport.view_pos.x, 0);
    assert_eq!(map_state.viewport.view_pos.y, 0);
    assert_eq!(map_state.viewport.screen_width, 0);
    assert_eq!(map_state.viewport.screen_height, 0);
    assert_eq!(map_state.notes_state.next_note_id, 0);
    assert!(map_state.notes_state.notes.is_empty());
    assert!(map_state.notes_state.render_order.is_empty());
    assert_eq!(map_state.notes_state.selected_note, None);
    assert_eq!(map_state.notes_state.cursor_pos, 0);
    assert_eq!(map_state.visual_mode.visual_move, false);
    assert_eq!(map_state.visual_mode.visual_connection, false);
    assert!(map_state.connections_state.connections.is_empty());
    assert!(map_state.connections_state.connection_index.is_empty());
    assert_eq!(map_state.connections_state.focused_connection, None);
    assert_eq!(map_state.connections_state.visual_editing_a_connection, false);
    assert_eq!(map_state.connections_state.editing_connection_index, None);
    assert_eq!(map_state.persistence.file_write_path, path);
    assert_eq!(map_state.ui_state.show_notification, None);
    assert_eq!(map_state.persistence.can_exit, true);
    assert_eq!(map_state.ui_state.confirm_discard_menu, None);
    assert_eq!(map_state.ui_state.help_screen, None);
    // Since get_setting_with_fs from map_state creation will
    // fail writing to mock directory - assume write error
    assert_eq!(map_state.settings_err_msg, Some(ErrMsg::FileWrite));
    assert_eq!(map_state.persistence.backup_res, None);
}

#[test]
fn test_clear_and_redraw() {
    let mut map_state = create_test_map_state(0, 0, 100, 50);
    map_state.ui_state.mark_redrawn();

    map_state.clear_and_redraw();

    assert_eq!(map_state.ui_state.needs_clear_and_redraw, true);
}

#[test]
fn test_switch_to_edit_mode_without_modal() {
    let mut map_state = create_test_map_state(0, 0, 100, 50);
    map_state.settings.edit_modal = false;
    map_state.current_mode = Mode::Normal;

    map_state.switch_to_edit_mode();

    assert_eq!(map_state.current_mode, Mode::Edit(None));
}

#[test]
fn test_switch_to_edit_mode_with_modal() {
    let mut map_state = create_test_map_state(0, 0, 100, 50);
    map_state.settings.edit_modal = true;
    map_state.current_mode = Mode::Normal;

    map_state.switch_to_edit_mode();

    assert_eq!(map_state.current_mode, Mode::Edit(Some(ModalEditMode::Normal)));
}

#[test]
fn test_select_note_empty_map() {
    let mut map_state = create_test_map_state(0, 0, 100, 50);

    map_state.select_note();

    assert_eq!(map_state.notes_state.selected_note, None);
}

#[test]
fn test_select_note_single_note() {
    let mut map_state = create_test_map_state(0, 0, 100, 50);
    
    // Add a note at position (40, 20) - close to screen center (50, 25)
    map_state.notes_state.notes.insert(0, Note::new(40, 20, String::from("test"), false, Color::White));
    map_state.notes_state.render_order.push(0);
    map_state.notes_state.next_note_id = 1;

    map_state.select_note();

    assert_eq!(map_state.notes_state.selected_note, Some(0));
    // Check that the note is marked as selected
    assert_eq!(map_state.notes_state.notes[&0].selected, true);
    // Check that render order was updated (note moved to back)
    assert_eq!(map_state.notes_state.render_order, vec![0]);
}

#[test]
fn test_select_note_multiple_notes() {
    let mut map_state = create_test_map_state(0, 0, 100, 50);
    
    // Screen center is at (50, 25)
    // Add notes at different distances from center
    map_state.notes_state.notes.insert(0, Note::new(10, 10, String::from("far"), false, Color::White));      // Distance: 40 + 15 = 55
    map_state.notes_state.notes.insert(1, Note::new(45, 20, String::from("close"), false, Color::White));    // Distance: 5 + 5 = 10
    map_state.notes_state.notes.insert(2, Note::new(80, 40, String::from("medium"), false, Color::White));   // Distance: 30 + 15 = 45
    
    map_state.notes_state.render_order = vec![0, 1, 2];
    map_state.notes_state.next_note_id = 3;

    map_state.select_note();

    // Should select note 1 (closest to center)
    assert_eq!(map_state.notes_state.selected_note, Some(1));
    assert_eq!(map_state.notes_state.notes[&1].selected, true);
    assert_eq!(map_state.notes_state.notes[&0].selected, false);
    assert_eq!(map_state.notes_state.notes[&2].selected, false);
    // Check that render order was updated (note 1 moved to back)
    assert_eq!(map_state.notes_state.render_order, vec![0, 2, 1]);
}

#[test]
fn test_stash_connection_with_target() {
    let mut map_state = create_test_map_state(0, 0, 100, 50);
    
    // Create a connection with a target
    let connection = Connection {
        from_id: 1,
        from_side: Side::Right,
        to_id: Some(2),
        to_side: Some(Side::Left),
        color: Color::White,
    };
    map_state.connections_state.focused_connection = Some(connection);

    map_state.connections_state.stash_connection();

    // Connection should be added to connections vector
    assert_eq!(map_state.connections_state.connections.len(), 1);
    assert_eq!(map_state.connections_state.connections[0], connection);
    
    // Connection should be added to connection_index for both from_id and to_id
    assert!(map_state.connections_state.connection_index.contains_key(&1));
    assert!(map_state.connections_state.connection_index.contains_key(&2));
    assert_eq!(map_state.connections_state.connection_index[&1].len(), 1);
    assert_eq!(map_state.connections_state.connection_index[&2].len(), 1);
    
    // focused_connection should be None
    assert_eq!(map_state.connections_state.focused_connection, None);
}

#[test]
fn test_stash_connection_without_target() {
    let mut map_state = create_test_map_state(0, 0, 100, 50);
    
    // Create a connection without a target
    let connection = Connection {
        from_id: 1,
        from_side: Side::Right,
        to_id: None,
        to_side: None,
        color: Color::White,
    };
    map_state.connections_state.focused_connection = Some(connection);

    map_state.connections_state.stash_connection();

    // Connection should not be added to connections vector (dropped)
    assert_eq!(map_state.connections_state.connections.len(), 0);
    assert!(map_state.connections_state.connection_index.is_empty());
    
    // focused_connection should be None
    assert_eq!(map_state.connections_state.focused_connection, None);
}

#[test]
fn test_take_out_connection() {
    let mut map_state = create_test_map_state(0, 0, 100, 50);
    
    // Set up a connection
    let connection = Connection {
        from_id: 1,
        from_side: Side::Right,
        to_id: Some(2),
        to_side: Some(Side::Left),
        color: Color::White,
    };
    
    // Add connection to vectors and index
    map_state.connections_state.connections.push(connection);
    map_state.connections_state.connection_index.entry(1).or_default().push(connection);
    map_state.connections_state.connection_index.entry(2).or_default().push(connection);

    map_state.connections_state.take_out_connection(0);

    // Connection should be removed from connections vector
    assert_eq!(map_state.connections_state.connections.len(), 0);
    
    // Connection should be removed from connection_index
    assert_eq!(map_state.connections_state.connection_index[&1].len(), 0);
    assert_eq!(map_state.connections_state.connection_index[&2].len(), 0);
    
    // focused_connection should now contain the removed connection
    assert_eq!(map_state.connections_state.focused_connection, Some(connection));
}

#[test]
fn test_on_tick_save_changes_disabled() {
    let mut map_state = create_test_map_state(0, 0, 100, 50);
    map_state.settings.save_interval = None;
    map_state.settings.runtime_backups_interval = None;
    map_state.persistence.mark_dirty(); // Simulate unsaved changes
    
    let old_last_save = map_state.persistence.last_save;

    map_state.on_tick_save_changes();

    // Timestamps should not change when saving is disabled
    assert_eq!(map_state.persistence.last_save, old_last_save);
}

#[test]
fn test_on_tick_save_changes_not_enough_time_passed() {
    let mut map_state = create_test_map_state(0, 0, 100, 50);
    map_state.settings.save_interval = Some(20); // 20 seconds
    map_state.settings.runtime_backups_interval = None;
    map_state.persistence.mark_dirty(); // Simulate unsaved changes
    map_state.persistence.last_save = Instant::now(); // Just saved
    
    let old_last_save = map_state.persistence.last_save;

    map_state.on_tick_save_changes();

    // Should not trigger save since not enough time has passed
    assert_eq!(map_state.persistence.last_save, old_last_save);
}

#[test]
fn test_on_tick_save_changes_no_unsaved_changes() {
    let mut map_state = create_test_map_state(0, 0, 100, 50);
    map_state.settings.save_interval = Some(20); // 20 seconds
    map_state.settings.runtime_backups_interval = None;
    map_state.persistence.mark_clean(); // No unsaved changes
    map_state.persistence.last_save = Instant::now() - Duration::from_secs(30); // Long time ago
    
    let old_last_save = map_state.persistence.last_save;

    map_state.on_tick_save_changes();

    // Should not trigger save since there are no unsaved changes
    assert_eq!(map_state.persistence.last_save, old_last_save);
}