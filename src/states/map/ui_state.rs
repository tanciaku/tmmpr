use crate::states::map::{DiscardMenuType, Notification};


#[derive(PartialEq, Debug)]
pub struct UIState {
    pub needs_clear_and_redraw: bool,
    pub show_notification: Option<Notification>,
    /// Whether to render a menu for confirming to discard 
    /// unsaved changes
    pub confirm_discard_menu: Option<DiscardMenuType>,
    /// Whether to show the help screen, and take all input for it
    /// usize - represents the page of the help screen
    pub help_screen: Option<usize>,
}

impl UIState {
    pub fn new() -> Self {
        Self {
            needs_clear_and_redraw: true,
            show_notification: None,
            confirm_discard_menu: None,
            help_screen: None,
        }
    }
    
    /// Sets the flag to clear and redraw the screen on the next frame.
    pub fn request_redraw(&mut self) {
        self.needs_clear_and_redraw = true;
    }

    /// Clears the redraw flag after the screen has been redrawn.
    pub fn mark_redrawn(&mut self) {
        self.needs_clear_and_redraw = false;
    }

    /// Sets a notification to display.
    pub fn set_notification(&mut self, notification: Notification) {
        self.show_notification = Some(notification);
    }

    /// Clears the current notification.
    pub fn clear_notification(&mut self) {
        self.show_notification = None;
    }

    /// Shows the discard changes confirmation menu.
    pub fn show_discard_menu(&mut self, menu_type: DiscardMenuType) {
        self.confirm_discard_menu = Some(menu_type);
    }

    /// Hides the discard changes confirmation menu.
    pub fn hide_discard_menu(&mut self) {
        self.confirm_discard_menu = None;
    }

    /// Shows the help screen at the specified page.
    pub fn show_help(&mut self, page: usize) {
        self.help_screen = Some(page);
    }

    /// Hides the help screen.
    pub fn hide_help(&mut self) {
        self.help_screen = None;
    }

    /// Returns true if the help screen is currently visible.
    pub fn is_help_visible(&self) -> bool {
        self.help_screen.is_some()
    }
}