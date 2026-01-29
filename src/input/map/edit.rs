use std::io::stdout;

use crossterm::{cursor::SetCursorStyle, event::{KeyCode, KeyEvent}, execute};

use crate::{input::{AppAction, map::{backspace_char, insert_char, jump_back_a_word, jump_forward_a_word, move_cursor_down, move_cursor_up, remove_char, switch_to_modal_insert_mode, switch_to_modal_normal_mode}}, states::{MapState, map::{ModalEditMode, Mode}}};


/// Handles keyboard input for Edit mode.
/// 
/// `modal`: Controls vim-style modal editing. `None` = always insert mode, `Some(mode)` = vim-style with normal/insert modes.
pub fn map_edit_kh(map_state: &mut MapState, key: KeyEvent, modal: Option<ModalEditMode>) -> AppAction {
    match modal {
        None | Some(ModalEditMode::Insert) => {
            if let Some(selected_note) = &map_state.notes_state.selected_note {
                match key.code {
                    KeyCode::Esc => {
                        match modal {
                            None => {
                                map_state.current_mode = Mode::Normal;
                                if let Some(note) = map_state.notes_state.notes.get_mut(selected_note) {
                                    note.selected = false;
                                    map_state.notes_state.cursor_pos = 0;
                                }
                            }
                            Some(ModalEditMode::Insert) => {
                                // Vim behavior: move cursor back one position when leaving insert mode
                                map_state.notes_state.cursor_pos = map_state.notes_state.cursor_pos.saturating_sub(1);

                                switch_to_modal_normal_mode(map_state);
                            }
                            _ => unreachable!(),
                        }
                    }

                    KeyCode::Char(c) => insert_char(map_state, *selected_note, c),
                    KeyCode::Enter => insert_char(map_state, *selected_note, '\n'),
                    KeyCode::Backspace => backspace_char(map_state, *selected_note),
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
        Some(ModalEditMode::Normal) => {
            if let Some(selected_note) = &map_state.notes_state.selected_note {
                match key.code {
                    KeyCode::Esc => {
                        map_state.current_mode = Mode::Normal;
                        
                        let _ = execute!(stdout(), SetCursorStyle::SteadyBar);

                        if let Some(note) = map_state.notes_state.notes.get_mut(selected_note) {
                            note.selected = false;
                            map_state.notes_state.cursor_pos = 0;
                        }
                    }
                    KeyCode::Char('i') => switch_to_modal_insert_mode(map_state),
                    KeyCode::Char('h') => {
                        if map_state.notes_state.cursor_pos > 0 { 
                            map_state.notes_state.cursor_pos -= 1 
                        }
                    }
                    KeyCode::Char('j') => move_cursor_down(map_state),
                    KeyCode::Char('k') => move_cursor_up(map_state),
                    KeyCode::Char('l') => {
                        if let Some(note) = map_state.notes_state.notes.get(selected_note) {
                            // Stop before last char to match vim's normal mode behavior
                            if map_state.notes_state.cursor_pos < note.content.len() - 1 {
                                map_state.notes_state.cursor_pos += 1;
                            }
                        }
                    }
                    KeyCode::Char('g') => map_state.notes_state.cursor_pos = 0,
                    KeyCode::Char('G') => {
                        if let Some(note) = map_state.notes_state.notes.get(selected_note) {    
                            map_state.notes_state.cursor_pos = note.content.len() - 1;
                        }
                    }
                    KeyCode::Char('w') => jump_forward_a_word(map_state),
                    KeyCode::Char('b') => jump_back_a_word(map_state),
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

    // Always redraw to reflect cursor movement and text changes
    map_state.clear_and_redraw();

    AppAction::Continue
}
