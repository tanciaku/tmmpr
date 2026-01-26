use std::{fs, path::PathBuf};
use tempfile::tempdir;
use ratatui::style::Color;

use crate::{
    app::{App, Screen},
    states::{
        MapState, 
        map::{BackupResult, Connection, Note, Notification, Side},
        start::{ErrMsg, StartState}
    },
    utils::{
        MapData, create_map_file_with_fs, filesystem::test_utils::TempFileSystem, load_map_file_with_fs,
        read_json_data, save_map_file, test_utils::MockFileSystem,
    },
};

// ============================================================================
// Helper Functions
// ============================================================================

fn create_map_state_using_mock_filesystem(path: PathBuf) -> MapState {
    let mock_fs = MockFileSystem::new();
    MapState::new_with_fs(path, &mock_fs)
}

/// Creates a test App with StartState screen
fn create_test_app_with_start_state() -> App {
    let temp_dir = tempdir().unwrap();
    let temp_fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    App {
        running: true,
        screen: Screen::Start(StartState::new_with_fs(&temp_fs)),
    }
}

/// Creates a MapState with some sample data for testing
fn create_populated_map_state(path: PathBuf) -> MapState {
    let mut map_state = create_map_state_using_mock_filesystem(path);
    
    // Add some notes
    let note1 = Note::new(10, 20, String::from("Test Note 1"), true, Color::White);
    let note2 = Note::new(50, 60, String::from("Test Note 2"), false, Color::Green);
    
    map_state.notes_state.notes.insert(0, note1);
    map_state.notes_state.notes.insert(1, note2);
    map_state.notes_state.render_order = vec![0, 1];
    map_state.notes_state.next_note_id = 2;
    
    // Add a connection
    let connection = Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: Some(1),
        to_side: Some(Side::Left),
        color: Color::White,
    };
    
    map_state.connections_state.connections.push(connection);
    map_state.connections_state.connection_index.entry(0).or_default().push(connection);
    map_state.connections_state.connection_index.entry(1).or_default().push(connection);
    
    map_state
}

/// Verifies that a MapData file has the expected default values
fn assert_default_map_data(data: &MapData) {
    assert_eq!(data.next_note_id, 0);
    assert!(data.notes.is_empty());
    assert!(data.render_order.is_empty());
    assert!(data.connections.is_empty());
    assert!(data.connection_index.is_empty());
}

// ============================================================================
// Tests for create_map_file
// ============================================================================

#[test]
fn test_create_map_file_creates_valid_json_file() {
    // Setup: Create temp directory and test app
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test_map.json");
    let mut app = create_test_app_with_start_state();
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    // Execute: Create the map file
    create_map_file_with_fs(&mut app, &file_path, &fs);
    
    // Verify: File exists
    assert!(file_path.exists(), "Map file should be created");
    
    // Verify: File contains valid JSON
    let contents = fs::read_to_string(&file_path).unwrap();
    assert!(!contents.is_empty(), "File should not be empty");
    
    // Verify: JSON can be parsed into MapData
    let data: MapData = serde_json::from_str(&contents).unwrap();
    
    // Verify: Data has expected default values
    assert_default_map_data(&data);
}

#[test]
fn test_create_map_file_transitions_to_map_screen() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test_map.json");
    let mut app = create_test_app_with_start_state();
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    // Before: App is in Start screen
    assert!(matches!(app.screen, Screen::Start(_)));
    
    create_map_file_with_fs(&mut app, &file_path, &fs);
    
    // After: App is in Map screen
    assert!(matches!(app.screen, Screen::Map(_)));
    
    // Verify: Map state has correct file path
    if let Screen::Map(map_state) = &app.screen {
        assert_eq!(map_state.persistence.file_write_path, file_path);
    }
}

#[test]
fn test_create_map_file_with_nested_directory_path() {
    let temp_dir = tempdir().unwrap();
    let nested_path = temp_dir.path().join("nested/dir/structure");
    fs::create_dir_all(&nested_path).unwrap();
    
    let file_path = nested_path.join("map.json");
    let mut app = create_test_app_with_start_state();
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    create_map_file_with_fs(&mut app, &file_path, &fs);
    
    // Verify: File created in nested directory
    assert!(file_path.exists());
    
    // Verify: Transition happened
    assert!(matches!(app.screen, Screen::Map(_)));
}

