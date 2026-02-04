
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::Rect,
    style::{Stylize, Color, Style},
    widgets::{Block, Clear, Padding, Paragraph},
    text::{Span, Line},
};

use crate::{
    states::{
        MapState,
        map::{DiscardMenuType, ModalEditMode, Mode, Notification}
    },
    utils::{IoErrorKind, get_color_name_in_string}
};

/// Renders the bottom information bar showing mode, viewport position, and transient notifications.
///
/// Note: This function clears one-time notifications/errors from the state after rendering them.
pub fn render_bar(frame: &mut Frame, map_state: &mut MapState) {

    let size = frame.area();

    let (mode_text, mode_text_color) = match &map_state.current_mode {
        Mode::Normal => (String::from("[ NORMAL ]"), Style::new().fg(Color::White)),
        Mode::Visual => (String::from("[ VISUAL ]"), Style::new().fg(Color::Yellow)),
        Mode::VisualMove => (String::from("[ VISUAL (MOVE) ]"), Style::new().fg(Color::Yellow)),
        Mode::VisualConnect => (String::from("[ VISUAL (CONNECT) ]"), Style::new().fg(Color::Yellow)),
        Mode::Edit(modal) => (
            match modal {
                None => String::from("[ EDIT ]"),
                Some(ModalEditMode::Normal) => String::from("[ EDIT (NORMAL) ]"),
                Some(ModalEditMode::Insert) => String::from("[ EDIT (INSERT) ]"),
            },
            Style::new().fg(Color::Blue)),
        Mode::Delete => (String::from("[ DELETE ]"), Style::new().fg(Color::Red)),
    };

    let mode_display = Paragraph::new(format!("{}", mode_text))
        .style(mode_text_color)
        .alignment(Alignment::Left)
        .block(Block::default().padding(Padding::new(2, 0, 0, 0)));

    let view_position_display = Paragraph::new(format!(
        "View: {},{}",
        map_state.viewport.view_pos.x,
        map_state.viewport.view_pos.y,
    ))
    .alignment(Alignment::Right)
    .block(Block::default().padding(Padding::new(0, 2, 0, 0)));
    
    // Bar occupies last 3 rows: one empty spacer, two content rows
    let bar_area = Rect {
        x: size.x,
        y: size.height - 3,
        width: size.width,
        height: 3,
    };

    let bar_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(bar_area);

    // Each row split into 3 columns: left/middle/right (middle ensures 70 cell minimum for messages)
    let row_1_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Min(70),
            Constraint::Fill(1),
        ])
        .split(bar_rows[1]);
    let row_2_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Min(70),
            Constraint::Fill(1),
        ])
        .split(bar_rows[2]);
    
    frame.render_widget(Clear, bar_area);
    frame.render_widget(mode_display, row_2_areas[0]);
    frame.render_widget(view_position_display, row_2_areas[2]);

    if let Mode::Delete = &map_state.current_mode {
        let delete_note_prompt = Line::from(Span::styled(
            String::from("d - Delete the selected note          Esc - Go back to Visual Mode"),
            Style::new().fg(Color::Red)
            ));
        
        frame.render_widget(delete_note_prompt, row_2_areas[1]);
    }

    // Show color of focused connection if one exists, otherwise show color of selected note
    if matches!(map_state.current_mode, Mode::Visual | Mode::VisualMove | Mode::VisualConnect) {
                
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

    // One-time error notification: rendered once then immediately cleared from state
    if let Some(err_msg) = &map_state.settings_err_msg {
        let settings_err_msg = match err_msg {
            IoErrorKind::DirFind => Line::from(Span::styled("Settings error: no home directory - using defaults.", Style::new().fg(Color::Red))).alignment(Alignment::Center),
            IoErrorKind::DirCreate => Line::from(Span::styled("Settings error: can't create config directory - using defaults.", Style::new().fg(Color::Red))).alignment(Alignment::Center),
            IoErrorKind::FileWrite => Line::from(Span::styled("Settings error: can't create settings file - using defaults.", Style::new().fg(Color::Red))).alignment(Alignment::Center),
            IoErrorKind::FileRead => Line::from(Span::styled("Settings error: can't read settings file - using defaults.", Style::new().fg(Color::Red))).alignment(Alignment::Center),
        };
        
        frame.render_widget(settings_err_msg, row_1_areas[1]);
        map_state.settings_err_msg = None;
    }

    // One-time success/failure notification: rendered once then immediately cleared from state
    if let Some(notification) = &map_state.ui_state.show_notification {
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

        map_state.ui_state.clear_notification();
    }

    if let Some(discard_menu_type) = &map_state.ui_state.confirm_discard_menu {
        let confirm_discard_menu_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(8),
            ])
            .split(frame.area());

        frame.render_widget(Clear, confirm_discard_menu_area[1]);
        frame.render_widget(Clear, confirm_discard_menu_area[2]);
        frame.render_widget(Block::bordered(), confirm_discard_menu_area[2]);

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
                let line_1 = Line::from("Discard unsaved changes to this map?").alignment(Alignment::Center);
                let line_2 = Line::from(
                    vec![
                        Span::styled("[ ESC ] - Cancel", Style::new().fg(Color::Green)), 
                        Span::raw("      "),
                        Span::styled("[ q ] - Confirm discard and exit", Style::new().fg(Color::Red)), 

                    ]).alignment(Alignment::Center);

                frame.render_widget(line_1, confirm_discard_menu_text_areas[1]);
                frame.render_widget(line_2, confirm_discard_menu_text_areas[4]);
            }
            DiscardMenuType::Settings => {
                let line_1 = Line::from("Discard unsaved changes to this map and go to settings?").alignment(Alignment::Center);
                let line_2 = Line::from("(You must save changes or discard them before you can open the settings menu)").alignment(Alignment::Center);
                let line_3 = Line::from(
                    vec![
                        Span::styled("[ ESC ] - Cancel", Style::new().fg(Color::Green)), 
                        Span::raw("      "),
                        Span::styled("[ q ] - Confirm discard and go to settings", Style::new().fg(Color::Red)), 

                    ]).alignment(Alignment::Center);

                frame.render_widget(line_1, confirm_discard_menu_text_areas[1]);
                frame.render_widget(line_2, confirm_discard_menu_text_areas[2]);
                frame.render_widget(line_3, confirm_discard_menu_text_areas[4]);
            }
        }
    }
}
