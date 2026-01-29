use std::{path::PathBuf, time::{Duration, Instant}};

use crate::{states::map::BackupResult, utils::get_duration_rt};

/// Tracks file persistence, auto-save timing, and backup state for a map.
#[derive(PartialEq, Debug)]
pub struct PersistenceState {
    /// The path provided by the user to write the map data to
    /// e.g /home/user/maps/map_0.json
    pub file_write_path: PathBuf,
    /// Gates exiting/switching screens: false when there are unsaved changes,
    /// true when all changes are saved.
    pub can_exit: bool,
    pub last_save: Instant,
    pub backup_res: Option<BackupResult>,
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

    pub fn mark_dirty(&mut self) {
        self.can_exit = false;
    }

    pub fn mark_clean(&mut self) {
        self.can_exit = true;
    }

    /// Only auto-saves if there are unsaved changes AND the interval has elapsed
    pub fn should_save(&self, interval_seconds: usize) -> bool {
        !self.can_exit && self.last_save.elapsed() > Duration::from_secs(interval_seconds as u64)
    }

    pub fn should_backup(&self, interval: &crate::states::settings::RuntimeBackupsInterval) -> bool {
        self.rt_backup_ts.elapsed() > get_duration_rt(interval)
    }

    pub fn reset_save_timer(&mut self) {
        self.last_save = Instant::now();
    }

    pub fn reset_backup_timer(&mut self) {
        self.rt_backup_ts = Instant::now();
    }
}