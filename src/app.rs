//! This module defines the core application state and data structures.
//! It is responsible for holding all the data needed to represent the
//! application's current state, from user interface modes to the content
//! of the notes on the canvas.

use std::collections::HashMap;

/// Represents the central state of the terminal application.
///
/// This struct holds all the necessary data for the application to run,
/// including the main loop control, UI state, viewport information, and the
/// collection of notes that make up the mind map.
pub struct App {
    /// Controls the main application loop. When set to `false`, the application will exit.
    pub running: bool,
    /// A flag indicating that the screen needs to be cleared and redrawn on the next frame.
    pub needs_clear_and_redraw: bool,
    /// The current input mode of the application, similar to Vim modes.
    pub current_mode: Mode,
    /// The position of the viewport (camera) on the infinite canvas.
    /// Position can only be positive.
    pub view_pos: ViewPos,
    /// The current width of the terminal screen in cells. Updated on every frame.
    pub screen_width: usize,
    /// The current height of the terminal screen in cells. Updated on every frame.
    pub screen_height: usize,
    /// A counter to ensure each new note gets a unique ID.
    pub next_note_id: usize,
    /// A collection of all notes in the mind map, keyed by their unique ID.
    pub notes: HashMap<usize, Note>,
    /// The unique ID of the currently selected note.
    pub selected_note: usize,
    pub cursor_pos: usize,
    pub visual_move: bool,
    pub connections: Vec<Connection>,
}

impl App {
    /// Constructs a new instance of `App`.
    ///
    /// Initializes the application state with default values, ready for the main loop.
    pub fn new() -> App {
        let mut app = App { 
            running: true, 
            needs_clear_and_redraw: true,
            current_mode: Mode::Normal,
            view_pos: ViewPos::new(),
            screen_width: 0,
            screen_height: 0,
            next_note_id: 0,
            notes: HashMap::new(),
            selected_note: 0,
            cursor_pos: 0,
            visual_move: false,
            connections: vec![],
        };

        // test, temp
        app.notes.insert(app.next_note_id, Note::new(10, 10, String::from(""), false));
        app.next_note_id += 1;
        app.notes.insert(app.next_note_id, Note::new(70, 30, String::from(""), false));
        app.next_note_id += 1;

        // Add a test connection to visualize
        app.connections.push(Connection {
            from_id: 0,
            from_side: Side::Right,
            to_id: Some(1),
            to_side: Some(Side::Left),
        });

        app
    }

    /// Sets the flag to force a screen clear and redraw on the next frame.
    pub fn clear_and_redraw(&mut self) {
        self.needs_clear_and_redraw = true;
    }

    /// Signals the application to exit the main loop.
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Adds a new, empty note to the canvas.
    ///
    /// The note is created at the center of the current viewport. It is immediately
    /// selected, and the application switches to `Mode::Insert` to allow for
    /// immediate text entry.
    pub fn add_note(&mut self) {
        let note_x = self.view_pos.x + self.screen_width/2;
        let note_y = self.view_pos.y + self.screen_height/2;
        self.notes.insert(self.next_note_id, Note::new(note_x, note_y, String::from(""), true));
        self.selected_note = self.next_note_id;
        self.current_mode = Mode::Insert;

        self.next_note_id += 1;
    }

    /// Finds and selects the note closest to the center of the viewport.
    ///
    /// This method calculates the "Manhattan distance" from the center of the screen
    /// to the top-left corner of each note and sets the `selected_note` field to the
    /// ID of the note with the smallest distance.
    pub fn select_note(&mut self) {
        let screen_center_x = self.view_pos.x + self.screen_width / 2;
        let screen_center_y = self.view_pos.y + self.screen_height / 2;
        
        // Start with a large distance and no selected note.
        let mut closest_note_id = 0;
        let mut min_distance = usize::MAX;

        // Find the note closest to the center of the screen
        for (id, note) in self.notes.iter() {
            // Calculate Manhattan distance: |x1 - x2| + |y1 - y2|.
            let distance = (note.x as isize - screen_center_x as isize).abs() 
                       + (note.y as isize - screen_center_y as isize).abs();

            if (distance as usize) < min_distance {
                min_distance = distance as usize;
                closest_note_id = *id;
            }
        }

        self.selected_note = closest_note_id;
        if let Some(note) = self.notes.get_mut(&self.selected_note) {
            note.selected = true;
        }
    }
}

