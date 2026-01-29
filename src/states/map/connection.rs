use ratatui::style::Color;
use serde::{Serialize, Deserialize};
use super::enums::Side;

/// Represents a directional connection between notes in the map.
///
/// Connections can be in-progress (only `from` specified) or complete (both `from` and `to`).
/// This allows drawing connections interactively before the user selects a target note.
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
pub struct Connection {
    pub from_id: usize,
    pub from_side: Side,
    /// None for in-progress connections being drawn by the user
    pub to_id: Option<usize>,
    /// None for in-progress connections being drawn by the user
    pub to_side: Option<Side>,
    /// Custom serde implementation in utils handles Color serialization
    #[serde(with = "crate::utils")]
    pub color: Color,
}
