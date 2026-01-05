//! This module is responsible for all rendering logic of the application.
//! It takes the application state (`App`) and a `ratatui` frame, and draws the UI.

use crate::states::settings::{BackupsErr, BackupsInterval, RuntimeBackupsInterval, SelectedToggle, SettingsNotification, SettingsType, side_to_string};
use crate::states::{MapState, SettingsState, StartState};
use crate::states::map::{DiscardMenuType, ModalEditMode, Mode, Note, Notification, Side, SignedRect};
use crate::states::start::{FocusedInputBox, SelectedStartButton, ErrMsg};
use crate::utils::{calculate_path, Point, get_color_name_in_string};
use ratatui::layout::Margin;
use ratatui::style::Stylize;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Position},
    prelude::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Padding, Paragraph, BorderType, List, ListItem},
    text::{Span, Line},
    Frame
};

const IN_PROGRESS_CHARSET: [&str; 6] = ["━", "┃", "┏", "┓", "┗", "┛"];
const NORMAL_CHARSET: [&str; 6] = ["─", "│", "┌", "┐", "└", "┘"];

const PLAIN_JUNCTIONS: [&str; 4] = ["┴", "┬", "┤", "├"];
const THICK_JUNCTIONS: [&str; 4] = ["┻", "┳", "┫", "┣"];
const DOUBLE_JUNCTIONS: [&str; 4] = ["╩", "╦", "╣", "╠"];

