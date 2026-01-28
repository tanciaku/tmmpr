//! This module handles terminal events, focusing on keyboard input
//! to control the application's state and behavior.

use crate::{
    app::{App, Screen},
    input::{map::{map_delete_kh, map_edit_kh, map_normal_kh, map_visual_kh}, settings_kh, start_kh},
    states::{MapState, map::Mode},
    utils::{RealFileSystem, create_map_file, load_map_file, save_map_file},
};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use std::path::PathBuf;

#[derive(PartialEq, Debug)]
pub enum AppAction {
    Continue,
    Quit,
    Switch(Screen),
    CreateMapFile(PathBuf),
    SaveMapFile(PathBuf),
    LoadMapFile(PathBuf),
}

/// Reads the terminal events.
///
/// This function is intentionally not tested because:
/// 1. It's a thin orchestrator over fully-tested components
/// 2. All called functions are tested individually (start_kh, settings_kh, map_kh, etc.)
/// 3. The integration points for crossterm are better tested via manual testing
/// 4. Adding mocks would add complexity without significant value
pub fn handle_events(app: &mut App) -> Result<()> {
    // Poll for an event with a timeout of 50ms. This is the main "tick" rate.
    if event::poll(std::time::Duration::from_millis(50))? {
        // Read the event
        match event::read()? {
            // Handle keyboard input
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                // Dispatch it to the appropriate handler.
                let app_action = match &mut app.screen {
                    Screen::Start(start_state) => start_kh(start_state, key, &RealFileSystem),
                    Screen::Settings(settings_state) => settings_kh(settings_state, key, &RealFileSystem),
                    Screen::Map(map_state) => map_kh(map_state, key),
                };

                match app_action {
                    AppAction::Continue => {}
                    AppAction::Quit => app.quit(),
                    AppAction::Switch(screen) => app.screen = screen,
                    AppAction::CreateMapFile(path) => create_map_file(app, &path),
                    AppAction::SaveMapFile(path) => {
                        // This match arm can only be reached from user input in map screen
                        if let Screen::Map(map_state) = &mut app.screen { // get the map state - guaranteed.
                            save_map_file(map_state, &path, true, false);
                        }
                    }
                    AppAction::LoadMapFile(path) => load_map_file(app, &path),
                }
            }

            // Redraw the UI if terminal window resized
            Event::Resize(_, _) => {
                match &mut app.screen {
                    Screen::Start(start_state) => start_state.needs_clear_and_redraw = true,
                    Screen::Settings(settings_state) => settings_state.needs_clear_and_redraw = true,
                    Screen::Map(map_state) => map_state.clear_and_redraw(),
                }
            }

            _ => {}
        }
    }
    Ok(())
}

/// Key handling depending on Map Screen Mode
pub fn map_kh(map_state: &mut MapState, key: KeyEvent) -> AppAction { 
    match &map_state.current_mode {
        // Normal mode is for navigation and high-level commands.
        Mode::Normal => map_normal_kh(map_state, key),

        // Visual mode for selections.
        Mode::Visual => map_visual_kh(map_state, key),

        // Edit mode is for editing the content of a note.
        Mode::Edit(modal) => map_edit_kh(map_state, key, *modal),
    
        // Delete mode is a confirmation to delete a note
        Mode::Delete => map_delete_kh(map_state, key),
    }
}
