use std::path::PathBuf;
use crate::{
    input::AppAction,
    states::start::{FocusedInputBox, RecentPaths, SelectedStartButton, get_recent_paths_with_fs}, utils::{FileSystem, RealFileSystem},
    utils::IoErrorKind
};

#[derive(PartialEq, Debug)]
pub struct StartState {
    pub needs_clear_and_redraw: bool,
    pub selected_button: SelectedStartButton,
    pub input_path: bool,
    pub focused_input_box: FocusedInputBox,
    pub input_path_string: Option<String>,
    pub input_path_name: Option<String>,
    pub display_err_msg: Option<IoErrorKind>,
    pub recent_paths: Result<RecentPaths, IoErrorKind>,
}

impl StartState {
    /// Creates a new start screen state with the real filesystem
    pub fn new() -> Self {
        Self::new_with_fs(&RealFileSystem)
    }

    /// Creates a new start screen state with a custom filesystem implementation (for testing)
    pub fn new_with_fs(fs: &dyn FileSystem) -> StartState {
        StartState {
            needs_clear_and_redraw: true,
            selected_button: SelectedStartButton::CreateSelect,
            input_path: false,
            focused_input_box: FocusedInputBox::InputBox1,
            input_path_string: None,
            input_path_name: None,
            display_err_msg: None,
            recent_paths: get_recent_paths_with_fs(fs),
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

    /// Handles path submission from either recent files or manual input.
    /// 
    /// For recent paths: validates existence before loading.
    /// For manual input: constructs full path from home dir + user input (e.g. maps/map_0),
    /// creates necessary directories, then loads or creates the map file.
    pub fn submit_path_with_fs(
        &mut self,
        recent_path: Option<PathBuf>,
        fs: &dyn FileSystem,
    ) -> AppAction {
        match recent_path {
            Some(path) => {
                if fs.path_exists(&path) {
                    AppAction::LoadMapFile(path)
                } else {
                    self.display_err_msg = Some(IoErrorKind::FileRead);
                    self.clear_and_redraw();
                    AppAction::Continue
                }
            }
            None => {
                // Both fields are guaranteed to be Some when submitting manual input
                let path = &self.input_path_string.as_ref().unwrap();
                let name = &self.input_path_name.as_ref().unwrap();

                let home_path = match fs.get_home_dir() {
                    Some(path) => path,
                    None => {
                        self.handle_submit_error(IoErrorKind::DirFind);
                        return AppAction::Continue
                    }
                };

                let map_path = home_path.join(path);

                if let Err(_) = fs.create_dir_all(&map_path) {
                    self.handle_submit_error(IoErrorKind::DirCreate);
                    return AppAction::Continue
                };

                let map_file_path = map_path.join(name).with_extension("json");

                if fs.path_exists(&map_file_path) {
                    AppAction::LoadMapFile(map_file_path)
                } else {
                    AppAction::CreateMapFile(map_file_path)
                }
            }
        }
    }
    
    /// Resets input fields and displays an error message when path submission fails
    pub fn handle_submit_error(&mut self, err_msg: IoErrorKind) {
        self.input_path_string = Some(String::new());
        self.input_path_name = Some(String::new());
        self.focused_input_box = FocusedInputBox::InputBox1;
        self.display_err_msg = Some(err_msg);
    }
}