/// Represents the application's current input mode, similar to Vim.
pub enum Mode {
    /// Default mode for navigation and commands.
    Normal,
    /// Mode for selecting or manipulating notes (not yet implemented).
    Visual,
    /// Mode for editing the text content of a note.
    Insert,
    Delete,
}

/// Represents a single note on the canvas.
pub struct Note {
    /// The absolute x-coordinate of the note's top-left corner on the canvas.
    pub x: usize,
    /// The absolute y-coordinate of the note's top-left corner on the canvas.
    pub y: usize,
    /// The text content of the note.
    pub content: String,
    /// A flag indicating whether this note is currently selected.
    pub selected: bool,
}

impl Note {
    /// Creates a new `Note` at a given position with initial content.
    pub fn new(x: usize, y: usize, content: String, selected: bool) -> Note {
        Note {
            x,
            y,
            content,
            selected,
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
        let (note_width, note_height) = self.get_dimensions();

        match side {
            Side::Right => {
                ((self.x + note_width as usize), (self.y + (note_height/2) as usize))
            }
            Side::Left => {
                (self.x, (self.y + (note_height/2) as usize))
            }
            Side::Top => {
                (self.x + (note_width/2) as usize, self.y)
            }
            Side::Bottom => {
                (self.x + (note_width/2) as usize, self.y + note_height as usize)
            }
        }
    }
}

/// Represents the top-left corner of the viewport on the infinite canvas.
pub struct ViewPos {
    pub x: usize,
    pub y: usize,
}

impl ViewPos {
    /// Default viewport position.
    pub fn new() -> ViewPos {
        ViewPos {
            x: 0,
            y: 0,
        }
    }
}

/// A rectangle representation that uses signed integers (`isize`) for its coordinates.
///
/// This is crucial for performing screen-space calculations where coordinates can
/// temporarily become negative (e.g., a note is partially off-screen to the left)
/// before being clipped to the viewport boundaries.
pub struct SignedRect {
    pub x: isize,
    pub y: isize,
    pub width: isize,
    pub height: isize,
}

impl SignedRect {
    /// Calculates the intersection of two `SignedRect`s.
    ///
    /// This method is the core of the clipping logic. It determines the overlapping
    /// area between two rectangles (typically a note and the viewport).
    ///
    /// Returns `Some(SignedRect)` representing the overlapping area, or `None` if
    /// the rectangles do not overlap at all.
    /// 
    /// * Reformat ?
    /// 
    pub fn intersection(&self, view: &SignedRect) -> Option<SignedRect> {
        // if the no part of the note rectangle is within the view rectangle
        // no part of the note will be drawn
        if self.x >= view.x + view.width || self.x + self.width <= view.x || self.y >= view.y + view.height || self.y + self.height <= view.y {
            return None
        // otherwise calculate the area of the note rectangle to draw
        } else { 
            // intersection area for x axis            
            let x_start = self.x.max(view.x);
            let x_end = (self.x + self.width).min(view.x + view.width);
            let x_width = x_end - x_start;

            // intersection area for y axis
            let y_start = self.y.max(view.y);
            let y_end = (self.y + self.height).min(view.y + view.height);
            let y_height = y_end - y_start;

            // return the visible area of the rectangle
            Some(SignedRect {
                x: x_start,
                y: y_start,
                width: x_width,
                height: y_height,
            })
        }
    }
}

pub struct Connection {
    pub from_id: usize,
    pub from_side: Side,
    pub to_id: Option<usize>,
    pub to_side: Option<Side>,
}

#[derive(Clone, Copy)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_note() {
        let mut app = App::new();
        app.screen_width = 80;
        app.screen_height = 24;
        app.view_pos.x = 10;
        app.view_pos.y = 10;

        app.add_note();

        assert_eq!(app.notes.len(), 1);
        assert!(matches!(app.current_mode, Mode::Insert));
        assert_eq!(app.selected_note, 0);
        assert_eq!(app.next_note_id, 1);

