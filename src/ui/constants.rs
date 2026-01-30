pub const IN_PROGRESS_CHARSET: [&str; 6] = ["━", "┃", "┏", "┓", "┗", "┛"];
pub const NORMAL_CHARSET: [&str; 6] = ["─", "│", "┌", "┐", "└", "┘"];

pub const PLAIN_JUNCTIONS: [&str; 4] = ["┴", "┬", "┤", "├"];
pub const THICK_JUNCTIONS: [&str; 4] = ["┻", "┳", "┫", "┣"];
pub const DOUBLE_JUNCTIONS: [&str; 4] = ["╩", "╦", "╣", "╠"];

/// Used to determine which corner character to draw when rendering connections
#[derive(Copy, Clone)]
pub enum SegDir {
    Right,
    Left,
    Up,
    Down,
}