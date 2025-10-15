//! This module is responsible for all rendering logic of the application.
//! It takes the application state (`App`) and a `ratatui` frame, and draws the UI.

use crate::app::{App, Mode, SignedRect};
use ratatui::{
    prelude::Rect,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// The main rendering entry point.
///
/// This function orchestrates the entire rendering process for a single frame.
/// It updates the app state with the current screen dimensions and then calls
/// the specific rendering functions for different parts of the UI.
pub fn render(frame: &mut Frame, app: &mut App) {

    // ...?

    // Update the app state with the current terminal size. This is crucial for
    // calculations that depend on screen dimensions, like centering new notes.
    app.screen_width = frame.area().width as usize;
    app.screen_height = frame.area().height as usize;

    // Render the main UI components.
    render_bar(frame, app);
    render_map(frame, app);
}

/// Renders the top information bar.
///
/// This bar displays debugging information and the current application state,
/// such as viewport position, mode, and selected note.
fn render_bar(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Determine the text to display for the current mode.
    let mode_text_to_display = match &app.current_mode {
        Mode::Normal => String::from("Normal"),
        Mode::Visual => String::from("Visual"),
        Mode::Insert => String::from("Insert"),
    };

    // Create the paragraph widget with formatted info.
    let size_display = Paragraph::new(format!(
        "Width: {}, Height: {}     x: {}  y: {}       Mode: {}            Selected note: {}",
        size.width,
        size.height,
        app.view_pos.x,
        app.view_pos.y,
        mode_text_to_display,
        app.selected_note
    ))
    .block(Block::default().borders(Borders::ALL).title("Terminal Info"));

    // Define the area for the top bar (first 3 rows).
    let top_rect = Rect {
        x: size.x,
        y: size.y,
        width: size.width,
        height: 3,
    };

    frame.render_widget(size_display, top_rect);
}

/// Renders the main canvas where notes are displayed.
///
/// This function iterates through all notes and performs a series of calculations
/// to determine if, where, and how each note should be rendered.
fn render_map(frame: &mut Frame, app: &App) {
    for (_id, note) in app.notes.iter() {
        // --- 1. Get Note Dimensions ---
        // Enforce a minimum size for readability.
        let (mut note_width, mut note_height) = note.get_dimensions();
        if note_width < 20 { note_width = 20; }
        if note_height < 4 { note_height = 4; }

        // --- 2. Translate to Screen Coordinates ---
        // Convert the note's absolute canvas coordinates into screen-relative coordinates.
        // This can result in negative values if the note is partially off-screen.
        let note_rect = SignedRect {
            x: note.x as isize - app.view_pos.x as isize,
            y: note.y as isize - app.view_pos.y as isize,
            width: note_width as isize,
            height: note_height as isize,
        };
        
        // The frame itself is a rectangle starting at (0,0) in screen space.
        let frame_rect = SignedRect {
            x: 0,
            y: 0,
            width: frame.area().width as isize,
            height: frame.area().height as isize,
        };

        // --- 3. Clipping ---
        // Calculate the intersection between the note and the frame. If there's no
        // overlap, `intersection_result` will be `None`, and drawing the note will be skipped.
        let intersection_result = note_rect.intersection(&frame_rect);

        if let Some(visible_part) = intersection_result {
            // Convert the clipped, screen-space rectangle back to a `ratatui::Rect`.
            // The coordinates are guaranteed to be non-negative at this point.
            let note_area = Rect::new(
                visible_part.x as u16,
                visible_part.y as u16,
                visible_part.width as u16,
                visible_part.height as u16,
            );

            // --- 4. Calculate Text Scrolling ---
            // If the note was clipped on the top or left, we need to scroll the text content
            // to show the correct portion.
            let horizontal_scroll = (visible_part.x - note_rect.x) as u16;
            let vertical_scroll = (visible_part.y - note_rect.y) as u16;

            // --- 5. Determine Dynamic Borders ---
            // Show only the borders for the sides of the note that are not clipped.
            let mut borders = Borders::NONE;
            if note_rect.x == visible_part.x {
                borders |= Borders::LEFT;
            }
            if note_rect.x + note_rect.width == visible_part.x + visible_part.width {
                borders |= Borders::RIGHT;
            }
            if note_rect.y == visible_part.y {
                borders |= Borders::TOP;
            }
            if note_rect.y + note_rect.height == visible_part.y + visible_part.height {
                borders |= Borders::BOTTOM;
            }

            // --- 6. Render the Widget ---
            let text_widget = Paragraph::new(note.content.as_str())
                .scroll((vertical_scroll, horizontal_scroll))
                .block(Block::default().borders(borders));

            frame.render_widget(text_widget, note_area);
        }
    }
}

/// Clears the entire frame before a redraw.
///
/// This is used to prevent artifacts from previous frames when the UI changes.
pub fn clear(f: &mut Frame) {
    f.render_widget(Clear, f.area());
}