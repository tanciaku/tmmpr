use std::{fs, path::PathBuf};

use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use crate::{states::start::ErrMsg, utils::{read_json_data, write_json_data}};
use tempfile::NamedTempFile;

#[derive(PartialEq)]
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
}

impl SettingsState {
    /// Go down a toggle in the settings menu.
    pub fn toggle_go_down(&mut self) {
        self.selected_toggle = match self.selected_toggle {
            SelectedToggle::Toggle1 => SelectedToggle::Toggle2,
            SelectedToggle::Toggle2 => SelectedToggle::Toggle1,
        }
    }

    /// Go up a toggle in the settings menu.
    pub fn toggle_go_up(&mut self) {
        self.selected_toggle = match self.selected_toggle {
            SelectedToggle::Toggle1 => SelectedToggle::Toggle2,
            SelectedToggle::Toggle2 => SelectedToggle::Toggle1,
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

        // Reset error if already isn't empty
        self.input_prompt_err = None;

        // Exit the input prompt
        self.input_prompt = false;
    }
}

#[derive(PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub save_interval: Option<usize>,
    pub backups_interval: Option<BackupsInterval>,
    pub last_backup_date: Option<DateTime<Local>>,
    pub backups_path: Option<String>,
}

impl Settings {
    // Get default setttings
    pub fn new() -> Settings {
        Settings {
            save_interval: Some(20),
            backups_interval: None,
            last_backup_date: None,
            backups_path: None,
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
    // Get the user's home directory path
    let home_path = match home::home_dir() {
        Some(path) => path,
        None => return,
    };

    // Make the full path to the file (/home/user/.config/tmmpr/settings.json)
    let settings_file_path = home_path.join(".config/tmmpr/settings").with_extension("json");

    // Write the data
    match &settings_state.settings {
        SettingsType::Custom(settings) => {
            match write_json_data(&settings_file_path, &settings) {
                Ok(_) => {
                    // Saved changes to a file - so can now exit the settings menu.
                    settings_state.can_exit = true;

                    settings_state.notification = Some(SettingsNotification::SaveSuccess);
                }
                Err(_) => settings_state.notification = Some(SettingsNotification::SaveFail),
            }
        }
        // Default settings are automatically written upon creation already.
        // Default settings become "custom" when you modify them or the settings file already existed.
        // Nothing happens if user tries rewrite the default settings to the file. (first boot)
        _ => {}
    }
}

/// Type to distinguish between whether successfully loaded the
/// settings file and to know to notify the user if didn't.
#[derive(PartialEq)]
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
#[derive(PartialEq)]
pub enum DiscardExitTo {
    StartScreen,
    MapScreen,
}

/// Which notification to show in the settings menu.
#[derive(PartialEq)]
pub enum SettingsNotification {
    SaveSuccess,
    SaveFail,
    BackupsSuccess,
}

/// Which toggle is selected in the settings menu.
#[derive(PartialEq)]
pub enum SelectedToggle {
    /// Save map interval
    Toggle1,
    Toggle2,
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

#[derive(PartialEq, Serialize, Deserialize)]
pub enum BackupsInterval {
    Daily,
    Every3Days,
    Weekly,
    Every2Weeks,
}

#[derive(PartialEq)]
pub enum BackupsErr {
    DirFind,
    DirCreate,
    FileWrite,
}