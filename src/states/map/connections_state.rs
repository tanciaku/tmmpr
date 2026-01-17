use std::collections::HashMap;

use crate::states::map::Connection;


#[derive(PartialEq, Debug)]
pub struct ConnectionsState {
    pub connections: Vec<Connection>,
    /// Separate type for connections, to be able to properly render
    /// connecting characters: ┴ ┬ ┤ ├
    pub connection_index: HashMap<usize, Vec<Connection>>,
    pub focused_connection: Option<Connection>,
    pub visual_editing_a_connection: bool,
    /// Index of the connection being edited, when it was taken out
    /// out the connections vector.
    pub editing_connection_index: Option<usize>,
}

impl ConnectionsState {
    pub fn new() -> Self {
        Self {
            connections: vec![],
            connection_index: HashMap::new(),
            focused_connection: None,
            visual_editing_a_connection: false,
            editing_connection_index: None,
        }
    }

    /// Stashes the currently focused connection into the connections list
    pub fn stash_connection(&mut self) {
        // Take the connection out, leaving None in its place.
        if let Some(connection) = self.focused_connection.take() {
            // Now we own the connection. We can check its fields.
            if connection.to_id.is_some() {
                // If it has a target, we finalize it.
                self.connections.push(connection);

                // Get the Vec for the key, or create a new empty Vec if it's not there
                let indexed_connection_start = self.connection_index.entry(connection.from_id).or_default();
                indexed_connection_start.push(connection); // Now push your item into the Vec

                // Again for the end point.
                let indexed_connection_end = self.connection_index.entry(connection.to_id.unwrap()).or_default();
                indexed_connection_end.push(connection);
            }
            // If it didn't have a target, we just drop it here.
        }
    }

    /// Takes out a connection from the list and makes it the focused connection
    pub fn take_out_connection(&mut self, index: usize) {
        let connection_removed = self.connections.remove(index);
        self.focused_connection = Some(connection_removed);

        // Edit values from corresponding keys associated with the connection
        // (removing the same connection from both indexes (from_id and to_id))
        if let Some(index_vec) = self.connection_index.get_mut(&connection_removed.from_id) {
            // Keep only the connections that are NOT the one we just removed.
            index_vec.retain(|c| c != &connection_removed);
        }

        if let Some(index_vec) = self.connection_index.get_mut(&connection_removed.to_id.unwrap()) {
            // Keep only the connections that are NOT the one we just removed.
            index_vec.retain(|c| c != &connection_removed);
        }
    }
}