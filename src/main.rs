use std::io::stdout;

use color_eyre::Result;
use crossterm::{cursor::SetCursorStyle, execute};
use ratatui::DefaultTerminal;
use tmmpr::{
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

/// Main event loop using on-demand rendering to reduce CPU usage.
/// Each screen state tracks whether it needs redrawing instead of rendering every frame.
fn run(mut terminal: DefaultTerminal, app: &mut App) -> Result<()> {
    let _ = execute!(stdout(), SetCursorStyle::SteadyBar);
    
    while app.running {
        match &mut app.screen {
            Screen::Start(start_state) => {
                if start_state.needs_clear_and_redraw {
                    terminal.draw(|frame| render_start(frame, start_state))?;
                    start_state.needs_clear_and_redraw = false;
                }
            }
            Screen::Settings(settings_state) => {
                if settings_state.needs_clear_and_redraw {
                    terminal.draw(|frame| render_settings(frame, settings_state))?;
                    settings_state.needs_clear_and_redraw = false;
                }
            }
            Screen::Map(map_state) => { 
                // Periodic auto-save and backup creation (respects user settings)
                map_state.auto_save_if_needed();
                map_state.auto_backup_if_needed();

                if map_state.ui_state.needs_clear_and_redraw {
                    terminal.draw(|frame| render_map(frame, map_state))?;
                    map_state.ui_state.mark_redrawn();
                }
            }
        };
         
        handle_events(app)?;
    }

    Ok(())
}