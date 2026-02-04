use crossterm::event::{KeyCode, KeyEvent};

use crate::{input::AppAction, states::{MapState, map::Mode}};


pub fn map_delete_kh(map_state: &mut MapState, key: KeyEvent) -> AppAction {
    match key.code {
        KeyCode::Esc => {
            map_state.current_mode = Mode::VisualSelect;
        }
        KeyCode::Char('d') => {
            if let Some(selected_note) = &map_state.notes_state.selected_note {
                map_state.persistence.mark_dirty();
            
                map_state.notes_state.notes.remove(selected_note);

                if let Some(pos) = map_state.notes_state.render_order.iter().position(|&x| x == *selected_note) {
                    map_state.notes_state.render_order.remove(pos);
                }

                map_state.connections_state.remove_note(*selected_note);

                map_state.notes_state.selected_note = None;

                map_state.current_mode = Mode::Normal;
            }
        }
        _ => {}
    }
    
    map_state.clear_and_redraw();
    AppAction::Continue
}