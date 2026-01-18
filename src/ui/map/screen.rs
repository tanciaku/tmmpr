
use ratatui::{
    Frame,
    widgets::Clear,
};

use crate::{
    states::MapState,
    ui::{render_bar, render_connections, render_map_help_page, render_notes}
};


pub fn render_map(frame: &mut Frame, map_state: &mut MapState) {
    // Clear the frame before drawing anything new.
    frame.render_widget(Clear, frame.area());

    // If help page toggled, show it and stop there.
    if let Some(page_number) = map_state.ui_state.help_screen {
        render_map_help_page(frame, page_number);
        return;
    }

    // Update the  map_state with the current terminal size. This is crucial for
    // calculations that depend on screen dimensions, like centering new notes.
    map_state.viewport.screen_width = frame.area().width as usize;
    map_state.viewport.screen_height = frame.area().height as usize;

    // Render the main UI components.
    render_connections(frame, map_state);
    render_notes(frame, map_state); // Notes will be drawn over connections (if any)
    render_bar(frame, map_state); // The bar will be drawn over everything
}
