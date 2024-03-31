use crate::{
    app::*,
    cache::*,
    components::*,
    data::{CrateResponse, VersionInfo},
};
use std::sync::Arc;
use yew::{prelude::*, suspense::*};

mod diff;
mod about;
mod home;
mod not_found;

pub use self::{diff::Diff, home::Home, not_found::NotFound, about::About};
