use std::collections::HashMap;
use ratatui::style::Color;
use super::note::Note;

#[derive(PartialEq, Debug)]
pub struct NotesState {
    notes: HashMap<usize, Note>,
    /// Used to generate unique IDs for new notes
    next_note_id_counter: usize,
    selected_note_id: Option<usize>,
    /// Z-index ordering for note rendering (back to front)
    render_order: Vec<usize>,
    cursor_pos: usize,
}

impl NotesState {
    pub fn new() -> Self {
        Self {
            notes: HashMap::new(),
            next_note_id_counter: 0,
            selected_note_id: None,
            render_order: vec![],
            cursor_pos: 0,
        }
    }

    /// Used when loading a map file from disk.
    pub fn from_map_data(
        notes: HashMap<usize, Note>,
        next_note_id_counter: usize,
        render_order: Vec<usize>,
    ) -> Self {
        Self {
            notes,
            next_note_id_counter,
            selected_note_id: None,
            render_order,
            cursor_pos: 0,
        }
    }

    pub fn notes(&self) -> &HashMap<usize, Note> {
        &self.notes
    }

    pub fn next_note_id_counter(&self) -> usize {
        self.next_note_id_counter
    }

    pub fn render_order(&self) -> &Vec<usize> {
        &self.render_order
    }

    pub fn selected_note_id(&self) -> Option<usize> {
        self.selected_note_id
    }

    /// Panics if no note is currently selected
    pub fn expect_selected_note_id(&self) -> usize {
        self.selected_note_id.expect("Bug: selected_note_id() called with no note selected")
    }

    /// Panics if no note is currently selected or if the selected note ID
    /// references a non-existent note.
    pub fn expect_selected_note(&self) -> &Note {
        let selected_note_id = self.selected_note_id
            .expect("Bug: get_selected_note() called with no note selected");
    
        self.notes
            .get(&selected_note_id)
            .expect("Bug: selected_note_id references non-existent note")
    }

    /// Panics if no note is currently selected or if the selected note ID
    /// references a non-existent note.
    pub fn expect_selected_note_mut(&mut self) -> &mut Note {
        let selected_note_id = self.selected_note_id
            .expect("Bug: get_selected_note_mut() called with no note selected");
    
        self.notes
            .get_mut(&selected_note_id)
            .expect("Bug: selected_note_id references non-existent note")
    }

    /// Creates a new note, returns its id
    pub fn add(&mut self, x: usize, y: usize, text: String, color: Color) -> usize {
        let id = self.next_note_id_counter;
        self.notes.insert(id, Note::new(x, y, text, color));
        self.render_order.push(id);
        self.next_note_id_counter += 1;
        id
    }
 
    /// Removes a note by ID and updates the render order
    pub fn remove(&mut self, id: usize) {
        self.notes.remove(&id);

        if let Some(pos) = self.render_order.iter().position(|&x| x == id) {
            self.render_order.remove(pos);
        }
        
        self.selected_note_id = None;
    }

    /// Panics if the note id doesn't exist or is not in the render order
    fn expect_id_exists(&self, id: usize) {
        assert!(
            self.notes.contains_key(&id),
            "Bug: note id {} not found in notes HashMap",
            id
        );
        assert!(
            self.render_order.contains(&id),
            "Bug: note id {} not found in render order",
            id
        );
    }

    /// Panics if the note id doesn't exist or is not in the render order
    pub fn select(&mut self, id: usize) {
        self.expect_id_exists(id);

        self.selected_note_id = Some(id);

        // Bring to front of render order
        let pos = self.render_order.iter().position(|&x| x == id).unwrap();
        let item = self.render_order.remove(pos);
        self.render_order.push(item);
    }

    /// Panics if no note is selected.
    pub fn deselect(&mut self) {
        let _ = self.selected_note_id
            .take()
            .expect("Bug: deselect() called with no note selected");
    }
    
    /// Finds the note closest to the given coordinates
    pub fn find_closest_note(&self, x: usize, y: usize) -> Option<usize> {
        self.notes().iter()
            .min_by_key(|(_, note)| {
                let distance = (note.x as isize - x as isize).abs()
                           + (note.y as isize - y as isize).abs();
                distance as usize
            })
            .map(|(id, _)| *id)
    }

    pub fn cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    /// If `pos` exceeds the content length, it will be set to the maximum valid position.
    /// # Panics
    /// If no note is selected.
    pub fn set_cursor_pos(&mut self, pos: usize) {
        let note = self.expect_selected_note();
        self.cursor_pos = pos.min(note.content.chars().count());
    }
}