pub fn render_start(frame: &mut Frame, start_state: &mut StartState) {
    // Clear the frame before drawing anything new.
    frame.render_widget(Clear, frame.area());

    // Determine the area to draw the text in
    let start_text_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(35),
            Constraint::Min(12),
            Constraint::Percentage(45),
            Constraint::Min(1),
        ]).split(frame.area());
    
    // Styling for buttons (whether selected or not)
    let create_select_style = SelectedStartButton::CreateSelect.get_style(&start_state.selected_button);
    let recent1_style = SelectedStartButton::Recent1.get_style(&start_state.selected_button);
    let recent2_style = SelectedStartButton::Recent2.get_style(&start_state.selected_button);
    let recent3_style = SelectedStartButton::Recent3.get_style(&start_state.selected_button);

    // "Recents:", "Error ... recents" message
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

    // "Paths" texts
    let (recent1_text, recent2_text, recent3_text) = if let Ok(recent_paths) = &start_state.recent_paths {
        (
            recent_paths.recent_path_1.as_ref().map_or(String::new(), |p| p.to_string_lossy().into_owned()),
            recent_paths.recent_path_2.as_ref().map_or(String::new(), |p| p.to_string_lossy().into_owned()),
            recent_paths.recent_path_3.as_ref().map_or(String::new(), |p| p.to_string_lossy().into_owned()),
        )
    } else {
        (String::new(), String::new(), String::new())
    };
    // Format them like so: [ /home/user/... ]
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

    // If entering a path for the map, also render this over everything else
    if start_state.input_path {
        // Set the area for the input menu block
        let input_menu_area = Layout::default() // Split the available area vertically
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Length(20),
                Constraint::Fill(1),
            ]).split(frame.area()); 
        let input_menu_area = Layout::default() // Split the previous again horizontally
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Length(70),
                Constraint::Fill(1),
            ]).split(input_menu_area[1]);
        
        // Split the area that's gonna cointain the input menu contents (text and input boxes)
        let input_menu_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min(2),
                Constraint::Length(1), // Text 1
                Constraint::Length(1), // Text 2
                Constraint::Length(3), // Input box 1 [3]
                Constraint::Length(2),
                Constraint::Length(1), // Text 3 [5]
                Constraint::Length(3), // Input box 2 [6]
                Constraint::Length(1),
                Constraint::Length(1), // Text 4 (Error msg) [8]
                Constraint::Min(2),
            ]).split(input_menu_area[1]);
        
        // Assign the areas for the input boxes
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


        // Create the text lines
        let text_line_1 = Paragraph::new(Line::from("Directory path for your map file:").alignment(Alignment::Center));
        let text_line_2 = Paragraph::new(Line::from("(Relative to your home directory, e.g. maps/):").alignment(Alignment::Center));
        let text_line_3 = Paragraph::new(Line::from("Enter the map name:").alignment(Alignment::Center));

        // Clear the area and draw the bordered block
        frame.render_widget(Clear, input_menu_area[1]);
        frame.render_widget(Block::bordered(), input_menu_area[1]);

        // Clear and update the help text area to input menu controls
        let info_text = Line::from("Esc - Cancel      Enter - confirm field").alignment(Alignment::Center);
        frame.render_widget(Clear, start_text_area[3]);
        frame.render_widget(info_text, start_text_area[3]);

        // Render the text
        frame.render_widget(text_line_1, input_menu_areas[1]);
        frame.render_widget(text_line_2, input_menu_areas[2]);
        frame.render_widget(text_line_3, input_menu_areas[5]);


        // Determine the block styles to use for input boxes
        let input_box_1_block = FocusedInputBox::InputBox1.get_style(&start_state.focused_input_box);
        let input_box_2_block = FocusedInputBox::InputBox2.get_style(&start_state.focused_input_box);

        // Create and render the input boxes
        if let Some(input_path_string) = &start_state.input_path_string {
            let input_box_1 = Paragraph::new(Line::from(input_path_string.as_str())).block(input_box_1_block);
            frame.render_widget(input_box_1, input_box_area_1[1]);
        }
        if let Some(input_path_name) = &start_state.input_path_name {
            let input_box_2 = Paragraph::new(Line::from(input_path_name.as_str())).block(input_box_2_block);
            frame.render_widget(input_box_2, input_box_area_2[1]);
        }

        // Draw the cursor for the selected input box
        match start_state.focused_input_box {
            FocusedInputBox::InputBox1 => {
                if let Some(input_path_string) = &start_state.input_path_string {
                    // x coordinate on screen of the input_box_area_1, 
                    //      +the length of the inputted string (path), +1 to clear border
                    let x = input_box_area_1[1].x + input_path_string.len() as u16 + 1;
                    // y coordinate on screen of the input_box_area_1, +1 to clear border
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

        // Display the error message if need to
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

/// Render the settings menu
pub fn render_settings(frame: &mut Frame, settings_state: &mut SettingsState) {

    // -- Error case --
    // If there was an error with using settings functionality -
    // render this and stop there.
    if let SettingsType::Default(_, error_message) = &settings_state.settings {
        if let Some(err_msg) = error_message {
            // Assign area for the settings error page
            let settings_error_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Fill(1),
                    Constraint::Length(1), // error message line 1
                    Constraint::Length(1), // error message line 2
                    Constraint::Length(1),
                    Constraint::Length(1), // keybinds 
                    Constraint::Fill(1),
                ])
                .split(frame.area());

            // Create the error and controls text
            let error_text1 = Line::from(Span::styled("There was an error with using the settings functionality:", Style::new().fg(Color::Red))).alignment(Alignment::Center);
            let error_text2 = match err_msg {
                ErrMsg::DirFind => Line::from(Span::styled("no home directory", Style::new().fg(Color::Red))).alignment(Alignment::Center),
                ErrMsg::DirCreate => Line::from(Span::styled("can't create config directory", Style::new().fg(Color::Red))).alignment(Alignment::Center),
                ErrMsg::FileWrite => Line::from(Span::styled("can't create settings file", Style::new().fg(Color::Red))).alignment(Alignment::Center),
                ErrMsg::FileRead => Line::from(Span::styled("can't read settings file", Style::new().fg(Color::Red))).alignment(Alignment::Center),
            };
            let settings_error_controls_text = Line::from("q - exit to start screen      o - go back to the map screen").alignment(Alignment::Center);

            // Render the error and controls text
            frame.render_widget(error_text1, settings_error_area[1]);
            frame.render_widget(error_text2, settings_error_area[2]);
            frame.render_widget(settings_error_controls_text, settings_error_area[4]);

            return; // Stop here
        }
    }

    // -- If context page toggled --
    if settings_state.context_page {
        // Assign area for the context page
        let context_page_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(40),
                Constraint::Length(1),
                Constraint::Length(1), // keybinds text
                Constraint::Length(3), // space to align with the settings menu
                Constraint::Fill(1),
            ])
            .split(frame.area());

        // Render the controls text, before splitting the area again
        let context_page_controls_text = Line::from("? / F1 - toggle context page").alignment(Alignment::Center);
        frame.render_widget(context_page_controls_text, context_page_area[3]);

        // Split the previous area for context page block
        let context_page_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(60),
                Constraint::Fill(1),
            ])
            .split(context_page_area[1]);

        // Render the bordered block (borders for the context page)
        frame.render_widget(Block::bordered(), context_page_area[1]);

        // Context page lines
        let context_page_lines = vec![
            Line::from("1. Map Changes Auto Save Interval"),
            Line::from("Automatically saves your map edits to the file at"),
            Line::from("regular intervals. This helps prevent data loss by "),
            Line::from("ensuring your recent changes are written to the file"),
            Line::from("periodically."),
            Line::from(""),
            Line::from("2. Backups Interval"),
            Line::from("Creates a backup copy of your map file each time you"),
            Line::from("open it, but only if enough time has passed since the"),
            Line::from("last backup. This protects against file corruption"),
            Line::from("and allows you to restore previous versions."),
            Line::from(""),
            Line::from("3. Runtime Backups Interval"),
            Line::from("(toggle only visible if backups enabled)"),
            Line::from("Creates periodic backups during long editing sessions"),
            Line::from("while the map file remains open. For example, if set"),
            Line::from("to 2 hours and you keep the application running for"),
            Line::from("several days, a backup will be created every 2 hours."),
            Line::from("This provides extra protection against data loss"),
            Line::from("during extended work sessions."),
            Line::from(""),
            Line::from("4. Default start side for making connections"),
            Line::from(""),
            Line::from("5. Default end side for making connections"),
        ];

        // Create a list widget to render
        let context_page_content: Vec<ListItem> = context_page_lines
            .into_iter()
            .map(ListItem::new)
            .collect();
        let context_page_content = List::new(context_page_content);

        // Render the page content
        frame.render_widget(context_page_content, context_page_area[1].inner(Margin::new(3, 3)));

        return; // Stop here
    }

    // -- Can use the settings functionality --
    // Assign area for the settings menu (split vertically)
    let settings_menu_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(40),
            Constraint::Length(1),
            Constraint::Length(1), // notification
            Constraint::Length(1),
            Constraint::Length(1), // optinal keybind hint
            Constraint::Length(1),
            Constraint::Length(1), // keybinds text 1
            Constraint::Length(1),
            Constraint::Length(1), // keybinds text 2
            Constraint::Fill(1),
        ])
        .split(frame.area());


    // Render the controls text, before splitting the area again
    let settings_screen_controls_text1 = Line::from("q - exit to start screen      o - go back to the map screen      s - save the settings").alignment(Alignment::Center);
    let settings_screen_controls_text2 = Line::from("Enter - toggle option      k / Up - go up       j / Down - go down       ? / F1 - toggle context page").alignment(Alignment::Center);

    frame.render_widget(settings_screen_controls_text1, settings_menu_area[7]);
    frame.render_widget(settings_screen_controls_text2, settings_menu_area[9]);


    // -- Render the notification if need to --
    if let Some(notification) = &settings_state.notification {
        // Create the notification text
        let notification_text = match notification {
            SettingsNotification::SaveSuccess => {
                Line::from(Span::styled("Settings saved successfully.", Style::new().fg(Color::Green))).alignment(Alignment::Center)
            }
            SettingsNotification::SaveFail => {
                Line::from(Span::styled("There was an error saving to settings file. (Write Error)", Style::new().fg(Color::Red))).alignment(Alignment::Center)
            }
        };

        // Render it
        frame.render_widget(notification_text, settings_menu_area[3]);

        // Render it only once
        settings_state.notification = None;
    }

    // -- Render a hint if need to --
    // Render a keybind hint if on backups toggle and backups are enabled
    if settings_state.settings.settings().backups_interval.is_some() && matches!(settings_state.selected_toggle, SelectedToggle::Toggle2) {
        let backups_toggle_hint = Line::from(String::from("Tab - to cycle backup intervals")).alignment(Alignment::Center);
        frame.render_widget(backups_toggle_hint, settings_menu_area[5]);
    }
    // Render a keybind hint if on runtime backups toogle and backups are enabled
    if settings_state.settings.settings().runtime_backups_interval.is_some() && matches!(settings_state.selected_toggle, SelectedToggle::Toggle3) {
        let runtime_backups_toggle_hint = Line::from(String::from("Tab - to cycle runtime backup intervals")).alignment(Alignment::Center);
        frame.render_widget(runtime_backups_toggle_hint, settings_menu_area[5]);
    }


    // Split the previous area (split horizontally)
    let settings_menu_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(60),
            Constraint::Fill(1),
        ])
        .split(settings_menu_area[1]);

    // Render the bordered block (borders for the settings menu)
    frame.render_widget(Block::bordered(), settings_menu_area[1]);


    // -- Create the text for toggles and toggles themselves --
    // (involves converting settings fields to different types 
    //      and figuring styles for toggles)

    // Toggle 1 - map changes save interval
    // Get the save interval value from the settings
    let toggle1_content = settings_state.settings.settings().save_interval;

    let toggle1_content_text = match toggle1_content {
        None => String::from("Disabled"),
        Some(interval) => format!("{} sec", interval),
    };
    // Determine it's style (whether it is selected or not)
    let toggle1_style = SelectedToggle::Toggle1.get_style(&settings_state.selected_toggle);


    // Toggle 2 - on load backups
    let toggle2_content = &settings_state.settings.settings().backups_interval;

    let toggle2_content_text = match toggle2_content {
        None => String::from("Disabled"),
        Some(BackupsInterval::Daily) => String::from("Daily"),
        Some(BackupsInterval::Every3Days) => String::from("Every 3 days"),
        Some(BackupsInterval::Weekly) => String::from("Weekly"),
        Some(BackupsInterval::Every2Weeks) => String::from("Every 2 weeks"),
    };
    // Determine it's style (whether it is selected or not)
    let toggle2_style = SelectedToggle::Toggle2.get_style(&settings_state.selected_toggle);


    // Toggle 3 - runtime backups (only visible if backups are enabled)
    // If backups enabled (which also enables runtime backups):
    let toggle3_line_text = if let Some(toggle3_content) = &settings_state.settings.settings().runtime_backups_interval {
        // Get the runtime backups set interval in String
        let toggle3_content_text = match toggle3_content {
            RuntimeBackupsInterval::Hourly => String::from("Hourly"),
            RuntimeBackupsInterval::Every2Hours => String::from("Every 2 hours"),
            RuntimeBackupsInterval::Every4Hours => String::from("Every 4 hours"),
            RuntimeBackupsInterval::Every6Hours => String::from("Every 6 hours"),
            RuntimeBackupsInterval::Every12Hours => String::from("Every 12 hours"),
        };
        // Determine it's style (whether it is selected or not)
        let toggle3_style = SelectedToggle::Toggle3.get_style(&settings_state.selected_toggle);
        // Create the line with appropriate styling for the toggle.
        vec![Span::raw("Runtime backups interval:  "), Span::styled(format!("{}", toggle3_content_text), toggle3_style)] 
    // If backups disabled - don't show the toggle.
    } else {
        vec![]
    };


    // Toggle 4 - default start side for making connections
    let toggle4_content_text = side_to_string(settings_state.settings.settings().default_start_side);
    // Determine it's style (whether it is selected or not)
    let toggle4_style = SelectedToggle::Toggle4.get_style(&settings_state.selected_toggle);


    // Toggle 5 - default end side for making connections
    let toggle5_content_text = side_to_string(settings_state.settings.settings().default_end_side);
    // Determine it's style (whether it is selected or not)
    let toggle5_style = SelectedToggle::Toggle5.get_style(&settings_state.selected_toggle);


    // Toggle 6 - Modal Editing for Edit Mode
    let toggle6_content_text = if settings_state.settings.settings().edit_modal {
        String::from("Enabled")
    } else {
        String::from("Disabled")
    };
    // Determine it's style (whether it is selected or not)
    let toggle6_style = SelectedToggle::Toggle6.get_style(&settings_state.selected_toggle);


    // Settings screen lines
    let settings_menu_content_lines = vec![
        Line::from(vec![Span::raw("Map changes auto save interval:  "), Span::styled(format!("{}", toggle1_content_text), toggle1_style)]),
        Line::from(""),
        Line::from(vec![Span::raw("Backups interval:  "), Span::styled(format!("{}", toggle2_content_text), toggle2_style)]),
        Line::from(""),
        Line::from(toggle3_line_text),
        Line::from(""),
        Line::from(vec![Span::raw("Default start side:  "), Span::styled(format!("{}", toggle4_content_text), toggle4_style)]),
        Line::from(""),
        Line::from(vec![Span::raw("Default end side:  "), Span::styled(format!("{}", toggle5_content_text), toggle5_style)]),
        Line::from(""),
        Line::from(vec![Span::raw("Modal Editing for Edit Mode:  "), Span::styled(format!("{}", toggle6_content_text), toggle6_style)]),
    ];

    // -- Rendering the toggles text and toggles themselves --
    // Create a list widget to render
    let settings_menu_content: Vec<ListItem> = settings_menu_content_lines
        .into_iter()
        .map(ListItem::new)
        .collect();
    let settings_menu_content = List::new(settings_menu_content);

    // Render the page content
    frame.render_widget(settings_menu_content, settings_menu_area[1].inner(Margin::new(3, 3)));

    // If entering a path for backups functionality - render this prompt over the menu.
    if settings_state.input_prompt {
        // Make a rectangular area for the input prompt
        let input_prompt_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(18),
                Constraint::Fill(1),
            ])
            .split(frame.area());
        let input_prompt_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(100),
                Constraint::Fill(1),
            ])
            .split(input_prompt_area[1]);
    
        // Clear the screen and render a bordered block (borders)
        frame.render_widget(Clear, frame.area());
        frame.render_widget(Block::bordered(), input_prompt_area[1]);

        // Input prompt lines
        let input_prompt_lines_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(2),
                Constraint::Length(5), // text1
                Constraint::Length(2),
                Constraint::Length(3), // input field
                Constraint::Length(1),
                Constraint::Length(1), // notification
                Constraint::Length(1),
                Constraint::Length(1), // keybinds
                Constraint::Min(2),
            ])
            .split(input_prompt_area[1]);
        let input_prompt_input_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(2),
                Constraint::Length(50),
                Constraint::Min(2),
            ])
            .split(input_prompt_lines_area[3]);

        // Text lines for the input prompt
        let input_prompt_text = vec![
            Line::from("Enter backups directory path").alignment(Alignment::Center),
            Line::from(""),
            Line::from("Empty field or directory name only - uses home directory as base path").alignment(Alignment::Center),
            Line::from("Path starting with / - uses absolute directory path").alignment(Alignment::Center),
            Line::from("Esc key - cancels path entry (if new) or removes existing path and disables backups").alignment(Alignment::Center)];
        // Creating a list widget
        let input_prompt_text: Vec<ListItem> = input_prompt_text.into_iter().map(ListItem::new).collect();
        let input_prompt_text = List::new(input_prompt_text);

        // Keybinds text for the input prompt
        let keybinds_text = Line::from("Esc - cancel          Enter - confirm path").alignment(Alignment::Center);

        // Render the text lines and the input bordered block
        frame.render_widget(input_prompt_text, input_prompt_lines_area[1]);
        frame.render_widget(keybinds_text, input_prompt_lines_area[7]);

        // Render the user entered string
        // .unwrap used here - because while in the input prompt - backups_path cannot be None 
        let user_input_path = Paragraph::new(Line::from(
            settings_state.settings.settings().backups_path.as_ref().unwrap().as_str()))
            .block(Block::bordered());
        frame.render_widget(user_input_path, input_prompt_input_area[1]);

        // Draw the cursor
        // x coordinate on screen of the input_prompt_input_area, +the length of the inputted string (path), +1 to clear border
        let cursor_x = input_prompt_input_area[1].x 
                        + settings_state.settings.settings().backups_path.as_ref().unwrap().len() as u16
                        + 1;
        // y coordinate on screen of the input_prompt_input_area, +1 to clear border
        let cursor_y = input_prompt_input_area[1].y + 1; 
        frame.set_cursor_position(Position::new(cursor_x, cursor_y));

        // Render error notification if need to
        if let Some(err) = &settings_state.input_prompt_err {
            match err {
                BackupsErr::DirFind => {
                    let err_text = Line::from("Error finding the home directory").fg(Color::Red).alignment(Alignment::Center);
                    frame.render_widget(err_text, input_prompt_lines_area[5]);
                }
                BackupsErr::DirCreate => {
                    let err_text = Line::from("Error creating backups directory").fg(Color::Red).alignment(Alignment::Center);
                    frame.render_widget(err_text, input_prompt_lines_area[5]);
                }
                BackupsErr::FileWrite => {
                    let err_text = Line::from("Error writing to the provided directory").fg(Color::Red).alignment(Alignment::Center);
                    frame.render_widget(err_text, input_prompt_lines_area[5]);
                }
            }
        }
    }

    // If attempted to exit without saving changes - show discard changes menu.
    if let Some(_) = &settings_state.confirm_discard_menu {
        // Define the area for the menu
        let confirm_discard_menu_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(8),
            ])
            .split(frame.area());

        // Clear the area and render an empty bordered block
        frame.render_widget(Clear, confirm_discard_menu_area[1]);
        frame.render_widget(Clear, confirm_discard_menu_area[2]);
        frame.render_widget(Block::bordered(), confirm_discard_menu_area[2]);

        // Define the text areas inside the menu area
        let confirm_discard_menu_text_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .split(confirm_discard_menu_area[2]);
        
        // Make the text itself
        let line_1 = Line::from("Exit without saving changes to settings?").alignment(Alignment::Center);
        let line_2 = Line::from(
            vec![
                Span::styled("[ ESC ] - Cancel", Style::new().fg(Color::Green)), 
                Span::raw("      "),
                Span::styled("[ q ] - Confirm discard and exit", Style::new().fg(Color::Red)), 

            ]).alignment(Alignment::Center);
        
        // Render the text
        frame.render_widget(line_1, confirm_discard_menu_text_areas[1]);
        frame.render_widget(line_2, confirm_discard_menu_text_areas[4]);
    }
}

