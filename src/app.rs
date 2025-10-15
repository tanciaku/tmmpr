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
}

impl App {
    /// Constructs a new instance of `App`.
    ///
    /// Initializes the application state with default values, ready for the main loop.
    pub fn new() -> App {
        App { 
            running: true, 
            needs_clear_and_redraw: true,
            current_mode: Mode::Normal,
            view_pos: ViewPos::new(),
            screen_width: 0,
            screen_height: 0,
            next_note_id: 0,
            notes: HashMap::new(),
            selected_note: 0,
        }
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
        self.notes.insert(self.next_note_id, Note::new(note_x, note_y, String::from("")));
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
    pub fn new(x: usize, y: usize, content: String) -> Note {
        Note {
            x,
            y,
            content,
            selected: false,
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
