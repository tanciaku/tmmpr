use std::{fs, path::PathBuf};

use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};

use crate::{states::start::ErrMsg, utils::{read_json_data, write_json_data}};


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
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub save_interval: Option<usize>
}

impl Settings {
    // Get default setttings
    pub fn new() -> Settings {
        Settings {
            save_interval: Some(20),
        }
    }

    /// Cycles through the available saving intervals, for changes made to the map
    pub fn cycle_save_intervals(&self) -> Option<usize> {
        match self.save_interval {
            None => Some(10),
            Some(10) => Some(20),
            Some(20) => Some(30),
            Some(30) => Some(60),
            Some(60) => None,
            _ => unreachable!(),
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
}

/// Which toggle is selected in the settings menu.
#[derive(PartialEq)]
pub enum SelectedToggle {
    /// Save map interval
    Toggle1,
}

impl SelectedToggle {
    /// Go down a toggle in the settings menu.
    pub fn toggle_go_down(&self) -> SelectedToggle {
        match self {
            SelectedToggle::Toggle1 => SelectedToggle::Toggle1
        }
    }

    /// Go up a toggle in the settings menu.
    pub fn toggle_go_up(&self) -> SelectedToggle {
        match self {
            SelectedToggle::Toggle1 => SelectedToggle::Toggle1
        }
    }

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

