use crossterm::event::{KeyCode, KeyEvent};

use crate::{input::AppAction, states::{MapState, map::Mode}};


pub fn map_delete_kh(map_state: &mut MapState, key: KeyEvent) -> AppAction {
    if let Some(selected_note) = &map_state.selected_note {
        match key.code {
            // Switch back to Visual Mode
            KeyCode::Esc => {
                map_state.current_mode = Mode::Visual;
            }
            // Confirm deleting the selected note
            KeyCode::Char('d') => {
                map_state.can_exit = false;
                
                // Remove that note from the notes HashMap by it's id  (id, note)
                map_state.notes.remove(selected_note);

                // Remove that note's id from the render_order
                if let Some(pos) = map_state.render_order.iter().position(|&x| x == *selected_note) {
                    map_state.render_order.remove(pos);
                }

                // -- Updating the connections Vec --
                // Remove any connections that were associated with that note
                // (Only keep the ones that aren't)
                map_state.connections.retain(|c| {
                    *selected_note != c.from_id && *selected_note != c.to_id.unwrap()
                    // .unwrap() is okay here since all the connections in the vector have an endpoint
                });

                // -- Updating the connection_index HashMap --
                // Get the Vec of connections for the deleted note AND remove it from the map in one step.
                if let Some(connections_to_delete) = map_state.connection_index.remove(selected_note) {
                    // Now loop through that Vec you just got back.
                    for connection in connections_to_delete {
                        // Figure out the ID of the other end in the connection.
                        let id_to_look_up = if connection.from_id != *selected_note {
                            connection.from_id
                        } else {
                            connection.to_id.unwrap()
                        };

                        // Go to the "other" note's entry and clean up the connections
                        // that involve the deleted note's id
                        if let Some(associated_vec) = map_state.connection_index.get_mut(&id_to_look_up) {
                            associated_vec.retain(|c| { c != &connection });
                        }
                    }
                }

                map_state.selected_note = None;

                map_state.current_mode = Mode::Normal;
            }
            _ => {}
        }
    }
    
    map_state.clear_and_redraw();
    AppAction::Continue
}