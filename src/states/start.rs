
use ratatui::style::{Color, Style};

pub struct StartState {
    pub needs_clear_and_redraw: bool,
    pub selected_button: SelectedStartButton,
}

impl StartState {
    pub fn new() -> StartState {
        StartState {
            needs_clear_and_redraw: true,
            selected_button: SelectedStartButton::CreateSelect,
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

impl SelectedStartButton {
    /// Determines the style based on if the button is selected
    pub fn get_style(&self, selected_button: &SelectedStartButton) -> Style {
        if self == selected_button {
            // Selected button style
            Style::new().bg(Color::White).fg(Color::Black)
        } else {
            // Default style
            Style::new()
        }
    }
}    