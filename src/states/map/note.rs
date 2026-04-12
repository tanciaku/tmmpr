use super::enums::Side;
use crate::graph::Node;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthStr;

/// The app-specific payload stored inside a graph node.
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct NoteData {
    pub content: String,
    #[serde(with = "crate::utils")]
    pub color: Color,
}

impl NoteData {
    pub fn new(content: String, color: Color) -> Self {
        Self { content, color }
    }
}

/// A note is a graph node whose data is NoteData.
pub type Note = Node<NoteData>;

pub fn new_note(x: usize, y: usize, content: String, color: Color) -> Note {
    Node::new(x, y, NoteData::new(content, color))
}

impl Note {
    /// Returns the rendered dimensions (width, height) including 2-cell border padding.
    ///
    /// Height is calculated by counting newlines rather than using `lines()` to
    /// preserve trailing empty lines that would otherwise be ignored.
    pub fn get_dimensions(&self) -> (u16, u16) {
        let height = (1 + self.data.content.matches('\n').count()) as u16;

        let width = self
            .data
            .content
            .lines()
            .map(|line| line.width())
            .max()
            .unwrap_or(0) as u16;

        enforce_note_dimensions(width, height)
    }

    /// Returns the canvas coordinates where a connection line should attach to this note.
    ///
    /// The point is centered on the specified side.
    pub fn get_connection_point(&self, side: Side) -> (usize, usize) {
        let (note_width, note_height) = self.get_dimensions();

        match side {
            Side::Right => (
                (self.x + note_width as usize - 1),
                (self.y + (note_height / 2) as usize),
            ),
            Side::Left => (self.x, (self.y + (note_height / 2) as usize)),
            Side::Top => (self.x + (note_width / 2) as usize, self.y),
            Side::Bottom => (
                self.x + (note_width / 2) as usize,
                self.y + note_height as usize - 1,
            ),
        }
    }
}

fn enforce_note_dimensions(width: u16, height: u16) -> (u16, u16) {
    let width = (width + 2).max(20) + 1; // borders, min, cursor
    let height = (height + 2).max(4); // borders, min

    (width, height)
}
