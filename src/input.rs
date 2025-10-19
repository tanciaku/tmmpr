//! This module handles terminal events, focusing on keyboard input
//! to control the application's state and behavior.

use crate::app::{App, Mode};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

/// Reads the terminal events.
pub fn handle_events(app: &mut App) -> Result<()> {
    // Poll for an event with a timeout of 50ms. This is the main "tick" rate.
    if event::poll(std::time::Duration::from_millis(50))? {
        // Read the event and dispatch to the appropriate handler.
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => on_key_event(app, key), // Handle keyboard input
            Event::Mouse(_) => {}
            // Redraw the UI if terminal window resized
            Event::Resize(_, _) => { app.needs_clear_and_redraw = true; }
            _ => {}
        }
    }
    Ok(())
}

/// Handles all keyboard input events and updates the application state accordingly.
///
/// This function is the central hub for all user commands. Its behavior is
/// determined by the application's current `Mode`.
fn on_key_event(app: &mut App, key: KeyEvent) {
    match app.current_mode {
        // Normal mode is for navigation and high-level commands.
        Mode::Normal => {
            match key.code {
                // --- Application Commands ---
                KeyCode::Char('q') => app.quit(),

                // --- Viewport Navigation ---
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

                // --- Note Manipulation ---
                // Add note
                KeyCode::Char('a') => app.add_note(),
                // Select note (first selects closest to the center of the screen)
                KeyCode::Char('v') => {
                    app.select_note();
                    app.current_mode = Mode::Visual;
                }

                _ => {}
            }
            // Any action in Normal mode triggers a redraw.
            app.clear_and_redraw();
        }

        // Visual mode for selections.
        Mode::Visual => {
            match key.code {
                // Switch back to Normal Mode
                KeyCode::Esc => {
                    app.current_mode = Mode::Normal;
                    if let Some(note) = app.notes.get_mut(&app.selected_note) {
                        note.selected = false;
                    }
                }
                KeyCode::Char('i') => app.current_mode = Mode::Insert,
                _ => {}
            }
            // Any action in Visual mode triggers a redraw.
            app.clear_and_redraw(); 
        }

        // Insert mode is for editing the content of a note.
        Mode::Insert => {
            match key.code {
                // Switch back to Normal Mode
                KeyCode::Esc => {
                    app.current_mode = Mode::Normal;
                    if let Some(note) = app.notes.get_mut(&app.selected_note) {
                        note.selected = false;
                        // Reset cursor position for the next time entering Insert mode.
                        app.cursor_pos = 0;
                    }
                }

                // --- Text Editing ---
                KeyCode::Char(c) => {
                    if let Some(note) = app.notes.get_mut(&app.selected_note) {
                        // Insert the typed character at the cursor's current position.
                        note.content.insert(app.cursor_pos, c);
                        // Move the cursor forward one position.
                        app.cursor_pos += 1;
                    }
                }
                KeyCode::Enter => {
                    if let Some(note) = app.notes.get_mut(&app.selected_note) {
                        // Insert a newline character at the cursor's position.
                        note.content.insert(app.cursor_pos, '\n');
                        // Move the cursor forward one position.
                        app.cursor_pos += 1;
                    }
                }
                KeyCode::Backspace => {
                    if let Some(note) = app.notes.get_mut(&app.selected_note) {
                        // We can only backspace if the cursor is not at the very beginning of the text.
                        if app.cursor_pos > 0 {
                            // To delete the character *before* the cursor, we must remove the character
                            // at the index `cursor_pos - 1`.
                            note.content.remove(app.cursor_pos - 1);
                            // After removing the character, we move the cursor's position back by one.
                            app.cursor_pos -= 1;
                        }
                    }
                }
                _ => {}
            }
            // Any action in Insert mode triggers a redraw.
            app.clear_and_redraw();
        }
    }
}