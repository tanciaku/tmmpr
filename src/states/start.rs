
use ratatui::{
    style::{Color, Style},
    widgets::{Block, BorderType},
};

pub struct StartState {
    pub needs_clear_and_redraw: bool,
    pub selected_button: SelectedStartButton,
    /// Is user entering a path for the map?
    pub input_path: bool,
    pub focused_input_box: FocusedInputBox,
    pub input_path_string: Option<String>,
    pub input_path_name: Option<String>,
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


