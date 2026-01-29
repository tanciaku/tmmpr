use crate::{
    states::{
        SettingsState,
        settings::{Settings, SettingsNotification, SettingsType},
        start::ErrMsg
    }, 
    utils::{read_json_data, save_settings_to_file_with_fs, write_json_data, filesystem::{FileSystem, RealFileSystem}}
};

/// Loads settings from disk or returns defaults with an error notification.
/// 
/// Returns:
/// - `Default` with no error: First boot, settings file created successfully
/// - `Default` with error: File system or I/O error (user should be notified)
/// - `Custom`: Existing settings loaded successfully
pub fn get_settings_with_fs(fs: &dyn FileSystem) -> SettingsType {
    let home_path = match fs.get_home_dir() {
        Some(path) => path,
        None => return SettingsType::Default(Settings::new(), Some(ErrMsg::DirFind)),
    };

    let config_dir_path = home_path.join(".config/tmmpr/");

    if let Err(_) = fs.create_dir_all(&config_dir_path) {
        return SettingsType::Default(Settings::new(), Some(ErrMsg::DirCreate))
    };

    let settings_file_path = config_dir_path.join("settings").with_extension("json");

    if fs.path_exists(&settings_file_path) {
        match read_json_data(&settings_file_path) {
            Ok(settings) => SettingsType::Custom(settings),
            Err(_) => SettingsType::Default(Settings::new(), Some(ErrMsg::FileRead)),
        }
    } else {
        let new_settings = Settings::new();
        match write_json_data(&settings_file_path, &new_settings) {
            Ok(_) => SettingsType::Default(Settings::new(), None),
            Err(_) => SettingsType::Default(Settings::new(), Some(ErrMsg::FileWrite)),
        }
    }
}

/// Saves settings to disk.
/// 
/// Precondition: Settings directories already exist (created during app initialization).
pub fn save_settings(settings_state: &mut SettingsState) {
    save_settings_with_fs(settings_state, &RealFileSystem)
}

pub fn save_settings_with_fs(settings_state: &mut SettingsState, fs: &dyn FileSystem) {
    let settings = &settings_state.settings.settings();

    match save_settings_to_file_with_fs(settings, fs) {
        Ok(_) => {
            // Only allow exiting the settings menu after successful save to prevent data loss
            settings_state.can_exit = true;
            settings_state.notification = Some(SettingsNotification::SaveSuccess);
        }
        Err(_) => settings_state.notification = Some(SettingsNotification::SaveFail),
    }
}