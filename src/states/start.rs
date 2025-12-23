
use std::{fs, path::{Path, PathBuf}};
use ratatui::{
    style::{Color, Style},
    widgets::{Block, BorderType},
};
use crate::{
    input::AppAction,
    utils::{read_json_data, write_json_data},
};
use serde::{Serialize, Deserialize};

pub struct StartState {
    pub needs_clear_and_redraw: bool,
    pub selected_button: SelectedStartButton,
    /// Is user entering a path for the map?
    pub input_path: bool,
    pub focused_input_box: FocusedInputBox,
    pub input_path_string: Option<String>,
    pub input_path_name: Option<String>,
    /// Which error message to display in case handling
    /// the map file fails
    pub display_err_msg: Option<ErrMsg>,
    pub recent_paths: Result<RecentPaths, ErrMsg>,
}

impl StartState {
    pub fn new() -> StartState {
        StartState {
            needs_clear_and_redraw: true,
            selected_button: SelectedStartButton::CreateSelect,
            input_path: false,
            focused_input_box: FocusedInputBox::InputBox1,
            input_path_string: None,
            input_path_name: None,
            display_err_msg: None,
            recent_paths: get_recent_paths(),
        }
    }
    
    pub fn clear_and_redraw(&mut self) {
        self.needs_clear_and_redraw = true;
    }

    pub fn navigate_start_buttons(&mut self, key: &str) {
        match key {
            "k" | "Up" => self.button_list_go_up(),
            "j" | "Down" => self.button_list_go_down(),
            _ => {}
        }
    }
    
    fn button_list_go_up(&mut self) {
        self.selected_button = match self.selected_button {
            SelectedStartButton::CreateSelect => SelectedStartButton::CreateSelect,
            SelectedStartButton::Recent1 => SelectedStartButton::CreateSelect,
            SelectedStartButton::Recent2 => SelectedStartButton::Recent1,
            SelectedStartButton::Recent3 => SelectedStartButton::Recent2,
        }
    }

    fn button_list_go_down(&mut self) {
        self.selected_button = match self.selected_button {
            SelectedStartButton::CreateSelect => SelectedStartButton::Recent1,
            SelectedStartButton::Recent1 => SelectedStartButton::Recent2,
            SelectedStartButton::Recent2 => SelectedStartButton::Recent3,
            SelectedStartButton::Recent3 => SelectedStartButton::Recent3,
        }
    }

    pub fn submit_path(&mut self, recent_path: Option<PathBuf>) -> AppAction {
        match recent_path {
            // Submitted a path from the "Recents" section
            Some(path) => {
                if path.exists() {
                    AppAction::LoadMapFile(path)
                } else {
                    self.display_err_msg = Some(ErrMsg::FileRead);
                    self.clear_and_redraw();
                    AppAction::Continue
                }
            }
            // Entered a new path
            None => {
                // Get the provided path and name
                // Both fields will always be Some in this scenario, so it's safe.
                let path = &self.input_path_string.as_ref().unwrap(); // provided path (relative to home path, e.g. maps/)
                let name = &self.input_path_name.as_ref().unwrap(); // provided map file name

                // Get the user's home directory path 
                //  or display an error and stop there
                let home_path = match home::home_dir() {
                    Some(path) => path,
                    None => {
                        self.handle_submit_error(ErrMsg::DirFind);
                        return AppAction::Continue
                    }
                };

                // Make the path to the file's directory (e.g. /home/user/maps/)
                let map_path = home_path.join(path);

                // Create the directory if it doesn't exist
                //  or display an error and stop there
                if let Err(_) = fs::create_dir_all(&map_path) {
                    self.handle_submit_error(ErrMsg::DirCreate);
                    return AppAction::Continue
                };

                // Make the full path to the file (e.g. /home/user/maps/map_0.json)
                let map_file_path = map_path.join(name).with_extension("json");

                // Load the file if it exits:
                if map_file_path.exists() {
                    AppAction::LoadMapFile(map_file_path)
                } else { // Otherwise create it
                    AppAction::CreateMapFile(map_file_path)
                }
            }
        }
    }         
    
    /// Helper function for clearing input fields and displaying error messages
    /// for the input menu
    pub fn handle_submit_error(&mut self, err_msg: ErrMsg) {
        self.input_path_string = Some(String::new()); // Reset fields
        self.input_path_name = Some(String::new()); // Reset fields
        self.focused_input_box = FocusedInputBox::InputBox1; // Switch back to the first input box 
        self.display_err_msg = Some(err_msg); // Show corresponding error message
    }
}             

