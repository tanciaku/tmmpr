use crate::app::{App, Mode};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Reads the terminal events.
pub fn handle_events(app: &mut App) -> Result<()> {
    // Only wait for keyboard events for 50ms - otherwise continue the loop iteration
    if event::poll(std::time::Duration::from_millis(50))? {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => on_key_event(app, key), // Handle keyboard input
            Event::Mouse(_) => {}
            Event::Resize(_, _) => { app.needs_redraw = true; } // Re-render if terminal window resized
            _ => {}
        }
    }
    Ok(())
}

/// Handles keyboard input.
fn on_key_event(app: &mut App, key: KeyEvent) {
    match app.current_mode {
        Mode::Normal => {
            match key.code {
                // Exit the app
                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => app.quit(),
                // Move left
                KeyCode::Char('h') => app.view_pos.x = app.view_pos.x.saturating_sub(1),
                KeyCode::Char('H') => app.view_pos.x = app.view_pos.x.saturating_sub(5),
                // Move down
                KeyCode::Char('j') => app.view_pos.y += 1,
                KeyCode::Char('J') => app.view_pos.y += 5,
                // Move up
                KeyCode::Char('k') => app.view_pos.y = app.view_pos.y.saturating_sub(1),
                KeyCode::Char('K') => app.view_pos.y = app.view_pos.y.saturating_sub(5),
                // Move right
                KeyCode::Char('l') => app.view_pos.x += 1,
                KeyCode::Char('L') => app.view_pos.x += 5,
                _ => {}
            }

            app.clear_and_redraw();
        }
        Mode::Visual => {

        }
        Mode::Insert => {

        }
    }
}