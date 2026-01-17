use std::collections::HashMap;
use ratatui::style::Color;
use super::note::Note;

#[derive(PartialEq, Debug)]
pub struct NotesState {
    /// A collection of all notes in the mind map, keyed by their unique ID.
    pub notes: HashMap<usize, Note>,
    /// A counter to ensure each new note gets a unique ID.
    pub next_note_id: usize,
    /// The unique ID of the currently selected note.
    pub selected_note: Option<usize>,
    /// Order in which to render the notes ("z index"). Ordered back to front.
    pub render_order: Vec<usize>,
    /// Cursor position within the currently selected note's text
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

    /// Create a new note at the given position
    /// Returns the ID of the newly created note
    pub fn create_note(&mut self, x: usize, y: usize, text: String, selected: bool, color: Color) -> usize {
        let id = self.next_note_id;
        self.notes.insert(id, Note::new(x, y, text, selected, color));
        self.render_order.push(id);
        self.next_note_id += 1;
        id
    }

    /// Select a note by ID
    /// Updates render order to bring the note to front
    /// Sets the note's selected flag to true
    pub fn select_note_by_id(&mut self, id: usize) {
        self.selected_note = Some(id);

        // Update the render order (bring to front)
        if let Some(pos) = self.render_order.iter().position(|&x| x == id) {
            let item = self.render_order.remove(pos);
            self.render_order.push(item);
        }

        // Mark as selected
        if let Some(note) = self.notes.get_mut(&id) {
            note.selected = true;
        }
    }

    // Deselect the currently selected note
    //pub fn deselect(&mut self) {
    //    if let Some(id) = self.selected_note {
    //        if let Some(note) = self.notes.get_mut(&id) {
    //            note.selected = false;
    //        }
    //    }
    //    self.selected_note = None;
    //    self.cursor_pos = 0;
    //}

    /// Find the note closest to the given coordinates
    /// Returns the ID of the closest note, or None if there are no notes
    pub fn find_closest_note(&self, x: usize, y: usize) -> Option<usize> {
        self.notes.iter()
            .min_by_key(|(_, note)| {
                // Calculate Manhattan distance: |x1 - x2| + |y1 - y2|
                let distance = (note.x as isize - x as isize).abs()
                           + (note.y as isize - y as isize).abs();
                distance as usize
            })
            .map(|(id, _)| *id)
    }
}