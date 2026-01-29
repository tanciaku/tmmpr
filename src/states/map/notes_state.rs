use std::collections::HashMap;
use ratatui::style::Color;
use super::note::Note;

#[derive(PartialEq, Debug)]
pub struct NotesState {
    pub notes: HashMap<usize, Note>,
    /// Used to generate unique IDs for new notes
    pub next_note_id: usize,
    pub selected_note: Option<usize>,
    /// Z-index ordering for note rendering (back to front)
    pub render_order: Vec<usize>,
    pub cursor_pos: usize,
}

impl NotesState {
    pub fn new() -> Self {
        Self {
            notes: HashMap::new(),
            next_note_id: 0,
            selected_note: None,
            render_order: vec![],
            cursor_pos: 0,
        }
    }

    /// Creates a new note and returns its ID
    pub fn create_note(&mut self, x: usize, y: usize, text: String, selected: bool, color: Color) -> usize {
        let id = self.next_note_id;
        self.notes.insert(id, Note::new(x, y, text, selected, color));
        self.render_order.push(id);
        self.next_note_id += 1;
        id
    }

    /// Selects a note by ID and brings it to the front of render order
    pub fn select_note_by_id(&mut self, id: usize) {
        self.selected_note = Some(id);

        // Bring to front of render order
        if let Some(pos) = self.render_order.iter().position(|&x| x == id) {
            let item = self.render_order.remove(pos);
            self.render_order.push(item);
        }

        if let Some(note) = self.notes.get_mut(&id) {
            note.selected = true;
        }
    }

    /// Finds the note closest to the given coordinates using Manhattan distance
    pub fn find_closest_note(&self, x: usize, y: usize) -> Option<usize> {
        self.notes.iter()
            .min_by_key(|(_, note)| {
                let distance = (note.x as isize - x as isize).abs()
                           + (note.y as isize - y as isize).abs();
                distance as usize
            })
            .map(|(id, _)| *id)
    }
}