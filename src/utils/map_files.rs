use std::{path::Path, collections::HashMap};
use chrono::Local;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use ratatui::style::Color;

use crate::{
    app::{App, Screen},
    states::{
        MapState,
        map::{Connection, ConnectionsState, Note, Notification, ViewPos},
    },
    utils::{
        IoErrorKind, filesystem::{FileSystem, RealFileSystem}, get_color_from_string, get_color_name_in_string, handle_on_load_backup_with_fs, read_json_data, write_json_data
    },
};

/// Serializable representation of map state for JSON persistence.
/// 
/// Separated from `MapState` to include only the data that needs to be persisted,
/// excluding runtime-only fields.
#[derive(Serialize, Deserialize)]
pub struct MapData {
    pub view_pos: ViewPos,
    #[serde(alias = "next_note_id")]
    pub next_note_id_counter: usize,
    pub notes: HashMap<usize, Note>,
    pub render_order: Vec<usize>,
    pub connections: Vec<Connection>,
}

/// Serializes `ratatui::style::Color` as a human-readable color name string.
pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let color_string = get_color_name_in_string(*color);
    serializer.serialize_str(&color_string)
}

/// Deserializes a color name string back into `ratatui::style::Color`.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    
    Ok(get_color_from_string(&s))
}


/// Creates a new map file at the given path and transitions to the Map screen.
pub fn create_map_file(app: &mut App, path: &Path) {
    create_map_file_with_fs(app, path, &RealFileSystem);
}

/// Creates a new map file with a custom filesystem (testable version).
pub fn create_map_file_with_fs(app: &mut App, path: &Path, fs: &impl FileSystem) {
    let map_state = MapState::new_with_fs(path.to_path_buf(), fs);
    let map_data = MapData {
        view_pos: map_state.viewport.view_pos,
        next_note_id_counter: map_state.notes_state.next_note_id_counter,
        notes: map_state.notes_state.notes,
        render_order: map_state.notes_state.render_order,
        connections: map_state.connections_state.connections().to_vec(),
    };

    if let Err(_) = write_json_data(path, &map_data) {
        if let Screen::Start(start_state) = &mut app.screen {
            start_state.handle_submit_error(IoErrorKind::FileWrite);
        }
        return
    }

    // Always called from Start screen
    if let Screen::Start(start_state) = &mut app.screen {
        if let Ok(recent_paths) = &mut start_state.recent_paths {
            if !recent_paths.contains_path(path) {
                recent_paths.add(path.to_path_buf());
                recent_paths.save_with_fs(fs);
            }
        }
    }

    app.screen = Screen::Map(MapState::new_with_fs(path.to_path_buf(), fs));
}

pub fn save_with_notification(
    map_state: &mut MapState,
    path: &Path,
    success_notif: Notification,
    fail_notif: Notification,
) -> Result<(), Box<dyn std::error::Error>> {
    match save_map_file(map_state, path) {
        Ok(_) => {
            map_state.ui_state.set_notification(success_notif);
            map_state.clear_and_redraw();
            Ok(())
        }
        Err(e) => {
            map_state.ui_state.set_notification(fail_notif);
            map_state.clear_and_redraw();
            Err(e)
        }
    }
}

/// Saves map data to a file.
/// 
/// Updates persistence state to allow exit after successful save.
pub fn save_map_file(map_state: &mut MapState, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let map_data = MapData {
        view_pos: map_state.viewport.view_pos.clone(),
        next_note_id_counter: map_state.notes_state.next_note_id_counter,
        notes: map_state.notes_state.notes.clone(),
        render_order: map_state.notes_state.render_order.clone(),
        connections: map_state.connections_state.connections().to_vec(),
    };

    write_json_data(path, &map_data).inspect(|_| {
        map_state.persistence.mark_clean();
    })
}

/// Loads a map file and transitions to the Map screen.
pub fn load_map_file(app: &mut App, path: &Path) {
    load_map_file_with_fs(app, path, &RealFileSystem);
}

/// Loads a map file with a custom filesystem (testable version).
/// 
/// Only called from the Start screen. On error, shows error message and remains
/// on Start screen to allow retry.
pub fn load_map_file_with_fs(app: &mut App, path: &Path, fs: &impl FileSystem) {
    let mut map_state = MapState::new_with_fs(path.to_path_buf(), fs);

    match read_json_data::<MapData>(path) {
        Ok(map_data) => {
            map_state.viewport.view_pos = map_data.view_pos;
            map_state.notes_state.next_note_id_counter = map_data.next_note_id_counter;
            map_state.notes_state.notes = map_data.notes;
            map_state.notes_state.render_order = map_data.render_order;
            map_state.connections_state = ConnectionsState::from_connections(map_data.connections);
        }
        Err(_) => {
            // Note: handle_submit_error resets input fields even when called from recent paths entry,
            // but this is harmless since the fields aren't visible in that context.
            if let Screen::Start(start_state) = &mut app.screen {
                start_state.handle_submit_error(IoErrorKind::FileRead);
            }
            return;
        }
    }

    // Always called from Start screen
    if let Screen::Start(start_state) = &mut app.screen {
        if let Ok(recent_paths) = &mut start_state.recent_paths {
            if !recent_paths.contains_path(path) {
                recent_paths.add(path.to_path_buf());
                recent_paths.save_with_fs(fs);
            }
        }
    }

    app.screen = Screen::Map(map_state);

    if let Screen::Map(map_state) = &mut app.screen {
        handle_on_load_backup_with_fs(map_state, fs, Local::now());
    }
}