#[test]
fn test_create_map_file_with_special_characters_in_filename() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("my-map_2024.json");
    let mut app = create_test_app_with_start_state();
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    create_map_file_with_fs(&mut app, &file_path, &fs);
    
    assert!(file_path.exists());
    
    // Verify: JSON is still valid
    let data: MapData = read_json_data(&file_path).unwrap();
    assert_default_map_data(&data);
}

#[test]
fn test_create_map_file_handles_write_error() {
    // Use an invalid path to trigger write error
    let temp_dir = tempdir().unwrap();
    let invalid_path = PathBuf::from("/invalid/nonexistent/path/map.json");
    let mut app = create_test_app_with_start_state();
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    create_map_file_with_fs(&mut app, &invalid_path, &fs);
    
    // Verify: File was not created
    assert!(!invalid_path.exists());
    
    // Verify: App stays on Start screen
    assert!(matches!(app.screen, Screen::Start(_)));
    
    // Verify: Error message is displayed
    if let Screen::Start(start_state) = &app.screen {
        assert_eq!(start_state.display_err_msg, Some(ErrMsg::FileWrite));
    }
}

#[test]
fn test_create_map_file_default_viewpos_values() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test_map.json");
    let mut app = create_test_app_with_start_state();
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    create_map_file_with_fs(&mut app, &file_path, &fs);
    
    // Read the created file
    let data: MapData = read_json_data(&file_path).unwrap();
    
    // ViewPos should have default values (from ViewPos::new())
    assert_eq!(data.view_pos.x, 0);
    assert_eq!(data.view_pos.y, 0);
}

// ============================================================================
// Tests for save_map_file
// ============================================================================

#[test]
fn test_save_map_file_writes_data_correctly() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("save_test.json");
    
    // Create a map state with some data
    let mut map_state = create_populated_map_state(file_path.clone());
    
    // Save the file
    save_map_file(&mut map_state, &file_path, false, false);
    
    // Verify: File exists
    assert!(file_path.exists());
    
    // Verify: File contains the correct data
    let loaded_data: MapData = read_json_data(&file_path).unwrap();
    
    assert_eq!(loaded_data.next_note_id, 2);
    assert_eq!(loaded_data.notes.len(), 2);
    assert_eq!(loaded_data.render_order, vec![0, 1]);
    assert_eq!(loaded_data.connections.len(), 1);
}

#[test]
fn test_save_map_file_sets_can_exit_flag() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.json");
    let mut map_state = create_populated_map_state(file_path.clone());
    
    // Initially can_exit should be true
    map_state.persistence.mark_dirty();
    
    save_map_file(&mut map_state, &file_path, false, false);
    
    // After successful save, can_exit should be true
    assert!(map_state.persistence.can_exit);
}

#[test]
fn test_save_map_file_shows_save_success_notification() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.json");
    let mut map_state = create_populated_map_state(file_path.clone());
    
    map_state.ui_state.clear_notification();
    
    // Save with notification enabled
    save_map_file(&mut map_state, &file_path, true, false);
    
    // Verify: Success notification shown
    assert_eq!(map_state.ui_state.show_notification, Some(Notification::SaveSuccess));
    
    // Verify: Needs redraw
    assert!(map_state.ui_state.needs_clear_and_redraw);
}

#[test]
fn test_save_map_file_without_notification() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.json");
    let mut map_state = create_populated_map_state(file_path.clone());
    
    map_state.ui_state.clear_notification();
    map_state.ui_state.mark_redrawn();
    
    // Save with notification disabled
    save_map_file(&mut map_state, &file_path, false, false);
    
    // Verify: No notification shown
    assert_eq!(map_state.ui_state.show_notification, None);
    
    // Verify: No redraw triggered
    assert!(!map_state.ui_state.needs_clear_and_redraw);
}

#[test]
fn test_save_map_file_backup_mode_success() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.json");
    let mut map_state = create_populated_map_state(file_path.clone());
    
    map_state.persistence.backup_res = None;
    
    // Save in backup mode with notification
    save_map_file(&mut map_state, &file_path, true, true);
    
    // Verify: Backup success result set
    assert_eq!(map_state.persistence.backup_res, Some(BackupResult::BackupSuccess));
    
    // Verify: Backup notification shown
    assert_eq!(map_state.ui_state.show_notification, Some(Notification::BackupSuccess));
}

