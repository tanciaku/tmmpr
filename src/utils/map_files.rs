use std::{path::Path, collections::HashMap};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use ratatui::style::Color;

use crate::{
    app::{App, Screen},
    states::{
        MapState, map::{BackupResult, Connection, Note, Notification, ViewPos}, start::ErrMsg
    },
    utils::{handle_on_load_backup, read_json_data, write_json_data, get_color_name_in_string, get_color_from_string},
};

/// A type for reading and writing relevant data from MapState
#[derive(Serialize, Deserialize)]
pub struct MapData {
    pub view_pos: ViewPos,
    pub next_note_id: usize,
    pub notes: HashMap<usize, Note>,
    pub render_order: Vec<usize>,
    pub connections: Vec<Connection>,
    pub connection_index: HashMap<usize, Vec<Connection>>,
}

pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let color_string = get_color_name_in_string(*color);
    serializer.serialize_str(&color_string)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    
    Ok(get_color_from_string(&s))
}


/// Creates a new map file.
/// 
/// Handles file write error by displaying appropriate error message to the user.
pub fn create_map_file(app: &mut App, path: &Path) {
    // Create a new Map State for creating a new map file
    let map_state = MapState::new(path.to_path_buf()); // Only clone when storing
    // Take the default values from that to write to the file
    let map_data = MapData {
        view_pos: map_state.viewport.view_pos,
        next_note_id: map_state.notes_state.next_note_id,
        notes: map_state.notes_state.notes,
        render_order: map_state.notes_state.render_order,
        connections: map_state.connections_state.connections,
        connection_index: map_state.connections_state.connection_index,
    };

    // Attempt to write that data to the file
    if let Err(_) = write_json_data(path, &map_data) {
        // Display an error
        if let Screen::Start(start_state) = &mut app.screen {
            start_state.handle_submit_error(ErrMsg::FileWrite);
        }
        return // Stop here without switching screens.
    }

    // Adding the path to the newly created map file to recent_paths
    if let Screen::Start(start_state) = &mut app.screen { // guaranteed
        // If recent_paths functionality available
        if let Ok(recent_paths) = &mut start_state.recent_paths {
            // Add the file path to recent_paths (if not already there)
            if !recent_paths.contains_path(path) {
                recent_paths.add(path.to_path_buf());

                // Save the recent_paths file
                recent_paths.save();
            }
        }
    }

    // If successful in the previous step -
    // switch to the Map Screen, with the newly created Map State
    app.screen = Screen::Map(MapState::new(path.to_path_buf())); // Only clone when storing
}

/// Saves map data to a file.
/// 
/// Handles file write error by displaying appropriate error message to the user. 
pub fn save_map_file(map_state: &mut MapState, path: &Path, show_save_notification: bool, making_backup: bool) {
    // Get the relevant values from the current Map State
    let map_data = MapData {
        view_pos: map_state.viewport.view_pos.clone(),
        next_note_id: map_state.notes_state.next_note_id,
        notes: map_state.notes_state.notes.clone(),
        render_order: map_state.notes_state.render_order.clone(),
        connections: map_state.connections_state.connections.clone(),
        connection_index: map_state.connections_state.connection_index.clone(),
    };

    // Attempt to write map data to the file
    match write_json_data(path, &map_data) {
        Ok(_) => {
            // Show successfully saved the map file message and redraw

            // Can exit the app - now that have successfully saved the map file.
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
            // Show failed saving the map file message and redraw

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

/// Loads map data from a file and transitions the application to the Map screen.
/// 
/// This function is exclusively called from the Start screen when the user wants to 
/// open an existing map file. It reads the map data from the specified file path,
/// populates a new MapState with the loaded data, and transitions the app to the Map screen.
/// 
/// If the file cannot be read or contains invalid data, the function will show
/// an error message via the Start screen's error handling and prevent screen transition.
pub fn load_map_file(app: &mut App, path: &Path) {
    // Initialize a default MapState that will be populated with loaded data.
    // This ensures we have valid defaults for any fields not present in the file.
    let mut map_state = MapState::new(path.to_path_buf()); // Only clone when storing

    match read_json_data::<MapData>(path) {
        Ok(map_data) => {
            // Successfully loaded data from file - now populate the MapState
            // with the saved values, overriding the defaults
            map_state.viewport.view_pos = map_data.view_pos;
            map_state.notes_state.next_note_id = map_data.next_note_id;
            map_state.notes_state.notes = map_data.notes;
            map_state.notes_state.render_order = map_data.render_order;
            map_state.connections_state.connections = map_data.connections;
            map_state.connections_state.connection_index = map_data.connection_index;
        }
        Err(_) => {
            // Failed to read or parse the map file - show error to user
            // and stay on the Start screen/Input menu to allow them to try again
            //
            // If an error occurs when using the "recent paths" functionality (in the start screen)
            // handle_submit_error will also unnecessarily reset the input fields, even though
            // the user is not in the input menu (affects nothing).
            if let Screen::Start(start_state) = &mut app.screen {
                start_state.handle_submit_error(ErrMsg::FileRead);
            }
            return; // Early return prevents screen transition
        }
    }

    // Adding the path to the map file to recent_paths
    if let Screen::Start(start_state) = &mut app.screen { // guaranteed
        // If recent_paths functionality available
        if let Ok(recent_paths) = &mut start_state.recent_paths {
            // Add the file path to recent_paths (if not already there)
            if !recent_paths.contains_path(path) {
                recent_paths.add(path.to_path_buf());

                // Save the recent_paths file
                recent_paths.save();
            }
        }
    }

    // File loaded successfully - transition to the Map screen with the 
    // populated MapState containing the loaded map data
    app.screen = Screen::Map(map_state);

    // If backups enabled - determine whether to create a load backup file.
    if let Screen::Map(map_state) = &mut app.screen {
        handle_on_load_backup(map_state);
    }
}
