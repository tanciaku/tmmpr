use std::path::PathBuf;
use crate::{
    input::AppAction,
    states::start::{ErrMsg, FocusedInputBox, RecentPaths, SelectedStartButton, get_recent_paths}, utils::FileSystem,
};

#[derive(PartialEq, Debug)]
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

    /// Submit a path with a custom filesystem implementation (for testing)
    pub fn submit_path_with_fs(
        &mut self,
        recent_path: Option<PathBuf>,
        fs: &dyn FileSystem,
    ) -> AppAction {
        match recent_path {
            // Submitted a path from the "Recents" section
            Some(path) => {
                if fs.path_exists(&path) {
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
                let home_path = match fs.get_home_dir() {
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
                if let Err(_) = fs.create_dir_all(&map_path) {
                    self.handle_submit_error(ErrMsg::DirCreate);
                    return AppAction::Continue
                };

                // Make the full path to the file (e.g. /home/user/maps/map_0.json)
                let map_file_path = map_path.join(name).with_extension("json");

                // Load the file if it exits:
                if fs.path_exists(&map_file_path) {
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