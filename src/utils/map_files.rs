use std::{path::Path, collections::HashMap};
use chrono::Local;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use ratatui::style::Color;

use crate::{
    app::{App, Screen},
    states::{
        MapState, map::{BackupResult, Connection, Note, Notification, ViewPos}, start::ErrMsg
    },
    utils::{handle_on_load_backup_with_fs, read_json_data, write_json_data, get_color_name_in_string, get_color_from_string, filesystem::{FileSystem, RealFileSystem}},
};

/// Serializable representation of map state for JSON persistence.
/// 
/// Separated from `MapState` to include only the data that needs to be persisted,
/// excluding runtime-only fields.
#[derive(Serialize, Deserialize)]
pub struct MapData {
    pub view_pos: ViewPos,
    pub next_note_id: usize,
    pub notes: HashMap<usize, Note>,
    pub render_order: Vec<usize>,
    pub connections: Vec<Connection>,
    pub connection_index: HashMap<usize, Vec<Connection>>,
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
        next_note_id: map_state.notes_state.next_note_id,
        notes: map_state.notes_state.notes,
        render_order: map_state.notes_state.render_order,
        connections: map_state.connections_state.connections,
        connection_index: map_state.connections_state.connection_index,
    };

    if let Err(_) = write_json_data(path, &map_data) {
        if let Screen::Start(start_state) = &mut app.screen {
            start_state.handle_submit_error(ErrMsg::FileWrite);
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

/// Saves map data to a file, optionally showing notifications.
/// 
/// Updates persistence state to allow exit after successful save.
pub fn save_map_file(map_state: &mut MapState, path: &Path, show_save_notification: bool, making_backup: bool) {
    let map_data = MapData {
        view_pos: map_state.viewport.view_pos.clone(),
        next_note_id: map_state.notes_state.next_note_id,
        notes: map_state.notes_state.notes.clone(),
        render_order: map_state.notes_state.render_order.clone(),
        connections: map_state.connections_state.connections.clone(),
        connection_index: map_state.connections_state.connection_index.clone(),
    };

    match write_json_data(path, &map_data) {
        Ok(_) => {
            // Mark as clean to allow exit without discard changes prompt
            map_state.persistence.mark_clean();

            if making_backup {
                map_state.persistence.backup_res = Some(BackupResult::BackupSuccess);
            }

            if show_save_notification {
                if making_backup {
                    map_state.ui_state.set_notification(Notification::BackupSuccess);
                } else {
                    map_state.ui_state.set_notification(Notification::SaveSuccess);
                }

                map_state.clear_and_redraw();
            }
        }
        Err(_) => {
            if making_backup {
                map_state.persistence.backup_res = Some(BackupResult::BackupFail);
            }

            if show_save_notification {
                if making_backup {
                    map_state.ui_state.set_notification(Notification::BackupFail);
                } else {
                    map_state.ui_state.set_notification(Notification::SaveFail);
                }

                map_state.clear_and_redraw();
            }
        }
    }            
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
            map_state.notes_state.next_note_id = map_data.next_note_id;
            map_state.notes_state.notes = map_data.notes;
            map_state.notes_state.render_order = map_data.render_order;
            map_state.connections_state.connections = map_data.connections;
            map_state.connections_state.connection_index = map_data.connection_index;
        }
        Err(_) => {
            // Note: handle_submit_error resets input fields even when called from recent paths entry,
            // but this is harmless since the fields aren't visible in that context.
            if let Screen::Start(start_state) = &mut app.screen {
                start_state.handle_submit_error(ErrMsg::FileRead);
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
