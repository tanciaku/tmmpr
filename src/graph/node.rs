use crate::graph::Side;
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

/// Trait for node data types that know their rendered layout dimensions.
///
/// Implement this on your data type `T` to plug it into `Node<T>`.
/// Only `dimensions()` is required - `connection_point()` is provided
/// automatically on `Node<T>` once this trait is satisfied.
pub trait NodeLayout {
    /// Returns the rendered (width, height) of this node, including any border padding.
    fn dimensions(&self) -> (u16, u16);
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

    /// Any `Node<T>` whose data implements `NodeLayout` gets `connection_point()` for free.
    ///
    /// This lives here rather than in the trait because it needs `self.x` and `self.y`,
    /// which only `Node<T>` has. The data type never needs to know about position.
    pub fn connection_point(&self, side: Side) -> (usize, usize)
    where
        T: NodeLayout,
    {
        let (w, h) = self.data.dimensions();

        match side {
            Side::Right => ((self.x + w as usize - 1), (self.y + (h / 2) as usize)),
            Side::Left => (self.x, (self.y + (h / 2) as usize)),
            Side::Top => (self.x + (w / 2) as usize, self.y),
            Side::Bottom => (self.x + (w / 2) as usize, self.y + h as usize - 1),
        }
    }
}