pub fn render_map(frame: &mut Frame, map_state: &mut MapState) {
    // Clear the frame before drawing anything new.
    frame.render_widget(Clear, frame.area());

    // If help page toggled, show it and stop there.
    if let Some(page_number) = map_state.help_screen {
        render_map_help_page(frame, page_number);
        return;
    }

    // Update the  map_state with the current terminal size. This is crucial for
    // calculations that depend on screen dimensions, like centering new notes.
    map_state.screen_width = frame.area().width as usize;
    map_state.screen_height = frame.area().height as usize;

    // Render the main UI components.
    render_connections(frame, map_state);
    render_notes(frame, map_state); // Notes will be drawn over connections (if any)
    render_bar(frame, map_state); // The bar will be drawn over everything
}

pub fn render_map_help_page(frame: &mut Frame, page_number: usize) {
        // Assign the area for page number, page area and control text's
        let help_screen_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Page number
                Constraint::Fill(1), // Page area itself
                Constraint::Length(1), // Controls
            ])
            .split(frame.area());

        // Render the controls text (same for all pages)
        let help_screen_controls_text = Line::from("? / F1 - toggle help page        l / Right / Tab - go forward a page        h / Left - go back a page").alignment(Alignment::Center);
        frame.render_widget(help_screen_controls_text, help_screen_layout[2]);

        // Render the page number _
        match page_number {
            1 => {
                // Render the bordered block (borders)
                frame.render_widget(Block::bordered().border_style(Color::White), help_screen_layout[1]);
                
                // Render the page number
                let page_ind_1_text = Line::from("  Page 1/5: General");
                frame.render_widget(page_ind_1_text, help_screen_layout[0]);

                // Page content lines
                let page_1_content = vec![
                    Line::from(""),
                    Line::from("Page contents:"),
                    Line::from(""),
                    Line::from("1 - General"),
                    Line::from("2 - Normal Mode"),
                    Line::from("3 - Visual Mode"),
                    Line::from("4 - Visual (Move), Visual (Connection)"),
                    Line::from("5 - Edit Mode"),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from("General info:"),
                    Line::from(""),
                    Line::from("Cannot exit the app from the help screen,"),
                    Line::from("  can do so from the Map Screen, Normal Mode."),
                    Line::from(""),
                    Line::from(""),
                    Line::from("Terminal cells are coordinates."),
                    Line::from(""),
                    Line::from(""),
                    Line::from("There are only positive coordinates, think of it as a whiteboard"),
                    Line::from("starting from the top left corner and going right and bottom infinitely."),
                    Line::from(""),
                    Line::from("0,0 x-------->"),
                    Line::from("y"),
                    Line::from("|"),
                    Line::from("|"),
                    Line::from("|"),
                    Line::from("v"),
                    Line::from(""),
                    Line::from(""),
                    Line::from("Changes are automatically saved to the map file every 20 seconds."),
                    Line::from("You can adjust the auto-save interval or disable it in the settings menu."),
                    Line::from(""),
                    Line::from("If you make changes and try to quit before saving them / before the changes are"),
                    Line::from("automatically saved - you will be prompted to either cancel exiting or discard those changes."),
                ];

                // Create a list widget to render
                let page_1_content: Vec<ListItem> = page_1_content
                    .into_iter()
                    .map(ListItem::new)
                    .collect();
                let page_1_content = List::new(page_1_content);

                // Render the page content
                frame.render_widget(page_1_content, help_screen_layout[1].inner(Margin::new(3, 1)));
            }
            2 => {
                // Render the bordered block (borders)
                frame.render_widget(Block::bordered().border_style(Color::White), help_screen_layout[1]);
                
                // Render the page number
                let page_ind_2_text = Line::from("  Page 2/5: Normal Mode");
                frame.render_widget(page_ind_2_text, help_screen_layout[0]);
                
                // Page content lines
                let page_2_content = vec![
                    Line::from(""),
                    Line::from("General Commands"),
                    Line::from(""),
                    Line::from("F1 / ?: Toggle help screen"),
                    Line::from("q:      Quit to start screen (if saved) or show confirm discard menu"),
                    Line::from("s:      Save map file"),
                    Line::from("o:      Open the settings"),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from("Viewport Navigation"),
                    Line::from(""),
                    Line::from("h / Left Arrow:        Move viewport left by 1"),
                    Line::from("H / Shift+Left Arrow:  Move viewport left by 5"),
                    Line::from(""),
                    Line::from("j / Down Arrow:        Move viewport down by 1"),
                    Line::from("J / Shift+Down Arrow:  Move viewport down by 5"),
                    Line::from(""),
                    Line::from("k / Up Arrow:          Move viewport up by 1"),
                    Line::from("K / Shift+Up Arrow:    Move viewport up by 5"),
                    Line::from(""),
                    Line::from("l / Right Arrow:       Move viewport right by 1"),
                    Line::from("L / Shift+Right Arrow: Move viewport right by 5"),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from("Note Operations"),
                    Line::from(""),
                    Line::from("a: Add a new note"),
                    Line::from("v: Select closest note to the center of the screen"),
                    Line::from("     and switch to Visual Mode"),
                ];

                // Create a list widget to render
                let page_2_content: Vec<ListItem> = page_2_content
                    .into_iter()
                    .map(ListItem::new)
                    .collect();
                let page_2_content = List::new(page_2_content);

                // Render the page content
                frame.render_widget(page_2_content, help_screen_layout[1].inner(Margin::new(3, 1)));
            }
            3 => {
                // Render the bordered block (borders)
                frame.render_widget(Block::bordered().border_style(Color::Yellow), help_screen_layout[1]);
                
                // Render the page number
                let page_ind_3_text = Line::from(vec![Span::raw("  Page 3/5: "), Span::styled("Visual Mode", Style::new().fg(Color::Yellow))]);
                frame.render_widget(page_ind_3_text, help_screen_layout[0]);

                // Page content lines
                let page_3_content = vec![
                    Line::from(""),
                    Line::from("General Commands"),
                    Line::from(""),
                    Line::from("ESC: Switch back to Normal mode"),
                    Line::from("i:   Switch to Edit mode"),
                    Line::from("m:   Switch to Move state"),
                    Line::from("c:   Switch to Connection state (edit existing connection(s))"),
                    Line::from("C:   Add a new connection from the selected note"),
                    Line::from("d:   Choose the selected note for deletion."),
                    Line::from("       (brings up the confirm to delete prompt)"),
                    Line::from("e:   Cycle through note colors"),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from("Note Focus Switching"),
                    Line::from(""),
                    Line::from("h / Left Arrow:  Switch focus to note on the left"),
                    Line::from("j / Down Arrow:  Switch focus to note below"),
                    Line::from("k / Up Arrow:    Switch focus to note above"),
                    Line::from("l / Right Arrow: Switch focus to note on the right"),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from("NOTE: Focus switching uses directional logic - notes must be primarily"),
                    Line::from("in the intended direction to be selectable. Meaning that sometimes you "),
                    Line::from("might not be able to switch focus to a particular note. In that case "),
                    Line::from("either move the view to be centered with the note and reselect, or move"),
                    Line::from("the note currently selected to a different spot and try to switching"),
                    Line::from("focus from it again."),
                ];

                // Create a list widget to render
                let page_3_content: Vec<ListItem> = page_3_content
                    .into_iter()
                    .map(ListItem::new)
                    .collect();
                let page_3_content = List::new(page_3_content);

                // Render the page content
                frame.render_widget(page_3_content, help_screen_layout[1].inner(Margin::new(3, 1)));
            }
            4 => {
                // Split the page area in two
                let help_page_4_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ])
                    .split(help_screen_layout[1]);
                // Render the two bordered blocks (borders)
                frame.render_widget(Block::bordered().border_style(Color::Yellow), help_page_4_layout[0]);
                frame.render_widget(Block::bordered().border_style(Color::Yellow), help_page_4_layout[1]);
                
                // Render the page number
                let page_ind_4_text = Line::from(vec![Span::raw("  Page 4/5: "), Span::styled("Visual (Move), Visual (Connection)", Style::new().fg(Color::Yellow))]);
                frame.render_widget(page_ind_4_text, help_screen_layout[0]);

                // Left page content lines
                let page_4_content_left = vec![
                    Line::from(""),
                    Line::from("Visual (Move)"),
                    Line::from(""),
                    Line::from(""),
                    Line::from("m:   Switch back to Visual mode normal state"),
                    Line::from("ESC: Switch back to Normal mode"),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from("h / Left Arrow:        Move note left by 1"),
                    Line::from("H / Shift+Left Arrow:  Move note left by 5"),
                    Line::from(""),
                    Line::from("j / Down Arrow:        Move note down by 1"),
                    Line::from("J / Shift+Down Arrow:  Move note down by 5"),
                    Line::from(""),
                    Line::from("k / Up Arrow:          Move note up by 1"),
                    Line::from("K / Shift+Up Arrow:    Move note up by 5"),
                    Line::from(""),
                    Line::from("l / Right Arrow:       Move note right by 1"),
                    Line::from("L / Shift+Right Arrow: Move note right by 5"),
                ];

                // Create a list widget to render
                let page_4_content_left: Vec<ListItem> = page_4_content_left
                    .into_iter()
                    .map(ListItem::new)
                    .collect();
                let page_4_content_left = List::new(page_4_content_left);

                // Render left page content
                frame.render_widget(page_4_content_left, help_page_4_layout[0].inner(Margin::new(3, 1)));


                // Right page content lines
                let page_4_content_right = vec![
                    Line::from(""),
                    Line::from("Visual (Connection)"),
                    Line::from(""),
                    Line::from(""),
                    Line::from("c: Switch back to Visual mode normal state"),
                    Line::from("r: Rotate connection start/end side"),
                    Line::from("n: Cycle through available connections on this note"),
                    Line::from("d: Delete selected connection"),
                    Line::from("e: Cycle through connection colors"),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from("Target Note Selection (selecting connection endpoint)"),
                    Line::from(""),
                    Line::from("h / Left Arrow:  Switch focus to note on the left"),
                    Line::from("j / Down Arrow:  Switch focus to note below"),
                    Line::from("k / Up Arrow:    Switch focus to note above"),
                    Line::from("l / Right Arrow: Switch focus to note on the right"),
                ];

                // Create a list widget to render
                let page_4_content_right: Vec<ListItem> = page_4_content_right
                    .into_iter()
                    .map(ListItem::new)
                    .collect();
                let page_4_content_right = List::new(page_4_content_right);

                // Render right page content
                frame.render_widget(page_4_content_right, help_page_4_layout[1].inner(Margin::new(3, 1)));
            }
            5 => {
                // Render the bordered block (borders)
                frame.render_widget(Block::bordered().border_style(Color::Blue), help_screen_layout[1]);
                
                // Render the page number
                let page_ind_5_text = Line::from(vec![Span::raw("  Page 5/5: "), Span::styled("Edit Mode", Style::new().fg(Color::Blue))]);
                frame.render_widget(page_ind_5_text, help_screen_layout[0]);

                // Page content lines
                let page_5_content = vec![
                    Line::from(""),
                    Line::from("Edit Mode (text editing)"),
                    Line::from(""),
                    Line::from(""),
                    Line::from("ESC:           Switch back to Normal mode"),
                    Line::from("Any character: Insert character at cursor position"),
                    Line::from("Enter:         Insert newline at cursor position"),
                    Line::from("Backspace:     Delete character before cursor"),
                    Line::from("Left Arrow:    Move cursor left"),
                    Line::from("Right Arrow:   Move cursor right"),
                    Line::from("Up Arrow:      Move cursor up one line"),
                    Line::from("Down Arrow:    Move cursor down one line"),
                ];

                // Create a list widget to render
                let page_5_content: Vec<ListItem> = page_5_content
                    .into_iter()
                    .map(ListItem::new)
                    .collect();
                let page_5_content = List::new(page_5_content);

                // Render page content
                frame.render_widget(page_5_content, help_screen_layout[1].inner(Margin::new(3, 1)));
            }
            _ => {}
        }
}

