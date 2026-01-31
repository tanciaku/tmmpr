//! # tmmpr
//!
//! A library for node graph data structures and algorithms.
//!
//! ## ⚠️ Work in Progress
//!
//! This library is currently under active development and extraction.
//! **Do not expect anything to be functional or stable at this time.**
//! The API is subject to breaking changes without notice.
//!
//! ## Library Usage
//!
//! The `graph` module provides core data structures for node graphs:
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