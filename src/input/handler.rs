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

/// Main event loop handler that polls terminal events and dispatches them to screen-specific handlers.
///
/// This function is intentionally not tested because:
/// 1. It's a thin orchestrator over fully-tested components
/// 2. All called functions are tested individually (start_kh, settings_kh, map_kh, etc.)
/// 3. The integration points for crossterm are better tested via manual testing
/// 4. Adding mocks would add complexity without significant value
pub fn handle_events(app: &mut App) -> Result<()> {
    // 50ms timeout balances responsiveness with CPU usage
    if event::poll(std::time::Duration::from_millis(50))? {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
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
                        // SaveMapFile can only be triggered from map screen, so this is guaranteed to succeed
                        if let Screen::Map(map_state) = &mut app.screen {
                            save_map_file(map_state, &path, true, false);
                        }
                    }
                    AppAction::LoadMapFile(path) => load_map_file(app, &path),
                }
            }

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

/// Dispatches key events to mode-specific handlers in the map screen.
pub fn map_kh(map_state: &mut MapState, key: KeyEvent) -> AppAction { 
    match &map_state.current_mode {
        Mode::Normal => map_normal_kh(map_state, key, &RealFileSystem),
        Mode::VisualSelect | Mode::VisualMove | Mode::VisualConnectAdd | Mode::VisualConnectEdit => map_visual_kh(map_state, key),
        Mode::Edit(modal) => map_edit_kh(map_state, key, *modal),
        // Delete mode requires user confirmation before actually deleting
        Mode::Delete => map_delete_kh(map_state, key),
    }
}