/// Renders the bottom information bar.
///
/// This bar displays debugging information and the current application state,
/// such as viewport position, mode, and selected note.
fn render_bar(frame: &mut Frame, map_state: &mut MapState) {

    // Get the total available screen area.
    let size = frame.area();

    // Determine the display text and color for the current application mode.
    let (mode_text, mode_text_color) = match &map_state.current_mode {
        Mode::Normal => (String::from("[ NORMAL ]"), Style::new().fg(Color::White)),
        Mode::Visual => {
            if map_state.visual_move {
                (String::from("[ VISUAL (MOVE) ]"), Style::new().fg(Color::Yellow))
            } else if map_state.visual_connection {
                (String::from("[ VISUAL (CONNECTION) ]"), Style::new().fg(Color::Yellow))
            } else {
                (String::from("[ VISUAL ]"), Style::new().fg(Color::Yellow))
            }
        }
        Mode::Edit(modal) => (
            match modal {
                None => String::from("[ EDIT ]"),
                Some(ModalEditMode::Normal) => String::from("[ EDIT (NORMAL) ]"),
                Some(ModalEditMode::Insert) => String::from("[ EDIT (INSERT) ]"),
            },
            Style::new().fg(Color::Blue)),
        Mode::Delete => (String::from("[ DELETE ]"), Style::new().fg(Color::Red)),
    };

    // --- Left-Aligned Widget: Mode Display ---
    // Create a Paragraph for the mode, styling it with the color determined above.
    // It's aligned to the left and given some padding.
    let mode_display = Paragraph::new(format!("{}", mode_text))
        .style(mode_text_color)
        .alignment(Alignment::Left)
        .block(Block::default().padding(Padding::new(2, 0, 0, 0)));

    // --- Right-Aligned Widget: View Position ---
    // Create a separate Paragraph to show the viewport's x/y coordinates.
    // This is aligned to the right, with padding on the right side.
    let view_position_display = Paragraph::new(format!(
        "View: {},{}",
        map_state.view_pos.x,
        map_state.view_pos.y,
    ))
    .alignment(Alignment::Right)
    .block(Block::default().padding(Padding::new(0, 2, 0, 0)));
    

    // --- Layout Management ---

    // Define the rectangular area for the entire bottom bar.
    let bar_area = Rect {
        x: size.x,
        y: size.height - 3, // Position it in the last three rows of the terminal.
        width: size.width,
        height: 3,
    };

    // Split the bar area into two set of rows
    let bar_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // 1 empty space row
            Constraint::Length(1), // 1 cell row
            Constraint::Length(1), // 1 cell row
        ])
        .split(bar_area);

    // Split the `bar_rows` into horizontal chunks.
    let row_1_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1), // Split the rest of the area in 2 sides
            Constraint::Min(70), // Middle area gets at least 70 cells
            Constraint::Fill(1), // Split the rest of the area in 2 sides
        ])
        .split(bar_rows[1]);
    let row_2_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1), // Split the rest of the area in 2 sides
            Constraint::Min(70), // Middle area gets at least 70 cells
            Constraint::Fill(1), // Split the rest of the area in 2 sides
        ])
        .split(bar_rows[2]);
    

    // Clear the area and render status widgets to the frame.
    frame.render_widget(Clear, bar_area); // First, clear the entire bar area.
    frame.render_widget(mode_display, row_2_areas[0]); // Render the mode display in the left chunk.
    frame.render_widget(view_position_display, row_2_areas[2]); // Render the position in the right chunk.

    // Render the confirm delete note prompt if need to  (Delete "Mode")
    if let Mode::Delete = &map_state.current_mode {
        let delete_note_prompt = Line::from(Span::styled(
            String::from("d - Delete the selected note          Esc - Go back to Visual Mode"),
            Style::new().fg(Color::Red)
            ));
        
        frame.render_widget(delete_note_prompt, row_2_areas[1]);
    }

    // (In Visual Mode only) 
    // -- Middle-Aligned Widget: Color currently set for the selected note/connection --
    if map_state.current_mode == Mode::Visual {
                
        let mut current_color_text = String::from("");
        let mut current_color_name = String::from("");
        let mut current_color = Color::White;

        if let Some(selected_note) = &map_state.selected_note {
            if let Some(focused_connection) = &map_state.focused_connection {
                current_color_text = String::from("Selected connection color: ");
                current_color_name = get_color_name_in_string(focused_connection.color);
                current_color = focused_connection.color;
            } else {
                if let Some(note) = map_state.notes.get(selected_note) {
                    current_color_text = String::from("Selected note color: ");
                    current_color_name = get_color_name_in_string(note.color);
                    current_color = note.color;
                }
            }
        }

        let current_color_widget = Line::from(vec![ 
            Span::from(current_color_text), 
            Span::styled(current_color_name, Style::new().fg(current_color))
        ]).alignment(Alignment::Center);

        frame.render_widget(current_color_widget, row_2_areas[1]);
    }

    // Whether to render a notification, that something went wrong with using
    // the settings functionality.
    if let Some(err_msg) = &map_state.settings_err_msg {
        // Create the error message text line.
        let settings_err_msg = match err_msg {
            ErrMsg::DirFind => Line::from(Span::styled("Settings error: no home directory - using defaults.", Style::new().fg(Color::Red))).alignment(Alignment::Center),
            ErrMsg::DirCreate => Line::from(Span::styled("Settings error: can't create config directory - using defaults.", Style::new().fg(Color::Red))).alignment(Alignment::Center),
            ErrMsg::FileWrite => Line::from(Span::styled("Settings error: can't create settings file - using defaults.", Style::new().fg(Color::Red))).alignment(Alignment::Center),
            ErrMsg::FileRead => Line::from(Span::styled("Settings error: can't read settings file - using defaults.", Style::new().fg(Color::Red))).alignment(Alignment::Center),
        };
        
        // Render the error message once
        frame.render_widget(settings_err_msg, row_1_areas[1]);

        // Reset to show settings error message
        map_state.settings_err_msg = None;
    }

    // Render a notification message if need to
    if let Some(notification) = &map_state.show_notification {
        // Render the corresponding notification message once
        match notification {
            Notification::SaveSuccess => {
                let notification_message = Line::from("Map file saved successfully").fg(Color::Green).alignment(Alignment::Center);
                frame.render_widget(notification_message, row_2_areas[1]);
            }
            Notification::SaveFail => {
                let notification_message = Line::from("Error saving the map file").fg(Color::Red).alignment(Alignment::Center);
                frame.render_widget(notification_message, row_2_areas[1]);
            }
            Notification::BackupSuccess => {
                let notification_message = Line::from("Backup file made successfully").fg(Color::Green).alignment(Alignment::Center);
                frame.render_widget(notification_message, row_2_areas[1]);
            }
            Notification::BackupFail => {
                let notification_message = Line::from("Error saving backup file").fg(Color::Red).alignment(Alignment::Center);
                frame.render_widget(notification_message, row_2_areas[1]);
            }
            Notification::BackupRecordFail => {
                let notification_message = Line::from("Backup created successfully, but failed to update backup records").fg(Color::Red).alignment(Alignment::Center);
                frame.render_widget(notification_message, row_2_areas[1]);
            }
        };

        // Reset what notification to show
        map_state.show_notification = None;
    }

    // Render a confirmation menu to discard changes if need to
    if let Some(discard_menu_type) = &map_state.confirm_discard_menu {
        // Define the area for the menu
        let confirm_discard_menu_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(8),
            ])
            .split(frame.area());

        // Clear the area and render an empty bordered block
        frame.render_widget(Clear, confirm_discard_menu_area[1]);
        frame.render_widget(Clear, confirm_discard_menu_area[2]);
        frame.render_widget(Block::bordered(), confirm_discard_menu_area[2]);

        // Define the text areas for inside the menu area
        let confirm_discard_menu_text_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .split(confirm_discard_menu_area[2]);
        
        match discard_menu_type {
            DiscardMenuType::Start => {
                // Make the text itself
                let line_1 = Line::from("Discard unsaved changes to this map?").alignment(Alignment::Center);
                let line_2 = Line::from(
                    vec![
                        Span::styled("[ ESC ] - Cancel", Style::new().fg(Color::Green)), 
                        Span::raw("      "),
                        Span::styled("[ q ] - Confirm discard and exit", Style::new().fg(Color::Red)), 

                    ]).alignment(Alignment::Center);
        
                // Render the text
                frame.render_widget(line_1, confirm_discard_menu_text_areas[1]);
                frame.render_widget(line_2, confirm_discard_menu_text_areas[4]);
            }
            DiscardMenuType::Settings => {
                // Make the text itself
                let line_1 = Line::from("Discard unsaved changes to this map and go to settings?").alignment(Alignment::Center);
                let line_2 = Line::from("(You must save changes or discard them before you can open the settings menu)").alignment(Alignment::Center);
                let line_3 = Line::from(
                    vec![
                        Span::styled("[ ESC ] - Cancel", Style::new().fg(Color::Green)), 
                        Span::raw("      "),
                        Span::styled("[ q ] - Confirm discard and go to settings", Style::new().fg(Color::Red)), 

                    ]).alignment(Alignment::Center);
        
                // Render the text
                frame.render_widget(line_1, confirm_discard_menu_text_areas[1]);
                frame.render_widget(line_2, confirm_discard_menu_text_areas[2]);
                frame.render_widget(line_3, confirm_discard_menu_text_areas[4]);
            }
        }
    }
}

