use serde::{Serialize, Deserialize};

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