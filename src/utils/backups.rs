use chrono::{Duration as ChronoDuration, Local};
use std::{path::PathBuf, time::Duration as StdDuration};

use crate::{
    states::{
        MapState,
        map::{BackupResult, Notification},
        settings::{BackupsInterval, RuntimeBackupsInterval}
    },
    utils::{save_map_file, save_settings_to_file},
};

/// Handles creating backups when loading a map file, if backups are enabled
pub fn handle_on_load_backup(map_state: &mut MapState) {
    // Extract backup configuration and file info
    // This function is structured like so - to prevent multiple borrow conflicts.
    // If backups are enabled - backups_path and backups_interval will be Some,
    // and backup_config contents will be Some(date).
    // If backups are disabled - backups_path and backups_interval will be None,
    // and backup_config contents will be None.
    let backup_config = if let (Some(backups_path), Some(backups_interval)) = 
        (&map_state.settings.backups_path, &map_state.settings.backups_interval) {
        
        // Get the name of the map file opened
        let filename = map_state.file_write_path.file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        // Get the current date
        let date = Local::now();
        
        // Backups functionality enabled -
        // return the data into a variable for use in this function.
        Some((
            PathBuf::from(backups_path.clone()), // Convert backups_path to an owned PathBuf for use here
            backups_interval,
            filename.to_string(),
            date,
            map_state.settings.backup_dates.get(filename).copied()
        ))
    } else {
        // Backups functionality disabled.
        None
    };


    // If backups enabled (configuration data exists)
    if let Some((backups_path, backups_interval, filename, date, last_backup_date)) = backup_config {
        // Whether to create a backup file
        let should_backup = match last_backup_date {
            // No previous backup (filename key) in the backup_dates HashMap.
            //      (No backup was ever made of this map file)
            None => true,
            // Check if set interval (time) has passed since last backup
            Some(last_date) => {
                let time_passed = date - last_date;
                time_passed >= get_duration(&backups_interval)
            }
        };


        if should_backup {
            let backups_file_path = backups_path
                .join(format!("{}-load-backup-{}", filename, date.format("%y-%m-%d")))
                .with_extension("json");

            // Attempt to create the backup file
            // (save_map_file changes map_state.backup_res depending 
            //      on the result of the write operation)
            save_map_file(map_state, &backups_file_path, true, true);

 
            // Handle the backup result and update settings if successful
            match &map_state.backup_res {
                Some(BackupResult::BackupSuccess) => {
                    // Update the backup date in settings
                    map_state.settings.backup_dates.insert(filename, date);
                    
                    // Save updated settings (backup_dates field) to file
                    if let Err(_) = save_settings_to_file(&map_state.settings) {
                        // If there was an error updating backup records - notify user.
                        map_state.show_notification = Some(Notification::BackupRecordFail);
                    }

                    // Reset the result of a backup operation
                    map_state.backup_res = None;
                }
                Some(BackupResult::BackupFail) => {
                    // Backup failed - notification already handled by save_map_file
                    
                    // Reset the result of a backup operation
                    map_state.backup_res = None;
                }
                None => unreachable!(), // save_map_file with backup flag always sets backup_res
            }
        }
    }
}

/// Handles creating backups the map file has been loaded and the application
/// was running for a while, if backups are enabled
pub fn handle_runtime_backup(map_state: &mut MapState) {
    // Extract backup configuration and file info
    // This function is structured like so - to prevent multiple borrow conflicts.
    // If backups are enabled - backups_path and backups_interval will be Some,
    // and backup_config contents will be Some(date).
    // If backups are disabled - backups_path and backups_interval will be None,
    // and backup_config contents will be None.
    let backup_config = if let (Some(backups_path), Some(_)) = 
        (&map_state.settings.backups_path, &map_state.settings.runtime_backups_interval) {
        
        // Get the name of the map file opened
        let filename = map_state.file_write_path.file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        // Get the current date
        let date = Local::now();
        
        // Backups functionality enabled -
        // return the data into a variable for use in this function.
        Some((
            PathBuf::from(backups_path.clone()), // Convert backups_path to an owned PathBuf for use here
            filename.to_string(),
            date,
            // Backup dates are not updated to reflect runtime backups
        ))
    } else {
        // Backups functionality disabled.
        None
    };


    // If backups enabled (configuration data exists)
    if let Some((backups_path, filename, date)) = backup_config {
        // Backup dates are not associated with making runtime backups.

        let backups_file_path = backups_path
            .join(format!("{}-session-backup-{}", filename, date.format("%y-%m-%d-%H%M")))
            .with_extension("json");

        // Attempt to create the backup file
        // (save_map_file changes map_state.backup_res depending 
        //      on the result of the write operation)
        save_map_file(map_state, &backups_file_path, true, true);

        // Handle the backup result and update settings if successful
        match &map_state.backup_res {
            Some(BackupResult::BackupSuccess) => {
                // Backup succeeded - notification already handled by save_map_file 

                // Reset the result of a backup operation
                map_state.backup_res = None;
            }
            Some(BackupResult::BackupFail) => {
                // Backup failed - notification already handled by save_map_file
                
                // Reset the result of a backup operation
                map_state.backup_res = None;
            }
            None => unreachable!(), // save_map_file with backup flag always sets backup_res
        }
    }
}

/// Get the Duration type from the BackupsInterval stored in Settings
pub fn get_duration(interval: &BackupsInterval) -> ChronoDuration {
    match interval {
        BackupsInterval::Daily => ChronoDuration::days(1),
        BackupsInterval::Every3Days => ChronoDuration::days(3),
        BackupsInterval::Weekly => ChronoDuration::weeks(1),
        BackupsInterval::Every2Weeks => ChronoDuration::weeks(2),
    }
}

/// Get the Duration type from the RuntimeBackupsInterval stored in Settings
pub fn get_duration_rt(interval: &RuntimeBackupsInterval) -> StdDuration {
    match interval {
        RuntimeBackupsInterval::Hourly => StdDuration::from_secs(3600),
        RuntimeBackupsInterval::Every2Hours => StdDuration::from_secs(7200),
        RuntimeBackupsInterval::Every4Hours => StdDuration::from_secs(14400),
        RuntimeBackupsInterval::Every6Hours => StdDuration::from_secs(21600),
        RuntimeBackupsInterval::Every12Hours => StdDuration::from_secs(43200),
    }
}