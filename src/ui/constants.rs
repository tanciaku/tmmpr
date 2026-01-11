// Character sets for different connection types
pub const IN_PROGRESS_CHARSET: [&str; 6] = ["━", "┃", "┏", "┓", "┗", "┛"];
pub const NORMAL_CHARSET: [&str; 6] = ["─", "│", "┌", "┐", "└", "┘"];

pub const PLAIN_JUNCTIONS: [&str; 4] = ["┴", "┬", "┤", "├"];
pub const THICK_JUNCTIONS: [&str; 4] = ["┻", "┳", "┫", "┣"];
pub const DOUBLE_JUNCTIONS: [&str; 4] = ["╩", "╦", "╣", "╠"];

/// Which direction the segment is going
/// Used to determine which corner character to draw
#[derive(Copy, Clone)]
pub enum SegDir {
    Right, // Horizontal going Right
    Left, // Horizontal going Left
    Up, // Vertical going Up
    Down, // Vertical going Down
}