use crate::app::{App, SignedRect};
use ratatui::{
    prelude::{Rect}, 
    widgets::{Block, Borders, Clear, Paragraph}, 
    Frame
};

pub fn render(frame: &mut Frame, app: &mut App) {

    // ?...

    app.screen_width = frame.area().width as usize;
    app.screen_height = frame.area().height as usize;

    render_bar(frame, app);
    render_map(frame, app);
}

fn render_bar(frame: &mut Frame, app: &App) {
    let size = frame.area();

    let size_display = Paragraph::new(format!(
        "Width: {}, Height: {}     x: {}  y: {}",
        size.width, size.height, app.view_pos.x, app.view_pos.y
    ))
    .block(Block::default().borders(Borders::ALL).title("Terminal Info"));

    // Render the paragraph in a small area at the top
    // Note: In a real app, you would use a layout manager.
    let top_rect = ratatui::layout::Rect {
        x: size.x,
        y: size.y,
        width: size.width,
        height: 3, // Only 3 rows high
    };

    frame.render_widget(size_display, top_rect);
}

fn render_map(frame: &mut Frame, app: &App) {
    for (_key, value) in app.notes.iter() {
        // Get the dimensions of the text inside the note
        let (mut note_width, mut note_height) = value.get_dimensions();
        if note_width < 20 { note_width = 20; }
        if note_height < 4 { note_height = 4; }

        // Calculate the position of the note in relation to
        // position of the view 
        let note_rect = SignedRect {
            x: value.x as isize - app.view_pos.x as isize, // position of the note in relation to the view
            y: value.y as isize - app.view_pos.y as isize, // position of the note in relation to the view
            width: note_width as isize,
            height: note_height as isize,
        };
        // Put the dimensions of the frame (available space)
        // in the same type to use later
        let frame_rect = SignedRect {
            x: 0, // field not used
            y: 0, // field not used
            width: frame.area().width as isize,
            height: frame.area().height as isize,
        };

        // With the intersection() method figure out if the note should be
        // drawn at all, and if so - how much of it and where
        let intersection_result = note_rect.intersection(&frame_rect);

        // If a part of the note is within the view
        if let Some(visible_part) = intersection_result {
            // Convert back to standard Ratatui Rect type
            let note_area = Rect::new(
                visible_part.x as u16,
                visible_part.y as u16,
                visible_part.width as u16,
                visible_part.height as u16,
            );

            // How much to scroll the Paragraph type (text) for each axis
            let horizontal_scroll = visible_part.x - (note_rect.x);
            let vertical_scroll = visible_part.y - (note_rect.y);


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

            let text_widget = Paragraph::new(format!("{}", value.content))
                .scroll((vertical_scroll as u16, horizontal_scroll as u16))
                .block(Block::default().borders(borders));

            frame.render_widget(text_widget, note_area);
        }
    }
}

/// Clear the entire area
pub fn clear(f: &mut Frame) {
    let area = f.area(); // The area of the entire frame
    f.render_widget(Clear, area);
}