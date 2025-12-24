//! This module handles terminal events, focusing on keyboard input
//! to control the application's state and behavior.

use crate::{
    app::{App, Screen},
    utils::{create_map_file, load_map_file, save_map_file},
    states::{
        MapState, StartState,
        start::{SelectedStartButton, FocusedInputBox},
        map::{Connection, Mode, Side},
    },
};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::{cmp::Reverse, path::PathBuf};
use ratatui::style::Color;

/// Reads the terminal events.
pub fn handle_events(app: &mut App) -> Result<()> {
    // Poll for an event with a timeout of 50ms. This is the main "tick" rate.
    if event::poll(std::time::Duration::from_millis(50))? {
        // Read the event
        match event::read()? {
            // Handle keyboard input
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                // Dispatch it to the appropriate handler.
                let app_action = match &mut app.screen {
                    Screen::Start(start_state) => start_kh(start_state, key),
                    Screen::Map(map_state) => map_kh(map_state, key),
                    _ => AppAction::Continue,
                };

                match app_action {
                    AppAction::Continue => {}
                    AppAction::Quit => app.quit(),
                    AppAction::Switch(screen) => app.screen = screen,
                    AppAction::CreateMapFile(path) => create_map_file(app, &path),
                    AppAction::SaveMapFile(path) => save_map_file(app, &path, true),
                    AppAction::LoadMapFile(path) => load_map_file(app, &path),
                }
            }

            // Redraw the UI if terminal window resized
            Event::Resize(_, _) => {
                match &mut app.screen {
                    Screen::Start(start_state) => start_state.needs_clear_and_redraw = true,
                    Screen::Map(map_state) => map_state.needs_clear_and_redraw = true,
                    _ => {},
                }
            }

            _ => {}
        }
    }
    Ok(())
}

