use crate::graph::{Node, NodeLayout, Side};
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

/// NoteData knows how large it renders. Position-dependent geometry
/// (connection_point) is handled by Node<T: NodeLayout> in graph/node.rs.
impl NodeLayout for NoteData {
    fn dimensions(&self) -> (u16, u16) {
        let height = (1 + self.content.matches('\n').count()) as u16;

        let width = self
            .content
            .lines()
            .map(|line| line.width())
            .max()
            .unwrap_or(0) as u16;

        enforce_note_dimensions(width, height)
    }
}

impl Note {
    /// Returns the rendered dimensions (width, height) including 2-cell border padding.
    pub fn get_dimensions(&self) -> (u16, u16) {
        self.data.dimensions()
    }

    /// Returns the canvas coordinates where a connection line should attach to this note.
    pub fn get_connection_point(&self, side: Side) -> (usize, usize) {
        self.connection_point(side)
    }
}

fn enforce_note_dimensions(width: u16, height: u16) -> (u16, u16) {
    let width = (width + 2).max(20) + 1; // borders, min, cursor
    let height = (height + 2).max(4); // borders, min

    (width, height)
}
