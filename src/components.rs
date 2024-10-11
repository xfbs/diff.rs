//! # Shared components
//!
//! This module contains shared components. These are components which are shared between multiple
//! views. Components which are only used by a single view can be kept inside the view's definition
//! itself, unless they are generic to too complex.

mod diff_view;
mod file_tree;
mod footer;
mod layout;
mod navigation;
mod non_ideal;
mod search;

pub use self::{
    diff_view::*, file_tree::*, footer::*, layout::*, navigation::*, non_ideal::*, search::*,
};