/// Renders the main canvas where notes are displayed.
///
/// This function iterates through all notes and performs a series of calculations
/// to determine if, where, and how each note should be rendered.
fn render_notes(frame: &mut Frame, map_state: &mut MapState) {
    // -- Render the notes in render order --
    for &note_id in &map_state.render_order {
        if let Some(note) = map_state.notes.get(&note_id) {
            // --- 1. Get Note Dimensions ---
            let (mut note_width, mut note_height) = note.get_dimensions();
            // Enforce a minimum size for readability.
            if note_width < 20 { note_width = 20; }
            if note_height < 4 { note_height = 4; } 
            // Add space for cursor
            note_width+=1;

            // --- 2. Translate to Screen Coordinates ---
            // Convert the note's absolute canvas coordinates into screen-relative coordinates.
            // This can result in negative values if the note is partially off-screen.
            let note_rect = SignedRect {
                x: note.x as isize - map_state.view_pos.x as isize,
                y: note.y as isize - map_state.view_pos.y as isize,
                width: note_width as isize,
                height: note_height as isize,
            };

            // The frame itself is a rectangle starting at (0,0) in screen space.
            let frame_rect = SignedRect {
                x: 0,
                y: 0,
                width: frame.area().width as isize,
                height: frame.area().height as isize,
            };

            // --- 3. Clipping ---
            // Calculate the intersection between the note and the frame. If there's no
            // overlap, `intersection_result` will be `None`, and drawing the note will be skipped.
            let intersection_result = note_rect.intersection(&frame_rect);
            if let Some(visible_part) = intersection_result {
                // Convert the clipped, screen-space rectangle back to a `ratatui::Rect`.
                // The coordinates are guaranteed to be non-negative at this point.
                let note_area = Rect::new(
                    visible_part.x as u16,
                    visible_part.y as u16,
                    visible_part.width as u16,
                    visible_part.height as u16,
                );

                // --- 4. Calculate Text Scrolling ---
                // If the note was clipped on the top or left, we need to scroll the text content
                // to show the correct portion.
                let horizontal_scroll = (visible_part.x - note_rect.x) as u16;
                let vertical_scroll = (visible_part.y - note_rect.y) as u16;

                // --- 5. Determine Dynamic Borders ---
                // Show only the borders for the sides of the note that are not clipped.
                let mut borders = Borders::NONE;
                if note_rect.x == visible_part.x {
                    borders |= Borders::LEFT;
                }
                if note_rect.x + note_rect.width == visible_part.x + visible_part.width {
                    borders |= Borders::RIGHT;
                }
                if note_rect.y == visible_part.y {
                    borders |= Borders::TOP;
                }
                if note_rect.y + note_rect.height == visible_part.y + visible_part.height {
                    borders |= Borders::BOTTOM;
                }

                // --- 6. Determine border color  ---
                // (based on selection and mode)
                let border_color = if note.selected {
                    match map_state.current_mode {
                        Mode::Normal => Color::White,
                        Mode::Visual => Color::Yellow,
                        Mode::Edit(_) => Color::Blue,
                        Mode::Delete => Color::Red,
                    }
                } else {
                    note.color
                };

                // Determine border type
                let border_type = if note.selected {
                    match map_state.current_mode {
                        Mode::Normal => BorderType::Plain,
                        Mode::Visual => BorderType::Thick,
                        Mode::Edit(_) => BorderType::Double,
                        Mode::Delete => BorderType::Rounded,
                    }
                } else {
                    BorderType::Plain
                };

                // Create a block for the note with borders.
                let block = Block::default()
                    .borders(borders)
                    .border_style(border_color)
                    .border_type(border_type);

                // --- 7. Create the widget itself ---
                let text_widget = Paragraph::new(note.content.as_str())
                    .scroll((vertical_scroll, horizontal_scroll))
                    .block(block);

                // --- 8. Render the Widget(s) ---
                // With a stable rendering order, we can now clear any content 
                // from notes rendered beneath the note to be drawn and then draw it.
                frame.render_widget(Clear, note_area);
                frame.render_widget(text_widget, note_area);

                // -- 9. Render the cursor if in Edit Mode on the selected note ---
                // This logic only runs if the app is in Edit Mode AND the note currently being
                // drawn is the one that's actively selected.
                if let Some(selected_note) = &map_state.selected_note {
                    if matches!(map_state.current_mode, Mode::Edit(_)) && note_id == *selected_note {

                        // To calculate the cursor's position, we first need a slice of the text
                        // from the beginning of the note's content up to the cursor's byte index.
                        let text_before_cursor = &note.content[..map_state.cursor_pos];

                        // --- Calculate cursor's position RELATIVE to the text inside the note ---

                        // The Y position (row) is the number of newline characters before the cursor.
                        let cursor_y_relative = text_before_cursor.matches('\n').count();

                        // The X position (column) is the number of characters since the last newline.
                        let cursor_x_relative = match text_before_cursor.rfind('\n') {
                            // If a newline is found, the X position is the number of characters
                            // between that newline and the cursor. `c+1` to skip the newline itself.
                            Some(c) => {
                                text_before_cursor[c+1..map_state.cursor_pos].chars().count()
                            }
                            // If no newline is found, we're on the first line. The X position is
                            // simply the total number of characters before the cursor.
                            None => { 
                                text_before_cursor[0..map_state.cursor_pos].chars().count()
                            }
                        };

                        // --- Check if the calculated position is VISIBLE on screen ---
                        // The cursor is only visible if its calculated row is within the scrolled view.
                        // `cursor_y_relative` must be at or after the `vertical_scroll` offset.
                        // It must also be within the visible height of the note area.
                        // `note_area.height - 2` accounts for the top and bottom borders.
                        if cursor_y_relative >= vertical_scroll as usize 
                           && cursor_y_relative <= (note_area.height - 2) as usize {

                            // --- Translate relative coordinates to absolute screen coordinates ---
                            let final_cursor_x = note_area.x as usize // Start at the note's visible edge
                                + 1                                   // Add 1 for the left border
                                + cursor_x_relative                   // Add the cursor's column in the text
                                - horizontal_scroll as usize;         // Subtract any horizontal text scrolling

                            let final_cursor_y = note_area.y as usize // Start at the note's visible edge
                                + 1                                   // Add 1 for the top border
                                + cursor_y_relative                   // Add the cursor's row in the text
                                - vertical_scroll as usize;           // Subtract any vertical text scrolling

                            // Finally, place the cursor at the calculated position in the frame.
                            frame.set_cursor_position(Position::new(final_cursor_x as u16, final_cursor_y as u16));
                        }
                    }
                }

                // -- 10. Render this note's connecting characters --
                // To fix the visual bug where connection characters were drawn over the notes "above",
                // this logic now runs *after* each note is drawn. It looks up the note's id in the
                // connection_index hash map and loops through the connections associated with that id.
                // Draws the appropriate character (`┬`, `┴`, etc.) on top of the border.
                // This entire block is inside the `if let` for visible notes as a key
                // optimization, avoiding any of this work for off-screen notes.

                // Get the connections associated with the note's id
                if let Some(connection_vec) = map_state.connection_index.get(&note_id) {
                    // Loop through the connections in the connection vector that are 
                    // associated with the note's id
                    // NOTE: if there are multiple connections to the same side - it draws
                    //        the connecting character that many times (not a performance issue whatsoever tho)
                    for connection in connection_vec {
                        if note_id == connection.from_id {
                            draw_connecting_character(note, connection.from_side, false, border_color, frame, map_state);
                        } else {
                            draw_connecting_character(note, connection.to_side.unwrap(), false, border_color, frame, map_state);
                        }
                    }
                }
            }
        }
    }

    // Render the start/end point for the "in progress" connection, if any
    if let Some(connection) = &map_state.focused_connection {
    
        if let Some(start_note) = map_state.notes.get(&connection.from_id){
            draw_connecting_character(start_note, connection.from_side, true, Color::Yellow, frame, map_state);

            if let Some(end_note_id) = connection.to_id {
                if let Some(end_note) = map_state.notes.get(&end_note_id) {
                    draw_connecting_character(end_note, connection.to_side.unwrap(), true, Color::Yellow, frame, map_state);
                }
            }
        }
    }
}

