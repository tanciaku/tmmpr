use std::{fs, path::PathBuf};
use tempfile::NamedTempFile;

use crate::states::settings::{
    BackupsErr, BackupsInterval, DiscardExitTo, RuntimeBackupsInterval,
    SelectedToggle, SettingsNotification, SettingsType, get_settings
};


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
        // User entered path for backups
        // .unwrap() used here - because while in the input prompt - backups_path cannot be None 
        let input_dir_path = self.settings.settings().backups_path.as_ref().unwrap();

        // If the path start with '/' - keep the path as it is (it's an absolute path e.g. /mnt/map_backups/)
        let backups_dir = if input_dir_path.starts_with('/') {
            PathBuf::from(input_dir_path)
        } else { // If it doesn't - append that path to the home user directory path (e.g. /home/user/map_backups/)
            // Get the user's home directory path
            let home_path = match home::home_dir() {
                Some(path) => path,
                None => {
                    self.input_prompt_err = Some(BackupsErr::DirFind);
                    return;
                }
            };
            // Construct the full path
            let backups_dir = home_path.join(input_dir_path);

            // Set that path in the Settings field
            // "Setting it" doesn't mean submitting succeded - it's for in case it does - use
            // the full path from home directory instead of the user entered directory path.
            self.settings.settings_mut().backups_path = Some(backups_dir.to_string_lossy().to_string());

            // Return it into a variable for use in this function
            backups_dir
        };
    
        // Create the directory
        if let Err(_) = fs::create_dir_all(&backups_dir) {
            self.input_prompt_err = Some(BackupsErr::DirCreate);
            return;
        };

        // Attempt to write data to that directory
        if let Err(_) = NamedTempFile::new_in(&backups_dir) {
            self.input_prompt_err = Some(BackupsErr::FileWrite);
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