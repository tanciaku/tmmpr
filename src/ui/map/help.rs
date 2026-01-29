use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Style},
    widgets::{Block, List, ListItem},
    text::{Span, Line},
};

/// Renders the help page UI with navigation controls and page-specific content.
pub fn render_map_help_page(frame: &mut Frame, page_number: usize) {
        let help_screen_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .split(frame.area());

        let help_screen_controls_text = Line::from("? / F1 - toggle help page        l / Right / Tab - go forward a page        h / Left - go back a page").alignment(Alignment::Center);
        frame.render_widget(help_screen_controls_text, help_screen_layout[2]);

        match page_number {
            1 => {
                frame.render_widget(Block::bordered().border_style(Color::White), help_screen_layout[1]);
                
                let page_ind_1_text = Line::from("  Page 1/5: General");
                frame.render_widget(page_ind_1_text, help_screen_layout[0]);

                let page_1_content = vec![
                    Line::from(""),
                    Line::from("Page contents:"),
                    Line::from(""),
                    Line::from("1 - General"),
                    Line::from("2 - Normal Mode"),
                    Line::from("3 - Visual Mode"),
                    Line::from("4 - Visual (Move), Visual (Connection)"),
                    Line::from("5 - Edit Mode"),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from("General info:"),
                    Line::from(""),
                    Line::from("Cannot exit the app from the help screen,"),
                    Line::from("can do so from the Map Screen, Normal Mode."),
                    Line::from(""),
                    Line::from(""),
                    Line::from("Terminal cells are coordinates."),
                    Line::from(""),
                    Line::from(""),
                    Line::from("There are only positive coordinates, think of it as a whiteboard"),
                    Line::from("starting from the top left corner and going right and bottom infinitely."),
                    Line::from(""),
                    Line::from("0,0 x-------->"),
                    Line::from("y"),
                    Line::from("|"),
                    Line::from("|"),
                    Line::from("|"),
                    Line::from("v"),
                    Line::from(""),
                    Line::from(""),
                    Line::from("To zoom in/out, adjust your terminal's font size (Ctrl +/- or Cmd +/- on macOS)."),
                    Line::from("The method varies by terminal emulator - check your terminal's documentation."),
                    Line::from(""),
                    Line::from(""),
                    Line::from("Changes are automatically saved to the map file every 20 seconds."),
                    Line::from("You can adjust the auto-save interval or disable it in the settings menu."),
                    Line::from(""),
                    Line::from("If you make changes and try to quit before saving them / before the changes are"),
                    Line::from("automatically saved - you will be prompted to either cancel exiting or discard those changes."),
                ];

                let page_1_content: Vec<ListItem> = page_1_content
                    .into_iter()
                    .map(ListItem::new)
                    .collect();
                let page_1_content = List::new(page_1_content);

                frame.render_widget(page_1_content, help_screen_layout[1].inner(Margin::new(3, 1)));
            }
            2 => {
                frame.render_widget(Block::bordered().border_style(Color::White), help_screen_layout[1]);
                
                let page_ind_2_text = Line::from("  Page 2/5: Normal Mode");
                frame.render_widget(page_ind_2_text, help_screen_layout[0]);
                
                let page_2_content = vec![
                    Line::from(""),
                    Line::from("General Commands"),
                    Line::from(""),
                    Line::from("F1 / ?: Toggle help screen"),
                    Line::from("q:      Quit to start screen (if saved) or show confirm discard menu"),
                    Line::from("s:      Save map file"),
                    Line::from("o:      Open the settings"),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from("Viewport Navigation"),
                    Line::from(""),
                    Line::from("h / Left Arrow:        Move viewport left by 1"),
                    Line::from("H / Shift+Left Arrow:  Move viewport left by 5"),
                    Line::from(""),
                    Line::from("j / Down Arrow:        Move viewport down by 1"),
                    Line::from("J / Shift+Down Arrow:  Move viewport down by 5"),
                    Line::from(""),
                    Line::from("k / Up Arrow:          Move viewport up by 1"),
                    Line::from("K / Shift+Up Arrow:    Move viewport up by 5"),
                    Line::from(""),
                    Line::from("l / Right Arrow:       Move viewport right by 1"),
                    Line::from("L / Shift+Right Arrow: Move viewport right by 5"),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from("Note Operations"),
                    Line::from(""),
                    Line::from("a: Add a new note"),
                    Line::from("v: Select closest note to the center of the screen"),
                    Line::from("     and switch to Visual Mode"),
                ];

                let page_2_content: Vec<ListItem> = page_2_content
                    .into_iter()
                    .map(ListItem::new)
                    .collect();
                let page_2_content = List::new(page_2_content);

                frame.render_widget(page_2_content, help_screen_layout[1].inner(Margin::new(3, 1)));
            }
            3 => {
                frame.render_widget(Block::bordered().border_style(Color::Yellow), help_screen_layout[1]);
                
                let page_ind_3_text = Line::from(vec![Span::raw("  Page 3/5: "), Span::styled("Visual Mode", Style::new().fg(Color::Yellow))]);
                frame.render_widget(page_ind_3_text, help_screen_layout[0]);

                let page_3_content = vec![
                    Line::from(""),
                    Line::from("General Commands"),
                    Line::from(""),
                    Line::from("ESC: Switch back to Normal mode"),
                    Line::from("i:   Switch to Edit mode"),
                    Line::from("m:   Switch to Move state"),
                    Line::from("c:   Switch to Connection state (edit existing connection(s))"),
                    Line::from("C:   Add a new connection from the selected note"),
                    Line::from("d:   Choose the selected note for deletion."),
                    Line::from("       (brings up the confirm to delete prompt)"),
                    Line::from("e:   Cycle through note colors"),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from("Note Focus Switching"),
                    Line::from(""),
                    Line::from("h / Left Arrow:  Switch focus to note on the left"),
                    Line::from("j / Down Arrow:  Switch focus to note below"),
                    Line::from("k / Up Arrow:    Switch focus to note above"),
                    Line::from("l / Right Arrow: Switch focus to note on the right"),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from("NOTE: Focus switching uses directional logic - notes must be primarily"),
                    Line::from("in the intended direction to be selectable. Meaning that sometimes you "),
                    Line::from("might not be able to switch focus to a particular note. In that case "),
                    Line::from("either move the view to be centered with the note and reselect, or move"),
                    Line::from("the note currently selected to a different spot and try to switching"),
                    Line::from("focus from it again."),
                ];

                let page_3_content: Vec<ListItem> = page_3_content
                    .into_iter()
                    .map(ListItem::new)
                    .collect();
                let page_3_content = List::new(page_3_content);

                frame.render_widget(page_3_content, help_screen_layout[1].inner(Margin::new(3, 1)));
            }
            4 => {
                // Split horizontally to show Move and Connection sub-modes side by side
                let help_page_4_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ])
                    .split(help_screen_layout[1]);
                frame.render_widget(Block::bordered().border_style(Color::Yellow), help_page_4_layout[0]);
                frame.render_widget(Block::bordered().border_style(Color::Yellow), help_page_4_layout[1]);
                
                let page_ind_4_text = Line::from(vec![Span::raw("  Page 4/5: "), Span::styled("Visual (Move), Visual (Connection)", Style::new().fg(Color::Yellow))]);
                frame.render_widget(page_ind_4_text, help_screen_layout[0]);

                let page_4_content_left = vec![
                    Line::from(""),
                    Line::from("Visual (Move)"),
                    Line::from(""),
                    Line::from(""),
                    Line::from("m:   Switch back to Visual mode normal state"),
                    Line::from("ESC: Switch back to Normal mode"),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from("h / Left Arrow:        Move note left by 1"),
                    Line::from("H / Shift+Left Arrow:  Move note left by 5"),
                    Line::from(""),
                    Line::from("j / Down Arrow:        Move note down by 1"),
                    Line::from("J / Shift+Down Arrow:  Move note down by 5"),
                    Line::from(""),
                    Line::from("k / Up Arrow:          Move note up by 1"),
                    Line::from("K / Shift+Up Arrow:    Move note up by 5"),
                    Line::from(""),
                    Line::from("l / Right Arrow:       Move note right by 1"),
                    Line::from("L / Shift+Right Arrow: Move note right by 5"),
                ];

                let page_4_content_left: Vec<ListItem> = page_4_content_left
                    .into_iter()
                    .map(ListItem::new)
                    .collect();
                let page_4_content_left = List::new(page_4_content_left);

                frame.render_widget(page_4_content_left, help_page_4_layout[0].inner(Margin::new(3, 1)));

                let page_4_content_right = vec![
                    Line::from(""),
                    Line::from("Visual (Connection)"),
                    Line::from(""),
                    Line::from(""),
                    Line::from("c: Switch back to Visual mode normal state"),
                    Line::from("r: Rotate connection start/end side"),
                    Line::from("n: Cycle through available connections on this note"),
                    Line::from("d: Delete selected connection"),
                    Line::from("e: Cycle through connection colors"),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from("Target Note Selection (selecting connection endpoint)"),
                    Line::from(""),
                    Line::from("h / Left Arrow:  Switch focus to note on the left"),
                    Line::from("j / Down Arrow:  Switch focus to note below"),
                    Line::from("k / Up Arrow:    Switch focus to note above"),
                    Line::from("l / Right Arrow: Switch focus to note on the right"),
                ];

                let page_4_content_right: Vec<ListItem> = page_4_content_right
                    .into_iter()
                    .map(ListItem::new)
                    .collect();
                let page_4_content_right = List::new(page_4_content_right);

                frame.render_widget(page_4_content_right, help_page_4_layout[1].inner(Margin::new(3, 1)));
            }
            5 => {
                frame.render_widget(Block::bordered().border_style(Color::Blue), help_screen_layout[1]);
                
                let page_ind_5_text = Line::from(vec![Span::raw("  Page 5/5: "), Span::styled("Edit Mode", Style::new().fg(Color::Blue))]);
                frame.render_widget(page_ind_5_text, help_screen_layout[0]);

                let page_5_content = vec![
                    Line::from(""),
                    Line::from("Edit Mode (text editing)"),
                    Line::from(""),
                    Line::from("By default, it operates like a standard text editor. When 'modal editing' is enabled in settings,"),
                    Line::from("Edit Mode becomes vim-inspired with two states: Edit Insert Mode (same functionality as default) and "),
                    Line::from("Edit Normal Mode with very limited vim navigation."),
                    Line::from(""),
                    Line::from(""),
                    Line::from("--------------------------------------------------------------------------------"),
                    Line::from(""),
                    Line::from("Normal Edit Mode (Default):"),
                    Line::from(""),
                    Line::from("Typing/Editing: Any character, Enter (new line), Backspace, arrow keys for navigation"),
                    Line::from("ESC - exit (returns to Normal Mode)"),
                    Line::from(""),
                    Line::from(""),
                    Line::from("--------------------------------------------------------------------------------"),
                    Line::from(""),
                    Line::from("Modal Edit Mode - Normal:"),
                    Line::from(""),
                    Line::from("Navigation:"),
                    Line::from("h/j/k/l (left/down/up/right)"),
                    Line::from("g - beginning"),
                    Line::from("G - end"),
                    Line::from("w - next word"),
                    Line::from("b - previous word"),
                    Line::from(""),
                    Line::from("Editing:"),
                    Line::from("i - enter Insert Mode"),
                    Line::from("a - move cursor after current character and enter Insert Mode"),
                    Line::from("x - delete character (just deletes it, there is no register/'clipboard')"),
                    Line::from("ESC - exit Edit Mode (returns to main Normal Mode)"),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from("Modal Edit Mode - Insert:"),
                    Line::from(""),
                    Line::from("Typing/Editing: Same as Normal Edit Mode - any character, Enter, Backspace, arrow keys"),
                    Line::from("ESC - switches to Edit Normal Mode"),
                ];

                let page_5_content: Vec<ListItem> = page_5_content
                    .into_iter()
                    .map(ListItem::new)
                    .collect();
                let page_5_content = List::new(page_5_content);

                frame.render_widget(page_5_content, help_screen_layout[1].inner(Margin::new(3, 1)));
            }
            _ => {}
        }
}
