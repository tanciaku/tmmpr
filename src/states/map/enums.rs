/// Represents the application's current input mode, similar to Vim.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Mode {
    Normal,
    Visual,
    VisualMove,
    VisualConnect,
    Edit,
    EditNormal,
    EditInsert,
    Delete,
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
    Settings,
}
