use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{app::Screen, input::{AppAction, map::{help_next_page, help_previous_page, move_viewport}}, states::{MapState, SettingsState, StartState, map::{DiscardMenuType, Mode}}};


/// Key handling for Normal Mode in the Map Screen
pub fn map_normal_kh(map_state: &mut MapState, key: KeyEvent) -> AppAction {
    // Showing help page (takes all input if toggled)
    if map_state.ui_state.is_help_visible() {
        match key.code {
            // F1, ? - toggle the help page
            KeyCode::F(1) | KeyCode::Char('?') | KeyCode::Esc => map_state.ui_state.hide_help(),

            // Right, l, Tab - go forward a page in the help screen.
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => help_next_page(map_state),

            // Left, h - go back a page in the help screen.
            KeyCode::Left | KeyCode::Char('h') => help_previous_page(map_state),

            _ => {}
        }

        map_state.clear_and_redraw(); 

        return AppAction::Continue // Stop here
    }
    
    // Confirm discard unsaved changes menu (takes all input if triggered)
    if let Some(discard_menu_type) = &map_state.ui_state.confirm_discard_menu {
        match key.code {
            // Cancel
            KeyCode::Esc => {
                map_state.ui_state.hide_discard_menu();
                map_state.clear_and_redraw();
            }
            // Confirm exiting and discarding unsaved changes
            KeyCode::Char('q') => {
                match discard_menu_type {
                    DiscardMenuType::Start => {
                        return AppAction::Switch(Screen::Start(StartState::new()))
                    }
                    DiscardMenuType::Settings => {
                        return AppAction::Switch(
                            Screen::Settings(SettingsState::new(
                                // Pass in the file path that was opened to return to it after closing settings
                                map_state.persistence.file_write_path.clone())))
                    }
                }
            }
            _ => {}
        }

        return AppAction::Continue // Stop here
    }
    
    // --- Map Screen Normal Mode Commands ---
    match key.code {

        // Exiting the app
        KeyCode::Char('q') => {
            // Can exit to start screen if saved the changes
            if map_state.persistence.can_exit {
                return AppAction::Switch(Screen::Start(StartState::new()))
            } else { // Otherwise show the confirmation to discard unsaved changes menu
                map_state.ui_state.show_discard_menu(DiscardMenuType::Start);
                map_state.clear_and_redraw();
            }
        }

        // F1, ? - toggle the help page
        KeyCode::F(1) | KeyCode::Char('?') => map_state.ui_state.show_help(1),

        // Save the map file
        KeyCode::Char('s') => return AppAction::SaveMapFile(map_state.persistence.file_write_path.clone()),

        // Open the settings
        KeyCode::Char('o') => {
            // Can exit to settings if saved the changes
            if map_state.persistence.can_exit {
                return AppAction::Switch(
                    Screen::Settings(SettingsState::new(
                        // Pass in the file path that was opened to return to it after closing settings
                        map_state.persistence.file_write_path.clone())))
            } else { // Otherwise show the confirmation to discard unsaved changes menu
                map_state.ui_state.show_discard_menu(DiscardMenuType::Settings);
                map_state.clear_and_redraw();
            }
        }

        // --- Viewport Navigation ---

        // Move left by 1   (h, Left)
        KeyCode::Char('h') => move_viewport(map_state, "x", -1),
        KeyCode::Left if key.modifiers == KeyModifiers::NONE => move_viewport(map_state, "x", -1),
        // Move left by 5   (H, Shift + Left)
        KeyCode::Char('H') => move_viewport(map_state, "x", -5),
        KeyCode::Left if key.modifiers == KeyModifiers::SHIFT => move_viewport(map_state, "x", -5),

        // Move down by 1   (j, Down)
        KeyCode::Char('j') => move_viewport(map_state, "y", 1),
        KeyCode::Down if key.modifiers == KeyModifiers::NONE => move_viewport(map_state, "y", 1),
        // Move down by 5   (J, Shift + Down)
        KeyCode::Char('J') => move_viewport(map_state, "y", 5),
        KeyCode::Down if key.modifiers == KeyModifiers::SHIFT => move_viewport(map_state, "y", 5),

        // Move up by 1   (k, Up)
        KeyCode::Char('k') => move_viewport(map_state, "y", -1),
        KeyCode::Up if key.modifiers == KeyModifiers::NONE => move_viewport(map_state, "y", -1),
        // Move up by 5   (K, Shift + Up)
        KeyCode::Char('K') => move_viewport(map_state, "y", -5),
        KeyCode::Up if key.modifiers == KeyModifiers::SHIFT => move_viewport(map_state, "y", -5),

        // Move right by 1   (l, Right)
        KeyCode::Char('l') => move_viewport(map_state, "x", 1),
        KeyCode::Right if key.modifiers == KeyModifiers::NONE => move_viewport(map_state, "x", 1),
        // Move right by 5   (L, Shift + Right)
        KeyCode::Char('L') => move_viewport(map_state, "x", 5),
        KeyCode::Right if key.modifiers == KeyModifiers::SHIFT => move_viewport(map_state, "x", 5),


        // --- Note Manipulation ---

        // Add a note
        KeyCode::Char('a') => map_state.add_note(),
        // Select note (selects the closest one to the center of the screen)
        KeyCode::Char('v') => {
            map_state.select_note();
            map_state.current_mode = Mode::Visual;
        }
    
        _ => {}
    }

    // Any action in Normal mode triggers a redraw.
    map_state.clear_and_redraw();

    AppAction::Continue
}
