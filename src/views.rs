//! # Views
//!
//! This module contains all of the views of the application.

use crate::{
    cache::*,
    components::*,
    data::{CrateResponse, VersionInfo},
    Link, Route,
};
use std::sync::Arc;
use yew::{prelude::*, suspense::*};

mod about;
mod diff;
mod home;
mod not_found;
mod search;

pub use self::{about::About, diff::Diff, home::Home, not_found::NotFound, search::Search};
