use ratatui::{
    style::{Color, Style},
    widgets::{Block, BorderType},
};

/// Buttons available in the start screen.
/// Limited to 3 recent files to keep the UI compact and focused.
#[derive(PartialEq, Debug)]
pub enum SelectedStartButton {
    CreateSelect,
    Recent1,
    Recent2,
    Recent3,
}

impl SelectedStartButton {
    pub fn get_style(&self, selected_button: &SelectedStartButton) -> Style {
        if self == selected_button {
            Style::new().bg(Color::White).fg(Color::Black)
        } else {
            Style::new()
        }
    }
}

/// Tracks which input box has focus when creating or selecting a file.
#[derive(PartialEq, Debug)]
pub enum FocusedInputBox {
    InputBox1,
    InputBox2,
}

// FIXME: Unclear why this attribute is needed - investigate and remove if possible
#[allow(mismatched_lifetime_syntaxes)]
impl FocusedInputBox {
    pub fn get_style(&self, focused_input_box: &FocusedInputBox) -> Block {
        if self == focused_input_box {
            Block::bordered()
                .border_style(Color::Blue)
                .border_type(BorderType::Double)
        } else {
            Block::bordered()
                .border_style(Color::White)
                .border_type(BorderType::Plain)
        }
    }
}

/// Error conditions that can occur during start screen operations.
#[derive(PartialEq, Debug)]
pub enum ErrMsg {
    DirFind,
    DirCreate,
    FileRead,
    FileWrite,
}