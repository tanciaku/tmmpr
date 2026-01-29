use std::path::PathBuf;

use crate::{
    states::settings::{
        BackupsErr, BackupsInterval, DiscardExitTo, RuntimeBackupsInterval,
        SelectedToggle, SettingsNotification, SettingsType, get_settings_with_fs
    },
    utils::{FileSystem, RealFileSystem}
};

/// Resolves backup path to absolute path.
/// Relative paths are resolved from home directory.
pub fn resolve_backup_path<F: FileSystem>(input_path: &str, fs: &F) -> Result<PathBuf, BackupsErr> {
    if input_path.starts_with('/') {
        Ok(PathBuf::from(input_path))
    } else {
        let home_path = fs.get_home_dir()
            .ok_or(BackupsErr::DirFind)?;
        Ok(home_path.join(input_path))
    }
}

/// Validates backup directory by attempting to create it and write to it.
pub fn validate_backup_directory<F: FileSystem>(path: &PathBuf, fs: &F) -> Result<(), BackupsErr> {
    fs.create_dir_all(path)
        .map_err(|_| BackupsErr::DirCreate)?;

    fs.test_write_to_dir(path)
        .map_err(|_| BackupsErr::FileWrite)?;

    Ok(())
}

#[derive(PartialEq, Debug)]
pub struct SettingsState {
    pub needs_clear_and_redraw: bool,
    pub settings_context_page: bool,
    /// Path to return to when exiting settings.
    pub map_file_path: PathBuf,
    pub settings: SettingsType,
    /// Whether unsaved changes have been committed to disk.
    pub can_exit: bool,
    /// Destination screen when discarding unsaved changes.
    pub confirm_discard_menu: Option<DiscardExitTo>,
    pub notification: Option<SettingsNotification>,
    pub selected_toggle: SelectedToggle,
    pub context_page: bool,
    pub input_prompt: bool,
    pub input_prompt_err: Option<BackupsErr>,
}

impl SettingsState {
    pub fn new_with_fs(map_file_path: PathBuf, fs: &dyn FileSystem) -> SettingsState {
        SettingsState {
            needs_clear_and_redraw: true,
            settings_context_page: false,
            map_file_path: map_file_path,
            settings: get_settings_with_fs(fs),
            can_exit: true,
            confirm_discard_menu: None,
            notification: None,
            selected_toggle: SelectedToggle::Toggle1,
            context_page: false,
            input_prompt: false,
            input_prompt_err: None,
        }
    }

    pub fn toggle_go_down(&mut self) {
        self.selected_toggle = match self.selected_toggle {
            SelectedToggle::Toggle1 => SelectedToggle::Toggle2,
            SelectedToggle::Toggle2 => {
                // Skip Toggle3 (runtime backups interval) if backups are disabled
                if let Some(_) = self.settings.settings().runtime_backups_interval {
                    SelectedToggle::Toggle3
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

    pub fn toggle_go_up(&mut self) {
        self.selected_toggle = match self.selected_toggle {
            SelectedToggle::Toggle1 => SelectedToggle::Toggle6,
            SelectedToggle::Toggle2 => SelectedToggle::Toggle1,
            SelectedToggle::Toggle3 => SelectedToggle::Toggle2,
            SelectedToggle::Toggle4 => {
                // Skip Toggle3 (runtime backups interval) if backups are disabled
                if let Some(_) = self.settings.settings().runtime_backups_interval {
                    SelectedToggle::Toggle3
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

    /// Submit path with injectable filesystem operations for testing.
    pub fn submit_path_with_fs<F: FileSystem>(&mut self, fs: &F) {
        // Safe unwrap: backups_path is guaranteed to be Some while input_prompt is active
        let input_dir_path = self.settings.settings().backups_path.as_ref().unwrap();

        let backups_dir = match resolve_backup_path(input_dir_path, fs) {
            Ok(path) => path,
            Err(err) => {
                self.input_prompt_err = Some(err);
                return;
            }
        };

        // Store absolute path for consistency
        if !input_dir_path.starts_with('/') {
            self.settings.settings_mut().backups_path = Some(backups_dir.to_string_lossy().to_string());
        }

        if let Err(err) = validate_backup_directory(&backups_dir, fs) {
            self.input_prompt_err = Some(err);
            return;
        }

        // Initialize backup intervals with sensible defaults
        self.settings.settings_mut().backups_interval = Some(BackupsInterval::Daily);
        self.settings.settings_mut().runtime_backups_interval = Some(RuntimeBackupsInterval::Every2Hours);

        self.input_prompt_err = None;
        self.input_prompt = false;
    }
}