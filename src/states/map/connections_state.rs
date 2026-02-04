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
/// Manages the bidirectional relationship between connections and notes.
/// 
/// Maintains two synchronized data structures:
/// - `connections`: The authoritative list of all connections
/// - `connection_index`: Fast lookup from note ID to connection indices
/// 
/// This encapsulation ensures the index is always consistent with the connections vector.
#[derive(PartialEq, Debug)]
struct ConnectionManager {
    connections: Vec<Connection>,
    /// Maps note IDs to indices in the connections vector.
    /// Both `from_id` and `to_id` are indexed for each connection.
    connection_index: HashMap<usize, Vec<usize>>,
}

impl ConnectionManager {
    fn new() -> Self {
        Self {
            connections: Vec::new(),
            connection_index: HashMap::new(),
        }
    }
    
    /// Creates a manager from existing data, rebuilding the index.
    /// Used when loading from disk.
    fn from_connections(connections: Vec<Connection>) -> Self {
        let mut manager = Self::new();
        for connection in connections {
            if connection.to_id.is_some() { // Validate
                manager.add(connection);
            }
        }
        manager
    }

    /// Adds a connection and updates the index for both endpoints.
    /// Returns the index where the connection was added.
    fn add(&mut self, connection: Connection) -> usize {
        let index = self.connections.len();
        
        self.connections.push(connection);
        
        self.connection_index
            .entry(connection.from_id)
            .or_default()
            .push(index);
        
        self.connection_index
            .entry(connection.to_id.expect("connections should always have an endpoint"))
            .or_default()
            .push(index);
        
        index
    }

    /// Removes a connection and updates the index accordingly.
    /// This is O(n) because we must fix indices after removal.
    fn remove(&mut self, index: usize) -> Connection {
        let connection = self.connections.remove(index);
        
        for indices in self.connection_index.values_mut() {
            if let Some(pos) = indices.iter().position(|&i| i == index) {
                indices.remove(pos);
            }
        }
        
        for indices in self.connection_index.values_mut() {
            for idx in indices.iter_mut() {
                if *idx > index {
                    *idx -= 1;
                }
            }
        }
        
        connection
    }

    /// Removes all connections involving the given note.
    /// Returns the number of connections removed.
    fn remove_all_for_note(&mut self, note_id: usize) -> usize {
        // Get all indices to remove (make a copy because we'll be mutating)
        let indices_to_remove: Vec<usize> = self.get_indices_for_note(note_id).to_vec();
        
        // Remove in reverse order so indices stay valid during iteration
        for &index in indices_to_remove.iter().rev() {
            self.remove(index);
        }
        
        indices_to_remove.len()
    }

    /// Gets all connection INDICES (within a vector of connections) for a given note.
    fn get_indices_for_note(&self, note_id: usize) -> &[usize] {
        self.connection_index
            .get(&note_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Gets references to all CONNECTIONS for a note.
    fn get_connections_for_note(&self, note_id: usize) -> Vec<&Connection> {
        self.get_indices_for_note(note_id)
            .iter()
            .map(|&i| &self.connections[i])
            .collect()
    }

    fn connections(&self) -> &[Connection] {
        &self.connections
    }

    #[cfg(test)]
    fn connection_index(&self) -> &HashMap<usize, Vec<usize>> {
        &self.connection_index
    }

    fn remove_note(&mut self, note_id: usize) {
        self.remove_all_for_note(note_id);
        self.connection_index.remove(&note_id);
    }
}

#[derive(PartialEq, Debug)]
pub struct ConnectionsState {
    manager: ConnectionManager,
    /// Connection currently being created or edited by the user.
    pub focused_connection: Option<Connection>,
    /// Original position in the connections vector when a connection was removed for editing.
    pub editing_connection_index: Option<usize>,
}

impl ConnectionsState {
    pub fn new() -> Self {
        Self {
            manager: ConnectionManager::new(),
            focused_connection: None,
            editing_connection_index: None,
        }
    }

    /// For serialization
    pub fn connections(&self) -> &[Connection] {
        self.manager.connections()
    }

    #[cfg(test)]
    pub fn connection_index(&self) -> &HashMap<usize, Vec<usize>> {
        self.manager.connection_index()
    }
    
    // For deserialization
    pub fn from_connections(connections: Vec<Connection>) -> Self {
        Self {
            manager: ConnectionManager::from_connections(connections),
            focused_connection: None,
            editing_connection_index: None,
        }
    }

    /// Finalizes the focused connection by adding it to the permanent connections list.
    /// Incomplete connections (missing `to_id`) are discarded.
    pub fn stash_connection(&mut self) {
        if let Some(connection) = self.focused_connection.take() {
            if connection.to_id.is_some() {
                self.manager.add(connection);
            }
        }
    }

    /// Removes a connection from permanent storage and makes it the focused connection for editing.
    pub fn take_out_connection(&mut self, index: usize) {
        let connection = self.manager.remove(index);
        self.focused_connection = Some(connection);
    }

    /// Gets all connection INDICES (within a vector of connections) for a given note
    pub fn get_indices_for_note(&self, note_id: usize) -> &[usize] {
        self.manager.get_indices_for_note(note_id)
    }

    /// Removes all connections involving the given note.
    /// Returns the number of connections removed.
    pub fn remove_all_for_note(&mut self, note_id: usize) -> usize {
        self.manager.remove_all_for_note(note_id)
    }
    
    pub fn remove_note(&mut self, note_id: usize) {
        self.manager.remove_note(note_id);
    }

    /// Get all connections touching a note (for rendering junction chars)
    pub fn get_connections_for_note(&self, note_id: usize) -> Vec<&Connection> {
        self.manager.get_connections_for_note(note_id)
    }
}