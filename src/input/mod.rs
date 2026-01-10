//! This module handles terminal events, focusing on keyboard input
//! to control the application's state and behavior.

mod handler;
mod start;
mod settings; 
mod map;

pub use start::start_kh;
pub use settings::settings_kh;
pub use handler::{handle_events, AppAction};