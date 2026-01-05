use std::{collections::HashMap, fs, path::PathBuf};

use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use crate::{states::{map::Side, start::ErrMsg}, utils::{read_json_data, write_json_data, save_settings_to_file}};
use tempfile::NamedTempFile;

#[derive(PartialEq, Debug)]
pub struct SettingsState {
    pub needs_clear_and_redraw: bool,
    pub settings_context_page: bool,
    /// To easily go back to the map file that was opened.
    pub map_file_path: PathBuf,
    pub settings: SettingsType,
    /// Determines whether the user has saved the changes
    /// to the settings file, before switching screens or exiting.
    pub can_exit: bool,
    /// Whether to render a menu for confirming to discard 
    /// unsaved changes
    pub confirm_discard_menu: Option<DiscardExitTo>,
    pub notification: Option<SettingsNotification>,
    /// Which toggle is selected in the settings menu
    pub selected_toggle: SelectedToggle,
    /// Whether to display context page (context for toggles)
    pub context_page: bool,
    /// Whether to display an input prompt for entering a path for backups.
    pub input_prompt: bool,
    /// Whether to notify user that using the entered path for backups failed.
    pub input_prompt_err: Option<BackupsErr>,
}

impl SettingsState {
    pub fn new(map_file_path: PathBuf) -> SettingsState {
        SettingsState {
            needs_clear_and_redraw: true,
            settings_context_page: false,
            map_file_path: map_file_path,
            settings: get_settings(),
            can_exit: true,
            confirm_discard_menu: None,
            notification: None,
            selected_toggle: SelectedToggle::Toggle1,
            context_page: false,
            input_prompt: false,
            input_prompt_err: None,
        }
    }

    /// Go down a toggle in the settings menu.
    pub fn toggle_go_down(&mut self) {
        self.selected_toggle = match self.selected_toggle {
            SelectedToggle::Toggle1 => SelectedToggle::Toggle2,
            SelectedToggle::Toggle2 => {
                // If backups enabled - toggle3 (runtime backups) is available - so can go to it.
                if let Some(_) = self.settings.settings().runtime_backups_interval {
                    SelectedToggle::Toggle3
                // If not - that toggle isn't available.
                } else {
                    SelectedToggle::Toggle4
                }
            }
            SelectedToggle::Toggle3 => SelectedToggle::Toggle4,
            SelectedToggle::Toggle4 => SelectedToggle::Toggle5,
            SelectedToggle::Toggle5 => SelectedToggle::Toggle6,
            SelectedToggle::Toggle6 => SelectedToggle::Toggle1,
        }
    }

    /// Go up a toggle in the settings menu.
    pub fn toggle_go_up(&mut self) {
        self.selected_toggle = match self.selected_toggle {
            SelectedToggle::Toggle1 => SelectedToggle::Toggle6,
            SelectedToggle::Toggle2 => SelectedToggle::Toggle1,
            SelectedToggle::Toggle3 => SelectedToggle::Toggle2,
            SelectedToggle::Toggle4 => {
                // If backups enabled - toggle3 (runtime backups) is available - so can go to it.
                if let Some(_) = self.settings.settings().runtime_backups_interval {
                    SelectedToggle::Toggle3
                // If not - that toggle isn't available.
                } else {
                    SelectedToggle::Toggle2
                }
            }
            SelectedToggle::Toggle5 => SelectedToggle::Toggle4,
            SelectedToggle::Toggle6 => SelectedToggle::Toggle5,
        }
    }

