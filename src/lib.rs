//! # diff.rs
//!
//! Web application to visualize code differences between different versions of Rust crates.  Fully
//! backend-less, works by downloading crate sources in the browser, validating the hash, unpacking
//! it, running a diff algorithm over the files and rendering the diff. Support syntax highlighting
//! provided by the `syntect` crate.

pub mod app;
mod cache;
pub mod components;
mod data;
mod syntax;
#[cfg(test)]
mod tests;
mod version;
pub mod views;

pub use app::App;
