use serde::{Deserialize, Serialize};

/// A node in a graph with position and arbitrary data.
/// 
/// # Coordinates
/// This library uses `usize` for coordinates, representing positions in a positive coordinate space.
/// 
/// # Type Parameters
/// * `T` - The type of data stored in this node
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Node<T> {
    pub x: usize,
    pub y: usize,
    pub data: T,
}

impl<T> Node<T> {
    /// Creates a new node at the specified position with the given data.
    pub fn new(x: usize, y: usize, data: T) -> Self {
        Self { x, y, data }
    }

    /// Returns the position of this node as a tuple.
    pub fn position(&self) -> (usize, usize) {
        (self.x, self.y)
    }
}