fn render_connections(frame: &mut Frame, map_state: &mut MapState) {

    for connection in &map_state.connections {
        if let Some(start_note) = map_state.notes.get(&connection.from_id){
            if let Some(end_note_id) = connection.to_id {
                if let Some(end_note) = map_state.notes.get(&end_note_id) {
                    let path = calculate_path(
                        start_note, 
                        connection.from_side, 
                        end_note, 
                        connection.to_side.unwrap(), // unwrap here, since if there is an
                                                     // end note - there is an end side
                    );

                    // For optimization, quickly check if the connection is visible before attempting
                    // to draw it. This avoids iterating over every cell of connections that are
                    // completely off-screen.
                    // The `.any()` iterator is efficient, stopping as soon as the first visible
                    // point is found.
                    let is_visible = path.iter().any(|point| {
                        let p_x = point.x - map_state.view_pos.x as isize;
                        let p_y = point.y - map_state.view_pos.y as isize;
                        p_x >= 0 && p_x < frame.area().width as isize && p_y >= 0 && p_y < frame.area().height as isize
                    });

                    // If no points in the path are within the visible screen area, skip
                    // the expensive drawing logic and move to the next connection.
                    if !is_visible {
                        continue
                    }

                    draw_connection(path, false, connection.color, frame, map_state);
                }
            }
        }
    }
        
    // Render the "in progress" connection, if any
    if let Some(focused_connection) = &map_state.focused_connection {

        if let Some(start_note) = map_state.notes.get(&focused_connection.from_id){

            if let Some(end_note_id) = focused_connection.to_id {
                if let Some(end_note) = map_state.notes.get(&end_note_id) {
                    let path = calculate_path(
                        start_note, 
                        focused_connection.from_side, 
                        end_note, 
                        focused_connection.to_side.unwrap(), // unwrap here, since if there is an
                                                             // end note - there is an end side
                    );

                    draw_connection(path, true, Color::Yellow, frame, map_state);
                }
            }
        }
    }
}

