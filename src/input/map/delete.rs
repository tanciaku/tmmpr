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

                map_state.connections_state.connections.retain(|c| {
                    // Safe to unwrap: all connections in the vector have both endpoints set
                    *selected_note != c.from_id && *selected_note != c.to_id.unwrap()
                });

                // Maintain bidirectional index: remove deleted note's entry, then clean up
                // references to it from all connected notes' entries
                if let Some(connections_to_delete) = map_state.connections_state.connection_index.remove(selected_note) {
                    for connection in connections_to_delete {
                        let id_to_look_up = if connection.from_id != *selected_note {
                            connection.from_id
                        } else {
                            connection.to_id.unwrap()
                        };

                        if let Some(associated_vec) = map_state.connections_state.connection_index.get_mut(&id_to_look_up) {
                            associated_vec.retain(|c| { c != &connection });
                        }
                    }
                }

                map_state.notes_state.selected_note = None;

                map_state.current_mode = Mode::Normal;
            }
        }
        _ => {}
    }
    
    map_state.clear_and_redraw();
    AppAction::Continue
}