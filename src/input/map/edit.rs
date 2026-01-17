use std::io::stdout;

use crossterm::{cursor::SetCursorStyle, event::{KeyCode, KeyEvent}, execute};

use crate::{input::{AppAction, map::{backspace_char, insert_char, jump_back_a_word, jump_forward_a_word, move_cursor_down, move_cursor_up, remove_char, switch_to_modal_insert_mode, switch_to_modal_normal_mode}}, states::{MapState, map::{ModalEditMode, Mode}}};


/// modal arg: Some() - Modal Editing for Edit Mode enabled, None - disabled.
pub fn map_edit_kh(map_state: &mut MapState, key: KeyEvent, modal: Option<ModalEditMode>) -> AppAction {
    match modal {
        // If Modal Editing is disabled for Edit Mode
        //  or it is enabled and is in Insert Mode.
        None | Some(ModalEditMode::Insert) => {
            if let Some(selected_note) = &map_state.notes_state.selected_note {
                match key.code {
                    KeyCode::Esc => {
                        match modal {
                            // If Modal Editing for Edit Mode is disabled - Esc switches back to Normal Mode.
                            None => {
                                map_state.current_mode = Mode::Normal;
                                if let Some(note) = map_state.notes_state.notes.get_mut(selected_note) {
                                    note.selected = false;
                                    // Reset cursor position for the next time entering Edit mode.
                                    map_state.notes_state.cursor_pos = 0;
                                }
                            }
                            // If it's enabled - switches mode to Modal Edit Mode - Normal.
                            Some(ModalEditMode::Insert) => {
                                // Move the cursor 1 space back
                                map_state.notes_state.cursor_pos = map_state.notes_state.cursor_pos.saturating_sub(1);

                                switch_to_modal_normal_mode(map_state);
                            }
                            _ => unreachable!(),
                        }
                    }

                    // --- Text Editing ---
                    KeyCode::Char(c) => insert_char(map_state, *selected_note, c),
                    KeyCode::Enter => insert_char(map_state, *selected_note, '\n'),
                    KeyCode::Backspace => {
                        map_state.persistence.mark_dirty();
                        backspace_char(map_state, *selected_note);
                    }
                    KeyCode::Left => {
                        if map_state.notes_state.cursor_pos > 0 { 
                            map_state.notes_state.cursor_pos -= 1 
                        }
                    }
                    KeyCode::Right => {
                        if let Some(note) = map_state.notes_state.notes.get(selected_note) {
                            if map_state.notes_state.cursor_pos < note.content.len() {
                                map_state.notes_state.cursor_pos += 1;
                            }
                        }
                    }
                    KeyCode::Up => move_cursor_up(map_state), 
                    KeyCode::Down => move_cursor_down(map_state),
                    _ => {}
                }
            }
        }
        // If Modal Editing for Edit Mode is enabled and is in Normal Mode.
        Some(ModalEditMode::Normal) => {
            if let Some(selected_note) = &map_state.notes_state.selected_note {
                match key.code {
                    // Switch back to Normal Mode.
                    KeyCode::Esc => {
                        map_state.current_mode = Mode::Normal;
                        
                        // Reset to a line cursor
                        let _ = execute!(stdout(), SetCursorStyle::SteadyBar);

                        if let Some(note) = map_state.notes_state.notes.get_mut(selected_note) {
                            // Deselect note (styling)
                            note.selected = false;
                            // Reset cursor position for the next time entering Edit mode.
                            map_state.notes_state.cursor_pos = 0;
                        }
                    }
                    // Switch to Insert mode
                    KeyCode::Char('i') => switch_to_modal_insert_mode(map_state),
                    // Move cursor left
                    KeyCode::Char('h') => {
                        if map_state.notes_state.cursor_pos > 0 { 
                            map_state.notes_state.cursor_pos -= 1 
                        }
                    }
                    // Move cursor down
                    KeyCode::Char('j') => move_cursor_down(map_state),
                    // Move cursor up
                    KeyCode::Char('k') => move_cursor_up(map_state),
                    // Move cursor right
                    KeyCode::Char('l') => {
                        if let Some(note) = map_state.notes_state.notes.get(selected_note) {
                            if map_state.notes_state.cursor_pos < note.content.len() - 1 {
                                map_state.notes_state.cursor_pos += 1;
                            }
                        }
                    }
                    // Move cursor to the very beginning
                    KeyCode::Char('g') => map_state.notes_state.cursor_pos = 0,
                    // Move cursor to the very end
                    KeyCode::Char('G') => {
                        if let Some(note) = map_state.notes_state.notes.get(selected_note) {    
                            map_state.notes_state.cursor_pos = note.content.len() - 1;
                        }
                    }
                    // Jump forward a word
                    KeyCode::Char('w') => jump_forward_a_word(map_state),
                    // Jump back a word
                    KeyCode::Char('b') => jump_back_a_word(map_state),
                    // Put cursor after the cursor position and switch to Insert mode
                    KeyCode::Char('a') => {
                        if let Some(note) = map_state.notes_state.notes.get(selected_note) {
                            if map_state.notes_state.cursor_pos + 1 <= note.content.len() {
                                map_state.notes_state.cursor_pos += 1;
                            }
                            
                            switch_to_modal_insert_mode(map_state);
                        }
                    }
                    KeyCode::Char('x') => remove_char(map_state, *selected_note),
                    _ => {}
                }
            }
        }
    }

    // Any action in Edit mode triggers a redraw.
    map_state.clear_and_redraw();

    AppAction::Continue
}
