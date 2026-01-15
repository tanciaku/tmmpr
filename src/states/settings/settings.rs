use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use crate::states::{
    map::Side,
    settings::{BackupsInterval, RuntimeBackupsInterval, cycle_side}
};

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
            edit_modal: false,
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