use ratatui::style::Color;
use serde::{Serialize, Deserialize};
use super::enums::Side;

/// A node on the mind map canvas with position, content, and visual styling.
///
/// Notes are the fundamental building blocks of the mind map. Each note occupies
/// a position on an infinite 2D plane and can be connected to other notes.
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct Note {
    pub x: usize,
    pub y: usize,
    pub content: String,
    pub selected: bool,
    /// Custom serialization needed to convert between ratatui's Color and a persistable format
    #[serde(with = "crate::utils")]
    pub color: Color,
}

impl Note {
    pub fn new(x: usize, y: usize, content: String, selected: bool, color: Color) -> Note {
        Note {
            x,
            y,
            content,
            selected,
            color,
        }
    }

    /// Returns the rendered dimensions (width, height) including 2-cell border padding.
    ///
    /// Height is calculated by counting newlines rather than using `lines()` to
    /// preserve trailing empty lines that would otherwise be ignored.
    pub fn get_dimensions(&self) -> (u16, u16) {
        let height = (1 + self.content.matches('\n').count()) as u16;
        
        let width = self.content
            .lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0) as u16;
        
        enforce_note_dimensions(width, height)
    }

    /// Returns the canvas coordinates where a connection line should attach to this note.
    ///
    /// The point is centered on the specified side.
    ///
    /// FIXME: Repetition, mixing concerns - enforcing minimum dimensions should be handled
    /// within get_dimensions(), this also occurs in other places
    pub fn get_connection_point(&self, side: Side) -> (usize, usize) {
        let (note_width, note_height) = self.get_dimensions();

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

fn enforce_note_dimensions(width: u16, height: u16) -> (u16, u16) {
    let width = (width + 2).max(20) + 1;  // borders, min, cursor
    let height = (height + 2).max(4);     // borders, min

    (width, height)
}