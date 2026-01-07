use std::{collections::HashMap, io::stdout, path::PathBuf, time::{Duration, Instant}};
use crossterm::{cursor::SetCursorStyle, execute};
use ratatui::style::Color;

use crate::{
    utils::{save_map_file, handle_runtime_backup, get_duration_rt}, 
    states::{settings::{Settings, SettingsType, get_settings}, start::ErrMsg},
};

use super::{
    enums::*,
    geometry::ViewPos,
    note::Note,
    connection::Connection,
};

#[derive(PartialEq, Debug)]
pub struct MapState {
    pub needs_clear_and_redraw: bool,
    /// A flag indicating that the screen needs to be cleared and redrawn on the next frame.
    /// The current input mode of the application, similar to Vim modes.
    pub current_mode: Mode,
    /// The position of the viewport (camera) on the infinite canvas.
    /// Position can only be positive.
    pub view_pos: ViewPos,
    /// Store screen dimensions in the MapState to be able to access them in modules besides ui.rs
    /// The current width of the terminal screen in cells. Updated on every frame.
    pub screen_width: usize,
    /// Store screen dimensions in the MapState to be able to access them in modules besides ui.rs
    /// The current height of the terminal screen in cells. Updated on every frame.
    pub screen_height: usize,
    /// A counter to ensure each new note gets a unique ID.
    pub next_note_id: usize,
    /// A collection of all notes in the mind map, keyed by their unique ID.
    pub notes: HashMap<usize, Note>,
    /// Order in which to render the notes ("z index")
    /// Ordered back to front.
    pub render_order: Vec<usize>,
    /// The unique ID of the currently selected note.
    pub selected_note: Option<usize>,
    pub cursor_pos: usize,
    pub visual_move: bool,
    pub visual_connection: bool,
    pub connections: Vec<Connection>,
    /// Separate type for connections, to be able to properly render
    /// connecting characters: ┴ ┬ ┤ ├
    pub connection_index: HashMap<usize, Vec<Connection>>,
    pub focused_connection: Option<Connection>,
    pub visual_editing_a_connection: bool,
    /// Index of the connection being edited, when it was taken out
    /// out the connections vector.
    pub editing_connection_index: Option<usize>,
    /// The path provided by the user to write the map data to
    /// e.g /home/user/maps/map_0.json
    pub file_write_path: PathBuf,
    pub show_notification: Option<Notification>,
    /// Determines whether the user has saved the changes
    /// to the map file, before switching screens or exiting.
    pub can_exit: bool,
    /// Whether to render a menu for confirming to discard 
    /// unsaved changes
    pub confirm_discard_menu: Option<DiscardMenuType>,
    /// Timestamp for automatically saving changes to the map file
    pub last_save: Instant,
    /// Whether to show the help screen, and take all input for it
    /// usize - represents the page of the help screen
    pub help_screen: Option<usize>,
    /// Settings
    pub settings: Settings,
    /// Whether to notify the user that something went wrong with
    /// using the settings functionality
    pub settings_err_msg: Option<ErrMsg>,
    /// The result of attempting to make a backup file
    pub backup_res: Option<BackupResult>,
    /// Timestamp for automatically making a runtime backup file
    pub rt_backup_ts: Instant,
}

impl MapState {
    pub fn new(file_write_path: PathBuf) -> MapState {

        // Set the settings, scenarios:
        // 1. Using default because settings file doesn't exist (first boot)
        // 2. Using default because there was an error (notify the user about it)
        // 3. Using custom - settings file exists
        let (settings, settings_err_msg) = match get_settings() {
            SettingsType::Default(settings, err_opt) => (settings, err_opt),
            SettingsType::Custom(settings) => (settings, None),
        };

        // Whether to notify the user that something went wrong with
        // using the settings functionality
        let settings_err_msg = match settings_err_msg {
            Some(err_msg) => Some(err_msg),
            None => None,
        };

        MapState {
            needs_clear_and_redraw: true,
            current_mode: Mode::Normal,
            view_pos: ViewPos::new(),
            screen_width: 0,
            screen_height: 0,
            next_note_id: 0,
            notes: HashMap::new(),
            render_order: vec![],
            selected_note: None,
            cursor_pos: 0,
            visual_move: false,
            visual_connection: false,
            connections: vec![],
            connection_index: HashMap::new(),
            focused_connection: None,
            visual_editing_a_connection: false,
            editing_connection_index: None,
            file_write_path,
            show_notification: None,
            can_exit: true,
            confirm_discard_menu: None,
            last_save: Instant::now(),
            help_screen: None,
            settings: settings, // set the settings
            settings_err_msg: settings_err_msg,
            backup_res: None,
            rt_backup_ts: Instant::now(),
        }
    }

    /// Sets the flag to force a screen clear and redraw on the next frame.
    pub fn clear_and_redraw(&mut self) {
        self.needs_clear_and_redraw = true;
    }

    /// Adds a new, empty note to the canvas.
    ///
    /// The note is created at the center of the current viewport. It is immediately
    /// selected, and the application switches to `Mode::Edit` to allow for
    /// immediate text entry.
    pub fn add_note(&mut self) {
        self.can_exit = false;

        let note_x = self.view_pos.x + self.screen_width/2;
        let note_y = self.view_pos.y + self.screen_height/2;
        self.notes.insert(self.next_note_id, Note::new(note_x, note_y, String::from(""), true, Color::White));
        self.render_order.push(self.next_note_id);
        self.selected_note = Some(self.next_note_id);
        
        // Switch to edit mode
        self.switch_to_edit_mode();

        self.next_note_id += 1;
    }

