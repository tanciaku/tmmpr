//! Settings screen input handling

use crate::{app::Screen, states::{SettingsState, StartState, settings::{DiscardExitTo, SelectedToggle, SettingsType, save_settings}}, utils::FileSystem};
use super::AppAction;
use crossterm::event::{KeyCode, KeyEvent};


/// Key handling for the Settings Screen
pub fn settings_kh(settings_state: &mut SettingsState, key: KeyEvent, fs: &dyn FileSystem) -> AppAction {

    // If there was an error with using settings functionality - take only this input
    if let SettingsType::Default(_, error_message) = &settings_state.settings {
        if let Some(_) = error_message {
            match key.code {
                KeyCode::Char('q') => return AppAction::Switch(Screen::Start(StartState::new_with_fs(fs))),
                KeyCode::Char('o') => return AppAction::LoadMapFile(settings_state.map_file_path.clone()),
                _ => {}
            }
        }
    }

    // If in the prompt to discard changes - take all input
    if let Some(exit_to) = &settings_state.confirm_discard_menu {
        match key.code {
            // Cancel
            KeyCode::Esc => {
                settings_state.confirm_discard_menu = None;
            }
            // Confirm exiting and discarding unsaved changes
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

    // If on the context page - take all input
    if settings_state.context_page {
        match key.code {
            // Close context page
            KeyCode::Char('?') | KeyCode::F(1) => settings_state.context_page = false,
            _ => {}
        }

        settings_state.needs_clear_and_redraw = true;
        return AppAction::Continue
    }

    // If in the input prompt for entering a path for backups - take all input
    if settings_state.input_prompt {
        // Get a mutable reference to Settings within SettingsType
        let settings = settings_state.settings.settings_mut();
        
        // .unwrap used here - because while in the input prompt - backups_path cannot be None 
        match key.code {
            // Cancel (Exit)
            // Esc - acts as both cancel entering a path
            //          and remove a path that was previously entered.
            KeyCode::Esc => {
                // Close input prompt
                settings_state.input_prompt = false;

                // Reset fields
                settings.backups_path = None;
                settings.backups_interval = None; // if already isn't
                settings.runtime_backups_interval = None; // if already isn't
                // Reset error
                settings_state.input_prompt_err = None; // if already isn't
            }
            // Typing
            KeyCode::Char(c) => {
                if settings.backups_path.as_ref().unwrap().len() < 46 {
                    settings.backups_path.as_mut().unwrap().push(c);
                }
            }
            KeyCode::Backspace => {
                if settings.backups_path.as_ref().unwrap().len() > 0 {
                    settings.backups_path.as_mut().unwrap().pop();
                }
            }
            // Submit
            KeyCode::Enter => settings_state.submit_path(),
            _ => {}
        }

        settings_state.needs_clear_and_redraw = true;
        return AppAction::Continue
    }

    match key.code {
        // Go to the start screen
        KeyCode::Char('q') => {
            if settings_state.can_exit {
                return AppAction::Switch(Screen::Start(StartState::new_with_fs(fs)))
            } else {
                settings_state.confirm_discard_menu = Some(DiscardExitTo::StartScreen);
            }            
        }
        // Go back to the map screen
        KeyCode::Char('o') => {
            if settings_state.can_exit {
                return AppAction::LoadMapFile(settings_state.map_file_path.clone())
            } else {
                settings_state.confirm_discard_menu = Some(DiscardExitTo::MapScreen);
            }            
        }

        // Show context page
        KeyCode::Char('?') | KeyCode::F(1) => settings_state.context_page = true,

        // Save settings
        KeyCode::Char('s') => save_settings(settings_state),

        // Go down a toggle
        KeyCode::Char('j') | KeyCode::Down => settings_state.toggle_go_down(),
        // Go up a toggle
        KeyCode::Char('k') | KeyCode::Up => settings_state.toggle_go_up(),

        // Toggle the selected setting
        KeyCode::Enter => {
            // Have to save or discard changes before exiting
            settings_state.can_exit = false;

            // Which setting is currently selected?
            match settings_state.selected_toggle {
                // Map changes save interval
                SelectedToggle::Toggle1 => {
                    settings_state.settings.settings_mut().cycle_save_intervals();
                }
                // Backups functionality
                SelectedToggle::Toggle2 => {
                    settings_state.input_prompt = true;

                    // If there isn't a path string already
                    let settings = settings_state.settings.settings_mut();
                    if let None = settings.backups_path {
                        settings.backups_path = Some(String::new());
                    }
                }
                // Cycle default start side
                SelectedToggle::Toggle4 => settings_state.settings.settings_mut().cycle_default_sides(true),
                // Cycle default end side
                SelectedToggle::Toggle5 => settings_state.settings.settings_mut().cycle_default_sides(false),
                // Toggle Modal Editing for Edit Mode
                SelectedToggle::Toggle6 => settings_state.settings.settings_mut().edit_modal = !settings_state.settings.settings().edit_modal,
                _ => {}
            }
        }

        // Cycle backup intervals
        KeyCode::Tab => {
            match settings_state.selected_toggle {
                SelectedToggle::Toggle2 => {
                    // If backups enabled and backups toggle is selected
                    if settings_state.settings.settings().backups_interval.is_some() {
                        settings_state.settings.settings_mut().cycle_backup_interval();
                        // Have to save or discard changes before exiting
                        settings_state.can_exit = false;
                    }
                }
                SelectedToggle::Toggle3 => {
                    // If backups enabled and runtime backups toggles is selected
                    if settings_state.settings.settings().runtime_backups_interval.is_some() {
                        settings_state.settings.settings_mut().cycle_runtime_backup_interval();
                        // Have to save or discard changes before exiting
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