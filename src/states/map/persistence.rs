use std::{path::PathBuf, time::{Duration, Instant}};

use crate::{states::map::BackupResult, utils::get_duration_rt};


#[derive(PartialEq, Debug)]
pub struct PersistenceState {
    /// The path provided by the user to write the map data to
    /// e.g /home/user/maps/map_0.json
    pub file_write_path: PathBuf,
    /// Determines whether the user has saved the changes
    /// to the map file, before switching screens or exiting.
    pub can_exit: bool,
    /// Timestamp for automatically saving changes to the map file
    pub last_save: Instant,
    /// The result of attempting to make a backup file
    pub backup_res: Option<BackupResult>,
    /// Timestamp for automatically making a runtime backup file
    pub rt_backup_ts: Instant,
}

impl PersistenceState {
    pub fn new(file_write_path: PathBuf) -> Self {
        Self {
            file_write_path,
            can_exit: true,
            last_save: Instant::now(),
            backup_res: None,
            rt_backup_ts: Instant::now(),
        }
    }

    /// Marks the map as having unsaved changes
    pub fn mark_dirty(&mut self) {
        self.can_exit = false;
    }

    /// Marks the map as saved (no unsaved changes)
    pub fn mark_clean(&mut self) {
        self.can_exit = true;
    }
    /// Checks if it's time to auto-save based on the interval
    pub fn should_save(&self, interval_seconds: usize) -> bool {
        !self.can_exit && self.last_save.elapsed() > Duration::from_secs(interval_seconds as u64)
    }

    /// Checks if it's time to make a runtime backup
    pub fn should_backup(&self, interval: &crate::states::settings::RuntimeBackupsInterval) -> bool {
        self.rt_backup_ts.elapsed() > get_duration_rt(interval)
    }

    /// Resets the save timer to now
    pub fn reset_save_timer(&mut self) {
        self.last_save = Instant::now();
    }

    /// Resets the backup timer to now
    pub fn reset_backup_timer(&mut self) {
        self.rt_backup_ts = Instant::now();
    }
}