use ratatui::style::Color;
use serde::{Serialize, Deserialize};
use super::enums::Side;

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
pub struct Connection {
    pub from_id: usize,
    pub from_side: Side,
    pub to_id: Option<usize>,
    pub to_side: Option<Side>,
    #[serde(with = "crate::serialization")]
    pub color: Color,
}
