//! Core application state and screen management.

use crate::states::{StartState, MapState, SettingsState};

pub struct App {
    /// Set to `false` to exit the main loop.
    pub running: bool,
    pub screen: Screen,
}

impl App {
    pub fn new() -> App {
        App {
            running: true, 
            screen: Screen::Start(StartState::new()),
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

}

/// Application screens. Each variant holds its own state to avoid
/// coupling between screens and enable independent development.
#[derive(PartialEq, Debug)]
pub enum Screen {
    Start(StartState),
    Map(MapState),
    Settings(SettingsState),
}