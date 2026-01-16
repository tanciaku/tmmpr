use ratatui::{
    style::{Color, Style},
    widgets::{Block, BorderType},
};

#[derive(PartialEq, Debug)]
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
            // Selected button
            Style::new().bg(Color::White).fg(Color::Black)
        } else {
            // Default
            Style::new()
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum FocusedInputBox {
    InputBox1,
    InputBox2,
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
#[derive(PartialEq, Debug)]
pub enum ErrMsg {
    DirFind,
    DirCreate,
    FileRead,
    FileWrite,
}