use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;

use crate::{input::{AppAction, map::{cycle_color, cycle_side, move_note, switch_notes_focus}}, states::{MapState, map::{Connection, Mode}}};


pub fn map_visual_kh(map_state: &mut MapState, key: KeyEvent) -> AppAction { 

    if map_state.current_mode == Mode::VisualMove {
        match key.code {
            KeyCode::Char('m') => map_state.current_mode = Mode::Visual,

            KeyCode::Esc => {
                map_state.current_mode = Mode::Normal;

                let note = map_state.notes_state.expect_selected_note_mut();
                note.selected = false;
            }

            // Vim-style movement: hjkl and arrow keys. Shift modifier increases step size to 5.
            KeyCode::Char('h') => move_note(map_state, "x", -1),
            KeyCode::Left if key.modifiers == KeyModifiers::NONE => move_note(map_state, "x", -1),
            KeyCode::Char('H') => move_note(map_state, "x", -5),
            KeyCode::Left if key.modifiers == KeyModifiers::SHIFT => move_note(map_state, "x", -5),
            
            KeyCode::Char('j') => move_note(map_state, "y", 1),
            KeyCode::Down if key.modifiers == KeyModifiers::NONE => move_note(map_state, "y", 1),
            KeyCode::Char('J') => move_note(map_state, "y", 5),
            KeyCode::Down if key.modifiers == KeyModifiers::SHIFT => move_note(map_state, "y", 5),
            
            KeyCode::Char('k') => move_note(map_state, "y", -1),
            KeyCode::Up if key.modifiers == KeyModifiers::NONE => move_note(map_state, "y", -1),
            KeyCode::Char('K') => move_note(map_state, "y", -5),
            KeyCode::Up if key.modifiers == KeyModifiers::SHIFT => move_note(map_state, "y", -5),
            
            KeyCode::Char('l') => move_note(map_state, "x", 1),
            KeyCode::Right if key.modifiers == KeyModifiers::NONE => move_note(map_state, "x", 1),
            KeyCode::Char('L') => move_note(map_state, "x", 5),
            KeyCode::Right if key.modifiers == KeyModifiers::SHIFT => move_note(map_state, "x", 5),

            _ => {}
        }

        // Early return to prevent falling through to other mode handlers
        map_state.clear_and_redraw(); 
        return AppAction::Continue
    }

    if map_state.current_mode == Mode::VisualConnect {
        match key.code {
            KeyCode::Char('c') => {
                map_state.connections_state.stash_connection();

                map_state.current_mode = Mode::Visual;
                map_state.connections_state.editing_connection_index = None;
            }

            KeyCode::Char('r') => {
                let selected_note_id = map_state.notes_state.expect_selected_note_id();
                map_state.persistence.mark_dirty();

                if let Some(focused_connection) = map_state.connections_state.focused_connection.as_mut() {
                    if focused_connection.from_id == selected_note_id {
                        focused_connection.from_side = cycle_side(focused_connection.from_side);
                    }

                    if let Some(to_id) = focused_connection.to_id {
                        if to_id == selected_note_id {
                            // Safe: to_side always exists when to_id exists
                            focused_connection.to_side = Some(cycle_side(focused_connection.to_side.unwrap()));
                        }
                    }
                }
            }

            // Cycle through all connections associated with the selected note (where it's either the start or end point).
            // This allows editing different connections on the same note.
            KeyCode::Char('n') => {
                let selected_note_id = map_state.notes_state.expect_selected_note_id();

                // Can only cycle when editing existing connections, not creating new ones
                if let Some(editing_idx) = map_state.connections_state.editing_connection_index { 
                    // The take out, then stash operation effectively cycles the list by moving 
                    // the current connection to the back
                    map_state.connections_state.stash_connection();

                    let indices = map_state.connections_state.get_indices_for_note(selected_note_id);
                    let pos = indices.iter().position(|&i| i == editing_idx).unwrap_or(0);
                    let next_index = indices[pos];
                    
                    map_state.connections_state.take_out_connection(next_index);
                    map_state.connections_state.editing_connection_index = Some(next_index);
                }
            }

            KeyCode::Char('d') => {
                // Can only delete when editing existing connections, not creating new ones
                if let Some(_) = map_state.connections_state.editing_connection_index {
                    map_state.persistence.mark_dirty();
                    map_state.connections_state.focused_connection = None;

                    map_state.current_mode = Mode::Visual;
                    map_state.connections_state.editing_connection_index = None;
                }
            }

            // Reuse note focus switching to select target endpoint for connection
            KeyCode::Char('j') => switch_notes_focus(map_state, "j"),
            KeyCode::Down => switch_notes_focus(map_state, "Down"),
            KeyCode::Char('k') => switch_notes_focus(map_state, "k"),
            KeyCode::Up => switch_notes_focus(map_state, "Up"),
            KeyCode::Char('h') => switch_notes_focus(map_state, "h"),
            KeyCode::Left => switch_notes_focus(map_state, "Left"),
            KeyCode::Char('l') => switch_notes_focus(map_state, "l"),
            KeyCode::Right => switch_notes_focus(map_state, "Right"),

            KeyCode::Char('e') => {
                if let Some(focused_connection) = map_state.connections_state.focused_connection.as_mut() {
                    focused_connection.color = cycle_color(focused_connection.color);
                    map_state.persistence.mark_dirty();
                }
            }

            _ => {}
        }

        // Early return to prevent falling through to other mode handlers
        map_state.clear_and_redraw(); 
        return AppAction::Continue
    }

    match key.code {
        KeyCode::Esc => {
            map_state.current_mode = Mode::Normal;

            let note = map_state.notes_state.expect_selected_note_mut();
            note.selected = false;
        }
        KeyCode::Char('i') => map_state.switch_to_edit_mode(),

        KeyCode::Char('m') => map_state.current_mode = Mode::VisualMove,

        // Enter connection edit mode. Finds and focuses the first connection associated with this note.
        KeyCode::Char('c') => {
            let selected_note_id = map_state.notes_state.expect_selected_note_id();

            if let Some(&index) = map_state.connections_state.get_indices_for_note(selected_note_id).first() {
                map_state.connections_state.take_out_connection(index);
                map_state.connections_state.editing_connection_index = Some(index);
                map_state.current_mode = Mode::VisualConnect;
            }
        }

        KeyCode::Char('C') => {
            let selected_note_id = map_state.notes_state.expect_selected_note_id();

            map_state.connections_state.focused_connection = Some(
                Connection {
                    from_id: selected_note_id,
                    from_side: map_state.settings.default_start_side,
                    to_id: None,
                    to_side: None,
                    color: Color::White,
                }
            );

            map_state.current_mode = Mode::VisualConnect;
            map_state.persistence.mark_dirty();
        }

        KeyCode::Char('d') => map_state.current_mode = Mode::Delete,

        KeyCode::Char('j') => switch_notes_focus(map_state, "j"),
        KeyCode::Down => switch_notes_focus(map_state, "Down"),
        KeyCode::Char('k') => switch_notes_focus(map_state, "k"),
        KeyCode::Up => switch_notes_focus(map_state, "Up"),
        KeyCode::Char('h') => switch_notes_focus(map_state, "h"),
        KeyCode::Left => switch_notes_focus(map_state, "Left"),
        KeyCode::Char('l') => switch_notes_focus(map_state, "l"),
        KeyCode::Right => switch_notes_focus(map_state, "Right"),

        KeyCode::Char('e') => {
            let note = map_state.notes_state.expect_selected_note_mut();

            note.color = cycle_color(note.color);
            map_state.persistence.mark_dirty();
        }

        _ => {}
    }

    // Always redraw to reflect selection and visual changes
    map_state.clear_and_redraw();
    AppAction::Continue
}