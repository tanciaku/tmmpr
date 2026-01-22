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
pub fn save_settings_to_file_with_fs(settings: &Settings, fs: &impl FileSystem) -> Result<(), Box<dyn std::error::Error>> { 
    // Get the user's home directory path
    let home_path = fs.get_home_dir()
        .ok_or("Could not find home directory")?;

    // Make the path to the settings directory (e.g. /home/user/.config/tmmpr/)
    let config_dir_path = home_path.join(".config/tmmpr/");

    // Create the directory if it doesn't exist
    fs.create_dir_all(&config_dir_path)?;

    // Make the full path to the file (/home/user/.config/tmmpr/settings.json)
    let settings_file_path = config_dir_path.join("settings").with_extension("json");

    // Write the data
    save_settings_to_path(settings, &settings_file_path)
}

// Add a more testable version that accepts a path
pub fn save_settings_to_path(settings: &Settings, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_json_data(path, settings)
}