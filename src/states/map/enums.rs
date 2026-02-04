use serde::{Serialize, Deserialize};

/// Represents the application's current input mode, similar to Vim.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Mode {
    Normal,
    VisualSelect,
    VisualMove,
    VisualConnectAdd, // FIXME?: redundant? - can just get that from .editing_connection_index being Some() or None?
    VisualConnectEdit, // FIXME?: redundant? - can just get that from .editing_connection_index being Some() or None?
    /// Supports both modal (Vim-style) and non-modal editing.
    /// `None` indicates non-modal editing, `Some` indicates modal with the current sub-mode.
    Edit(Option<ModalEditMode>),
    Delete,
}

/// Modal editing sub-modes within Edit mode.
/// Allows Vim-style navigation (Normal) and text insertion (Insert) while editing a note.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ModalEditMode {
    Normal,
    Insert,
}

/// Represents which side of a note a connection is attached to.
///
/// Used to specify the connection point on both the source and target notes.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

/// Notifications displayed in the status bar.
#[derive(PartialEq, Debug)]
pub enum Notification {
    SaveSuccess,
    SaveFail,
    BackupSuccess,
    BackupFail,
    BackupRecordFail,
}

/// Tracks the user's intended destination when discarding unsaved changes.
/// Used to route correctly after the discard confirmation.
#[derive(PartialEq, Debug)]
pub enum DiscardMenuType {
    Start,
    Settings
}

#[derive(PartialEq, Debug)]
pub enum BackupResult {
    BackupSuccess,
    BackupFail,
}
