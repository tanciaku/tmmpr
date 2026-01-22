use crate::{
    states::{
        SettingsState,
        settings::{Settings, SettingsNotification, SettingsType},
        start::ErrMsg
    }, 
    utils::{read_json_data, save_settings_to_file_with_fs, write_json_data, filesystem::{FileSystem, RealFileSystem}}
};

pub fn get_settings() -> SettingsType {
    get_settings_with_fs(&RealFileSystem)
}

/// Get settings using a custom FileSystem (for testing or production).
pub fn get_settings_with_fs(fs: &impl FileSystem) -> SettingsType {
    // Get the user's home directory path
    let home_path = match fs.get_home_dir() {
        Some(path) => path,
        None => return SettingsType::Default(Settings::new(), Some(ErrMsg::DirFind)),
    };

    //// Make the path to the settings directory (e.g. /home/user/.config/tmmpr/)
    let config_dir_path = home_path.join(".config/tmmpr/");

    // Create the directory if it doesn't exist
    if let Err(_) = fs.create_dir_all(&config_dir_path) {
        return SettingsType::Default(Settings::new(), Some(ErrMsg::DirCreate))
    };

    // Make the full path to the file (e.g. /home/user/.config/tmmpr/settings.json)
    let settings_file_path = config_dir_path.join("settings").with_extension("json");

    // Load the file if it exits:
    if fs.path_exists(&settings_file_path) {
        match read_json_data(&settings_file_path) {
            Ok(settings) => SettingsType::Custom(settings),
            Err(_) => SettingsType::Default(Settings::new(), Some(ErrMsg::FileRead)),
        }
    } else { // Otherwise create it
        let new_settings = Settings::new();
        match write_json_data(&settings_file_path, &new_settings) {
            Ok(_) => SettingsType::Default(Settings::new(), None),
            Err(_) => SettingsType::Default(Settings::new(), Some(ErrMsg::FileWrite)),
        }
    }
}

/// This is (can) only be called if user can use the settings 
/// functionality - directories (already) exist in that case.
pub fn save_settings(settings_state: &mut SettingsState) {
    save_settings_with_fs(settings_state, &RealFileSystem)
}

/// Save settings using a custom FileSystem (for testing or production).
pub fn save_settings_with_fs(settings_state: &mut SettingsState, fs: &impl FileSystem) {
    // Reference to Settings in the SettingsType
    let settings = &settings_state.settings.settings();

    // Save the settings data
    match save_settings_to_file_with_fs(settings, fs) {
        Ok(_) => {
            // Saved changes to a file - so can now exit the settings menu.
            settings_state.can_exit = true;
            settings_state.notification = Some(SettingsNotification::SaveSuccess);
        }
        Err(_) => settings_state.notification = Some(SettingsNotification::SaveFail),
    }
}