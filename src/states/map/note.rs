use ratatui::style::Color;
use serde::{Serialize, Deserialize};
use super::enums::Side;

/// Represents a single note (node) on the plain.
///
/// Notes are the fundamental building blocks of the mind map, each positioned at
/// specific coordinates on the canvas. They can contain text content, be visually
/// styled with colors, and can be selected for editing or manipulation.
///
/// # Fields
///
/// * `x`, `y` - The absolute position of the note's top-left corner on the canvas
/// * `content` - The text content displayed within the note
/// * `selected` - Whether this note is currently selected by the user
/// * `color` - The border color of the note when rendered in the TUI
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct Note {
    pub x: usize,
    pub y: usize,
    pub content: String,
    pub selected: bool,
    #[serde(with = "crate::utils")]
    pub color: Color,
}

impl Note {
    /// Creates a new `Note` at a given position with initial content.
    pub fn new(x: usize, y: usize, content: String, selected: bool, color: Color) -> Note {
        Note {
            x,
            y,
            content,
            selected,
            color,
        }
    }

    /// Calculates the width and height of the note's bounding box for rendering.
    ///
    /// The dimensions include a 2-cell padding for the borders.
    /// The height is calculated by counting newline characters to correctly handle
    /// trailing empty lines, which the `lines()` iterator would otherwise ignore.
    pub fn get_dimensions(&self) -> (u16, u16) {
        // Height is 1 (for the first line) + number of newline characters.
        let height = (1 + self.content.matches('\n').count()) as u16;
        
        // Width is the character count of the longest line.
        let width = self.content
            .lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0) as u16;
        
        // Add 2 to each dimension for a 1-cell border on all sides.
        (width + 2, height + 2)
    }

    pub fn get_connection_point(&self, side: Side) -> (usize, usize) {
        let (mut note_width, mut note_height) = self.get_dimensions();

        // Enforce a minimum size
        if note_width < 20 { note_width = 20; }
        if note_height < 4 { note_height = 4; } 
        // Add space for cursor
        note_width+=1;

        match side {
            Side::Right => {
                ((self.x + note_width as usize - 1), (self.y + (note_height/2) as usize))
            }
            Side::Left => {
                (self.x, (self.y + (note_height/2) as usize))
            }
            Side::Top => {
                (self.x + (note_width/2) as usize, self.y)
            }
            Side::Bottom => {
                (self.x + (note_width/2) as usize, self.y + note_height as usize - 1)
            }
        }
    }
}
