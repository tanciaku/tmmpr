use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

use crate::{
    states::start::ErrMsg, 
    utils::{read_json_data, write_json_data, filesystem::{FileSystem, RealFileSystem}},
};

// PathBuf because the state needs to own it's fields.
#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct RecentPaths {
    pub recent_path_1: Option<PathBuf>,
    pub recent_path_2: Option<PathBuf>,
    pub recent_path_3: Option<PathBuf>,
}

impl RecentPaths {
    pub fn new() -> RecentPaths {
        RecentPaths { 
            recent_path_1: None,
            recent_path_2: None,
            recent_path_3: None,
        }
    }

    /// Adds a new recent_path and moves the other down by 1, discarding the last one
    pub fn add(&mut self, path: PathBuf) {
        // "Move" the other two "down by 1", discarding the one in 3 (if any)
        self.recent_path_3 = self.recent_path_2.clone();
        self.recent_path_2 = self.recent_path_1.clone();

        // Add the new one
        self.recent_path_1 = Some(path);
    }

    /// Returns true if the given path exists in any of the recent paths
    pub fn contains_path(&self, path: &Path) -> bool {
        self.recent_path_1.as_deref() == Some(path) ||
        self.recent_path_2.as_deref() == Some(path) ||
        self.recent_path_3.as_deref() == Some(path)
    }

    /// Save recent paths using a filesystem abstraction (testable version).
    /// There cannot be an error here since - if the user cannot use the
    /// recent_paths functionality - this will never be called.
    /// If the directories didn't exist before - they would at this point.
    pub fn save_with_fs(&self, fs: &impl FileSystem) {
        // Get the user's home directory path
        let home_path = match fs.get_home_dir() {
            Some(path) => path,
            None => return,
        };

        // Make the full path to the file (/home/user/.config/tmmpr/recent_paths.json)
        let recent_paths_file_path = home_path.join(".config/tmmpr/recent_paths").with_extension("json");

        // Write the data (guaranteed at this point)
        let _ = write_json_data(&recent_paths_file_path, &self);
    }
}

/// Gets the recent paths from the filesystem using a filesystem abstraction (testable version).
/// Or creates an empty one if it doesn't exist
/// If there is an error somewhere along the way - returns an error message
///   (can't use recent_paths functionality in that case)
pub fn get_recent_paths_with_fs(fs: &impl FileSystem) -> Result<RecentPaths, ErrMsg> {
    // Get the user's home directory path
    let home_path = match fs.get_home_dir() {
        Some(path) => path,
        None => return Err(ErrMsg::DirFind),
    };

    // Make the path to the config directory (e.g. /home/user/.config/tmmpr/)
    let config_dir_path = home_path.join(".config/tmmpr/");

    // Create the directory if it doesn't exist
    if let Err(_) = fs.create_dir_all(&config_dir_path) {
        return Err(ErrMsg::DirCreate)
    };

    // Make the full path to the file (e.g. /home/user/.config/tmmpr/recent_paths.json)
    let recent_paths_file_path = config_dir_path.join("recent_paths").with_extension("json");

    // Load the file if it exits:
    if fs.path_exists(&recent_paths_file_path) {
        match read_json_data(&recent_paths_file_path) {
            Ok(recent_paths) => Ok(recent_paths),
            Err(_) => Err(ErrMsg::FileRead),
        }
    } else { // Otherwise create it
        let new_recent_paths = RecentPaths::new();
        match write_json_data(&recent_paths_file_path, &new_recent_paths) {
            Ok(_) => Ok(new_recent_paths),
            Err(_) => Err(ErrMsg::FileWrite),
        }
    }
}

/// Gets the recent paths from the ~/.config/tmmpr/recent_paths.json file (production version).
/// Calls get_recent_paths_with_fs with RealFileSystem.
pub fn get_recent_paths() -> Result<RecentPaths, ErrMsg> {
    get_recent_paths_with_fs(&RealFileSystem)
}