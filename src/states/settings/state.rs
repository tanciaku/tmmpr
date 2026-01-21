use std::path::PathBuf;

use crate::{
    states::settings::{
        BackupsErr, BackupsInterval, DiscardExitTo, RuntimeBackupsInterval,
        SelectedToggle, SettingsNotification, SettingsType, get_settings
    },
    utils::{FileSystem, RealFileSystem}
};

/// Resolve the backup path from user input
/// If absolute (starts with '/'), returns as-is
/// If relative, joins with home directory
pub fn resolve_backup_path<F: FileSystem>(input_path: &str, fs: &F) -> Result<PathBuf, BackupsErr> {
    if input_path.starts_with('/') {
        // Absolute path - use as-is
        Ok(PathBuf::from(input_path))
    } else {
        // Relative path - resolve from home directory
        let home_path = fs.get_home_dir()
            .ok_or(BackupsErr::DirFind)?;
        Ok(home_path.join(input_path))
    }
}

/// Validate that the backup directory can be created and written to
pub fn validate_backup_directory<F: FileSystem>(path: &PathBuf, fs: &F) -> Result<(), BackupsErr> {
    // Create the directory
    fs.create_dir_all(path)
        .map_err(|_| BackupsErr::DirCreate)?;

    // Test writing to the directory
    fs.test_write_to_dir(path)
        .map_err(|_| BackupsErr::FileWrite)?;

    Ok(())
}

#[derive(PartialEq, Debug)]
pub struct SettingsState {
    pub needs_clear_and_redraw: bool,
    pub settings_context_page: bool,
    /// To easily go back to the map file that was opened.
    pub map_file_path: PathBuf,
    pub settings: SettingsType,
    /// Determines whether the user has saved the changes
    /// to the settings file, before switching screens or exiting.
    pub can_exit: bool,
    /// Whether to render a menu for confirming to discard 
    /// unsaved changes
    pub confirm_discard_menu: Option<DiscardExitTo>,
    pub notification: Option<SettingsNotification>,
    /// Which toggle is selected in the settings menu
    pub selected_toggle: SelectedToggle,
    /// Whether to display context page (context for toggles)
    pub context_page: bool,
    /// Whether to display an input prompt for entering a path for backups.
    pub input_prompt: bool,
    /// Whether to notify user that using the entered path for backups failed.
    pub input_prompt_err: Option<BackupsErr>,
}

impl SettingsState {
    pub fn new(map_file_path: PathBuf) -> SettingsState {
        SettingsState {
            needs_clear_and_redraw: true,
            settings_context_page: false,
            map_file_path: map_file_path,
            settings: get_settings(),
            can_exit: true,
            confirm_discard_menu: None,
            notification: None,
            selected_toggle: SelectedToggle::Toggle1,
            context_page: false,
            input_prompt: false,
            input_prompt_err: None,
        }
    }

    /// Go down a toggle in the settings menu.
    pub fn toggle_go_down(&mut self) {
        self.selected_toggle = match self.selected_toggle {
            SelectedToggle::Toggle1 => SelectedToggle::Toggle2,
            SelectedToggle::Toggle2 => {
                // If backups enabled - toggle3 (runtime backups) is available - so can go to it.
                if let Some(_) = self.settings.settings().runtime_backups_interval {
                    SelectedToggle::Toggle3
                // If not - that toggle isn't available.
                } else {
                    SelectedToggle::Toggle4
                }
            }
            SelectedToggle::Toggle3 => SelectedToggle::Toggle4,
            SelectedToggle::Toggle4 => SelectedToggle::Toggle5,
            SelectedToggle::Toggle5 => SelectedToggle::Toggle6,
            SelectedToggle::Toggle6 => SelectedToggle::Toggle1,
        }
    }

    /// Go up a toggle in the settings menu.
    pub fn toggle_go_up(&mut self) {
        self.selected_toggle = match self.selected_toggle {
            SelectedToggle::Toggle1 => SelectedToggle::Toggle6,
            SelectedToggle::Toggle2 => SelectedToggle::Toggle1,
            SelectedToggle::Toggle3 => SelectedToggle::Toggle2,
            SelectedToggle::Toggle4 => {
                // If backups enabled - toggle3 (runtime backups) is available - so can go to it.
                if let Some(_) = self.settings.settings().runtime_backups_interval {
                    SelectedToggle::Toggle3
                // If not - that toggle isn't available.
                } else {
                    SelectedToggle::Toggle2
                }
            }
            SelectedToggle::Toggle5 => SelectedToggle::Toggle4,
            SelectedToggle::Toggle6 => SelectedToggle::Toggle5,
        }
    }

    pub fn submit_path(&mut self) {
        self.submit_path_with_fs(&RealFileSystem)
    }

    /// Submit path with injectable filesystem operations for testing
    pub fn submit_path_with_fs<F: FileSystem>(&mut self, fs: &F) {
        // User entered path for backups
        // .unwrap() used here - because while in the input prompt - backups_path cannot be None 
        let input_dir_path = self.settings.settings().backups_path.as_ref().unwrap();

        // Resolve the full path
        let backups_dir = match resolve_backup_path(input_dir_path, fs) {
            Ok(path) => path,
            Err(err) => {
                self.input_prompt_err = Some(err);
                return;
            }
        };

        // Update the path in settings if it was resolved from relative to absolute
        if !input_dir_path.starts_with('/') {
            self.settings.settings_mut().backups_path = Some(backups_dir.to_string_lossy().to_string());
        }

        // Validate the directory (create and test write)
        if let Err(err) = validate_backup_directory(&backups_dir, fs) {
            self.input_prompt_err = Some(err);
            return;
        }

        // Set the default backup interval
        self.settings.settings_mut().backups_interval = Some(BackupsInterval::Daily);

        // Set the default runtime backup interval
        self.settings.settings_mut().runtime_backups_interval = Some(RuntimeBackupsInterval::Every2Hours);

        // Reset error if already isn't empty
        self.input_prompt_err = None;

        // Exit the input prompt
        self.input_prompt = false;
    }
}