    /// Switches the map state to Edit Mode for editing note content.
    ///
    /// Use modal editing for Edit Mode if it is enabled.
    pub fn switch_to_edit_mode(&mut self) {
        self.current_mode = Mode::Edit(
            if self.settings.edit_modal {
                // Set a block cursor
                let _ = execute!(stdout(), SetCursorStyle::SteadyBlock);

                Some(ModalEditMode::Normal)
            } else {
                None
            }
        );
    }

    /// Finds and selects the note closest to the center of the viewport.
    ///
    /// This method calculates the "Manhattan distance" from the center of the screen
    /// to the top-left corner of each note and sets the `selected_note` field to the
    /// ID of the note with the smallest distance.
    pub fn select_note(&mut self) {
        let screen_center_x = self.view_pos.x + self.screen_width / 2;
        let screen_center_y = self.view_pos.y + self.screen_height / 2;
        
        // Use an iterator to find the closest note ID.
        let closest_note_id_opt = self.notes.iter()
            .min_by_key(|(_, note)| {
                // Calculate Manhattan distance: |x1 - x2| + |y1 - y2|.
                let distance = (note.x as isize - screen_center_x as isize).abs()
                           + (note.y as isize - screen_center_y as isize).abs();
                distance as usize
            })
            .map(|(id, _)| *id); // We only care about the ID.

        // The result is an Option<usize>
        self.selected_note = closest_note_id_opt;

        if let Some(id) = self.selected_note {
            // Update the render order
            // (put the just selected note's id to the back of the render_order vector -
            //      so it renders it over every other note "below")
            if let Some(pos) = self.render_order.iter().position(|&x| x == id) {
                let item = self.render_order.remove(pos);  // Remove from current position
                self.render_order.push(item);              // Add to back
            }

            // Render that note in "selected" style
            if let Some(note) = self.notes.get_mut(&id) {
                note.selected = true;
            }
        }
    }

    pub fn stash_connection(&mut self) {
        // Take the connection out, leaving None in its place.
        if let Some(connection) = self.focused_connection.take() {
            // Now we own the connection. We can check its fields.
            if connection.to_id.is_some() {
                // If it has a target, we finalize it.
                self.connections.push(connection);

                // Get the Vec for the key, or create a new empty Vec if it's not there
                let indexed_connection_start = self.connection_index.entry(connection.from_id).or_default();
                indexed_connection_start.push(connection); // Now push your item into the Vec

                // Again for the end point.
                let indexed_connection_end = self.connection_index.entry(connection.to_id.unwrap()).or_default();
                indexed_connection_end.push(connection);
            }
            // If it didn't have a target, we just drop it here.
        }
    }

    pub fn take_out_connection(&mut self, index: usize) {
        let connection_removed = self.connections.remove(index);
        self.focused_connection = Some(connection_removed);

        // Edit values from corresponding keys associated with the connection
        // (removing the same connection from both indexes (from_id and to_id))
        if let Some(index_vec) = self.connection_index.get_mut(&connection_removed.from_id) {
            // Keep only the connections that are NOT the one we just removed.
            index_vec.retain(|c| c != &connection_removed);
        }

        if let Some(index_vec) = self.connection_index.get_mut(&connection_removed.to_id.unwrap()) {
            // Keep only the connections that are NOT the one we just removed.
            index_vec.retain(|c| c != &connection_removed);
        }
    }

    /// Make a runtime map backup file every set interval (if enabled)
    /// 
    /// Save changes to the map file every set interval (if enabled)
    pub fn on_tick_save_changes(&mut self) {
        // Saving changes to map file (every 20s by default)
        match self.settings.save_interval {
            // If it is disabled - don't periodically save changes.
            None => {}
            // Save changes every _ seconds
            Some(interval) => {
                // If there were changes made to the map and _ seconds have passed
                if self.can_exit == false && self.last_save.elapsed() > Duration::from_secs(interval as u64) { 
                    // Copy the write path for use here
                    let map_file_path = self.file_write_path.clone();
                    // Attempt to save changes to the map file
                    // Notifications handled by save_map_file itself
                    save_map_file(self, &map_file_path, false, false);
                    self.last_save = Instant::now(); // Restart the timer (take another timestamp) 
                }
            }
        }

        // Making a runtime backup file (every 2h by default)
        match &self.settings.runtime_backups_interval {
            // If it is disabled - don't make periodic runtime backups.
            None => {}
            // Make a backup every set interval
            Some(interval) => {
                // If a set duration has passed since opening the map file or last runtime backup:
                if self.rt_backup_ts.elapsed() > get_duration_rt(interval) {
                    // NOTE: if there is a runtime backups interval - there is a backups path.
                    // Notifications handled by save_map_file, which is within handle_runtime_backup.
                    handle_runtime_backup(self);
                    self.rt_backup_ts = Instant::now(); // Restart the timer (take another timestamp)
                }
            }
        }
    }
}