#[derive(PartialEq)]
pub enum SelectedStartButton {
    CreateSelect,
    Recent1,
    Recent2,
    Recent3,
}

#[derive(PartialEq)]
pub enum FocusedInputBox {
    InputBox1,
    InputBox2,
}

impl SelectedStartButton {
    /// Determines the style based on if the button is selected
    pub fn get_style(&self, selected_button: &SelectedStartButton) -> Style {
        if self == selected_button {
            // Selected button
            Style::new().bg(Color::White).fg(Color::Black)
        } else {
            // Default
            Style::new()
        }
    }
}

#[allow(mismatched_lifetime_syntaxes)]
impl FocusedInputBox {
    /// Determines the block style based on if the input box is in focus
    pub fn get_style(&self, focused_input_box: &FocusedInputBox) -> Block {
        if self == focused_input_box {
            // Input box is in focus
            Block::bordered()
                .border_style(Color::Blue)
                .border_type(BorderType::Double)
        } else {
            // Default
            Block::bordered()
                .border_style(Color::White)
                .border_type(BorderType::Plain)
        }
    }
}

/// Which error message to display
pub enum ErrMsg {
    DirFind,
    DirCreate,
    FileRead,
    FileWrite,
}

/// Gets the recent paths from the ~/.config/tmmpr/recent_paths.json file.
/// Or creates an empty one if it doesn't exist
/// If there is an error somewhere along the way - returns None
///   (can't use recent_paths functionality in that case)
pub fn get_recent_paths() -> Result<RecentPaths, ErrMsg> {
    // Get the user's home directory path
    let home_path = match home::home_dir() {
        Some(path) => path,
        None => return Err(ErrMsg::DirFind),
    };

    // Make the path to the config directory (e.g. /home/user/.config/tmmpr/)
    let config_dir_path = home_path.join(".config/tmmpr/");

    // Create the directory if it doesn't exist
    if let Err(_) = fs::create_dir_all(&config_dir_path) {
        return Err(ErrMsg::DirCreate)
    };

    // Make the full path to the file (e.g. /home/user/.config/tmmpr/recent_paths.json)
    let recent_paths_file_path = config_dir_path.join("recent_paths").with_extension("json");

    // Load the file if it exits:
    if recent_paths_file_path.exists() {
        match read_json_data(&recent_paths_file_path) {
            Ok(recent_paths) => Ok(recent_paths),
            Err(_) => Err(ErrMsg::FileRead),
        }
    } else { // Otherwise create it
        let new_recent_paths = RecentPaths::new();
        match write_json_data(&recent_paths_file_path, &new_recent_paths) {
            Ok(_) => Ok(new_recent_paths),
            Err(_) => Err(ErrMsg::FileWrite),
        }
    }
}

// PathBuf because the state needs to own it's fields.
#[derive(Serialize, Deserialize)]
pub struct RecentPaths {
    pub recent_path_1: Option<PathBuf>,
    pub recent_path_2: Option<PathBuf>,
    pub recent_path_3: Option<PathBuf>,
}

impl RecentPaths {
    pub fn new() -> RecentPaths {
        RecentPaths { 
            recent_path_1: None,
            recent_path_2: None,
            recent_path_3: None,
        }
    }

    /// Adds a new recent_path and moves the other down by 1, discarding the last one
    pub fn add(&mut self, path: PathBuf) {
        // "Move" the other two "down by 1", discarding the one in 3 (if any)
        self.recent_path_3 = self.recent_path_2.clone();
        self.recent_path_2 = self.recent_path_1.clone();

        // Add the new one
        self.recent_path_1 = Some(path);
    }

    /// Returns true if the given path exists in any of the recent paths
    pub fn contains_path(&self, path: &Path) -> bool {
        self.recent_path_1.as_deref() == Some(path) ||
        self.recent_path_2.as_deref() == Some(path) ||
        self.recent_path_3.as_deref() == Some(path)
    }

    /// There cannot be an error here since - if the user cannot use the
    /// recent_paths functionality - this will never be called.
    /// If the directories didn't exist before, they would at this point.
    pub fn save(&self) {
        // Get the user's home directory path
        let home_path = match home::home_dir() {
            Some(path) => path,
            None => return,
        };

        // Make the full path to the file (e.g. /home/user/.config/tmmpr/recent_paths.json)
        let recent_paths_file_path = home_path.join(".config/tmmpr/recent_paths").with_extension("json");

        // Write the data (guaranteed at this point)
        let _ = write_json_data(&recent_paths_file_path, &self);
    }
}

