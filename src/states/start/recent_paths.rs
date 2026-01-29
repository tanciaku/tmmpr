use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

use crate::{
    states::start::ErrMsg, 
    utils::{read_json_data, write_json_data, filesystem::FileSystem},
};

/// Stores up to 3 most recently opened map files.
/// Uses PathBuf for owned data that persists across the application lifecycle.
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

    /// Adds a path to the top of the recent list (position 1).
    /// Shifts existing paths down, discarding the oldest if list is full.
    pub fn add(&mut self, path: PathBuf) {
        self.recent_path_3 = self.recent_path_2.clone();
        self.recent_path_2 = self.recent_path_1.clone();
        self.recent_path_1 = Some(path);
    }

    pub fn contains_path(&self, path: &Path) -> bool {
        self.recent_path_1.as_deref() == Some(path) ||
        self.recent_path_2.as_deref() == Some(path) ||
        self.recent_path_3.as_deref() == Some(path)
    }

    /// Persists recent paths to `~/.config/tmmpr/recent_paths.json`.
    /// 
    /// Errors are silently ignored: this function is only called after successful
    /// initialization by `get_recent_paths_with_fs`, which ensures the config directory
    /// exists and is writable. If saving fails, recent paths simply won't persist.
    pub fn save_with_fs(&self, fs: &impl FileSystem) {
        let home_path = match fs.get_home_dir() {
            Some(path) => path,
            None => return,
        };

        let recent_paths_file_path = home_path.join(".config/tmmpr/recent_paths").with_extension("json");

        let _ = write_json_data(&recent_paths_file_path, &self);
    }
}

/// Loads recent paths from `~/.config/tmmpr/recent_paths.json`, creating an empty file
/// if it doesn't exist.
/// 
/// Returns an error if the config directory cannot be created or accessed, which disables
/// recent paths functionality for the session. Uses FileSystem abstraction for testability.
pub fn get_recent_paths_with_fs(fs: &dyn FileSystem) -> Result<RecentPaths, ErrMsg> {
    let home_path = match fs.get_home_dir() {
        Some(path) => path,
        None => return Err(ErrMsg::DirFind),
    };

    let config_dir_path = home_path.join(".config/tmmpr/");

    if let Err(_) = fs.create_dir_all(&config_dir_path) {
        return Err(ErrMsg::DirCreate)
    };

    let recent_paths_file_path = config_dir_path.join("recent_paths").with_extension("json");

    if fs.path_exists(&recent_paths_file_path) {
        match read_json_data(&recent_paths_file_path) {
            Ok(recent_paths) => Ok(recent_paths),
            Err(_) => Err(ErrMsg::FileRead),
        }
    } else {
        let new_recent_paths = RecentPaths::new();
        match write_json_data(&recent_paths_file_path, &new_recent_paths) {
            Ok(_) => Ok(new_recent_paths),
            Err(_) => Err(ErrMsg::FileWrite),
        }
    }
}