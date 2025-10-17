//! This module is responsible for all rendering logic of the application.
//! It takes the application state (`App`) and a `ratatui` frame, and draws the UI.

use crate::app::{App, Mode, SignedRect};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
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

            let mut block_border_color = Block::default();
            if note.selected { 
                match app.current_mode {
                    Mode::Visual => { block_border_color = Block::default().borders(borders).border_style(Color::Yellow) } 
                    Mode::Insert => { block_border_color = Block::default().borders(borders).border_style(Color::Blue) } 
                    _ => {}
                } 
            } else {
                block_border_color = Block::default().borders(borders).border_style(Color::White) 
            }

            // --- 6. Render the Widget ---
            let text_widget = Paragraph::new(note.content.as_str())
                .scroll((vertical_scroll, horizontal_scroll))
                .block(block_border_color);

            frame.render_widget(text_widget, note_area);
        }
    }
}