    pub fn submit_path(&mut self) {
        // User entered path for backups
        // .unwrap() used here - because while in the input prompt - backups_path cannot be None 
        let input_dir_path = self.settings.settings().backups_path.as_ref().unwrap();

        // If the path start with '/' - keep the path as it is (it's an absolute path e.g. /mnt/map_backups/)
        let backups_dir = if input_dir_path.starts_with('/') {
            PathBuf::from(input_dir_path)
        } else { // If it doesn't - append that path to the home user directory path (e.g. /home/user/map_backups/)
            // Get the user's home directory path
            let home_path = match home::home_dir() {
                Some(path) => path,
                None => {
                    self.input_prompt_err = Some(BackupsErr::DirFind);
                    return;
                }
            };
            // Construct the full path
            let backups_dir = home_path.join(input_dir_path);

            // Set that path in the Settings field
            // "Setting it" doesn't mean submitting succeded - it's for in case it does - use
            // the full path from home directory instead of the user entered directory path.
            self.settings.settings_mut().backups_path = Some(backups_dir.to_string_lossy().to_string());

            // Return it into a variable for use in this function
            backups_dir
        };
    
        // Create the directory
        if let Err(_) = fs::create_dir_all(&backups_dir) {
            self.input_prompt_err = Some(BackupsErr::DirCreate);
            return;
        };

        // Attempt to write data to that directory
        if let Err(_) = NamedTempFile::new_in(&backups_dir) {
            self.input_prompt_err = Some(BackupsErr::FileWrite);
            return;
        }

        // Set the default backup interval
        self.settings.settings_mut().backups_interval = Some(BackupsInterval::Daily);

        // Set the default runtime backup interval
        self.settings.settings_mut().runtime_backups_interval = Some(RuntimeBackupsInterval::Every2Hours);

        // Reset error if already isn't empty
        self.input_prompt_err = None;

        // Exit the input prompt
        self.input_prompt = false;
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct Settings {
    /// Interval at which to auto-save changes to a map file
    pub save_interval: Option<usize>,
    /// Interval at which to backup map file on loading it
    pub backups_interval: Option<BackupsInterval>,
    /// Path to a directory in which to save backups
    pub backups_path: Option<String>, 
    /// Last backup date for each map file opened
    pub backup_dates: HashMap<String, DateTime<Local>>,
    /// Interval at which to backup map file while the application
    /// is running.
    pub runtime_backups_interval: Option<RuntimeBackupsInterval>,
    /// Default start side for creating connections
    pub default_start_side: Side,
    /// Default end side for creating connections
    pub default_end_side: Side,
    /// Whether modal editing for Edit Mode is enabled
    pub edit_modal: bool,
}

impl Settings {
    // Get default setttings
    pub fn new() -> Settings {
        Settings {
            save_interval: Some(20),
            backups_interval: None,
            backups_path: None,
            backup_dates: HashMap::new(),
            runtime_backups_interval: None,
            default_start_side: Side::Right,
            default_end_side: Side::Right,
            edit_modal: true,
        }
    }

    /// Cycles through the available saving intervals, for changes made to the map
    pub fn cycle_save_intervals(&mut self) {
        self.save_interval = match self.save_interval {
            None => Some(10),
            Some(10) => Some(20),
            Some(20) => Some(30),
            Some(30) => Some(60),
            Some(60) => None,
            _ => unreachable!(),
        }; 
    }

    pub fn cycle_backup_interval(&mut self) {
        self.backups_interval = match self.backups_interval {
            Some(BackupsInterval::Daily) => Some(BackupsInterval::Every3Days),
            Some(BackupsInterval::Every3Days) => Some(BackupsInterval::Weekly),
            Some(BackupsInterval::Weekly) => Some(BackupsInterval::Every2Weeks),
            Some(BackupsInterval::Every2Weeks) => Some(BackupsInterval::Daily),
            None => unreachable!(), // cannot cycle backup interval if backups are not enabled
        };
    }

    pub fn cycle_runtime_backup_interval(&mut self) {
        self.runtime_backups_interval = match self.runtime_backups_interval {
            Some(RuntimeBackupsInterval::Hourly) => Some(RuntimeBackupsInterval::Every2Hours),
            Some(RuntimeBackupsInterval::Every2Hours) => Some(RuntimeBackupsInterval::Every4Hours),
            Some(RuntimeBackupsInterval::Every4Hours) => Some(RuntimeBackupsInterval::Every6Hours),
            Some(RuntimeBackupsInterval::Every6Hours) => Some(RuntimeBackupsInterval::Every12Hours), 
            Some(RuntimeBackupsInterval::Every12Hours) => Some(RuntimeBackupsInterval::Hourly), 
            None => unreachable!(), // cannot cycle runtime backup interval if backups are not enabled
        }
    }

    pub fn cycle_default_sides(&mut self, start_side: bool) {
        if start_side {
            self.default_start_side = cycle_side(self.default_start_side);
        } else {
            self.default_end_side = cycle_side(self.default_end_side);
        }
    }
}

pub fn get_settings() -> SettingsType {
    // Get the user's home directory path
    let home_path = match home::home_dir() {
        Some(path) => path,
        None => return SettingsType::Default(Settings::new(), Some(ErrMsg::DirFind)),
    };

    //// Make the path to the settings directory (e.g. /home/user/.config/tmmpr/)
    let config_dir_path = home_path.join(".config/tmmpr/");

    // Create the directory if it doesn't exist
    if let Err(_) = fs::create_dir_all(&config_dir_path) {
        return SettingsType::Default(Settings::new(), Some(ErrMsg::DirCreate))
    };

    // Make the full path to the file (e.g. /home/user/.config/tmmpr/settings.json)
    let settings_file_path = config_dir_path.join("settings").with_extension("json");

    // Load the file if it exits:
    if settings_file_path.exists() {
        match read_json_data(&settings_file_path) {
            Ok(settings) => SettingsType::Custom(settings),
            Err(_) => SettingsType::Default(Settings::new(), Some(ErrMsg::FileRead)),
        }
    } else { // Otherwise create it
        let new_settings = Settings::new();
        match write_json_data(&settings_file_path, &new_settings) {
            Ok(_) => SettingsType::Default(Settings::new(), None),
            Err(_) => SettingsType::Default(Settings::new(), Some(ErrMsg::FileWrite)),
        }
    }
}

/// This is (can) only be called if user can use the settings 
/// functionality - directories (already) exist in that case.
pub fn save_settings(settings_state: &mut SettingsState) {
    // Reference to Settings in the SettingsType
    let settings = &settings_state.settings.settings();

    // Save the settings data (use the shared utility function)
    match save_settings_to_file(settings) {
        Ok(_) => {
            // Saved changes to a file - so can now exit the settings menu.
            settings_state.can_exit = true;
            settings_state.notification = Some(SettingsNotification::SaveSuccess);
        }
        Err(_) => settings_state.notification = Some(SettingsNotification::SaveFail),
    }
}

/// Type to distinguish between whether successfully loaded the
/// settings file and to know to notify the user if didn't.
#[derive(PartialEq, Debug)]
pub enum SettingsType {
    Default(Settings, Option<ErrMsg>),
    Custom(Settings),
}

impl SettingsType {
    /// Get a reference to the Settings regardless of variant
    pub fn settings(&self) -> &Settings {
        match self {
            SettingsType::Default(settings, _) => settings,
            SettingsType::Custom(settings) => settings,
        }
    }

    /// Get a mutable reference to the Settings regardless of variant
    pub fn settings_mut(&mut self) -> &mut Settings {
        match self {
            SettingsType::Default(settings, _) => settings,
            SettingsType::Custom(settings) => settings,
        }
    }
}

/// If exiting from the confirm discard menu - where to exit to.
#[derive(PartialEq, Debug)]
pub enum DiscardExitTo {
    StartScreen,
    MapScreen,
}

/// Which notification to show in the settings menu.
#[derive(PartialEq, Debug)]
pub enum SettingsNotification {
    SaveSuccess,
    SaveFail,
}

/// Which toggle is selected in the settings menu.
#[derive(PartialEq, Debug)]
pub enum SelectedToggle {
    /// Save map interval
    Toggle1,
    /// Load backups
    Toggle2,
    /// Runtime backups
    Toggle3,
    /// Default start side for making connections
    Toggle4,
    /// Default end side for making connections
    Toggle5,
    /// Modal Editing for Edit Mode
    Toggle6,
}

impl SelectedToggle {
    /// Determines the style based on if the toggle is selected
    pub fn get_style(&self, selected_button: &SelectedToggle) -> Style {
        if self == selected_button {
            // Selected button
            Style::new().bg(Color::White).fg(Color::Black)
        } else {
            // Default
            Style::new()
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub enum BackupsInterval {
    Daily,
    Every3Days,
    Weekly,
    Every2Weeks,
}

#[derive(PartialEq, Debug)]
pub enum BackupsErr {
    DirFind,
    DirCreate,
    FileWrite,
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub enum RuntimeBackupsInterval {
    Hourly,
    Every2Hours,
    Every4Hours,
    Every6Hours,
    Every12Hours,
}

fn cycle_side(side: Side) -> Side {
    match side {
        Side::Right => Side::Bottom,
        Side::Bottom => Side::Left,
        Side::Left => Side::Top,
        Side::Top => Side::Right,
    }
}

pub fn side_to_string(side: Side) -> String {
    match side {
        Side::Right => String::from("Right"),
        Side::Bottom => String::from("Bottom"),
        Side::Left => String::from("Left"),
        Side::Top => String::from("Top"),
    }
}