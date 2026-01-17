
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::Rect,
    style::{Stylize, Color, Style},
    widgets::{Block, Clear, Padding, Paragraph},
    text::{Span, Line},
};

use crate::{
    utils::get_color_name_in_string,
    states::{
        MapState,
        start::ErrMsg,
        map::{DiscardMenuType, ModalEditMode, Mode, Notification},
    },
};

/// Renders the bottom information bar.
///
/// This bar displays debugging information and the current application state,
/// such as viewport position, mode, and selected note.
pub fn render_bar(frame: &mut Frame, map_state: &mut MapState) {

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
        map_state.viewport.view_pos.x,
        map_state.viewport.view_pos.y,
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

        if let Some(selected_note) = &map_state.notes_state.selected_note {
            if let Some(focused_connection) = &map_state.connections_state.focused_connection {
                current_color_text = String::from("Selected connection color: ");
                current_color_name = get_color_name_in_string(focused_connection.color);
                current_color = focused_connection.color;
            } else {
                if let Some(note) = map_state.notes_state.notes.get(selected_note) {
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