// `in_progess` is a bool argument for whether it is the "in progress" (of making/editing)
// connection being drawn.
fn draw_connection(path: Vec<Point>, in_progress: bool, color: Color, frame: &mut Frame, map_state: &MapState) {
    let connection_charset = if in_progress {
        &IN_PROGRESS_CHARSET
    } else {
        &NORMAL_CHARSET
    };

    // Draw the horizontal and vertical line segments that make
    // up a connection (.windows(2) - 2 points that make up a line)
    for points in path.windows(2) {
        // Translate first point absolute coordinates to screen coordinates
        let p1_x = points[0].x - map_state.view_pos.x as isize;
        let p1_y = points[0].y - map_state.view_pos.y as isize;

        // -- Determine line characters and draw them --
        
        // If the difference is in x coordinates - draw horizontal segment characters
        if points[0].x != points[1].x {
            let x_diff = (points[1].x - points[0].x).abs(); // difference in point 1 to point 2
            let mut x_coor: isize;
            
            for offset in 0..x_diff {
                if points[1].x > points[0].x { // +difference (going right)
                    x_coor = p1_x + offset;
                } else { // -difference (going left)
                    x_coor = p1_x - offset;
                }

                if x_coor >= 0 && x_coor < frame.area().width as isize && p1_y >= 0 && p1_y < frame.area().height as isize {
                    if let Some(cell) = frame.buffer_mut().cell_mut((x_coor as u16, p1_y as u16)) {
                        cell.set_symbol(connection_charset[0])
                            .set_fg(color);
                    }
                }
            }
        } else { // If the difference is in y coordinates - draw vertical segment characters                        
            let y_diff = (points[1].y - points[0].y).abs(); // difference in point 1 to point 2
            let mut y_coor: isize;

            for offset in 0..y_diff {
                if points[1].y > points[0].y { // +difference (going down)
                    y_coor = p1_y + offset;
                } else { // -difference (going up)
                    y_coor = p1_y - offset;
                }
                
                if y_coor >= 0 && y_coor < frame.area().height as isize && p1_x >= 0 && p1_x < frame.area().width as isize {
                    if let Some(cell) = frame.buffer_mut().cell_mut((p1_x as u16, y_coor as u16)) {
                        cell.set_symbol(connection_charset[1])
                            .set_fg(color);
                    }
                }
            }
        }
    }

    // -- Determine segment directions --
    // (p1->p2, p2->p3, ...)
    // Used later to determine corner characters (┌ ┐ └ ┘)
    let mut segment_directions: Vec<SegDir> = vec![];

    for points in path.windows(2) {
        if points[0].x != points[1].x { // horizontal difference
            if points[1].x > points[0].x { // +difference (going right)
                segment_directions.push(SegDir::Right);
            } else { // -difference (going left)
                segment_directions.push(SegDir::Left);
            }
        } else if points[0].y != points[1].y { // vertical difference
            if points[1].y > points[0].y { // +difference (going down)
                segment_directions.push(SegDir::Down);
            } else { // -difference (going up)
                segment_directions.push(SegDir::Up);
            } 
        } else { // no difference, difference on the same axis
            if let Some(last_direction) = segment_directions.last() {
                segment_directions.push(*last_direction); 
            }
        }
    }

    // -- Draw the corner characters for segments --
    for (i, points) in path.windows(3).enumerate() {
        // Translate points absolute coordinates to screen coordinates
        // points[1] - to draw every 2nd point, so all besides the first and last [1, 0, 0, 0, 1]
        let p_x = points[1].x - map_state.view_pos.x as isize;
        let p_y = points[1].y - map_state.view_pos.y as isize;

        let incoming = segment_directions[i];
        let outgoing = segment_directions[i + 1];
        
        let corner_character = match (incoming, outgoing) {
            // ┌
            (SegDir::Left, SegDir::Down) => { connection_charset[2] }
            (SegDir::Up, SegDir::Right) => { connection_charset[2] }
            // ┐
            (SegDir::Right, SegDir::Down) => { connection_charset[3] }
            (SegDir::Up, SegDir::Left) => { connection_charset[3] }
            // └
            (SegDir::Down, SegDir::Right) => { connection_charset[4] }
            (SegDir::Left, SegDir::Up) => { connection_charset[4] }
            // ┘
            (SegDir::Down, SegDir::Left) => { connection_charset[5] }
            (SegDir::Right, SegDir::Up) => { connection_charset[5] }
            // ─
            (SegDir::Left, SegDir::Left) => { connection_charset[0] }
            (SegDir::Left, SegDir::Right) => { connection_charset[0] }
            (SegDir::Right, SegDir::Right) => { connection_charset[0] }
            (SegDir::Right, SegDir::Left) => { connection_charset[0] }
            // │
            (SegDir::Up, SegDir::Up) => { connection_charset[1] }
            (SegDir::Up, SegDir::Down) => { connection_charset[1] }
            (SegDir::Down, SegDir::Down) => { connection_charset[1] }
            (SegDir::Down, SegDir::Up) => { connection_charset[1] }
        };

        if p_x >= 0 && p_x < frame.area().width as isize && p_y >= 0 && p_y < frame.area().height as isize {
            if let Some(cell) = frame.buffer_mut().cell_mut((p_x as u16, p_y as u16)) {
                cell.set_symbol(corner_character)
                    .set_fg(color);
            }
        }
    }
}

