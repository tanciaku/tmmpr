//! This module is responsible for all rendering logic of the application.
//! It takes the application state (`App`) and a `ratatui` frame, and draws the UI.

use crate::app::{App, Mode, SignedRect};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Position},
    prelude::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Padding, Paragraph},
    Frame
};

/// The main rendering entry point.
///
/// This function orchestrates the entire rendering process for a single frame.
/// It updates the app state with the current screen dimensions and then calls
/// the specific rendering functions for different parts of the UI.
pub fn render(frame: &mut Frame, app: &mut App) {
    // Clear the frame before drawing anything new.
    frame.render_widget(Clear, frame.area());

    // Update the app state with the current terminal size. This is crucial for
    // calculations that depend on screen dimensions, like centering new notes.
    app.screen_width = frame.area().width as usize;
    app.screen_height = frame.area().height as usize;

    // Render the main UI components.
    render_map(frame, app);
    render_bar(frame, app);
}

/// Renders the bottom information bar.
///
/// This bar displays debugging information and the current application state,
/// such as viewport position, mode, and selected note.
fn render_bar(frame: &mut Frame, app: &App) {

    // Get the total available screen area.
    let size = frame.area();

    // Determine the display text and color for the current application mode.
    let (mode_text, mode_text_color) = match &app.current_mode {
        Mode::Normal => (String::from("NORMAL"), Style::new().fg(Color::White)),
        Mode::Visual => (String::from("VISUAL"), Style::new().fg(Color::Yellow)),
        Mode::Insert => (String::from("INSERT"), Style::new().fg(Color::Blue)),
    };

    // --- Left-Aligned Widget: Mode Display ---
    // Create a Paragraph for the mode, styling it with the color determined above.
    // It's aligned to the left and given some padding.
    let mode_display = Paragraph::new(format!("\n[ {} ]", mode_text))
        .style(mode_text_color)
        .alignment(Alignment::Left)
        .block(Block::default().padding(Padding::new(2, 0, 0, 0)));

    // --- Right-Aligned Widget: View Position ---
    // Create a separate Paragraph to show the viewport's x/y coordinates.
    // This is aligned to the right, with padding on the right side.
    let view_position_display = Paragraph::new(format!(
        "\nView: {},{}",
        app.view_pos.x,
        app.view_pos.y,
    ))
    .alignment(Alignment::Right)
    .block(Block::default().padding(Padding::new(0, 2, 0, 0)));
    
    // Define the rectangular area for the entire bottom bar.
    let bottom_rect = Rect {
        x: size.x,
        y: size.height - 2, // Position it in the last two rows of the terminal.
        width: size.width,
        height: 2,
    };

    // --- Layout Management ---
    // Split the `bottom_rect` into two equal horizontal chunks.
    // This prevents the styling of one widget from affecting the other.
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left side gets 50% of the width.
            Constraint::Percentage(50), // Right side gets the other 50%.
        ])
        .split(bottom_rect);
    
    // Assign the created chunks to variables for clarity.
    let left_bar = chunks[0];
    let right_bar = chunks[1];

    // --- Rendering ---
    // Finally, render the widgets to the frame.
    frame.render_widget(Clear, bottom_rect); // First, clear the entire bar area.
    frame.render_widget(mode_display, left_bar); // Render the mode display to the left chunk.
    frame.render_widget(view_position_display, right_bar); // Render the position to the right chunk.
}

/// Renders the main canvas where notes are displayed.
///
/// This function iterates through all notes and performs a series of calculations
/// to determine if, where, and how each note should be rendered.
fn render_map(frame: &mut Frame, app: &App) {
    // To ensure consistent rendering and prevent flickering, we must draw notes
    // in a stable order. A HashMap's iterator is not guaranteed to be stable,
    // so we collect the notes into a Vec and sort them by their ID (the key).
    // This acts as a stable z-index, where higher IDs are drawn on top.
    let mut sorted_notes: Vec<_> = app.notes.iter().collect();
    sorted_notes.sort_by_key(|(id, _note)| **id);

    for (id, note) in sorted_notes {
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

            // --- 6. Determine border color  ---
            // (based on selection and mode)
            let border_color = if note.selected {
                match app.current_mode {
                    Mode::Normal => Color::White,
                    Mode::Visual => Color::Yellow,
                    Mode::Insert => Color::Blue,
                }
            } else {
                Color::White
            };

            // Create a block for the note with borders.
            let block = Block::default()
                .borders(borders)
                .border_style(border_color);

            // --- 7. Create the widget itself ---
            let text_widget = Paragraph::new(note.content.as_str())
                .scroll((vertical_scroll, horizontal_scroll))
                .block(block);

            // --- 8. Render the Widget(s) ---
            // With a stable rendering order, we can now clear any content 
            // from notes rendered beneath the note to be drawn and then draw it.
            frame.render_widget(Clear, note_area);
            frame.render_widget(text_widget, note_area);

            // -- 9. Render the cursor if in Insert Mode on the selected note ---
            // This logic only runs if the app is in Insert mode AND the note currently being
            // drawn is the one that's actively selected.
            if matches!(app.current_mode, Mode::Insert) && *id == app.selected_note {

                // To calculate the cursor's position, we first need a slice of the text
                // from the beginning of the note's content up to the cursor's byte index.
                let text_before_cursor = &note.content[..app.cursor_pos];

                // --- Calculate cursor's position RELATIVE to the text inside the note ---

                // The Y position (row) is the number of newline characters before the cursor.
                let cursor_y_relative = text_before_cursor.matches('\n').count();

                // The X position (column) is the number of characters since the last newline.
                let cursor_x_relative = match text_before_cursor.rfind('\n') {
                    // If a newline is found, the X position is the number of characters
                    // between that newline and the cursor. `c+1` to skip the newline itself.
                    Some(c) => {
                        text_before_cursor[c+1..app.cursor_pos].chars().count()
                    }
                    // If no newline is found, we're on the first line. The X position is
                    // simply the total number of characters before the cursor.
                    None => { 
                        text_before_cursor[0..app.cursor_pos].chars().count()
                    }
                };
                
                // --- Check if the calculated position is VISIBLE on screen ---
                // The cursor is only visible if its calculated row is within the scrolled view.
                // `cursor_y_relative` must be at or after the `vertical_scroll` offset.
                // It must also be within the visible height of the note area.
                // `note_area.height - 2` accounts for the top and bottom borders.
                if cursor_y_relative >= vertical_scroll as usize 
                   && cursor_y_relative <= (note_area.height - 2) as usize {

                    // --- Translate relative coordinates to absolute screen coordinates ---
                    let final_cursor_x = note_area.x as usize // Start at the note's visible edge
                        + 1                                   // Add 1 for the left border
                        + cursor_x_relative                   // Add the cursor's column in the text
                        - horizontal_scroll as usize;         // Subtract any horizontal text scrolling

                    let final_cursor_y = note_area.y as usize // Start at the note's visible edge
                        + 1                                   // Add 1 for the top border
                        + cursor_y_relative                   // Add the cursor's row in the text
                        - vertical_scroll as usize;           // Subtract any vertical text scrolling

                    // Finally, place the cursor at the calculated position in the frame.
                    frame.set_cursor_position(Position::new(final_cursor_x as u16, final_cursor_y as u16));
                }
            }
        }
    }
}

