use color_eyre::Result;
use ratatui::DefaultTerminal;

mod app;
mod input;
mod ui;
mod utils;
use crate::{
    app::App,
    input::handle_events,
    ui::{render},
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
        // Draw/Redraw the ui
        if app.needs_clear_and_redraw {
            terminal.draw(|frame| render(frame, app))?;
            app.needs_clear_and_redraw = false;
        }

        // Read terminal events
        handle_events(app)?;
    }

    Ok(())
}