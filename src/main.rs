use color_eyre::Result;
use ratatui::DefaultTerminal;

mod app;
mod input;
mod ui;
mod utils;
mod serialization;
use crate::{
    app::{App, Screen},
    input::handle_events,
    ui::{render_map},
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
    //app.setup()?;

    // Main application loop
    while app.running {

        match &mut app.screen {
            Screen::Map(map_state) => {
                if map_state.needs_clear_and_redraw {
                    terminal.draw(|frame| render_map(frame, map_state))?;
                    map_state.needs_clear_and_redraw = false;
                }
            }
            _ => {},
        }

        // Read terminal events
        handle_events(app)?;
    }

    //app.exit();

    Ok(())
}