#[test]
fn test_save_map_file_handles_write_failure() {
    // Use an invalid path to trigger write error
    let invalid_path = PathBuf::from("/invalid/path/map.json");
    let mut map_state = create_populated_map_state(invalid_path.clone());
    
    map_state.ui_state.clear_notification();
    
    // Attempt to save with notification enabled
    save_map_file(&mut map_state, &invalid_path, true, false);
    
    // Verify: Save failure notification shown
    assert_eq!(map_state.ui_state.show_notification, Some(Notification::SaveFail));
    
    // Verify: Redraw triggered
    assert!(map_state.ui_state.needs_clear_and_redraw);
}

#[test]
fn test_save_map_file_backup_mode_failure() {
    let invalid_path = PathBuf::from("/invalid/path/map.json");
    let mut map_state = create_populated_map_state(invalid_path.clone());
    
    map_state.persistence.backup_res = None;
    
    // Attempt backup save with notification
    save_map_file(&mut map_state, &invalid_path, true, true);
    
    // Verify: Backup failure result set
    assert_eq!(map_state.persistence.backup_res, Some(BackupResult::BackupFail));
    
    // Verify: Backup failure notification shown
    assert_eq!(map_state.ui_state.show_notification, Some(Notification::BackupFail));
}

#[test]
fn test_save_map_file_overwrites_existing_file() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("overwrite_test.json");
    
    // Create initial map state and save
    let mut map_state1 = create_map_state_using_mock_filesystem(file_path.clone());
    save_map_file(&mut map_state1, &file_path, false, false);
    
    // Read the initial data
    let initial_data: MapData = read_json_data(&file_path).unwrap();
    assert_eq!(initial_data.next_note_id, 0);
    
    // Create a new map state with different data and save to same file
    let mut map_state2 = create_populated_map_state(file_path.clone());
    map_state2.notes_state.next_note_id = 5;
    save_map_file(&mut map_state2, &file_path, false, false);
    
    // Verify: File was overwritten with new data
    let updated_data: MapData = read_json_data(&file_path).unwrap();
    assert_eq!(updated_data.next_note_id, 5);
    assert_eq!(updated_data.notes.len(), 2);
}

#[test]
fn test_save_map_file_preserves_note_properties() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.json");
    
    let mut map_state = create_map_state_using_mock_filesystem(file_path.clone());
    
    // Add a note with specific properties
    let note = Note::new(100, 200, String::from("Important Note"), true, Color::Cyan);
    map_state.notes_state.notes.insert(42, note);
    map_state.notes_state.render_order.push(42);
    map_state.notes_state.next_note_id = 43;
    
    save_map_file(&mut map_state, &file_path, false, false);
    
    // Verify: All note properties are preserved
    let loaded_data: MapData = read_json_data(&file_path).unwrap();
    
    let loaded_note = loaded_data.notes.get(&42).unwrap();
    assert_eq!(loaded_note.x, 100);
    assert_eq!(loaded_note.y, 200);
    assert_eq!(loaded_note.content, "Important Note");
    assert_eq!(loaded_note.selected, true);
    assert_eq!(loaded_note.color, Color::Cyan);
}

#[test]
fn test_save_map_file_preserves_connections() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.json");
    
    let mut map_state = create_map_state_using_mock_filesystem(file_path.clone());
    
    // Add notes
    map_state.notes_state.notes.insert(0, Note::new(10, 10, String::from("A"), false, Color::White));
    map_state.notes_state.notes.insert(1, Note::new(20, 20, String::from("B"), false, Color::White));
    
    // Add multiple connections
    let conn1 = Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: Some(1),
        to_side: Some(Side::Left),
        color: Color::Red,
    };
    
    let conn2 = Connection {
        from_id: 1,
        from_side: Side::Bottom,
        to_id: None,
        to_side: None,
        color: Color::Blue,
    };
    
    map_state.connections_state.connections.push(conn1);
    map_state.connections_state.connections.push(conn2);
    
    save_map_file(&mut map_state, &file_path, false, false);
    
    // Verify: Connections preserved
    let loaded_data: MapData = read_json_data(&file_path).unwrap();
    assert_eq!(loaded_data.connections.len(), 2);
    assert_eq!(loaded_data.connections[0].from_id, 0);
    assert_eq!(loaded_data.connections[0].color, Color::Red);
    assert_eq!(loaded_data.connections[1].to_id, None);
}

