//! # tmmpr
//!
//! Terminal-based mind mapping and graph visualization.
//!
//! ## Library Usage
//! Use the `graph` module for core data structures and algorithms:
//! ```rust
//! use tmmpr::graph::{Note, Point, calculate_path};
//! ```
//!
//! Other modules are internal and subject to change.

pub mod graph {
    pub use crate::states::map::{Note, Side};
    pub use crate::utils::geometry::{Point, calculate_path};
}

#[doc(hidden)]
pub mod app;
#[doc(hidden)]
pub mod states;
#[doc(hidden)]
pub mod utils;
#[doc(hidden)]
pub mod ui;
#[doc(hidden)]
pub mod input;