        let note = app.notes.get(&0).unwrap();
        assert_eq!(note.x, 10 + 80 / 2); // view_pos.x + screen_width / 2
        assert_eq!(note.y, 10 + 24 / 2); // view_pos.y + screen_height / 2
        assert_eq!(note.content, "");
        assert_eq!(note.selected, true);
    }

    #[test]
    fn test_select_note() {
        let mut app = App::new();
        app.screen_width = 80;
        app.screen_height = 24;

        // --- Scenario 1: No notes ---
        app.select_note();
        assert_eq!(app.selected_note, 0); // Should remain default

        // --- Scenario 2: One note ---
        app.notes.insert(0, Note::new(50, 20, "".to_string(), false));
        app.select_note();
        assert_eq!(app.selected_note, 0);
        assert_eq!(app.notes.get(&0).unwrap().selected, true);

        // --- Scenario 3: Multiple notes ---
        // Center of screen is (40, 12)
        // Note 0 is at (50, 20), distance = |50-40| + |20-12| = 10 + 8 = 18
        // Note 1 is at (45, 15), distance = |45-40| + |15-12| = 5 + 3 = 8  <-- Closest
        // Note 2 is at (10, 10), distance = |10-40| + |10-12| = 30 + 2 = 32
        app.notes.insert(1, Note::new(45, 15, "".to_string(), false));
        app.notes.insert(2, Note::new(10, 10, "".to_string(), false));
        
        app.select_note();
        assert_eq!(app.selected_note, 1);
        assert_eq!(app.notes.get(&1).unwrap().selected, true);
    }

    #[test]
    fn test_get_dimensions() {
        // --- Scenario 1: Empty note ---
        // Internal size (0, 1) + border (2, 2) = (2, 3)
        let note1 = Note::new(0, 0, "".to_string(), false);
        assert_eq!(note1.get_dimensions(), (2, 3));

        // --- Scenario 2: Single line ---
        // Internal size (11, 1) + border (2, 2) = (13, 3)
        let note2 = Note::new(0, 0, "hello world".to_string(), false);
        assert_eq!(note2.get_dimensions(), (13, 3));

        // --- Scenario 3: Multi-line, varied length ---
        // Internal size (longest line is 9, height is 3 lines) = (9, 3)
        // Dimensions + border (2, 2) = (11, 5)
        let note3 = Note::new(0, 0, "short\nloooonger\nmedium".to_string(), false);
        assert_eq!(note3.get_dimensions(), (11, 5));

        // --- Scenario 4: Trailing newline ---
        // Internal size (5, 2 lines) = (5, 2)
        // Dimensions + border (2, 2) = (7, 4)
        let note4 = Note::new(0, 0, "hello\n".to_string(), false);
        assert_eq!(note4.get_dimensions(), (7, 4));
    }

    #[test]
    fn test_signed_rect_intersection() {
        let view = SignedRect { x: 10, y: 10, width: 20, height: 20 };

        // --- Scenario 1: Partial overlap ---
        let rect1 = SignedRect { x: 5, y: 5, width: 10, height: 10 };
        let intersection1 = rect1.intersection(&view).unwrap();
        assert_eq!(intersection1.x, 10);
        assert_eq!(intersection1.y, 10);
        assert_eq!(intersection1.width, 5);
        assert_eq!(intersection1.height, 5);

        // --- Scenario 2: Rect fully contained in view ---
        let rect2 = SignedRect { x: 12, y: 12, width: 5, height: 5 };
        let intersection2 = rect2.intersection(&view).unwrap();
        assert_eq!(intersection2.x, 12);
        assert_eq!(intersection2.y, 12);
        assert_eq!(intersection2.width, 5);
        assert_eq!(intersection2.height, 5);

        // --- Scenario 3: No overlap ---
        let rect3 = SignedRect { x: 100, y: 100, width: 10, height: 10 };
        assert!(rect3.intersection(&view).is_none());

        // --- Scenario 4: Touching edges ---
        let rect4 = SignedRect { x: 0, y: 10, width: 10, height: 10 }; // Touches left edge
        assert!(rect4.intersection(&view).is_none());

        // --- Scenario 5: Touching corners ---
        let rect5 = SignedRect { x: 30, y: 30, width: 10, height: 10 }; // Touches bottom-right corner
        assert!(rect5.intersection(&view).is_none());
    }
}