#[test]
fn test_save_map_file_empty_state() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("empty.json");
    
    // Save an empty map state
    let mut map_state = create_map_state_using_mock_filesystem(file_path.clone());
    save_map_file(&mut map_state, &file_path, false, false);
    
    // Verify: File created with default/empty values
    let loaded_data: MapData = read_json_data(&file_path).unwrap();
    assert_default_map_data(&loaded_data);
}

// ============================================================================
// Tests for load_map_file
// ============================================================================

#[test]
fn test_load_map_file_loads_valid_file() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("load_test.json");
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    // Create and save a map file first
    let mut map_state = create_populated_map_state(file_path.clone());
    save_map_file(&mut map_state, &file_path, false, false);
    
    // Now load it with a fresh app
    let mut app = create_test_app_with_start_state();
    
    load_map_file_with_fs(&mut app, &file_path, &fs);
    
    // Verify: App transitioned to Map screen
    assert!(matches!(app.screen, Screen::Map(_)));
    
    // Verify: Data was loaded correctly
    if let Screen::Map(loaded_state) = &app.screen {
        assert_eq!(loaded_state.notes_state.next_note_id, 2);
        assert_eq!(loaded_state.notes_state.notes.len(), 2);
        assert_eq!(loaded_state.notes_state.render_order, vec![0, 1]);
        assert_eq!(loaded_state.connections_state.connections.len(), 1);
        assert_eq!(loaded_state.persistence.file_write_path, file_path);
    }
}

#[test]
fn test_load_map_file_loads_note_properties() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.json");
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    // Create a map with specific note properties
    let mut map_state = create_map_state_using_mock_filesystem(file_path.clone());
    let note = Note::new(123, 456, String::from("Specific Note"), true, Color::Magenta);
    map_state.notes_state.notes.insert(7, note);
    map_state.notes_state.render_order.push(7);
    map_state.notes_state.next_note_id = 8;
    
    save_map_file(&mut map_state, &file_path, false, false);
    
    // Load the file
    let mut app = create_test_app_with_start_state();
    load_map_file_with_fs(&mut app, &file_path, &fs);
    
    // Verify: Note properties preserved
    if let Screen::Map(loaded_state) = &app.screen {
        let loaded_note = loaded_state.notes_state.notes.get(&7).unwrap();
        assert_eq!(loaded_note.x, 123);
        assert_eq!(loaded_note.y, 456);
        assert_eq!(loaded_note.content, "Specific Note");
        assert_eq!(loaded_note.selected, true);
        assert_eq!(loaded_note.color, Color::Magenta);
    }
}

#[test]
fn test_load_map_file_loads_connections() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.json");
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    // Create a map with connections
    let mut map_state = create_map_state_using_mock_filesystem(file_path.clone());
    map_state.notes_state.notes.insert(0, Note::new(10, 10, String::from("A"), false, Color::White));
    map_state.notes_state.notes.insert(1, Note::new(20, 20, String::from("B"), false, Color::White));
    
    let conn = Connection {
        from_id: 0,
        from_side: Side::Bottom,
        to_id: Some(1),
        to_side: Some(Side::Top),
        color: Color::Yellow,
    };
    
    map_state.connections_state.connections.push(conn);
    map_state.connections_state.connection_index.entry(0).or_default().push(conn);
    map_state.connections_state.connection_index.entry(1).or_default().push(conn);
    
    save_map_file(&mut map_state, &file_path, false, false);
    
    // Load the file
    let mut app = create_test_app_with_start_state();
    load_map_file_with_fs(&mut app, &file_path, &fs);
    
    // Verify: Connections loaded
    if let Screen::Map(loaded_state) = &app.screen {
        assert_eq!(loaded_state.connections_state.connections.len(), 1);
        assert_eq!(loaded_state.connections_state.connections[0].from_id, 0);
        assert_eq!(loaded_state.connections_state.connections[0].to_id, Some(1));
        assert_eq!(loaded_state.connections_state.connections[0].from_side, Side::Bottom);
        assert_eq!(loaded_state.connections_state.connections[0].color, Color::Yellow);
        
        // Connection index should also be loaded
        assert_eq!(loaded_state.connections_state.connection_index.len(), 2);
    }
}

