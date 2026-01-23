use std::{io::stdout, path::PathBuf};
use crossterm::{cursor::SetCursorStyle, execute};
use ratatui::style::Color;

use crate::{
    states::{
        map::{ConnectionsState, ModalEditMode, Mode, NotesState, PersistenceState, UIState, ViewportState, VisualModeState},
        settings::{Settings, SettingsType, get_settings},
        start::ErrMsg
    },
    utils::{handle_runtime_backup, save_map_file}
};


#[derive(PartialEq, Debug)]
pub struct MapState {
    /// A flag indicating that the screen needs to be cleared and redrawn on the next frame.
    /// The current input mode of the application, similar to Vim modes.
    pub current_mode: Mode,
    pub viewport: ViewportState,
    pub notes_state: NotesState,
    pub visual_mode: VisualModeState,
    pub connections_state: ConnectionsState,
    pub persistence: PersistenceState,
    pub ui_state: UIState,
    /// Settings
    pub settings: Settings,
    /// Whether to notify the user that something went wrong with
    /// using the settings functionality
    pub settings_err_msg: Option<ErrMsg>,
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
            current_mode: Mode::Normal,
            viewport: ViewportState::new(),
            notes_state: NotesState::new(),
            visual_mode: VisualModeState::new(),
            connections_state: ConnectionsState::new(),
            persistence: PersistenceState::new(file_write_path),
            ui_state: UIState::new(),
            settings: settings,
            settings_err_msg: settings_err_msg,
        }
    }

    /// Sets the flag to clear and redraw the screen on the next frame.
    pub fn clear_and_redraw(&mut self) {
        self.ui_state.request_redraw();
    }

    /// Adds a new, empty note to the canvas.
    ///
    /// The note is created at the center of the current viewport. It is immediately
    /// selected, and the application switches to `Mode::Edit` to allow for
    /// immediate text entry.
    pub fn add_note(&mut self) {
        self.persistence.mark_dirty();

        let (note_x, note_y) = self.viewport.center();

        let note_id = self.notes_state.create_note(note_x, note_y, String::from(""), true, Color::White);
        self.notes_state.selected_note = Some(note_id);
        
        // Switch to edit mode
        self.switch_to_edit_mode();
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
        let (screen_center_x, screen_center_y) = self.viewport.center();
        
        // Use the new helper methods
        if let Some(id) = self.notes_state.find_closest_note(screen_center_x, screen_center_y) {
            // When/If found the closest note - select it and switch to Visual Mode
            self.notes_state.select_note_by_id(id);
            self.current_mode = Mode::Visual;
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
                if self.persistence.should_save(interval) { 
                    // Copy the write path for use here
                    let map_file_path = self.persistence.file_write_path.clone();
                    // Attempt to save changes to the map file
                    // Notifications handled by save_map_file itself
                    save_map_file(self, &map_file_path, false, false);
                    self.persistence.reset_save_timer();
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
                if self.persistence.should_backup(interval) {
                    // NOTE: if there is a runtime backups interval - there is a backups path.
                    // Notifications handled by save_map_file, which is within handle_runtime_backup.
                    handle_runtime_backup(self);
                    self.persistence.reset_backup_timer();
                }
            }
        }
    }
}