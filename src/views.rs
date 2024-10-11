//! # Application views
//!
//! This module contains all of the views of the application. Views are pages that
//! can be rendered (selected via the active route). Any components which are used
//! by more than one view (or are sufficiently complex) should go into the `components`
//! module, which contains components shared between views.

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

pub use self::{about::*, diff::*, home::Home, not_found::NotFound, search::Search};
