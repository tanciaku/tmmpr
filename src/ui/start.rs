
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Position},
    style::{Color, Style},
    widgets::{Block, Clear, Paragraph, List, ListItem},
    text::{Span, Line},
};

use crate::{
    states::{
        StartState,
        start::{FocusedInputBox, SelectedStartButton, ErrMsg},
    },
};


/// Renders the start screen with menu options and optional path input dialog.
pub fn render_start(frame: &mut Frame, start_state: &mut StartState) {
    frame.render_widget(Clear, frame.area());

    let start_text_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(35),
            Constraint::Min(12),
            Constraint::Percentage(45),
            Constraint::Min(1),
        ]).split(frame.area());
    
    let create_select_style = SelectedStartButton::CreateSelect.get_style(&start_state.selected_button);
    let recent1_style = SelectedStartButton::Recent1.get_style(&start_state.selected_button);
    let recent2_style = SelectedStartButton::Recent2.get_style(&start_state.selected_button);
    let recent3_style = SelectedStartButton::Recent3.get_style(&start_state.selected_button);

    // Display error if getting recent paths failed, otherwise show recents header
    let recents_text = match &start_state.display_err_msg {
        Some(_) => {
            Line::from(Span::styled("File doesn't exist or there was an error reading it", Style::new().fg(Color::Red))).alignment(Alignment::Center)
        }
        None => {
            match &start_state.recent_paths {
                Ok(_) => Line::from("Recents:").alignment(Alignment::Center),
                Err(ErrMsg::DirFind) => Line::from(Span::styled("Error finding the home directory", Style::new().fg(Color::Red))).alignment(Alignment::Center),
                Err(ErrMsg::DirCreate) => Line::from(Span::styled("Error creating the config directory", Style::new().fg(Color::Red))).alignment(Alignment::Center),
                Err(ErrMsg::FileRead) => Line::from(Span::styled("Error reading recent_paths file", Style::new().fg(Color::Red))).alignment(Alignment::Center),
                Err(ErrMsg::FileWrite) => Line::from(Span::styled("Error creating recent_paths file", Style::new().fg(Color::Red))).alignment(Alignment::Center), 
            }
        }
    };

    let (recent1_text, recent2_text, recent3_text) = if let Ok(recent_paths) = &start_state.recent_paths {
        (
            recent_paths.recent_path_1.as_ref().map_or(String::new(), |p| p.to_string_lossy().into_owned()),
            recent_paths.recent_path_2.as_ref().map_or(String::new(), |p| p.to_string_lossy().into_owned()),
            recent_paths.recent_path_3.as_ref().map_or(String::new(), |p| p.to_string_lossy().into_owned()),
        )
    } else {
        (String::new(), String::new(), String::new())
    };
    // Wrap paths in brackets to visually distinguish as selectable buttons
    let recent1_text = format!("[ {} ]", recent1_text);
    let recent2_text = format!("[ {} ]", recent2_text);
    let recent3_text = format!("[ {} ]", recent3_text);

    
    let start_menu = vec![
        Line::from("tmmpr  v0.1.0").alignment(Alignment::Center),
        Line::from(""),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("[ Create a new map / Select existing map ]", create_select_style)).alignment(Alignment::Center),
        Line::from(""),
        Line::from(""),
        recents_text,
        Line::from(""),
        Line::from(Span::styled(recent1_text, recent1_style)).alignment(Alignment::Center),
        Line::from(Span::styled(recent2_text, recent2_style)).alignment(Alignment::Center),
        Line::from(Span::styled(recent3_text, recent3_style)).alignment(Alignment::Center),
    ];
        
    let start_menu: Vec<ListItem> = start_menu
        .into_iter()
        .map(ListItem::new)
        .collect();

    let start_menu = List::new(start_menu);

    let info_text = Line::from("q - quit      Enter - choose option      k / Up - go up       j / Down - go down").alignment(Alignment::Center);

    frame.render_widget(start_menu, start_text_area[1]);
    frame.render_widget(info_text, start_text_area[3]);

    // Overlay input dialog when user chooses to create/select a map
    if start_state.input_path {
        let input_menu_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Length(20),
                Constraint::Fill(1),
            ]).split(frame.area()); 
        let input_menu_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Length(70),
                Constraint::Fill(1),
            ]).split(input_menu_area[1]);
        
        // Layout indices: [1,2]=labels, [3]=input1, [5]=label, [6]=input2, [8]=error
        let input_menu_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min(2),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(2),
            ]).split(input_menu_area[1]);
        
        let input_box_area_1 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Length(50),
                Constraint::Fill(1),
            ]).split(input_menu_areas[3]);
        let input_box_area_2 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Length(50),
                Constraint::Fill(1),
            ]).split(input_menu_areas[6]);

        let text_line_1 = Paragraph::new(Line::from("Directory path for your map file:").alignment(Alignment::Center));
        let text_line_2 = Paragraph::new(Line::from("(Relative to your home directory, e.g. maps/):").alignment(Alignment::Center));
        let text_line_3 = Paragraph::new(Line::from("Enter the map name:").alignment(Alignment::Center));

        frame.render_widget(Clear, input_menu_area[1]);
        frame.render_widget(Block::bordered(), input_menu_area[1]);

        // Override start menu help text with input-specific controls
        let info_text = Line::from("Esc - Cancel      Enter - confirm field").alignment(Alignment::Center);
        frame.render_widget(Clear, start_text_area[3]);
        frame.render_widget(info_text, start_text_area[3]);

        frame.render_widget(text_line_1, input_menu_areas[1]);
        frame.render_widget(text_line_2, input_menu_areas[2]);
        frame.render_widget(text_line_3, input_menu_areas[5]);

        let input_box_1_block = FocusedInputBox::InputBox1.get_style(&start_state.focused_input_box);
        let input_box_2_block = FocusedInputBox::InputBox2.get_style(&start_state.focused_input_box);

        if let Some(input_path_string) = &start_state.input_path_string {
            let input_box_1 = Paragraph::new(Line::from(input_path_string.as_str())).block(input_box_1_block);
            frame.render_widget(input_box_1, input_box_area_1[1]);
        }
        if let Some(input_path_name) = &start_state.input_path_name {
            let input_box_2 = Paragraph::new(Line::from(input_path_name.as_str())).block(input_box_2_block);
            frame.render_widget(input_box_2, input_box_area_2[1]);
        }

        // Position cursor at end of text in the focused input box
        match start_state.focused_input_box {
            FocusedInputBox::InputBox1 => {
                if let Some(input_path_string) = &start_state.input_path_string {
                    // +1 offset accounts for border width
                    let x = input_box_area_1[1].x + input_path_string.len() as u16 + 1;
                    let y = input_box_area_1[1].y + 1;

                    frame.set_cursor_position(Position::new(x, y));
                }
            }
            FocusedInputBox::InputBox2 => {
                if let Some(input_path_name) = &start_state.input_path_name {
                    let x = input_box_area_2[1].x + input_path_name.len() as u16 + 1;
                    let y = input_box_area_2[1].y + 1;

                    frame.set_cursor_position(Position::new(x, y));
                }
            }
        }

        if let Some(err) = &start_state.display_err_msg {
            match err {
                ErrMsg::DirFind => {
                  let error_text = Line::from(Span::styled("Error finding the home directory", Style::new().fg(Color::Red))).alignment(Alignment::Center);
                  frame.render_widget(error_text, input_menu_areas[8]);
                }
                ErrMsg::DirCreate => {
                  let error_text = Line::from(Span::styled("Error creating the directory", Style::new().fg(Color::Red))).alignment(Alignment::Center);
                  frame.render_widget(error_text, input_menu_areas[8]);
                }
                ErrMsg::FileWrite => {
                  let error_text = Line::from(Span::styled("Error creating the map file", Style::new().fg(Color::Red))).alignment(Alignment::Center);
                  frame.render_widget(error_text, input_menu_areas[8]);
                }
                ErrMsg::FileRead => {
                  let error_text = Line::from(Span::styled("Error reading the map file", Style::new().fg(Color::Red))).alignment(Alignment::Center);
                  frame.render_widget(error_text, input_menu_areas[8]);
                }
            }
        }
    }
}