#[test]
fn test_load_map_file_loads_view_position() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.json");
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    // Create a map with modified view position
    let mut map_state = create_map_state_using_mock_filesystem(file_path.clone());
    map_state.viewport.view_pos.x = 500;
    map_state.viewport.view_pos.y = 750;
    
    save_map_file(&mut map_state, &file_path, false, false);
    
    // Load the file
    let mut app = create_test_app_with_start_state();
    load_map_file_with_fs(&mut app, &file_path, &fs);
    
    // Verify: View position loaded
    if let Screen::Map(loaded_state) = &app.screen {
        assert_eq!(loaded_state.viewport.view_pos.x, 500);
        assert_eq!(loaded_state.viewport.view_pos.y, 750);
    }
}

#[test]
fn test_load_map_file_handles_missing_file() {
    let temp_dir = tempdir().unwrap();
    let missing_file = temp_dir.path().join("nonexistent.json");
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    let mut app = create_test_app_with_start_state();
    
    load_map_file_with_fs(&mut app, &missing_file, &fs);
    
    // Verify: App stays on Start screen
    assert!(matches!(app.screen, Screen::Start(_)));
    
    // Verify: Error message displayed
    if let Screen::Start(start_state) = &app.screen {
        assert_eq!(start_state.display_err_msg, Some(ErrMsg::FileRead));
    }
}

#[test]
fn test_load_map_file_handles_invalid_json() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("invalid.json");
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    // Write invalid JSON to the file
    fs::write(&file_path, "{ this is not valid json }").unwrap();
    
    let mut app = create_test_app_with_start_state();
    
    load_map_file_with_fs(&mut app, &file_path, &fs);
    
    // Verify: App stays on Start screen
    assert!(matches!(app.screen, Screen::Start(_)));
    
    // Verify: Error message displayed
    if let Screen::Start(start_state) = &app.screen {
        assert_eq!(start_state.display_err_msg, Some(ErrMsg::FileRead));
    }
}

#[test]
fn test_load_map_file_handles_corrupt_json() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("corrupt.json");
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    // Write JSON that's valid but missing required fields
    fs::write(&file_path, r#"{"next_note_id": 5}"#).unwrap();
    
    let mut app = create_test_app_with_start_state();
    
    load_map_file_with_fs(&mut app, &file_path, &fs);
    
    // Verify: App stays on Start screen due to deserialization error
    assert!(matches!(app.screen, Screen::Start(_)));
    
    // Verify: Error message displayed
    if let Screen::Start(start_state) = &app.screen {
        assert_eq!(start_state.display_err_msg, Some(ErrMsg::FileRead));
    }
}

#[test]
fn test_load_map_file_empty_map() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("empty.json");
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    // Save an empty map
    let mut map_state = create_map_state_using_mock_filesystem(file_path.clone());
    save_map_file(&mut map_state, &file_path, false, false);
    
    // Load it
    let mut app = create_test_app_with_start_state();
    load_map_file_with_fs(&mut app, &file_path, &fs);
    
    // Verify: Loaded successfully with empty data
    if let Screen::Map(loaded_state) = &app.screen {
        assert_eq!(loaded_state.notes_state.next_note_id, 0);
        assert!(loaded_state.notes_state.notes.is_empty());
        assert!(loaded_state.notes_state.render_order.is_empty());
        assert!(loaded_state.connections_state.connections.is_empty());
    }
}

#[test]
fn test_load_map_file_with_many_notes() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("many_notes.json");
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    // Create a map with many notes
    let mut map_state = create_map_state_using_mock_filesystem(file_path.clone());
    
    for i in 0..50 {
        let note = Note::new(i * 10, i * 20, format!("Note {}", i), false, Color::White);
        map_state.notes_state.notes.insert(i, note);
        map_state.notes_state.render_order.push(i);
    }
    map_state.notes_state.next_note_id = 50;
    
    save_map_file(&mut map_state, &file_path, false, false);
    
    // Load it
    let mut app = create_test_app_with_start_state();
    load_map_file_with_fs(&mut app, &file_path, &fs);
    
    // Verify: All notes loaded
    if let Screen::Map(loaded_state) = &app.screen {
        assert_eq!(loaded_state.notes_state.notes.len(), 50);
        assert_eq!(loaded_state.notes_state.render_order.len(), 50);
        assert_eq!(loaded_state.notes_state.next_note_id, 50);
        
        // Spot check a few notes
        assert_eq!(loaded_state.notes_state.notes.get(&0).unwrap().content, "Note 0");
        assert_eq!(loaded_state.notes_state.notes.get(&25).unwrap().content, "Note 25");
        assert_eq!(loaded_state.notes_state.notes.get(&49).unwrap().content, "Note 49");
    }
}

