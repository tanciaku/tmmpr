//! This module defines the core application state and data structures.
//! It is responsible for holding all the data needed to represent the
//! application's current state, from user interface modes to the content
//! of the notes on the canvas.

use crate::states::{StartState, MapState, SettingsState};

/// Represents the central state of the terminal application.
///
/// This struct holds all the necessary data for the application to run,
/// including the main loop control, UI state, viewport information, and the
/// collection of notes that make up the mind map.
pub struct App {
    /// Controls the main application loop. When set to `false`, the application will exit.
    pub running: bool,
    pub screen: Screen,
}

impl App {
    /// Constructs a new instance of `App`.
    ///
    /// Initializes the application state with default values, ready for the main loop.
    pub fn new() -> App {
        App {
            running: true, 
            screen: Screen::Start(StartState::new()),
        }
    }

    /// Signals the application to exit the main loop.
    pub fn quit(&mut self) {
        self.running = false;
    }

}

pub enum Screen {
    Start(StartState),
    Map(MapState),
    Settings(SettingsState),
    Help,
}