use std::collections::HashMap;

pub struct App {
    pub running: bool,
    pub needs_clear_and_redraw: bool,
    pub current_mode: Mode,
    pub view_pos: ViewPos,
    pub screen_width: usize,
    pub screen_height: usize,
    pub next_note_id: usize,
    pub notes: HashMap<usize, Note>,
}

pub enum Mode {
    Normal,
    Visual,
    Insert,
}

pub struct Note {
    pub x: usize,
    pub y: usize,
    pub content: String,
}

impl Note {
    // move left 1, ... (move with view if reached border)
    pub fn new(x: usize, y: usize, content: String) -> Note {
        Note {
            x,
            y,
            content,
        }
    }

    pub fn get_dimensions(&self) -> (u16, u16) {
        let height = self.content.lines().count() as u16;
        let width = self.content
            .lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0) as u16;
        (width+2, height+2)  // (+2 is for the borders around the note)
    }
}

pub struct ViewPos {
    pub x: usize,
    pub y: usize,
}

impl ViewPos {
    pub fn new() -> ViewPos {
        ViewPos {
            x: 0,
            y: 0,
        }
    }
}

/// * ... (custom type with values that could be negative because...)
pub struct SignedRect {
    pub x: isize,
    pub y: isize,
    pub width: isize,
    pub height: isize,
}

impl SignedRect {
    /// Is called on a SignedRect instance of a note
    /// self - in this case is the note rectangle instance
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

impl App {
    /// Construct a new instance of App
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
        };
        app
    }

    pub fn clear_and_redraw(&mut self) {
        self.needs_clear_and_redraw = true;
    }

    /// Stop the application
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn add_note(&mut self) {
        let note_x = self.view_pos.x + self.screen_width/2;
        let note_y = self.view_pos.y + self.screen_height/2;
        self.notes.insert(self.next_note_id, Note::new(note_x, note_y, String::from("")));
        self.next_note_id += 1;
        //self.notes.insert(self.next_note_id, Note::new(200, 75, String::from("This is a test sentence.\nA new line.")));
    }
}