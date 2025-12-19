
use std::fs;
use ratatui::{
    style::{Color, Style},
    widgets::{Block, BorderType},
};

use crate::{input::AppAction};

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
        }
    }
    
    pub fn clear_and_redraw(&mut self) {
        self.needs_clear_and_redraw = true;
    }

    pub fn navigate_start_buttons(&mut self, key: &str) {
        match key {
            "k" => self.button_list_go_up(),
            "j" => self.button_list_go_down(),
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

    pub fn submit_path(&mut self) -> AppAction {
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

        // Make the full path to the file (e.g. /home/user/maps/map)
        let map_file_path = map_path.join(name).with_extension("json");

        // -- 2. File Creation (Conditional) --
        // Load the file if it exits:
        if map_file_path.exists() {
            AppAction::LoadMapFile(map_file_path)
        } else { // Otherwise create it
            AppAction::CreateMapFile(map_file_path)
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

/// Which error message to display when accessing/creating a map fails
pub enum ErrMsg {
    DirFind,
    DirCreate,
    FileRead,
    FileWrite,
}

//pub struct RecentPaths {
//    pub recent_path_1: Option<String>,
//    pub recent_path_2: Option<String>,
//    pub recent_path_3: Option<String>,
//}