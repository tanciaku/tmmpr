use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;

use crate::{input::{AppAction, map::{cycle_color, cycle_side, move_note, switch_notes_focus}}, states::{MapState, map::{Connection, Mode}}};


pub fn map_visual_kh(map_state: &mut MapState, key: KeyEvent) -> AppAction { 

    // -- If Move State for Visual Mode is enabled --
    if map_state.visual_mode.visual_move {
        match key.code {
            // Switch back to Visual Mode Normal State
            KeyCode::Char('m') => map_state.visual_mode.visual_move = false,

            // Switch back to Normal Mode
            KeyCode::Esc => {
                map_state.current_mode = Mode::Normal;
                map_state.visual_mode.visual_move = false;

                if let Some(selected_note) = map_state.notes_state.selected_note {
                    if let Some(note) = map_state.notes_state.notes.get_mut(&selected_note) {
                        note.selected = false;
                    }
                }
            }


            // --- Moving the note ---

            // Move the note left by 1   (h, Left)
            KeyCode::Char('h') => move_note(map_state, "x", -1),
            KeyCode::Left if key.modifiers == KeyModifiers::NONE => move_note(map_state, "x", -1),
            // Move the note left by 5   (H, Shift + Left)
            KeyCode::Char('H') => move_note(map_state, "x", -5),
            KeyCode::Left if key.modifiers == KeyModifiers::SHIFT => move_note(map_state, "x", -5),
            
            // Move the note down by 1   (j, Down)
            KeyCode::Char('j') => move_note(map_state, "y", 1),
            KeyCode::Down if key.modifiers == KeyModifiers::NONE => move_note(map_state, "y", 1),
            // Move the note down by 5   (J, Shift + Down)
            KeyCode::Char('J') => move_note(map_state, "y", 5),
            KeyCode::Down if key.modifiers == KeyModifiers::SHIFT => move_note(map_state, "y", 5),
            
            // Move the note up by 1   (k, Up)
            KeyCode::Char('k') => move_note(map_state, "y", -1),
            KeyCode::Up if key.modifiers == KeyModifiers::NONE => move_note(map_state, "y", -1),
            // Move the note up by 5   (K, Shift + Up)
            KeyCode::Char('K') => move_note(map_state, "y", -5),
            KeyCode::Up if key.modifiers == KeyModifiers::SHIFT => move_note(map_state, "y", -5),
            
            // Move the note right by 1   (l, Right)
            KeyCode::Char('l') => move_note(map_state, "x", 1),
            KeyCode::Right if key.modifiers == KeyModifiers::NONE => move_note(map_state, "x", 1),
            // Move the note right by 5   (L, Shift + Right)
            KeyCode::Char('L') => move_note(map_state, "x", 5),
            KeyCode::Right if key.modifiers == KeyModifiers::SHIFT => move_note(map_state, "x", 5),

            _ => {}
        }

        // Trigger a redraw and stop there
        map_state.clear_and_redraw(); 
        return AppAction::Continue
    }

    // -- If Connection State for Visual Mode is enabled -- 
    if map_state.visual_mode.visual_connection {
        match key.code {
            // Switch back to Visual Mode Normal State
            KeyCode::Char('c') => {

                map_state.connections_state.stash_connection();

                map_state.visual_mode.visual_connection = false;
                map_state.connections_state.visual_editing_a_connection = false; // (if already isn't)
                map_state.connections_state.editing_connection_index = None; // (if already isn't)
            }

            // Rotating the start/end side of the connection 
            KeyCode::Char('r') => {
                if let Some(selected_note) = map_state.notes_state.selected_note {
                    map_state.can_exit = false;
                    if let Some(focused_connection) = map_state.connections_state.focused_connection.as_mut() {
                        if focused_connection.from_id == selected_note {
                            focused_connection.from_side = cycle_side(focused_connection.from_side);
                        }

                        if let Some(to_id) = focused_connection.to_id {
                            if to_id == selected_note {
                                focused_connection.to_side = Some(cycle_side(focused_connection.to_side.unwrap()));
                                // .unwrap() okay here - since if there is a to_id, there is a to_side
                            }
                        }
                    }
                }
            }

            // Cycling through the available connections (to select the one the
            // user wants) associated with this note - so this note can be the 
            // start point or end point of a connection the user can edit.
            KeyCode::Char('n') => {
                if let Some(selected_note) = map_state.notes_state.selected_note {
                    // Can only cycle through the available connections on this note if
                    // entered the visual_connection mode to edit existing connections
                    // and not currently making a new one
                    if map_state.connections_state.visual_editing_a_connection {

                        // Stash the Current Connection
                        map_state.connections_state.stash_connection();
                        // Index of the connection just stashed
                        let start_index = map_state.connections_state.editing_connection_index.unwrap();

                        // Start by assuming we haven't found it.
                        let mut next_index_option = None;

                        // Only search the latter part of the vector if it's safe to do so.
                        if start_index < map_state.connections_state.connections.len() {
                            next_index_option = map_state.connections_state.connections[start_index..]
                                .iter()
                                .position(|c| {
                                    selected_note == c.from_id || selected_note == c.to_id.unwrap()
                                })
                                .map(|i| i + start_index);
                        }

                        // If that connection was last in the vector or no match was found after it -
                        // search from the start
                        if next_index_option.is_none() {
                            next_index_option = map_state.connections_state.connections
                                .iter()
                                .position(|c| {
                                    selected_note == c.from_id || selected_note == c.to_id.unwrap()
                                });
                        }

                        if let Some(next_index) = next_index_option {
                            // If found one - remove it and put it in focus.
                            // Note: it will always "find" one - since
                            map_state.connections_state.take_out_connection(next_index);
                            map_state.connections_state.editing_connection_index = Some(next_index);
                        }
                    }
                }
            }

            // Delete the selected connection
            KeyCode::Char('d') => {
                if map_state.connections_state.visual_editing_a_connection {
                    map_state.can_exit = false;

                    // Delete that connection
                    map_state.connections_state.focused_connection = None;

                    // Exit
                    map_state.visual_mode.visual_connection = false;
                    map_state.connections_state.visual_editing_a_connection = false;
                    map_state.connections_state.editing_connection_index = None;
                }
            }

            // -- Target Note Selection --
            // Reuse the focus switching logic to select a target note for the new connection.
            // Below
            KeyCode::Char('j') => switch_notes_focus(map_state, "j"),
            KeyCode::Down => switch_notes_focus(map_state, "Down"),
            // Above
            KeyCode::Char('k') => switch_notes_focus(map_state, "k"),
            KeyCode::Up => switch_notes_focus(map_state, "Up"),
            // Left
            KeyCode::Char('h') => switch_notes_focus(map_state, "h"),
            KeyCode::Left => switch_notes_focus(map_state, "Left"),
            // Right
            KeyCode::Char('l') => switch_notes_focus(map_state, "l"),
            KeyCode::Right => switch_notes_focus(map_state, "Right"),

            // Cycle through colors for the "in progress"/focused connection
            KeyCode::Char('e') => {
                if let Some(focused_connection) = map_state.connections_state.focused_connection.as_mut() {
                    focused_connection.color = cycle_color(focused_connection.color);
                    map_state.can_exit = false;
                }
            }

            _ => {}
        }

        // Trigger a redraw and stop there
        map_state.clear_and_redraw(); 
        return AppAction::Continue
    }

    // If Visual Mode is in Normal State
    match key.code {
        // Switch back to Normal Mode
        KeyCode::Esc => {
            map_state.current_mode = Mode::Normal;

            if let Some(selected_note) = map_state.notes_state.selected_note {
                if let Some(note) = map_state.notes_state.notes.get_mut(&selected_note) {
                    note.selected = false;
                }
            }
        }
        // Switch to Edit mode
        KeyCode::Char('i') => map_state.switch_to_edit_mode(),

        // Switch to Move State for the Visual Mode
        KeyCode::Char('m') => map_state.visual_mode.visual_move = true,

        // Switch to Connection Sate for Visual Mode
        // This block selects the "first" connection that this note
        // is associated with, if it has any.
        KeyCode::Char('c') => {
            if let Some(selected_note) = map_state.notes_state.selected_note {
                if let Some(index) = map_state.connections_state.connections.iter().position(|c| {
                    selected_note == c.from_id || selected_note == c.to_id.unwrap()
                    // unwrap() is safe here since all the connections have an endpoint if
                    // they are in the connections vector.
                }) {
                    map_state.connections_state.take_out_connection(index);
                    map_state.connections_state.editing_connection_index = Some(index);
                    map_state.visual_mode.visual_connection = true;
                    map_state.connections_state.visual_editing_a_connection = true;
                }
            }
        }

        // Add a new Connection for the selected note
        KeyCode::Char('C') => {
            if let Some(selected_note) = map_state.notes_state.selected_note {
                map_state.connections_state.focused_connection = Some(
                    Connection {
                        from_id: selected_note,
                        from_side: map_state.settings.default_start_side, // default side
                        to_id: None,
                        to_side: None,
                        color: Color::White,
                    }
                );

                map_state.visual_mode.visual_connection = true;
                
                map_state.can_exit = false;
            }
        }

        // Switch to Delete Mode
        KeyCode::Char('d') => map_state.current_mode = Mode::Delete,

        // -- Switching focus between notes --
        // Switch focus to the closest note below the currently selected one
        // Below
        KeyCode::Char('j') => switch_notes_focus(map_state, "j"),
        KeyCode::Down => switch_notes_focus(map_state, "Down"),
        // Above
        KeyCode::Char('k') => switch_notes_focus(map_state, "k"),
        KeyCode::Up => switch_notes_focus(map_state, "Up"),
        // Left
        KeyCode::Char('h') => switch_notes_focus(map_state, "h"),
        KeyCode::Left => switch_notes_focus(map_state, "Left"),
        // Right
        KeyCode::Char('l') => switch_notes_focus(map_state, "l"),
        KeyCode::Right => switch_notes_focus(map_state, "Right"),

        // Cycle through colors for the selected note
        KeyCode::Char('e') => {
            if let Some(selected_note) = map_state.notes_state.selected_note {
                if let Some(note) = map_state.notes_state.notes.get_mut(&selected_note) {
                    note.color = cycle_color(note.color);
                    
                    map_state.can_exit = false;
                }
            }
        }

        _ => {}
    }

    // Any action in Visual mode triggers a redraw.
    map_state.clear_and_redraw();
    AppAction::Continue
}