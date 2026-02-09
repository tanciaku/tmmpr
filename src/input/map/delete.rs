use crossterm::event::{KeyCode, KeyEvent};

use crate::{input::AppAction, states::{MapState, map::Mode}};


pub fn map_delete_kh(map_state: &mut MapState, key: KeyEvent) -> AppAction {
    match key.code {
        KeyCode::Esc => {
            map_state.mode = Mode::Visual;
        }
        KeyCode::Char('d') => {
            let selected_note_id = map_state.notes_state.expect_selected_note_id();

            map_state.persistence.mark_dirty(); 
            map_state.notes_state.remove(selected_note_id);
            map_state.connections_state.remove_note(selected_note_id);
            map_state.mode = Mode::Normal;
        }
        _ => {}
    }
    
    map_state.clear_and_redraw();
    AppAction::Continue
}