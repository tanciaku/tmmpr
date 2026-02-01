use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};

use crate::{
    states::settings::Settings,
    utils::IoErrorKind,
};

/// Tracks whether settings were loaded from a custom file or fell back to defaults.
/// Carries an optional error message with defaults to notify the user of load failures.
#[derive(PartialEq, Debug)]
pub enum SettingsType {
    Default(Settings, Option<IoErrorKind>),
    Custom(Settings),
}

impl SettingsType {
    pub fn settings(&self) -> &Settings {
        match self {
            SettingsType::Default(settings, _) => settings,
            SettingsType::Custom(settings) => settings,
        }
    }

    pub fn settings_mut(&mut self) -> &mut Settings {
        match self {
            SettingsType::Default(settings, _) => settings,
            SettingsType::Custom(settings) => settings,
        }
    }
}

/// Destination when user confirms discarding unsaved settings changes.
#[derive(PartialEq, Debug)]
pub enum DiscardExitTo {
    StartScreen,
    MapScreen,
}

#[derive(PartialEq, Debug)]
pub enum SettingsNotification {
    SaveSuccess,
    SaveFail,
}

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
    /// Returns highlighted style if this toggle is currently selected.
    pub fn get_style(&self, selected_button: &SelectedToggle) -> Style {
        if self == selected_button {
            Style::new().bg(Color::White).fg(Color::Black)
        } else {
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