// ============================================================================
// Integration Tests (Roundtrip: Save -> Load)
// ============================================================================

#[test]
fn test_roundtrip_save_and_load_preserves_all_data() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("roundtrip.json");
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    // Create a map with comprehensive data
    let mut original_state = create_map_state_using_mock_filesystem(file_path.clone());
    
    // Add diverse notes
    original_state.notes_state.notes.insert(0, Note::new(10, 20, String::from("First"), true, Color::Red));
    original_state.notes_state.notes.insert(1, Note::new(30, 40, String::from("Second"), false, Color::Green));
    original_state.notes_state.notes.insert(5, Note::new(50, 60, String::from("Third"), false, Color::Blue));
    
    original_state.notes_state.render_order = vec![0, 5, 1]; // Non-sequential order
    original_state.notes_state.next_note_id = 6;
    
    // Add connections
    let conn1 = Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: Some(1),
        to_side: Some(Side::Left),
        color: Color::White,
    };
    
    let conn2 = Connection {
        from_id: 5,
        from_side: Side::Bottom,
        to_id: None,
        to_side: None,
        color: Color::Cyan,
    };
    
    original_state.connections_state.connections.push(conn1);
    original_state.connections_state.connections.push(conn2);
    
    original_state.connections_state.connection_index.entry(0).or_default().push(conn1);
    original_state.connections_state.connection_index.entry(1).or_default().push(conn1);
    original_state.connections_state.connection_index.entry(5).or_default().push(conn2);
    
    // Set view position
    original_state.viewport.view_pos.x = 100;
    original_state.viewport.view_pos.y = 200;
    
    // Save the file
    save_map_file(&mut original_state, &file_path, false, false);
    
    // Load the file
    let mut app = create_test_app_with_start_state();
    load_map_file_with_fs(&mut app, &file_path, &fs);
    
    // Verify: Everything matches
    if let Screen::Map(loaded_state) = &app.screen {
        // Check basic state
        assert_eq!(loaded_state.notes_state.next_note_id, 6);
        assert_eq!(loaded_state.viewport.view_pos.x, 100);
        assert_eq!(loaded_state.viewport.view_pos.y, 200);
        
        // Check notes
        assert_eq!(loaded_state.notes_state.notes.len(), 3);
        assert_eq!(loaded_state.notes_state.notes.get(&0).unwrap().content, "First");
        assert_eq!(loaded_state.notes_state.notes.get(&0).unwrap().selected, true);
        assert_eq!(loaded_state.notes_state.notes.get(&1).unwrap().content, "Second");
        assert_eq!(loaded_state.notes_state.notes.get(&5).unwrap().content, "Third");
        
        // Check render order preserved
        assert_eq!(loaded_state.notes_state.render_order, vec![0, 5, 1]);
        
        // Check connections
        assert_eq!(loaded_state.connections_state.connections.len(), 2);
        assert_eq!(loaded_state.connections_state.connections[0].from_id, 0);
        assert_eq!(loaded_state.connections_state.connections[0].to_id, Some(1));
        assert_eq!(loaded_state.connections_state.connections[1].from_id, 5);
        assert_eq!(loaded_state.connections_state.connections[1].to_id, None);
        
        // Check connection index
        assert_eq!(loaded_state.connections_state.connection_index.len(), 3);
    }
}

#[test]
fn test_roundtrip_create_save_load() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("full_cycle.json");
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    // Step 1: Create a new map file
    let mut app = create_test_app_with_start_state();
    create_map_file_with_fs(&mut app, &file_path, &fs);
    
    // Step 2: Modify the map state
    if let Screen::Map(map_state) = &mut app.screen {
        let note = Note::new(100, 200, String::from("Created Note"), false, Color::Magenta);
        map_state.notes_state.notes.insert(0, note);
        map_state.notes_state.render_order.push(0);
        map_state.notes_state.next_note_id = 1;
        map_state.viewport.view_pos.x = 50;
        
        // Save the changes
        save_map_file(map_state, &file_path, false, false);
    }
    
    // Step 3: Load the file in a fresh app
    let mut fresh_app = create_test_app_with_start_state();
    load_map_file_with_fs(&mut fresh_app, &file_path, &fs);
    
    // Step 4: Verify the modifications persisted
    if let Screen::Map(loaded_state) = &fresh_app.screen {
        assert_eq!(loaded_state.notes_state.next_note_id, 1);
        assert_eq!(loaded_state.notes_state.notes.len(), 1);
        assert_eq!(loaded_state.notes_state.notes.get(&0).unwrap().content, "Created Note");
        assert_eq!(loaded_state.viewport.view_pos.x, 50);
    }
}

