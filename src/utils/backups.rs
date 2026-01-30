use chrono::{Duration as ChronoDuration, Local};
use std::{path::PathBuf, time::Duration as StdDuration};

use crate::{
    states::{
        MapState,
        map::{BackupResult, Notification},
        settings::{BackupsInterval, RuntimeBackupsInterval}
    },
    utils::{save_map_file, save_settings_to_file_with_fs, filesystem::FileSystem},
};

/// Creates a backup snapshot when a map file is loaded, respecting the configured backup interval.
/// 
/// Uses a custom filesystem implementation for testability.
pub fn handle_on_load_backup_with_fs(
    map_state: &mut MapState, 
    fs: &impl FileSystem,
    current_date: chrono::DateTime<Local>
) {
    // Extract configuration upfront to avoid multiple mutable borrows of map_state
    // throughout the function. All data needed for backup decision is pulled into
    // backup_config, or None if backups are disabled.
    let backup_config = if let (Some(backups_path), Some(backups_interval)) = 
        (&map_state.settings.backups_path, &map_state.settings.backups_interval) {
        
        let filename = map_state.persistence.file_write_path.file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        
        Some((
            PathBuf::from(backups_path.clone()),
            backups_interval,
            filename.to_string(),
            current_date,
            map_state.settings.backup_dates.get(filename).copied()
        ))
    } else {
        None
    };

    if let Some((backups_path, backups_interval, filename, date, last_backup_date)) = backup_config {
        let should_backup = match last_backup_date {
            None => true,
            Some(last_date) => {
                let time_passed = date - last_date;
                time_passed >= get_duration(&backups_interval)
            }
        };

        if should_backup {
            let backups_file_path = backups_path
                .join(format!("{}-load-backup-{}", filename, date.format("%y-%m-%d")))
                .with_extension("json");

            save_map_file(map_state, &backups_file_path, true, true);
 
            match &map_state.persistence.backup_res {
                Some(BackupResult::BackupSuccess) => {
                    map_state.settings.backup_dates.insert(filename, date);
                    
                    if let Err(_) = save_settings_to_file_with_fs(&map_state.settings, fs) {
                        map_state.ui_state.set_notification(Notification::BackupRecordFail);
                    }

                    map_state.persistence.backup_res = None;
                }
                Some(BackupResult::BackupFail) => {
                    // Notification already handled by save_map_file
                    map_state.persistence.backup_res = None;
                }
                None => unreachable!(), // save_map_file with backup flag always sets backup_res
            }
        }
    }
}

/// Creates periodic backups during an active editing session at the configured runtime interval.
/// 
/// Unlike on-load backups, runtime backups:
/// - Include hours/minutes in the filename for multiple backups per day
/// - Do not update the backup_dates registry (only on-load backups do)
/// - Are triggered by elapsed time since the last runtime backup
/// - Always makes a backup when called, interval handled outside
pub fn handle_runtime_backup(map_state: &mut MapState) {
    // Extract configuration upfront to avoid multiple mutable borrows of map_state
    let backup_config = if let (Some(backups_path), Some(_)) = 
        (&map_state.settings.backups_path, &map_state.settings.runtime_backups_interval) {
        
        let filename = map_state.persistence.file_write_path.file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        let date = Local::now();
        
        Some((
            PathBuf::from(backups_path.clone()),
            filename.to_string(),
            date,
        ))
    } else {
        None
    };

    if let Some((backups_path, filename, date)) = backup_config {
        let backups_file_path = backups_path
            .join(format!("{}-session-backup-{}", filename, date.format("%y-%m-%d-%H%M")))
            .with_extension("json");

        save_map_file(map_state, &backups_file_path, true, true);

        match &map_state.persistence.backup_res {
            Some(BackupResult::BackupSuccess) => {
                // Notification already handled by save_map_file
                map_state.persistence.backup_res = None;
            }
            Some(BackupResult::BackupFail) => {
                // Notification already handled by save_map_file
                map_state.persistence.backup_res = None;
            }
            None => unreachable!(), // save_map_file with backup flag always sets backup_res
        }
    }
}

/// Converts a BackupsInterval enum to its equivalent chrono::Duration for date arithmetic.
pub fn get_duration(interval: &BackupsInterval) -> ChronoDuration {
    match interval {
        BackupsInterval::Daily => ChronoDuration::days(1),
        BackupsInterval::Every3Days => ChronoDuration::days(3),
        BackupsInterval::Weekly => ChronoDuration::weeks(1),
        BackupsInterval::Every2Weeks => ChronoDuration::weeks(2),
    }
}

/// Converts a RuntimeBackupsInterval enum to its equivalent std::Duration for timer operations.
pub fn get_duration_rt(interval: &RuntimeBackupsInterval) -> StdDuration {
    match interval {
        RuntimeBackupsInterval::Hourly => StdDuration::from_secs(3600),
        RuntimeBackupsInterval::Every2Hours => StdDuration::from_secs(7200),
        RuntimeBackupsInterval::Every4Hours => StdDuration::from_secs(14400),
        RuntimeBackupsInterval::Every6Hours => StdDuration::from_secs(21600),
        RuntimeBackupsInterval::Every12Hours => StdDuration::from_secs(43200),
    }
}
