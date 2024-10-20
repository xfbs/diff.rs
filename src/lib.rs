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
use camino::Utf8PathBuf;
use yew::prelude::*;
use yew_router::prelude::*;

/// Link component which uses this crate's [Route].
pub type Link<R = Route> = yew_router::components::Link<R>;

/// Application routes.
///
/// This struct declares all valid routes in the app, and has a [render()](Self::render) method to render the
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
    #[at("/search/:query")]
    Search { query: String },

    /// Browse view, will load crate source and redirect to default file.
    #[at("/browse/:krate/:version")]
    Browse { krate: String, version: VersionId },

    /// File browse view.
    #[at("/browse/:krate/:version/*path")]
    BrowseFile {
        krate: String,
        version: VersionId,
        path: Utf8PathBuf,
    },

    /// Crate view, will make request to get most recent version and redirect.
    #[at("/:krate/")]
    Crate { krate: String },

    /// Crates view, allows for diffing two crates.
    #[at("/:old_krate/:new_krate")]
    Crates {
        old_krate: String,
        new_krate: String,
    },

    /// File diff view between `old` and `new` versions.
    #[at("/:krate/:old_version/:new_version")]
    SingleSourceDiff {
        krate: String,
        old_version: VersionId,
        new_version: VersionId,
    },

    /// File diff view between `old` and `new` versions.
    #[at("/:krate/:old_version/:new_version/*path")]
    SingleSourceFile {
        krate: String,
        old_version: VersionId,
        new_version: VersionId,
        path: Utf8PathBuf,
    },

    /// File diff view, render differences in the file path between the crate versions.
    #[at("/:old_krate/:old_version/:new_krate/:new_version/*path")]
    File {
        old_krate: String,
        old_version: VersionId,
        new_krate: String,
        new_version: VersionId,
        path: Utf8PathBuf,
    },

    #[at("/repo/:krate/:version/files/*path")]
    RepoFile {
        krate: String,
        version: VersionId,
        path: Utf8PathBuf,
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
            Route::Browse { krate, version } => html! {
                <Diff
                    src_name={krate.clone()}
                    dst_name={krate}
                    old={version.clone()}
                    new={version}
                />
            },
            Route::BrowseFile {
                krate,
                version,
                path,
            } => html! {
                <Diff
                    src_name={krate.clone()}
                    dst_name={krate}
                    old={version.clone()}
                    new={version}
                    {path}
                />
            },
            Route::Crate { krate } => html! {
                <Diff
                    src_name={krate.clone()}
                    dst_name={krate}
                    old={VersionId::Named(VersionNamed::Previous)}
                    new={VersionId::Named(VersionNamed::Latest)}
                />
            },
            Route::Crates {
                old_krate,
                new_krate,
            } => html! {
                <Diff
                    src_name={old_krate}
                    dst_name={new_krate}
                    old={VersionId::Named(VersionNamed::Latest)}
                    new={VersionId::Named(VersionNamed::Latest)}
                />
            },
            Route::SingleSourceDiff {
                krate,
                old_version,
                new_version,
            } => html! {
                <Diff src_name={krate.clone()} dst_name={krate} old={old_version} new={new_version} />
            },
            Route::SingleSourceFile {
                krate,
                old_version,
                new_version,
                path,
            } => html! {
                <Diff src_name={krate.clone()} dst_name={krate} old={old_version} new={new_version} {path} />
            },
            Route::File {
                old_krate,
                old_version,
                new_krate,
                new_version,
                path,
            } => html! {
                <Diff src_name={old_krate} dst_name={new_krate} old={old_version} new={new_version} {path} />
            },
            Route::NotFound => html! { <NotFound /> },
            Route::Search { query } => html! { <Search search={query} /> },
            Route::RepoFile {
                krate,
                version,
                path,
            } => html! {
                <RepoFileView {krate} {version} {path} />
            },
        }
    }

    /// Try to simplify a route.
    ///
    /// If the route is a multi-crate route, and the crates are identical, then simplify it to
    /// using a single-crate route.
    pub fn simplify(self) -> Self {
        match self {
            Route::File {
                old_krate,
                old_version,
                new_krate,
                new_version,
                path,
            } if old_krate == new_krate => Route::SingleSourceFile {
                krate: old_krate,
                old_version,
                new_version,
                path,
            },
            Route::Crates {
                old_krate,
                new_krate,
            } if old_krate == new_krate => Route::Crate { krate: old_krate },
            other => other,
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
