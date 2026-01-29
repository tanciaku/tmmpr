use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{app::Screen, input::{AppAction, map::{help_next_page, help_previous_page, move_viewport}}, states::{MapState, SettingsState, StartState, map::DiscardMenuType}, utils::FileSystem};


/// Handles keyboard input for Normal Mode in the Map Screen.
/// 
/// Help and discard confirmation menus take priority and intercept all input when shown.
pub fn map_normal_kh(map_state: &mut MapState, key: KeyEvent, fs: &dyn FileSystem) -> AppAction {
    // Help menu intercepts all input when visible
    if map_state.ui_state.is_help_visible() {
        match key.code {
            KeyCode::F(1) | KeyCode::Char('?') | KeyCode::Esc => map_state.ui_state.hide_help(),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => help_next_page(map_state),
            KeyCode::Left | KeyCode::Char('h') => help_previous_page(map_state),
            _ => {}
        }

        map_state.clear_and_redraw(); 

        return AppAction::Continue
    }
    
    // Discard confirmation menu intercepts all input when triggered
    if let Some(discard_menu_type) = &map_state.ui_state.confirm_discard_menu {
        match key.code {
            KeyCode::Esc => {
                map_state.ui_state.hide_discard_menu();
                map_state.clear_and_redraw();
            }
            KeyCode::Char('q') => {
                match discard_menu_type {
                    DiscardMenuType::Start => {
                        return AppAction::Switch(Screen::Start(StartState::new_with_fs(fs)))
                    }
                    DiscardMenuType::Settings => {
                        // Preserve file path to return to after closing settings
                        return AppAction::Switch(
                            Screen::Settings(SettingsState::new_with_fs(map_state.persistence.file_write_path.clone(), fs)))
                    }
                }
            }
            _ => {}
        }

        return AppAction::Continue
    }
    
    match key.code {
        KeyCode::Char('q') => {
            // Require saving or explicit confirmation before exiting
            if map_state.persistence.can_exit {
                return AppAction::Switch(Screen::Start(StartState::new_with_fs(fs)))
            } else {
                map_state.ui_state.show_discard_menu(DiscardMenuType::Start);
                map_state.clear_and_redraw();
            }
        }

        KeyCode::F(1) | KeyCode::Char('?') => map_state.ui_state.show_help(1),

        KeyCode::Char('s') => return AppAction::SaveMapFile(map_state.persistence.file_write_path.clone()),

        KeyCode::Char('o') => {
            // Require saving or explicit confirmation before opening settings
            if map_state.persistence.can_exit {
                // Preserve file path to return to after closing settings
                return AppAction::Switch(
                    Screen::Settings(SettingsState::new_with_fs(map_state.persistence.file_write_path.clone(), fs)))
            } else {
                map_state.ui_state.show_discard_menu(DiscardMenuType::Settings);
                map_state.clear_and_redraw();
            }
        }

        // Vim-style hjkl navigation with arrow key alternatives
        // Shifted versions move 5x faster for quicker navigation
        KeyCode::Char('h') => move_viewport(map_state, "x", -1),
        KeyCode::Left if key.modifiers == KeyModifiers::NONE => move_viewport(map_state, "x", -1),
        KeyCode::Char('H') => move_viewport(map_state, "x", -5),
        KeyCode::Left if key.modifiers == KeyModifiers::SHIFT => move_viewport(map_state, "x", -5),

        KeyCode::Char('j') => move_viewport(map_state, "y", 1),
        KeyCode::Down if key.modifiers == KeyModifiers::NONE => move_viewport(map_state, "y", 1),
        KeyCode::Char('J') => move_viewport(map_state, "y", 5),
        KeyCode::Down if key.modifiers == KeyModifiers::SHIFT => move_viewport(map_state, "y", 5),

        KeyCode::Char('k') => move_viewport(map_state, "y", -1),
        KeyCode::Up if key.modifiers == KeyModifiers::NONE => move_viewport(map_state, "y", -1),
        KeyCode::Char('K') => move_viewport(map_state, "y", -5),
        KeyCode::Up if key.modifiers == KeyModifiers::SHIFT => move_viewport(map_state, "y", -5),

        KeyCode::Char('l') => move_viewport(map_state, "x", 1),
        KeyCode::Right if key.modifiers == KeyModifiers::NONE => move_viewport(map_state, "x", 1),
        KeyCode::Char('L') => move_viewport(map_state, "x", 5),
        KeyCode::Right if key.modifiers == KeyModifiers::SHIFT => move_viewport(map_state, "x", 5),

        KeyCode::Char('a') => map_state.add_note(),
        // Selects the note closest to viewport center
        KeyCode::Char('v') => map_state.select_note(),
    
        _ => {}
    }

    // Redraw ensures UI reflects any state changes from input
    map_state.clear_and_redraw();

    AppAction::Continue
}
