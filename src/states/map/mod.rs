// Public API exports
pub use state::*;
pub use note::*;
pub use connection::*;
pub use geometry::*;
pub use enums::*;

// Implementation modules
mod state;
mod note;
mod connection;
mod geometry;
mod enums;

// Test modules
#[cfg(test)]
mod tests;