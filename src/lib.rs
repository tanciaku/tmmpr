//! # tmmpr
//!
//! Graph visualization library for terminal applications.
//!
//! ## For Library Users
//! Use the `graph` module for core data structures and algorithms:
//! ```rust
//! use tmmpr::graph::{Note, Point, calculate_path};
//! ```
//!
//! ## Note
//! Other modules are internal implementation details and may change.

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