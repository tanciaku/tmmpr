//! Settings screen input handling

use crate::{app::Screen, states::{SettingsState, StartState, settings::{DiscardExitTo, SelectedToggle, SettingsType, save_settings_with_fs}}, utils::FileSystem};
use super::AppAction;
use crossterm::event::{KeyCode, KeyEvent};

pub fn settings_kh(settings_state: &mut SettingsState, key: KeyEvent, fs: &dyn FileSystem) -> AppAction {

    // Restrict input to only error acknowledgment keys if settings couldn't be loaded
    if let SettingsType::Default(_, error_message) = &settings_state.settings {
        if let Some(_) = error_message {
            match key.code {
                KeyCode::Char('q') => return AppAction::Switch(Screen::Start(StartState::new_with_fs(fs))),
                KeyCode::Char('o') => return AppAction::LoadMapFile(settings_state.map_file_path.clone()),
                _ => {}
            }
        }
    }

    // Discard confirmation dialog takes precedence over other input
    if let Some(exit_to) = &settings_state.confirm_discard_menu {
        match key.code {
            KeyCode::Esc => {
                settings_state.confirm_discard_menu = None;
            }
            KeyCode::Char('q') => {
                match exit_to {
                    DiscardExitTo::StartScreen => return AppAction::Switch(Screen::Start(StartState::new_with_fs(fs))),
                    DiscardExitTo::MapScreen => return AppAction::LoadMapFile(settings_state.map_file_path.clone()),
                }
            }
            _ => {}
        }

        settings_state.needs_clear_and_redraw = true;
        return AppAction::Continue
    }

    // Context help page takes all input when shown
    if settings_state.context_page {
        match key.code {
            KeyCode::Char('?') | KeyCode::F(1) => settings_state.context_page = false,
            _ => {}
        }

        settings_state.needs_clear_and_redraw = true;
        return AppAction::Continue
    }

    // Backup path input prompt takes all input when shown
    if settings_state.input_prompt {
        let settings = settings_state.settings.settings_mut();
        
        // Safety: backups_path is always Some while input_prompt is true
        match key.code {
            // Esc cancels input AND clears any previously entered path
            KeyCode::Esc => {
                settings_state.input_prompt = false;

                settings.backups_path = None;
                settings.backups_interval = None;
                settings.runtime_backups_interval = None;
                settings_state.input_prompt_err = None;
            }
            KeyCode::Char(c) => {
                // Limit to 46 chars to fit UI display width
                if settings.backups_path.as_ref().unwrap().len() < 46 {
                    settings.backups_path.as_mut().unwrap().push(c);
                }
            }
            KeyCode::Backspace => {
                if !settings.backups_path.as_ref().unwrap().is_empty() {
                    settings.backups_path.as_mut().unwrap().pop();
                }
            }
            KeyCode::Enter => settings_state.submit_path(),
            _ => {}
        }

        settings_state.needs_clear_and_redraw = true;
        return AppAction::Continue
    }

    match key.code {
        KeyCode::Char('q') => {
            if settings_state.can_exit {
                return AppAction::Switch(Screen::Start(StartState::new_with_fs(fs)))
            } else {
                settings_state.confirm_discard_menu = Some(DiscardExitTo::StartScreen);
            }            
        }
        KeyCode::Char('o') => {
            if settings_state.can_exit {
                return AppAction::LoadMapFile(settings_state.map_file_path.clone())
            } else {
                settings_state.confirm_discard_menu = Some(DiscardExitTo::MapScreen);
            }            
        }

        KeyCode::Char('?') | KeyCode::F(1) => settings_state.context_page = true,

        KeyCode::Char('s') => save_settings_with_fs(settings_state, fs),

        KeyCode::Char('j') | KeyCode::Down => settings_state.toggle_go_down(),
        KeyCode::Char('k') | KeyCode::Up => settings_state.toggle_go_up(),

        KeyCode::Enter => {
            // Prevent exiting without saving or discarding changes
            settings_state.can_exit = false;

            match settings_state.selected_toggle {
                SelectedToggle::Toggle1 => {
                    settings_state.settings.settings_mut().cycle_save_intervals();
                }
                SelectedToggle::Toggle2 => {
                    settings_state.input_prompt = true;

                    let settings = settings_state.settings.settings_mut();
                    if let None = settings.backups_path {
                        settings.backups_path = Some(String::new());
                    }
                }
                SelectedToggle::Toggle4 => settings_state.settings.settings_mut().cycle_default_sides(true),
                SelectedToggle::Toggle5 => settings_state.settings.settings_mut().cycle_default_sides(false),
                SelectedToggle::Toggle6 => settings_state.settings.settings_mut().edit_modal = !settings_state.settings.settings().edit_modal,
                _ => {}
            }
        }

        // Tab cycles sub-options for backup interval settings
        KeyCode::Tab => {
            match settings_state.selected_toggle {
                SelectedToggle::Toggle2 => {
                    if settings_state.settings.settings().backups_interval.is_some() {
                        settings_state.settings.settings_mut().cycle_backup_interval();
                        settings_state.can_exit = false;
                    }
                }
                SelectedToggle::Toggle3 => {
                    if settings_state.settings.settings().runtime_backups_interval.is_some() {
                        settings_state.settings.settings_mut().cycle_runtime_backup_interval();
                        settings_state.can_exit = false;
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }

    settings_state.needs_clear_and_redraw = true;
    AppAction::Continue
}
