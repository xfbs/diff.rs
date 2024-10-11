//! # diff.rs
//!
//! Web application to visualize code differences between different versions of Rust crates.  Fully
//! backend-less, works by downloading crate sources in the browser, validating the hash, unpacking
//! it, running a diff algorithm over the files and rendering the diff. Support syntax highlighting
//! provided by the `syntect` crate.

mod cache;
pub mod components;
mod data;
mod syntax;
#[cfg(test)]
mod tests;
mod version;
pub mod views;

use crate::{
    version::{VersionId, VersionNamed},
    views::*,
};
use yew::prelude::*;
use yew_router::prelude::*;

/// Link component which uses this crate's [Route].
pub type Link<R = Route> = yew_router::components::Link<R>;

/// Application routes.
///
/// This struct declares all valid routes in the app, and has a [switch()] method to render the
/// appropriate view.
///
/// The default route for `/:name` is to render a crate. Therefore, when adding new routes, one
/// must be careful to not alias an existing crate name. For example, adding a route with the path
/// `/serde` would mask the crate view for the `serde` crate.
#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    /// Home view, shows search bar and summary.
    #[at("/")]
    Home,

    /// About view, shows information about the application.
    #[at("/about")]
    About,

    /// Search view, shows search results.
    #[at("/search/:krate")]
    Search { krate: String },

    /// Browese view, will load crate source and redirect to default file.
    #[at("/browse/:name/:version")]
    Browse { name: String, version: VersionId },

    /// File browse view.
    #[at("/browse/:name/:version/*path")]
    BrowseFile {
        name: String,
        version: VersionId,
        path: String,
    },

    /// Crate view, will make request to get most recent version and redirect.
    #[at("/:name/")]
    Crate { name: String },

    /// Crates view, allows for diffing two crates.
    #[at("/:src_name/:dst_name")]
    Crates { src_name: String, dst_name: String },

    /// File diff view between `old` and `new` versions.
    #[at("/:name/:old/:new")]
    SingleSourceDiff {
        name: String,
        old: VersionId,
        new: VersionId,
    },

    /// File diff view between `old` and `new` versions.
    #[at("/:name/:old/:new/*path")]
    SingleSourceFile {
        name: String,
        old: VersionId,
        new: VersionId,
        path: String,
    },

    /// File diff view, render differences in the file path between the crate versions.
    #[at("/:src_name/:old/:dst_name/:new/*path")]
    File {
        src_name: String,
        old: VersionId,
        dst_name: String,
        new: VersionId,
        path: String,
    },

    /// Route that is matched if no other route matches, shows error message.
    #[not_found]
    #[at("/404")]
    NotFound,
}

impl Route {
    /// Render this route to a view.
    pub fn render(route: Route) -> Html {
        match route {
            Route::Home => html! { <Home /> },
            Route::About => html! { <About /> },
            Route::Browse { name, version } => html! {
                <Diff
                    src_name={name.clone()}
                    dst_name={name}
                    old={version.clone()}
                    new={version}
                />
            },
            Route::BrowseFile {
                name,
                version,
                path,
            } => html! {
                <Diff
                    src_name={name.clone()}
                    dst_name={name}
                    old={version.clone()}
                    new={version}
                    {path}
                />
            },
            Route::Crate { name } => html! {
                <Diff
                    src_name={name.clone()}
                    dst_name={name}
                    old={VersionId::Named(VersionNamed::Previous)}
                    new={VersionId::Named(VersionNamed::Latest)}
                />
            },
            Route::Crates { src_name, dst_name } => html! {
                <Diff
                    src_name={src_name}
                    dst_name={dst_name}
                    old={VersionId::Named(VersionNamed::Latest)}
                    new={VersionId::Named(VersionNamed::Latest)}
                />
            },
            Route::SingleSourceDiff { name, old, new } => html! {
                <Diff src_name={name.clone()} dst_name={name} {old} {new} />
            },
            Route::SingleSourceFile {
                name,
                old,
                new,
                path,
            } => html! {
                <Diff src_name={name.clone()} dst_name={name} {old} {new} {path} />
            },
            Route::File {
                src_name,
                old,
                dst_name,
                new,
                path,
            } => html! {
                <Diff {src_name} {dst_name} {old} {new} {path} />
            },
            Route::NotFound => html! { <NotFound /> },
            Route::Search { krate } => html! { <Search search={krate} /> },
        }
    }
}

/// Render application.
#[function_component]
pub fn App() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={Route::render} />
        </BrowserRouter>
    }
}
