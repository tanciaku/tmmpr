
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Position, Margin},
    style::{Stylize, Color, Style},
    widgets::{Block, Clear, Paragraph, List, ListItem},
    text::{Span, Line},
};

use crate::{
    states::{
        SettingsState,
        start::ErrMsg,
        settings::{BackupsErr, BackupsInterval, RuntimeBackupsInterval, SelectedToggle, SettingsNotification, SettingsType, side_to_string},
    },
};

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
            Line::from(""),
            Line::from("6. Modal Edit Mode"),
            Line::from("Enables basic modal editing with limited vim-inspired"),
            Line::from("keybindings. See help screen for available commands."),
            Line::from("(? / F1  in Map Screen to open Help Screen)"),
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
