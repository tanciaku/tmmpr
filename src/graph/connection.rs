use serde::{Deserialize, Serialize};

/// Represents a directional connection between notes in the map.
///
/// Connections can be in-progress (only `from` specified) or complete (both `from` and `to`).
/// This allows drawing connections interactively before the user selects a target note.
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
pub struct Connection<T> {
    pub from_id: usize,
    pub from_side: Side,
    /// None for in-progress connections being drawn by the user
    pub to_id: Option<usize>,
    /// None for in-progress connections being drawn by the user
    pub to_side: Option<Side>,
    pub data: T,
}

impl<T> Connection<T> {
    pub fn new(
        from_id: usize,
        from_side: Side,
        to_id: Option<usize>,
        to_side: Option<Side>,
        data: T,
    ) -> Self {
        Self {
            from_id,
            from_side,
            to_id,
            to_side,
            data,
        }
    }
}

/// Represents which side of a note a connection is attached to.
///
/// Used to specify the connection point on both the source and target notes.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}
