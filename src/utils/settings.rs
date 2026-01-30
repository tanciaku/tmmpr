use std::path::Path;

use crate::{
    states::settings::Settings,
    utils::{write_json_data, filesystem::FileSystem},
};

/// Saves the application settings to the settings file using a custom FileSystem.
/// 
/// This function is designed for testing with MockFileSystem or production use with RealFileSystem.
/// 
/// # Errors
/// 
/// Returns an error if:
/// - The home directory cannot be found
/// - The directory cannot be created
/// - The file cannot be written to
pub fn save_settings_to_file_with_fs(settings: &Settings, fs: &dyn FileSystem) -> Result<(), Box<dyn std::error::Error>> { 
    let home_path = fs.get_home_dir()
        .ok_or("Could not find home directory")?;

    // Using XDG Base Directory specification for config files
    let config_dir_path = home_path.join(".config/tmmpr/");

    fs.create_dir_all(&config_dir_path)?;

    let settings_file_path = config_dir_path.join("settings").with_extension("json");

    save_settings_to_path(settings, &settings_file_path)
}

/// Lower-level function for saving settings to an arbitrary path.
///
/// Separated from `save_settings_to_file_with_fs` to allow testing without
/// filesystem abstraction and to enable custom save locations if needed.
pub fn save_settings_to_path(settings: &Settings, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_json_data(path, settings)
}