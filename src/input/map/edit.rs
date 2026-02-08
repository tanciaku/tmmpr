use std::io::stdout;

use crossterm::{cursor::SetCursorStyle, event::{KeyCode, KeyEvent}, execute};

use crate::{input::{AppAction, map::{append, backspace_char, cursor_pos_beginning, cursor_pos_end, insert_char, jump_back_a_word, jump_forward_a_word, move_cursor_down, move_cursor_left, move_cursor_right, move_cursor_right_norm, move_cursor_up, remove_char, switch_to_modal_insert_mode, switch_to_modal_normal_mode}}, states::{MapState, map::{ModalEditMode, Mode}}};

/// Handles keyboard input for Edit mode.
/// 
/// `modal`: Controls vim-style modal editing. `None` = always insert mode, `Some(mode)` = vim-style with normal/insert modes.
pub fn map_edit_kh(map_state: &mut MapState, key: KeyEvent, modal: Option<ModalEditMode>) -> AppAction {
    match modal {
        None | Some(ModalEditMode::Insert) => {
            match key.code {
                KeyCode::Esc => {
                    match modal {
                        None => {
                            cursor_pos_beginning(&mut map_state.notes_state);
                            map_state.notes_state.deselect();

                            let _ = execute!(stdout(), SetCursorStyle::SteadyBar);
                            map_state.current_mode = Mode::Normal;
                        }
                        Some(ModalEditMode::Insert) => {
                            // Vim behavior: move cursor back one position when leaving insert mode
                            move_cursor_left(&mut map_state.notes_state);

                            switch_to_modal_normal_mode(map_state);
                        }
                        _ => unreachable!(),
                    }
                }

                KeyCode::Char(c) => insert_char(map_state, c),
                KeyCode::Enter => insert_char(map_state, '\n'),
                KeyCode::Backspace => backspace_char(map_state),
                KeyCode::Left => move_cursor_left(&mut map_state.notes_state),
                KeyCode::Right => move_cursor_right(&mut map_state.notes_state),
                KeyCode::Up => move_cursor_up(&mut map_state.notes_state), 
                KeyCode::Down => move_cursor_down(&mut map_state.notes_state),
                _ => {}
            }
        }
        Some(ModalEditMode::Normal) => {
            match key.code {
                KeyCode::Esc => {
                    cursor_pos_beginning(&mut map_state.notes_state);
                    map_state.notes_state.deselect();
                    
                    let _ = execute!(stdout(), SetCursorStyle::SteadyBar);
                    map_state.current_mode = Mode::Normal;
                }
                KeyCode::Char('i') => switch_to_modal_insert_mode(map_state),
                KeyCode::Char('h') => move_cursor_left(&mut map_state.notes_state),
                KeyCode::Char('j') => move_cursor_down(&mut map_state.notes_state),
                KeyCode::Char('k') => move_cursor_up(&mut map_state.notes_state),
                KeyCode::Char('l') => move_cursor_right_norm(&mut map_state.notes_state),
                KeyCode::Char('g') => cursor_pos_beginning(&mut map_state.notes_state),
                KeyCode::Char('G') => cursor_pos_end(&mut map_state.notes_state),
                KeyCode::Char('w') => jump_forward_a_word(&mut map_state.notes_state),
                KeyCode::Char('b') => jump_back_a_word(&mut map_state.notes_state),
                KeyCode::Char('a') => append(map_state),
                KeyCode::Char('x') => remove_char(map_state),
                _ => {}
            }
        }
    }

    // Always redraw to reflect cursor movement and text changes
    map_state.clear_and_redraw();
    AppAction::Continue
}
