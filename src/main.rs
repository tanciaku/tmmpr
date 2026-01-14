mod app;
mod input;
mod ui;
mod states;
mod utils;

use color_eyre::Result;
use ratatui::DefaultTerminal;
use crate::{
    app::{App, Screen},
    input::handle_events,
    ui::{render_map, render_settings, render_start} 
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
        // Which screen is app in?
        match &mut app.screen {
            Screen::Start(start_state) => {
                // Clear and redraw the screen if need to
                if start_state.needs_clear_and_redraw {
                    terminal.draw(|frame| render_start(frame, start_state))?;
                    start_state.needs_clear_and_redraw = false;
                }
            }
            Screen::Settings(settings_state) => {
                // Clear and redraw the screen if need to
                if settings_state.needs_clear_and_redraw {
                    terminal.draw(|frame| render_settings(frame, settings_state))?;
                    settings_state.needs_clear_and_redraw = false;
                }
            }
            Screen::Map(map_state) => { 
                // (If enabled)
                // Saving changes to map file, Making runtime backup files
                map_state.on_tick_save_changes();

                // Clear and redraw the screen if need to
                if map_state.needs_clear_and_redraw {
                    terminal.draw(|frame| render_map(frame, map_state))?;
                    map_state.needs_clear_and_redraw = false;
                }
            }
        };
         
        // Read terminal events
        handle_events(app)?;
    }

    Ok(())
}