use ratatui::{
    Frame,
    widgets::Clear,
};

use crate::{
    states::MapState,
    ui::{render_bar, render_connections, render_map_help_page, render_notes}
};

pub fn render_map(frame: &mut Frame, map_state: &mut MapState) {
    frame.render_widget(Clear, frame.area());

    if let Some(page_number) = map_state.ui_state.help_screen {
        render_map_help_page(frame, page_number);
        return;
    }

    // Viewport needs current dimensions for calculations like centering new notes
    map_state.viewport.screen_width = frame.area().width as usize;
    map_state.viewport.screen_height = frame.area().height as usize;

    render_connections(frame, map_state);
    render_notes(frame, map_state); // Notes drawn over connections
    render_bar(frame, map_state); // Bar drawn over everything
}
