use std::path::Path;

use crate::{
    states::settings::Settings,
    utils::write_json_data,
};

/// Saves the application settings to the settings file in the user's config directory.
/// 
/// This function writes the settings to `~/.config/tmmpr/settings.json`.
/// 
/// # Errors
/// 
/// Returns an error if:
/// - The home directory cannot be found
/// - The file cannot be written to
pub fn save_settings_to_file(settings: &Settings) -> Result<(), Box<dyn std::error::Error>> { 
    // Get the user's home directory path
    let home_path = home::home_dir()
        .ok_or("Could not find home directory")?;

    // Make the full path to the file (/home/user/.config/tmmpr/settings.json)
    let settings_file_path = home_path.join(".config/tmmpr/settings").with_extension("json");

    // Write the data
    save_settings_to_path(settings, &settings_file_path)
}

// Add a more testable version that accepts a path
pub fn save_settings_to_path(settings: &Settings, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_json_data(path, settings)
}