#[test]
fn test_multiple_save_load_cycles() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("cycles.json");
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    // Create initial map
    let mut map_state = create_map_state_using_mock_filesystem(file_path.clone());
    map_state.notes_state.notes.insert(0, Note::new(10, 10, String::from("V1"), false, Color::White));
    map_state.notes_state.next_note_id = 1;
    save_map_file(&mut map_state, &file_path, false, false);
    
    // Load and modify (cycle 1)
    let mut app = create_test_app_with_start_state();
    load_map_file_with_fs(&mut app, &file_path, &fs);
    
    if let Screen::Map(state) = &mut app.screen {
        state.notes_state.notes.insert(1, Note::new(20, 20, String::from("V2"), false, Color::White));
        state.notes_state.next_note_id = 2;
        save_map_file(state, &file_path, false, false);
    }
    
    // Load and modify (cycle 2)
    let mut app2 = create_test_app_with_start_state();
    load_map_file_with_fs(&mut app2, &file_path, &fs);
    
    if let Screen::Map(state) = &mut app2.screen {
        state.notes_state.notes.insert(2, Note::new(30, 30, String::from("V3"), false, Color::White));
        state.notes_state.next_note_id = 3;
        save_map_file(state, &file_path, false, false);
    }
    
    // Final load - verify all changes persisted
    let mut final_app = create_test_app_with_start_state();
    load_map_file_with_fs(&mut final_app, &file_path, &fs);
    
    if let Screen::Map(final_state) = &final_app.screen {
        assert_eq!(final_state.notes_state.notes.len(), 3);
        assert_eq!(final_state.notes_state.next_note_id, 3);
        assert!(final_state.notes_state.notes.contains_key(&0));
        assert!(final_state.notes_state.notes.contains_key(&1));
        assert!(final_state.notes_state.notes.contains_key(&2));
    }
}

#[test]
fn test_connection_index_roundtrip() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("conn_index.json");
    let fs = TempFileSystem { home_path: temp_dir.path().to_path_buf() };
    
    // Create complex connection structure
    let mut map_state = create_map_state_using_mock_filesystem(file_path.clone());
    
    map_state.notes_state.notes.insert(0, Note::new(10, 10, String::from("A"), false, Color::White));
    map_state.notes_state.notes.insert(1, Note::new(20, 20, String::from("B"), false, Color::White));
    map_state.notes_state.notes.insert(2, Note::new(30, 30, String::from("C"), false, Color::White));
    
    let conn1 = Connection {
        from_id: 0,
        from_side: Side::Right,
        to_id: Some(1),
        to_side: Some(Side::Left),
        color: Color::White,
    };
    
    let conn2 = Connection {
        from_id: 1,
        from_side: Side::Right,
        to_id: Some(2),
        to_side: Some(Side::Left),
        color: Color::White,
    };
    
    map_state.connections_state.connections.extend_from_slice(&[conn1, conn2]);
    
    // Build connection index
    map_state.connections_state.connection_index.entry(0).or_default().push(conn1);
    map_state.connections_state.connection_index.entry(1).or_default().extend_from_slice(&[conn1, conn2]);
    map_state.connections_state.connection_index.entry(2).or_default().push(conn2);
    
    save_map_file(&mut map_state, &file_path, false, false);
    
    // Load and verify
    let mut app = create_test_app_with_start_state();
    load_map_file_with_fs(&mut app, &file_path, &fs);
    
    if let Screen::Map(loaded_state) = &app.screen {
        // Verify connection index structure
        assert_eq!(loaded_state.connections_state.connection_index.len(), 3);
        assert_eq!(loaded_state.connections_state.connection_index.get(&0).unwrap().len(), 1);
        assert_eq!(loaded_state.connections_state.connection_index.get(&1).unwrap().len(), 2);
        assert_eq!(loaded_state.connections_state.connection_index.get(&2).unwrap().len(), 1);
    }
}
