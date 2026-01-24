//! Start screen input handling

use crate::{input::AppAction, states::{StartState, start::{FocusedInputBox, SelectedStartButton}}, utils::FileSystem};
use crossterm::event::{KeyCode, KeyEvent};


/// Key handling for the Start Screen
pub fn start_kh(start_state: &mut StartState, key: KeyEvent, fs: &impl FileSystem) -> AppAction {
    // Take all input if in the Input Menu screen
    // (Entering a path for the map file)
    if start_state.input_path {

        // Keys independent of which input box is in focus
        match key.code {
            KeyCode::Esc => {
                start_state.input_path = false;
                start_state.focused_input_box = FocusedInputBox::InputBox1; // if already isn't
                start_state.input_path_string = None; // reset input fields
                start_state.input_path_name = None; // reset input fields
            }
            _ => {}
        }

        // Which input box is in focus?
        match start_state.focused_input_box {
            FocusedInputBox::InputBox1 => {
                if let Some(path) = &mut start_state.input_path_string {
                    match key.code {
                        KeyCode::Char(c) => {
                            if path.len() < 46 {
                                path.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if path.len() > 0 {
                                path.pop();
                            }
                        }
                        KeyCode::Enter => {
                            start_state.focused_input_box = FocusedInputBox::InputBox2;
                        }
                        _ => {}
                    }
                }
            }
            FocusedInputBox::InputBox2 => {
                if let Some(map_name) = &mut start_state.input_path_name {
                    match key.code {
                        KeyCode::Char(c) => {
                            if map_name.len() < 46 {
                                map_name.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if map_name.len() > 0 {
                                map_name.pop();
                            }
                        }
                        KeyCode::Enter => {
                            start_state.clear_and_redraw();
                            return start_state.submit_path_with_fs(None, fs)
                        }
                        _ => {}
                    }
                }
            }
        }

        start_state.clear_and_redraw();
        return AppAction::Continue
    }

    // If in the start menu
    match key.code {

        KeyCode::Char('q') => return AppAction::Quit,

        KeyCode::Char('k') => start_state.navigate_start_buttons("k"),
        KeyCode::Up => start_state.navigate_start_buttons("Up"),

        KeyCode::Char('j') => start_state.navigate_start_buttons("j"),
        KeyCode::Down => start_state.navigate_start_buttons("Down"),

        KeyCode::Enter => {
            match start_state.selected_button {
                SelectedStartButton::CreateSelect => {
                    start_state.input_path = true;
                    start_state.display_err_msg = None; // if already isn't
                    start_state.input_path_string = Some(String::new());
                    start_state.input_path_name = Some(String::new());
                }
                _ => {}
            }
        }

        _ => {}
    }

    // If able to use the "recent paths" functionality (no errors)
    if let Ok(recent_paths) = &start_state.recent_paths {
        match key.code {
            KeyCode::Enter => {
                match start_state.selected_button {
                    SelectedStartButton::Recent1 => {
                        if let Some(path) = &recent_paths.recent_path_1 {
                            return start_state.submit_path_with_fs(Some(path.to_path_buf()), fs)
                        }
                    }
                    SelectedStartButton::Recent2 => {
                        if let Some(path) = &recent_paths.recent_path_2 {
                            return start_state.submit_path_with_fs(Some(path.to_path_buf()), fs)
                        }
                    }
                    SelectedStartButton::Recent3 => {
                        if let Some(path) = &recent_paths.recent_path_3 {
                            return start_state.submit_path_with_fs(Some(path.to_path_buf()), fs)
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    
    start_state.clear_and_redraw();
    AppAction::Continue
}
