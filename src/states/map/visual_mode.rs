/// Tracks which visual mode operations are currently active.
///
/// Visual modes allow batch operations on selected notes and connections.
#[derive(PartialEq, Debug, Clone)]
pub struct VisualModeState {
    /// Whether visual move mode is active for repositioning notes
    pub visual_move: bool,
    /// Whether visual connection mode is active for managing connections
    pub visual_connection: bool,
}

impl VisualModeState {
    pub fn new() -> Self {
        Self {
            visual_move: false,
            visual_connection: false,
        }
    }
}