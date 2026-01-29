use crate::states::map::{DiscardMenuType, Notification};


#[derive(PartialEq, Debug)]
pub struct UIState {
    pub needs_clear_and_redraw: bool,
    pub show_notification: Option<Notification>,
    pub confirm_discard_menu: Option<DiscardMenuType>,
    /// Page number of the currently visible help screen
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
    
    pub fn request_redraw(&mut self) {
        self.needs_clear_and_redraw = true;
    }

    pub fn mark_redrawn(&mut self) {
        self.needs_clear_and_redraw = false;
    }

    pub fn set_notification(&mut self, notification: Notification) {
        self.show_notification = Some(notification);
    }

    pub fn clear_notification(&mut self) {
        self.show_notification = None;
    }

    pub fn show_discard_menu(&mut self, menu_type: DiscardMenuType) {
        self.confirm_discard_menu = Some(menu_type);
    }

    pub fn hide_discard_menu(&mut self) {
        self.confirm_discard_menu = None;
    }

    pub fn show_help(&mut self, page: usize) {
        self.help_screen = Some(page);
    }

    pub fn hide_help(&mut self) {
        self.help_screen = None;
    }

    pub fn is_help_visible(&self) -> bool {
        self.help_screen.is_some()
    }
}