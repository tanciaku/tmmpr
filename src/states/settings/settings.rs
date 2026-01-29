use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use crate::states::{
    map::Side,
    settings::{BackupsInterval, RuntimeBackupsInterval, cycle_side}
};

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct Settings {
    /// Interval (in seconds) at which to auto-save changes
    pub save_interval: Option<usize>,
    /// How often to create backups when loading a map file
    pub backups_interval: Option<BackupsInterval>,
    pub backups_path: Option<String>, 
    /// Tracks last backup timestamp per map file path
    pub backup_dates: HashMap<String, DateTime<Local>>,
    /// How often to create backups during an active editing session
    pub runtime_backups_interval: Option<RuntimeBackupsInterval>,
    pub default_start_side: Side,
    pub default_end_side: Side,
    pub edit_modal: bool,
}

impl Settings {
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

    /// Cycles through available save intervals: 10s -> 20s -> 30s -> 60s -> off
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

    /// Cycles through backup intervals for map loading backups
    pub fn cycle_backup_interval(&mut self) {
        self.backups_interval = match self.backups_interval {
            Some(BackupsInterval::Daily) => Some(BackupsInterval::Every3Days),
            Some(BackupsInterval::Every3Days) => Some(BackupsInterval::Weekly),
            Some(BackupsInterval::Weekly) => Some(BackupsInterval::Every2Weeks),
            Some(BackupsInterval::Every2Weeks) => Some(BackupsInterval::Daily),
            None => unreachable!(), // cannot cycle if backups are disabled
        };
    }

    /// Cycles through runtime backup intervals during active sessions
    pub fn cycle_runtime_backup_interval(&mut self) {
        self.runtime_backups_interval = match self.runtime_backups_interval {
            Some(RuntimeBackupsInterval::Hourly) => Some(RuntimeBackupsInterval::Every2Hours),
            Some(RuntimeBackupsInterval::Every2Hours) => Some(RuntimeBackupsInterval::Every4Hours),
            Some(RuntimeBackupsInterval::Every4Hours) => Some(RuntimeBackupsInterval::Every6Hours),
            Some(RuntimeBackupsInterval::Every6Hours) => Some(RuntimeBackupsInterval::Every12Hours), 
            Some(RuntimeBackupsInterval::Every12Hours) => Some(RuntimeBackupsInterval::Hourly), 
            None => unreachable!(), // cannot cycle if runtime backups are disabled
        }
    }

    /// Cycles default connection side. If `start_side` is true, cycles start side; otherwise cycles end side.
    pub fn cycle_default_sides(&mut self, start_side: bool) {
        if start_side {
            self.default_start_side = cycle_side(self.default_start_side);
        } else {
            self.default_end_side = cycle_side(self.default_end_side);
        }
    }
}