#[derive(PartialEq, Debug, Clone)]
pub struct VisualModeState {
    pub visual_move: bool,
    pub visual_connection: bool,
}

impl VisualModeState {
    /// Creates a new VisualModeState with both modes disabled
    pub fn new() -> Self {
        Self {
            visual_move: false,
            visual_connection: false,
        }
    }
}