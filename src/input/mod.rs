//! This module handles terminal events, focusing on keyboard input
//! to control the application's state and behavior.

mod handler;
mod map;
mod settings;
mod start;
#[cfg(test)]
mod tests;

pub use handler::{AppAction, handle_events};
pub use settings::settings_kh;
pub use start::start_kh;