/// Key handling for the Start Screen
fn start_kh(start_state: &mut StartState, key: KeyEvent) -> AppAction {
    // Take all input if in the Input Menu screen
    // (Entering a path for the map file)
    if start_state.input_path {

        // Keys independent of which input box is in focus
        match key.code {
            KeyCode::Esc => {
                start_state.input_path = false;
                start_state.focused_input_box = FocusedInputBox::InputBox1; // if already isn't
                start_state.input_path_string = None; // reset input fields
                start_state.input_path_name = None; // reset input fields
            }
            _ => {}
        }

        // Which input box is in focus?
        match start_state.focused_input_box {
            FocusedInputBox::InputBox1 => {
                if let Some(path) = &mut start_state.input_path_string {
                    match key.code {
                        KeyCode::Char(c) => {
                            if path.len() < 46 {
                                path.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if path.len() > 0 {
                                path.pop();
                            }
                        }
                        KeyCode::Enter => {
                            start_state.focused_input_box = FocusedInputBox::InputBox2;
                        }
                        _ => {}
                    }
                }
            }
            FocusedInputBox::InputBox2 => {
                if let Some(map_name) = &mut start_state.input_path_name {
                    match key.code {
                        KeyCode::Char(c) => {
                            if map_name.len() < 26 {
                                map_name.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if map_name.len() > 0 {
                                map_name.pop();
                            }
                        }
                        KeyCode::Enter => {
                            start_state.clear_and_redraw();
                            return start_state.submit_path(None)
                        }
                        _ => {}
                    }
                }
            }
        }

        start_state.clear_and_redraw();
        return AppAction::Continue
    }

    // If in the start menu
    match key.code {

        KeyCode::Char('q') => return AppAction::Quit,

        KeyCode::Char('k') => start_state.navigate_start_buttons("k"),
        KeyCode::Up => start_state.navigate_start_buttons("Up"),

        KeyCode::Char('j') => start_state.navigate_start_buttons("j"),
        KeyCode::Down => start_state.navigate_start_buttons("Down"),

        KeyCode::Enter => {
            match start_state.selected_button {
                SelectedStartButton::CreateSelect => {
                    start_state.input_path = true;
                    start_state.display_err_msg = None; // if already isn't
                    start_state.input_path_string = Some(String::new());
                    start_state.input_path_name = Some(String::new());
                }
                _ => {}
            }
        }

        _ => {}
    }

    // If able to use the "recent paths" functionality (no errors)
    if let Ok(recent_paths) = &start_state.recent_paths {
        match key.code {
            KeyCode::Enter => {
                match start_state.selected_button {
                    SelectedStartButton::Recent1 => {
                        if let Some(path) = &recent_paths.recent_path_1 {
                            return start_state.submit_path(Some(path.to_path_buf()))
                        }
                    }
                    SelectedStartButton::Recent2 => {
                        if let Some(path) = &recent_paths.recent_path_2 {
                            return start_state.submit_path(Some(path.to_path_buf()))
                        }
                    }
                    SelectedStartButton::Recent3 => {
                        if let Some(path) = &recent_paths.recent_path_3 {
                            return start_state.submit_path(Some(path.to_path_buf()))
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    
    start_state.clear_and_redraw();
    AppAction::Continue
}

/// Key handling for the Map Screen
fn map_kh(map_state: &mut MapState, key: KeyEvent) -> AppAction { 
    match map_state.current_mode {
        // Normal mode is for navigation and high-level commands.
        Mode::Normal => map_normal_kh(map_state, key),

        // Visual mode for selections.
        Mode::Visual => map_visual_kh(map_state, key),

        // Insert mode is for editing the content of a note.
        Mode::Insert => map_insert_kh(map_state, key),
    
        // Delete mode is a confirmation to delete a note
        Mode::Delete => map_delete_kh(map_state, key),
    }
}

#[derive(PartialEq)]
pub enum AppAction {
    Continue,
    Quit,
    Switch(Screen),
    CreateMapFile(PathBuf),
    SaveMapFile(PathBuf),
    LoadMapFile(PathBuf),
}

/// Key handling for Normal Mode in the Map Screen
fn map_normal_kh(map_state: &mut MapState, key: KeyEvent) -> AppAction {
    // Showing help page (takes all input if toggled)
    if let Some(_) = map_state.help_screen {
        match key.code {
            // F1, ? - toggle the help page
            KeyCode::F(1) | KeyCode::Char('?') | KeyCode::Esc => map_state.help_screen = None,

            // Right, l, Tab - go forward a page in the help screen.
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => help_next_page(map_state),

            // Left, h - go back a page in the help screen.
            KeyCode::Left | KeyCode::Char('h') => help_previous_page(map_state),

            _ => {}
        }

        map_state.clear_and_redraw(); 

        return AppAction::Continue // Stop here
    }
    
    // Confirm discard unsaved changes menu (takes all input if triggered)
    if map_state.confirm_discard_menu {
        match key.code {
            // Cancel
            KeyCode::Esc => {
                map_state.confirm_discard_menu = false;
                map_state.needs_clear_and_redraw = true;
            }
            // Confirm exiting and discarding unsaved changes
            KeyCode::Char('q') => {
                return AppAction::Switch(Screen::Start(StartState::new()))
            }
            _ => {}
        }

        return AppAction::Continue // Stop here
    }
    
    // --- Map Screen Normal Mode Commands ---
    match key.code {

        // Exiting the app
        KeyCode::Char('q') => {
            // Can exit the app if saved the changes
            if map_state.can_exit {
                return AppAction::Switch(Screen::Start(StartState::new()))
            } else { // Otherwise show the confirmation to discard unsaved changes menu
                map_state.confirm_discard_menu = true;
                map_state.needs_clear_and_redraw = true;
            }
        }

        // F1, ? - toggle the help page
        KeyCode::F(1) | KeyCode::Char('?') => map_state.help_screen = Some(1),

        // Save the map file
        KeyCode::Char('s') => return AppAction::SaveMapFile(map_state.file_write_path.clone()),

        // --- Viewport Navigation ---

        // Move left by 1   (h, Left)
        KeyCode::Char('h') => move_viewport(map_state, "x", -1),
        KeyCode::Left if key.modifiers == KeyModifiers::NONE => move_viewport(map_state, "x", -1),
        // Move left by 5   (H, Shift + Left)
        KeyCode::Char('H') => move_viewport(map_state, "x", -5),
        KeyCode::Left if key.modifiers == KeyModifiers::SHIFT => move_viewport(map_state, "x", -5),

        // Move down by 1   (j, Down)
        KeyCode::Char('j') => move_viewport(map_state, "y", 1),
        KeyCode::Down if key.modifiers == KeyModifiers::NONE => move_viewport(map_state, "y", 1),
        // Move down by 5   (J, Shift + Down)
        KeyCode::Char('J') => move_viewport(map_state, "y", 5),
        KeyCode::Down if key.modifiers == KeyModifiers::SHIFT => move_viewport(map_state, "y", 5),

        // Move up by 1   (k, Up)
        KeyCode::Char('k') => move_viewport(map_state, "y", -1),
        KeyCode::Up if key.modifiers == KeyModifiers::NONE => move_viewport(map_state, "y", -1),
        // Move up by 5   (K, Shift + Up)
        KeyCode::Char('K') => move_viewport(map_state, "y", -5),
        KeyCode::Up if key.modifiers == KeyModifiers::SHIFT => move_viewport(map_state, "y", -5),

        // Move right by 1   (l, Right)
        KeyCode::Char('l') => move_viewport(map_state, "x", 1),
        KeyCode::Right if key.modifiers == KeyModifiers::NONE => move_viewport(map_state, "x", 1),
        // Move right by 5   (L, Shift + Right)
        KeyCode::Char('L') => move_viewport(map_state, "x", 5),
        KeyCode::Right if key.modifiers == KeyModifiers::SHIFT => move_viewport(map_state, "x", 5),


        // --- Note Manipulation ---

        // Add a note
        KeyCode::Char('a') => map_state.add_note(),
        // Select note (selects the closest one to the center of the screen)
        KeyCode::Char('v') => {
            map_state.select_note();
            map_state.current_mode = Mode::Visual;
        }
    
        _ => {}
    }

    // Any action in Normal mode triggers a redraw.
    map_state.clear_and_redraw();

    AppAction::Continue
}

fn map_visual_kh(map_state: &mut MapState, key: KeyEvent) -> AppAction { 

    // -- If Move State for Visual Mode is enabled --
    if map_state.visual_move {
        match key.code {
            // Switch back to Visual Mode Normal State
            KeyCode::Char('m') => map_state.visual_move = false,

            // Switch back to Normal Mode
            KeyCode::Esc => {
                map_state.current_mode = Mode::Normal;
                map_state.visual_move = false;

                if let Some(selected_note) = map_state.selected_note {
                    if let Some(note) = map_state.notes.get_mut(&selected_note) {
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
    if map_state.visual_connection {
        match key.code {
            // Switch back to Visual Mode Normal State
            KeyCode::Char('c') => {

                map_state.stash_connection();

                map_state.visual_connection = false;
                map_state.visual_editing_a_connection = false; // (if already isn't)
                map_state.editing_connection_index = None; // (if already isn't)
            }

            // Rotating the start/end side of the connection 
            KeyCode::Char('r') => {
                if let Some(selected_note) = map_state.selected_note {
                    map_state.can_exit = false;
                    if let Some(focused_connection) = map_state.focused_connection.as_mut() {
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
                if let Some(selected_note) = map_state.selected_note {
                    // Can only cycle through the available connections on this note if
                    // entered the visual_connection mode to edit existing connections
                    // and not currently making a new one
                    if map_state.visual_editing_a_connection {

                        // Stash the Current Connection
                        map_state.stash_connection();
                        // Index of the connection just stashed
                        let start_index = map_state.editing_connection_index.unwrap();

                        // Start by assuming we haven't found it.
                        let mut next_index_option = None;

                        // Only search the latter part of the vector if it's safe to do so.
                        if start_index < map_state.connections.len() {
                            next_index_option = map_state.connections[start_index..]
                                .iter()
                                .position(|c| {
                                    selected_note == c.from_id || selected_note == c.to_id.unwrap()
                                })
                                .map(|i| i + start_index);
                        }

                        // If that connection was last in the vector or no match was found after it -
                        // search from the start
                        if next_index_option.is_none() {
                            next_index_option = map_state.connections
                                .iter()
                                .position(|c| {
                                    selected_note == c.from_id || selected_note == c.to_id.unwrap()
                                });
                        }

                        if let Some(next_index) = next_index_option {
                            // If found one - remove it and put it in focus.
                            // Note: it will always "find" one - since
                            map_state.take_out_connection(next_index);
                            map_state.editing_connection_index = Some(next_index);
                        }
                    }
                }
            }

            // Delete the selected connection
            KeyCode::Char('d') => {
                if map_state.visual_editing_a_connection {
                    map_state.can_exit = false;

                    // Delete that connection
                    map_state.focused_connection = None;

                    // Exit
                    map_state.visual_connection = false;
                    map_state.visual_editing_a_connection = false;
                    map_state.editing_connection_index = None;
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
                if let Some(focused_connection) = map_state.focused_connection.as_mut() {
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

            if let Some(selected_note) = map_state.selected_note {
                if let Some(note) = map_state.notes.get_mut(&selected_note) {
                    note.selected = false;
                }
            }
        }
        // Switch to Insert mode
        KeyCode::Char('i') => map_state.current_mode = Mode::Insert,

        // Switch to Move State for the Visual Mode
        KeyCode::Char('m') => map_state.visual_move = true,

        // Switch to Connection Sate for Visual Mode
        // This block selects the "first" connection that this note
        // is associated with, if it has any.
        KeyCode::Char('c') => {
            if let Some(selected_note) = map_state.selected_note {
                if let Some(index) = map_state.connections.iter().position(|c| {
                    selected_note == c.from_id || selected_note == c.to_id.unwrap()
                    // unwrap() is safe here since all the connections have an endpoint if
                    // they are in the connections vector.
                }) {
                    map_state.take_out_connection(index);
                    map_state.editing_connection_index = Some(index);
                    map_state.visual_connection = true;
                    map_state.visual_editing_a_connection = true;
                }
            }
        }

        // Add a new Connection for the selected note
        KeyCode::Char('C') => {
            if let Some(selected_note) = map_state.selected_note {
                map_state.focused_connection = Some(
                    Connection {
                        from_id: selected_note,
                        from_side: Side::Right, // default side
                        to_id: None,
                        to_side: None,
                        color: Color::White,
                    }
                );

                map_state.visual_connection = true;
                
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
            if let Some(selected_note) = map_state.selected_note {
                if let Some(note) = map_state.notes.get_mut(&selected_note) {
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

fn map_insert_kh(map_state: &mut MapState, key: KeyEvent) -> AppAction {
    if let Some(selected_note) = &map_state.selected_note {
        match key.code {
            // Switch back to Normal Mode
            KeyCode::Esc => {
                map_state.current_mode = Mode::Normal;
                if let Some(note) = map_state.notes.get_mut(selected_note) {
                    note.selected = false;
                    // Reset cursor position for the next time entering Insert mode.
                    map_state.cursor_pos = 0;
                }
            }

            // --- Text Editing ---
            KeyCode::Char(c) => {
                map_state.can_exit = false;
                if let Some(note) = map_state.notes.get_mut(selected_note) {
                    // Insert the typed character at the cursor's current position.
                    note.content.insert(map_state.cursor_pos, c);
                    // Move the cursor forward one position.
                    map_state.cursor_pos += 1;
                }
            }
            KeyCode::Enter => {
                map_state.can_exit = false;
                if let Some(note) = map_state.notes.get_mut(selected_note) {
                    // Insert a newline character at the cursor's position.
                    note.content.insert(map_state.cursor_pos, '\n');
                    // Move the cursor forward one position.
                    map_state.cursor_pos += 1;
                }
            }
            KeyCode::Backspace => {
                map_state.can_exit = false;
                if let Some(note) = map_state.notes.get_mut(selected_note) {
                    // We can only backspace if the cursor is not at the very beginning of the text.
                    if map_state.cursor_pos > 0 {
                        // To delete the character *before* the cursor, we must remove the character
                        // at the index `cursor_pos - 1`.
                        note.content.remove(map_state.cursor_pos - 1);
                        // After removing the character, we move the cursor's position back by one.
                        map_state.cursor_pos -= 1;
                    }
                }
            }
            KeyCode::Left => {
                if map_state.cursor_pos > 0 { 
                    map_state.cursor_pos -= 1 
                }
            }
            KeyCode::Right => {
                if let Some(note) = map_state.notes.get(selected_note) {
                    if map_state.cursor_pos < note.content.len() {
                        map_state.cursor_pos += 1;
                    }
                }
            }
            KeyCode::Up => move_cursor_up(map_state), 
            KeyCode::Down => move_cursor_down(map_state),
            _ => {}
        }
    }

    // Any action in Insert mode triggers a redraw.
    map_state.clear_and_redraw();

    AppAction::Continue
}

fn map_delete_kh(map_state: &mut MapState, key: KeyEvent) -> AppAction {
    if let Some(selected_note) = &map_state.selected_note {
        match key.code {
            // Switch back to Visual Mode
            KeyCode::Esc => {
                map_state.current_mode = Mode::Visual;
            }
            // Confirm deleting the selected note
            KeyCode::Char('d') => {
                map_state.can_exit = false;
                
                map_state.notes.remove(selected_note);

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


/// Go forward a page in the help screen
fn help_next_page(map_state: &mut MapState) {
    if let Some(current_page) = &mut map_state.help_screen {
        map_state.help_screen = Some(
            match current_page {
                1 => 2,
                2 => 3,
                3 => 4,
                4 => 5,
                5 => 1,
                _ => unreachable!(),
        });
    }
}

/// Go back a page in the help screen
fn help_previous_page(map_state: &mut MapState) {
    if let Some(current_page) = &mut map_state.help_screen {
        map_state.help_screen = Some(
            match current_page {
                1 => 5,
                2 => 1,
                3 => 2,
                4 => 3,
                5 => 4,
                _ => unreachable!(),
        });
    }
}

fn move_viewport(map_state: &mut MapState, axis: &str, amount: isize) {
    match axis {
        "x" => {
            if amount > 0 {
                map_state.view_pos.x += amount as usize;
            } else {
                map_state.view_pos.x = map_state.view_pos.x.saturating_sub(amount.abs() as usize);
            }
        }
        "y" => {
            if amount > 0 {
                map_state.view_pos.y += amount as usize;
            } else {
                map_state.view_pos.y = map_state.view_pos.y.saturating_sub(amount.abs() as usize);
            }
        }
        _ => {}
    }
    
    map_state.can_exit = false;
}

fn move_note(map_state: &mut MapState, axis: &str, amount: isize) {    
    if let Some(selected_note) = map_state.selected_note {
        // Get note dimensions for:
        // When a note moves beyond the screen edge, automatically adjust the viewport to keep it visible.
        let (mut note_width,
            mut note_height) = if let Some(note) = map_state.notes.get_mut(&selected_note) { 
                note.get_dimensions()
        } else {
            unreachable!()
        };
        // Enforce a minimum size for readability.
        if note_width < 20 { note_width = 20; }
        if note_height < 4 { note_height = 4; }
        // Add space for cursor
        note_width+=1;

        if let Some(note) = map_state.notes.get_mut(&selected_note) {
            match axis {
                "x" => {
                    if amount > 0 {
                        // First, update the note's x-coordinate.
                        note.x += amount as usize;
                        // Check if the right edge of the note is past the right edge of the screen.
                        if note.x + note_width as usize > map_state.view_pos.x + map_state.screen_width {
                            // If it is, move the viewport right to keep the note in view.
                            map_state.view_pos.x += amount as usize;
                        }
                    } else {
                        // First, update the note's x-coordinate.
                        note.x = note.x.saturating_sub(amount.abs() as usize);
                        // Then, check if the left edge of the note is now to the left of the viewport's edge.
                        if note.x < map_state.view_pos.x {
                            // If it is, move the viewport left to keep the note in view.
                            map_state.view_pos.x = map_state.view_pos.x.saturating_sub(amount.abs() as usize);
                        }
                    }
                }
                "y" => {
                    if amount > 0 {
                        // Update the note's y-coordinate.
                        note.y += amount as usize; 
                        // Check if the bottom edge of the note is below the visible screen area.
                        // We subtract 2 from the screen height to account for the bottom info bar.
                        if note.y as isize + note_height as isize > map_state.view_pos.y as isize + map_state.screen_height as isize - 2 {
                            // If it is, move the viewport down to keep the note in view.
                            map_state.view_pos.y += amount as usize;
                        }
                    } else {
                        // Update the note's y-coordinate.
                        note.y = note.y.saturating_sub(amount.abs() as usize);
                        // Then, check if the top edge of the note is now above the top edge of the viewport.
                        if note.y < map_state.view_pos.y {
                            // If it is, move the viewport down to up the note in view.
                            map_state.view_pos.y = map_state.view_pos.y.saturating_sub(amount.abs() as usize);
                        }
                    }
                }
                _ => {}
            }
            
            map_state.can_exit = false;
        }
    }
}

fn switch_notes_focus(map_state: &mut MapState, key: &str) {
    if let Some(selected_note) = map_state.selected_note {
        // --- 1. Get the starting position ---
        // Safely get the coordinates of the currently selected note.
        // We copy the `x` and `y` values into local variables so we are
        // no longer borrowing `app.notes`, which allows us to borrow it again later.
        let (selected_note_x, selected_note_y) = if let Some(note) = map_state.notes.get(&selected_note) {
            (note.x, note.y)
        } else {
            // If there's no selected note for some reason, we can't proceed.
            return;
        };

        // --- 2. Find all candidate notes ---
        // Use an iterator chain to declaratively find all valid notes to jump to.
        let candidate_ids: Vec<usize> = map_state.notes.iter()
            .filter(|(id, note)| {
                let dx = (note.x as isize - selected_note_x as isize).abs();
                let dy = (note.y as isize - selected_note_y as isize).abs();

                // This logic defines a "cone of selection" to find intuitive candidates.
                // A note is a valid candidate only if it's both in the correct direction
                // AND the distance along the primary axis of movement is greater than the
                // distance on the secondary axis. For example, when moving right ('l'),
                // a note is only a candidate if it is truly "more to the right" than it is
                // "up" or "down".

                // First, determine if the note is a valid candidate based on the direction.
                let is_in_direction = match key {
                    // For vertical movement ('j'/'k'), the vertical distance must be greater.
                    "j" | "Down" => note.y > selected_note_y && dy > dx,
                    "k" | "Up" => note.y < selected_note_y && dy > dx,
                    // For horizontal movement ('l'/'h'), the horizontal distance must be greater.
                    "l" | "Right" => note.x > selected_note_x && dx > dy,
                    "h" | "Left" => note.x < selected_note_x && dx > dy,
                    // If an invalid character is passed, no notes will be candidates.
                    _ => false,
                };
            
                // The final condition is: is it in the right direction AND not the note we started on?
                is_in_direction && **id != selected_note
            })
            // Only need the IDs.
            .map(|(id, _)| *id)
            .collect();

        // --- 3. Find the single best candidate ---
        let closest_note_id_option = match key {
            "j" | "Down" => {
                // Find the closest note below
                candidate_ids.iter().min_by_key(|&&id| {
                    let note = &map_state.notes[&id];
                    // Calculate horizontal distance.
                    let x_dist = (note.x as isize - selected_note_x as isize).abs() as usize;
                
                    // The key is a tuple: `(vertical_position, horizontal_distance)`.
                    // It will compare tuples element by element, find the note with the
                    // minimum `y` value. If there's a tie, it will use `x_dist` to find the winner.
                    (note.y, x_dist)
                })
            }
            "k" | "Up" => {
                // Find the closest note above
                candidate_ids.iter().max_by_key(|&&id| {
                    let note = &map_state.notes[&id];
                    let x_dist = (note.x as isize - selected_note_x as isize).abs() as usize;
                
                    (note.y, Reverse(x_dist))
                })
            }
            "l" | "Right" => {
                // Find the closest note to the right
                candidate_ids.iter().min_by_key(|&&id| {
                    let note = &map_state.notes[&id];
                    let y_dist = (note.y as isize - selected_note_y as isize).abs() as usize;

                    (note.x, y_dist)
                })
            }
            "h" | "Left" => {
                candidate_ids.iter().max_by_key(|&&id| {
                    let note = &map_state.notes[&id];
                    let y_dist = (note.y as isize - selected_note_y as isize).abs() as usize;

                    (note.x, Reverse(y_dist))
                })
            }
            _ => None,
        };
    

        // --- 4. Perform the selection switch and view update ---
        // This block only runs if `closest_note_id_option` is `Some`, meaning a note was found.
        if let Some(&id) = closest_note_id_option { 
            // First, deselect the old note. This mutable borrow is short-lived.
            if let Some(note) = map_state.notes.get_mut(&selected_note) {
                note.selected = false;
            }

            // Then, update the application's state to the new ID.
            map_state.selected_note = Some(id);

            // Finally, select the new note. This is another, separate mutable borrow.
            if let Some(note) = map_state.notes.get_mut(&id) {
                note.selected = true;
            }

            // As a final step, center the viewport on the newly selected note.
            if let Some(note) = map_state.notes.get(&id) {
                map_state.view_pos.x = note.x.saturating_sub(map_state.screen_width/2);
                map_state.view_pos.y = note.y.saturating_sub(map_state.screen_height/2);
            }

            // If in the middle of creating a connection:
            if map_state.visual_connection {
                if let Some(focused_connection) = map_state.focused_connection.as_mut() {
                    // only create a connection on note other than the start note
                    // (otherwise could have a connection going from start note to itself)
                    if id == focused_connection.from_id {
                        // if tried to make a connection (jumped to) from the start note
                        // to itself - just set the "to" fields to None (the default)
                        focused_connection.to_id = None;
                        focused_connection.to_side = None;
                    } else {
                        // update the `to_id` of "in-progress" connection to point to the newly found note.
                        focused_connection.to_id = Some(id); // id of the note that just jumped to
                        focused_connection.to_side = Some(Side::Right); // default side
                    }

                    map_state.can_exit = false;
                }
            }
        }
    }
}

fn move_cursor_up(map_state: &mut MapState) {
    if let Some(selected_note) = &map_state.selected_note {
        if let Some(note) = map_state.notes.get(selected_note) {
            // --- 1. Find the start of the current and previous lines ---

            // `current_line_start` will hold the starting index of the line the cursor is on.
            let mut current_line_start = 0;
            // `previous_line_start` will hold the starting index of the line *above* the cursor.
            let mut previous_line_start = 0;

            // Iterate through the lines of the note to find the cursor's position.
            for line in note.content.lines() {
                // Check if the end of the current line is past the cursor's position.
                // If it is, we've found the line the cursor is on.
                if current_line_start + line.chars().count() >= map_state.cursor_pos {
                    break;
                }

                // If we haven't found the cursor's line yet, we update our variables.
                // The current line's start becomes the new 'previous' line start.
                previous_line_start = current_line_start;
                // We update the current line's start to the beginning of the *next* line,
                // accounting for the current line's length plus the newline character.
                current_line_start += line.chars().count() + 1;
            }

            // --- 2. Handle the edge case of being on the first line ---

            // If `current_line_start` is still 0, it means the loop broke on the first
            // line. We can't move up, so we exit early.
            if current_line_start == 0 { return }

            // --- 3. Calculate the new cursor position ---

            // Determine the cursor's horizontal position (column) within its current line.
            let index_in_the_current_line = map_state.cursor_pos - current_line_start;

            // Calculate the character length of the previous line.
            let previous_line_length = current_line_start - previous_line_start - 1;

            // --- 4. Set the new cursor position, snapping if necessary ---

            // Check if the previous line is long enough to place the cursor at the same column.
            if previous_line_length > index_in_the_current_line {
                // If it is, the new position is the start of the previous line plus the column offset.
                map_state.cursor_pos = previous_line_start + index_in_the_current_line;
            } else {
                // If the previous line is shorter, "snap" the cursor to the end of that line.
                map_state.cursor_pos = previous_line_start + previous_line_length;
            }
        }
    }
}

fn move_cursor_down(map_state: &mut MapState) {
    if let Some(selected_note) = &map_state.selected_note {
        if let Some(note) = map_state.notes.get(selected_note) {
            // --- 1. Find the start of the current and next lines ---
            let mut current_line_start = 0;
            let mut next_line_start = 0;

            // Iterate through the lines to find the cursor's current line and the start of the next.
            for line in note.content.lines() {
                // The `if` condition checks if the cursor is on the current line being processed.
                // We use `next_line_start` for the check because it holds the starting index
                // of the line we are currently evaluating in the loop.
                if next_line_start + line.chars().count() >= map_state.cursor_pos {
                    // Once we find the correct line, we perform one final update.
                    // `current_line_start` gets the correct value for the cursor's actual line.
                    current_line_start = next_line_start;
                    // `next_line_start` is pushed forward to the start of the *following* line.
                    next_line_start += line.chars().count() + 1;
                    // We've found what we need, so we exit the loop.
                    break;
                }

                // If the cursor isn't on this line, we update the variables for the next iteration.
                current_line_start = next_line_start;
                next_line_start += line.chars().count() + 1;
            }

            // --- 2. Handle the edge case of being on the last line ---

            // If the calculated `next_line_start` is beyond the total length of the note,
            // it means there is no next line to move to, so we exit early.
            if next_line_start > note.content.len() { return }

            // --- 3. Calculate the new cursor position ---

            // Determine the cursor's horizontal position (column) within its current line.
            let index_in_the_current_line = map_state.cursor_pos - current_line_start;

            // To find the length of the next line, we first create a slice of the note
            // content starting from the beginning of the next line.
            let remaining_content = &note.content[next_line_start..];

            // We then search for a newline character within that remaining slice.
            let next_line_length = match remaining_content.find('\n') {
                // If a newline is found, its index within the slice is the length of the next line.
                Some(newline_pos) => newline_pos,
                // If no newline is found, it's the last line, so its length is the length of the entire remaining slice.
                None => remaining_content.len(),
            };

            // --- 4. Set the new cursor position, snapping if necessary ---

            // Check if the next line is long enough to place the cursor at the same column.
            if next_line_length > index_in_the_current_line {
                // If it is, the new position is the start of the next line plus the column offset.
                map_state.cursor_pos = next_line_start + index_in_the_current_line;
            } else {
                // If the next line is shorter, "snap" the cursor to the end of that line.
                map_state.cursor_pos = next_line_start + next_line_length;
            }
        }
    }
}

fn cycle_side(side: Side) -> Side {
    match side {
        Side::Right => Side::Bottom,
        Side::Bottom => Side::Left,
        Side::Left => Side::Top,
        Side::Top => Side::Right,
    }
}

fn cycle_color(color: Color) -> Color {
    match color {
        Color::Red => Color::Green,
        Color::Green => Color::Yellow,
        Color::Yellow => Color::Blue,
        Color::Blue => Color::Magenta,
        Color::Magenta => Color::Cyan,
        Color::Cyan => Color::White,
        Color::White => Color::Black,
        Color::Black => Color::Red,
        _ => Color::White,
    }
}