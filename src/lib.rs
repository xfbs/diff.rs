//! # diff.rs
//!
//! Web application to visualize code differences between different versions of Rust crates.  Fully
//! backend-less, works by downloading crate sources in the browser, validating the hash, unpacking
//! it, running a diff algorithm over the files and rendering the diff. Support syntax highlighting
//! provided by the `syntect` crate.

mod cache;
pub mod components;
mod data;
mod maybe_parsed;
mod syntax;
#[cfg(test)]
mod tests;
mod version;
pub mod views;

use crate::{
    maybe_parsed::MaybeParsed,
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
    #[at("/browse/:krate/:version/files/*path")]
    BrowseFile {
        krate: String,
        version: VersionId,
        path: Utf8PathBuf,
    },

    /// Legacy file browse view (without `/files/`), redirects to [Route::BrowseFile].
    ///
    /// Needed because otherwise [Route::SingleSourceFile] would win for paths starting with a
    /// `files` directory.
    #[at("/browse/:krate/:version/*path")]
    LegacyBrowseFile {
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
    ///
    /// The static `files` segment makes this route outrank [Route::File] in the route recognizer
    /// (which prefers routes with more static segments, then more dynamic ones, and does not fall
    /// back to other routes if parameters fail to parse).
    #[at("/:krate/:old_version/:new_version/files/*path")]
    SingleSourceFile {
        krate: String,
        old_version: VersionId,
        new_version: VersionId,
        path: Utf8PathBuf,
    },

    /// Legacy single-crate file diff view (without `/files/`), redirects to
    /// [Route::SingleSourceFile].
    ///
    /// Only matches paths with a single component (`/serde/1.0.0/1.0.1/Cargo.toml`), longer ones
    /// are matched by [Route::File] and handled there.
    #[at("/:krate/:old_version/:new_version/*path")]
    LegacySingleSourceFile {
        krate: String,
        old_version: VersionId,
        new_version: VersionId,
        path: Utf8PathBuf,
    },

    /// File diff view, render differences in the file path between the crate versions.
    ///
    /// This also matches legacy single-crate links with nested paths, such as
    /// `/serde/1.0.0/1.0.1/src/lib.rs`, which are parsed as `new_krate = "1.0.1"` and
    /// `new_version = "src"`. Therefore `new_version` is parsed optionally, and
    /// [Route::render] redirects these to [Route::SingleSourceFile].
    #[at("/:old_krate/:old_version/:new_krate/:new_version/*path")]
    File {
        old_krate: String,
        old_version: VersionId,
        new_krate: String,
        new_version: MaybeParsed<VersionId>,
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
            route @ (Route::LegacyBrowseFile { .. } | Route::LegacySingleSourceFile { .. }) => {
                match route.upgrade() {
                    Some(to) => html! { <ReplaceRedirect {to} /> },
                    None => html! { <NotFound /> },
                }
            }
            route @ Route::File { .. } => match route.upgrade() {
                Some(to) => html! { <ReplaceRedirect {to} /> },
                None => match route {
                    Route::File {
                        old_krate,
                        old_version,
                        new_krate,
                        new_version,
                        path,
                    } if is_crate_name(&new_krate) => match new_version.parsed() {
                        Some(new_version) => html! {
                            <Diff src_name={old_krate} dst_name={new_krate} old={old_version} new={new_version.clone()} {path} />
                        },
                        None => html! { <NotFound /> },
                    },
                    _ => html! { <NotFound /> },
                },
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

    /// If this route was matched from a legacy URL, returns the route to redirect to.
    ///
    /// Legacy URLs are single-crate file links without the `/files/` segment
    /// (`/serde/1.0.0/1.0.1/src/lib.rs`) and browse links without it
    /// (`/browse/serde/1.0.0/src/lib.rs`).
    pub fn upgrade(&self) -> Option<Route> {
        match self {
            Route::LegacyBrowseFile {
                krate,
                version,
                path,
            } => Some(Route::BrowseFile {
                krate: krate.clone(),
                version: version.clone(),
                path: path.clone(),
            }),
            Route::LegacySingleSourceFile {
                krate,
                old_version,
                new_version,
                path,
            } => Some(Route::SingleSourceFile {
                krate: krate.clone(),
                old_version: old_version.clone(),
                new_version: new_version.clone(),
                path: path.clone(),
            }),
            // Not a valid two-crate link: this may be a legacy single-crate link, where the
            // "crate name" is actually the new version (crate names cannot start with a digit)
            // and the "version" is the first path segment, which is kept verbatim.
            Route::File {
                old_krate,
                old_version,
                new_krate,
                new_version,
                path,
            } if !is_crate_name(new_krate) || new_version.parsed().is_none() => {
                let legacy_new_version: VersionId = new_krate.parse().ok()?;
                Some(Route::SingleSourceFile {
                    krate: old_krate.clone(),
                    old_version: old_version.clone(),
                    new_version: legacy_new_version,
                    path: Utf8PathBuf::from(new_version.raw()).join(path),
                })
            }
            _ => None,
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
            } if old_krate == new_krate && new_version.parsed().is_some() => {
                Route::SingleSourceFile {
                    krate: old_krate,
                    old_version,
                    new_version: new_version.parsed().cloned().unwrap(),
                    path,
                }
            }
            Route::Crates {
                old_krate,
                new_krate,
            } if old_krate == new_krate => Route::Crate { krate: old_krate },
            other => other,
        }
    }
}

/// Determines if `name` is a valid crate name (as enforced by crates.io).
///
/// Crate names must start with a letter, which is what distinguishes them from versions.
fn is_crate_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[derive(Properties, PartialEq)]
struct ReplaceRedirectProps {
    to: Route,
}

/// Redirect which replaces the current history entry (unlike [Redirect], which pushes one), so
/// that legacy links do not break the back button.
#[function_component]
fn ReplaceRedirect(props: &ReplaceRedirectProps) -> Html {
    let navigator = use_navigator();
    use_effect_with(props.to.clone(), move |to| {
        if let Some(navigator) = navigator {
            navigator.replace(to);
        }
    });
    Html::default()
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
