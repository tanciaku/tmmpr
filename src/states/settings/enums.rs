use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};
use crate::states::{settings::Settings, start::ErrMsg};

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
#[derive(PartialEq, Debug, Clone, Copy)]
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