use serde::{Deserialize, Serialize};

/// Represents the top-left corner of the viewport on the infinite canvas.
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
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

#[derive(PartialEq, Debug)]
pub struct ViewportState {
    /// The position of the viewport (camera) on the infinite canvas.
    /// Position can only be positive.
    pub view_pos: ViewPos,
    /// The current width of the terminal screen in cells. Updated on every frame.
    pub screen_width: usize,
    /// The current height of the terminal screen in cells. Updated on every frame.
    pub screen_height: usize,
}

impl ViewportState {
    pub fn new() -> Self {
        Self {
            view_pos: ViewPos::new(),
            screen_width: 0,
            screen_height: 0,
        }
    }
    
    /// Get the center coordinates of the viewport
    pub fn center(&self) -> (usize, usize) {
        (
            self.view_pos.x + self.screen_width / 2,
            self.view_pos.y + self.screen_height / 2,
        )
    }

    /// Convert world to screen coordinates
    pub fn to_screen_coords(&self, p_x: isize , p_y: isize) -> (isize, isize) {
        let p_x = p_x - self.view_pos.x as isize;
        let p_y = p_y - self.view_pos.y as isize;
        (p_x, p_y)
    }
}