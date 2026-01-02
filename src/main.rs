use color_eyre::Result;
use ratatui::DefaultTerminal;

mod app;
mod input;
mod states;
mod ui;
mod utils;
mod serialization;
use crate::{
    app::{App, Screen},
    input::{AppAction, handle_events},
    ui::{render_map, render_settings, render_start}, 
    utils::{save_map_file, handle_runtime_backup},
};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut app = App::new();
    let result = run(terminal, &mut app);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, app: &mut App) -> Result<()> {
    // Main application loop
    while app.running {
        // Extract any actions before the match to avoid borrow conflicts
        let action = match &mut app.screen {
            Screen::Start(start_state) => {
                // Clear and redraw the screen if need to
                if start_state.needs_clear_and_redraw {
                    terminal.draw(|frame| render_start(frame, start_state))?;
                    start_state.needs_clear_and_redraw = false;
                }

                None // No save action for start screen
            }
            Screen::Settings(settings_state) => {
                // Clear and redraw the screen if need to
                if settings_state.needs_clear_and_redraw {
                    terminal.draw(|frame| render_settings(frame, settings_state))?;
                    settings_state.needs_clear_and_redraw = false;
                }

                None // No save action for settings screen
            }
            Screen::Map(map_state) => { 
                // Clear and redraw the screen if need to
                if map_state.needs_clear_and_redraw {
                    terminal.draw(|frame| render_map(frame, map_state))?;
                    map_state.needs_clear_and_redraw = false;
                }
                
                // Return the action to handle outside the match
                Some(map_state.on_tick_save_changes())
            }
        };
         
        // Handle the action outside the match (no borrow conflicts)
        if let Some(action) = action {
            match action {
                AppAction::SaveMapFile(path) => save_map_file(app, &path, false, false),
                AppAction::MakeRTBackupFile => handle_runtime_backup(app),
                AppAction::Continue => {}
                _ => {} // .on_tick_save_changes() can only return the three above
            }
        }

        // Read terminal events
        handle_events(app)?;
    }

    Ok(())
}