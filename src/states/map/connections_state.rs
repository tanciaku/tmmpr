use std::collections::HashMap;
use ratatui::style::Color;
use serde::{Serialize, Deserialize};

use super::enums::Side;

/// Represents a directional connection between notes in the map.
///
/// Connections can be in-progress (only `from` specified) or complete (both `from` and `to`).
/// This allows drawing connections interactively before the user selects a target note.
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
pub struct Connection {
    pub from_id: usize,
    pub from_side: Side,
    /// None for in-progress connections being drawn by the user
    pub to_id: Option<usize>,
    /// None for in-progress connections being drawn by the user
    pub to_side: Option<Side>,
    /// Custom serde implementation in utils handles Color serialization
    #[serde(with = "crate::utils")]
    pub color: Color,
}

#[derive(PartialEq, Debug)]
pub struct ConnectionsState {
    pub connections: Vec<Connection>,
    /// Maps note IDs to all connections touching that note (both from and to).
    /// Used for efficient lookup when rendering junction characters: ┴ ┬ ┤ ├
    pub connection_index: HashMap<usize, Vec<Connection>>,
    /// Connection currently being created or edited by the user.
    pub focused_connection: Option<Connection>,
    /// Original position in the connections vector when a connection was removed for editing.
    pub editing_connection_index: Option<usize>,
}

impl ConnectionsState {
    pub fn new() -> Self {
        Self {
            connections: vec![],
            connection_index: HashMap::new(),
            focused_connection: None,
            editing_connection_index: None,
        }
    }

    /// Finalizes the focused connection by adding it to the permanent connections list.
    /// Incomplete connections (missing `to_id`) are discarded.
    /// Updates the connection_index for both endpoints.
    pub fn stash_connection(&mut self) {
        if let Some(connection) = self.focused_connection.take() {
            if connection.to_id.is_some() {
                self.connections.push(connection);

                let indexed_connection_start = self.connection_index.entry(connection.from_id).or_default();
                indexed_connection_start.push(connection);

                let indexed_connection_end = self.connection_index.entry(connection.to_id.unwrap()).or_default();
                indexed_connection_end.push(connection);
            }
        }
    }

    /// Removes a connection from permanent storage and makes it the focused connection for editing.
    /// Updates the connection_index for both endpoints.
    pub fn take_out_connection(&mut self, index: usize) {
        let connection_removed = self.connections.remove(index);
        self.focused_connection = Some(connection_removed);

        if let Some(index_vec) = self.connection_index.get_mut(&connection_removed.from_id) {
            index_vec.retain(|c| c != &connection_removed);
        }

        if let Some(index_vec) = self.connection_index.get_mut(&connection_removed.to_id.unwrap()) {
            index_vec.retain(|c| c != &connection_removed);
        }
    }
}