// `is_editing` argument is to determine whether the function is called from the
// block that is responosible for drawing the "in progress" connection (being made or edited)
fn draw_connecting_character(note: &Note, side: Side, is_editing: bool, color: Color, frame: &mut Frame, map_state: &MapState) {
    // Set of connection characters for the selected note (depends on the current_mode)
    let connection_charset = if note.selected || is_editing {
        match map_state.current_mode {
            Mode::Visual => &THICK_JUNCTIONS,
            Mode::Edit(_) => &DOUBLE_JUNCTIONS,
            // For Normal and Delete, we use the plain set
            _ => &PLAIN_JUNCTIONS,
        }
    } else { // Default set of connection characters (if note or the connection is not selected)
        &PLAIN_JUNCTIONS
    };

    let connection_point_character = match side {
        Side::Top => connection_charset[0],
        Side::Bottom => connection_charset[1],
        Side::Left => connection_charset[2],
        Side::Right => connection_charset[3],
    };

    let p = note.get_connection_point(side);
    let p_x = p.0 as isize - map_state.view_pos.x as isize; // connection start point relative x
    let p_y = p.1 as isize - map_state.view_pos.y as isize; // connection start point relative y

    if p_x >= 0 && p_x < frame.area().width as isize && p_y >= 0 && p_y < frame.area().height as isize {
        if let Some(cell) = frame.buffer_mut().cell_mut((p_x as u16, p_y as u16)) {
            cell.set_symbol(connection_point_character)
                .set_fg(color);
        }
    }
}

// Which direction the segment is going
// Used to determine which corner character to draw
#[derive(Copy, Clone)]
enum SegDir {
    Right, // Horizontal going Right
    Left, // Horizontal going Left
    Up, // Vertical going Up
    Down, // Vertical going Down
}
