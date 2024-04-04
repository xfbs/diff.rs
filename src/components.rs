//! # Shared components
//!
//! This module contains shared components.

mod diff_view;
mod file_tree;
mod layout;
mod navigation;
mod non_ideal;
mod search;

pub use self::{diff_view::*, file_tree::*, layout::*, navigation::*, non_ideal::*, search::*};
