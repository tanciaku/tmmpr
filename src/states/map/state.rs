use std::{io::stdout, path::PathBuf};
use crossterm::{cursor::SetCursorStyle, execute};
use ratatui::style::Color;

use crate::{
    states::{
        map::{
            ConnectionsState, ModalEditMode, Mode, NotesState, PersistenceState,
            UIState, ViewportState
        },
        settings::{Settings, SettingsType, get_settings_with_fs},
    },
    utils::{IoErrorKind, FileSystem, handle_runtime_backup, save_map_file}
};


/// Core state for the map view where users create and edit notes and connections.
///
/// This is the main working state of the application, handling note creation/editing,
/// viewport navigation, visual selection, and auto-save/backup functionality.
#[derive(PartialEq, Debug)]
pub struct MapState {
    pub current_mode: Mode,
    pub viewport: ViewportState,
    pub notes_state: NotesState,
    pub connections_state: ConnectionsState,
    pub persistence: PersistenceState,
    pub ui_state: UIState,
    pub settings: Settings,
    pub settings_err_msg: Option<IoErrorKind>,
}

impl MapState {
    pub fn new_with_fs(file_write_path: PathBuf, fs: &dyn FileSystem) -> MapState {
        let (settings, settings_err_msg) = match get_settings_with_fs(fs) {
            SettingsType::Default(settings, err_opt) => (settings, err_opt),
            SettingsType::Custom(settings) => (settings, None),
        };

        MapState {
            current_mode: Mode::Normal,
            viewport: ViewportState::new(),
            notes_state: NotesState::new(),
            connections_state: ConnectionsState::new(),
            persistence: PersistenceState::new(file_write_path),
            ui_state: UIState::new(),
            settings: settings,
            settings_err_msg: settings_err_msg,
        }
    }

    pub fn clear_and_redraw(&mut self) {
        self.ui_state.request_redraw();
    }

    /// Adds a new, empty note at the center of the viewport and enters edit mode.
    pub fn add_note(&mut self) {
        self.persistence.mark_dirty();

        let (note_x, note_y) = self.viewport.center();

        self.notes_state.add(note_x, note_y, String::from(""), true, Color::White);
        
        self.switch_to_edit_mode();
    }

    /// Switches to Edit mode, using modal editing if enabled in settings.
    ///
    /// Block cursor provides visual feedback that modal editing is active (vim-style).
    pub fn switch_to_edit_mode(&mut self) {
        self.current_mode = Mode::Edit(
            if self.settings.edit_modal {
                let _ = execute!(stdout(), SetCursorStyle::SteadyBlock);

                Some(ModalEditMode::Normal)
            } else {
                None
            }
        );
    }

    /// Selects the note closest to the viewport center and enters Visual mode.
    ///
    /// Uses Manhattan distance from viewport center to note top-left corner.
    pub fn select_note(&mut self) {
        let (screen_center_x, screen_center_y) = self.viewport.center();
        
        if let Some(id) = self.notes_state.find_closest_note(screen_center_x, screen_center_y) {
            self.notes_state.select(id);
            self.current_mode = Mode::Visual;
        }
    }

    /// Handles periodic auto-save operations based on configured intervals.
    pub fn auto_save_if_needed(&mut self) {
        if let Some(interval) = self.settings.save_interval {
            if self.persistence.should_save(interval) { 
                let map_file_path = self.persistence.file_write_path.clone();
                let _ = save_map_file(self, &map_file_path); // No notification for auto-save
                self.persistence.reset_save_timer();
            }
        }
    }

    /// Handles periodic backup operations based on configured intervals.
    pub fn auto_backup_if_needed(&mut self) {
        if let Some(interval) = &self.settings.runtime_backups_interval {
            if self.persistence.should_backup(interval) {
                // Runtime backups interval implies backups path exists in settings
                handle_runtime_backup(self);
                self.persistence.reset_backup_timer();
            }
        }
    }
}