use serde::{Serialize, Deserialize};

/// Represents the application's current input mode, similar to Vim.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Mode {
    /// Default mode for navigation and commands.
    Normal,
    /// Mode for selecting or manipulating notes (not yet implemented).
    Visual,
    /// Mode for editing the text content of a note.
    /// Option<> - represents whether Modal Editing is enabled for Edit Mode.
    Edit(Option<ModalEditMode>),
    Delete,
}

/// Represents the two possible Modal Editing modes for Edit Mode
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

/// Which notification to display at the bottom of the screen
#[derive(PartialEq, Debug)]
pub enum Notification {
    SaveSuccess,
    SaveFail,
    BackupSuccess,
    BackupFail,
    BackupRecordFail,
}

/// A type to determine where the user is trying to exit to
/// without saving changes to the map file.
#[derive(PartialEq, Debug)]
pub enum DiscardMenuType {
    Start,
    Settings
}

/// A type to represent the outcome of attempting to write
/// a backup file
#[derive(PartialEq, Debug)]
pub enum BackupResult {
    BackupSuccess,
    BackupFail,
}
