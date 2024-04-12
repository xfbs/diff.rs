//! # Application
//!
//! This module contains the root component of the application. The root component contains
//! the routing logic, and renders the views depending on the current route.

use crate::{
    version::{VersionId, VersionNamed},
    views::*,
};
use yew::prelude::*;
use yew_router::prelude::*;
pub use yew_router::prelude::{use_navigator, Link, Redirect};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/about")]
    About,
    #[at("/search/:krate")]
    Search { krate: String },
    #[at("/:name/")]
    Crate { name: String },
    #[at("/:src_name/:dst_name")]
    Crates {
        src_name: String,
        dst_name: String,
    },
    #[at("/:name/:old/:new")]
    SingleSourceDiff {
        name: String,
        old: VersionId,
        new: VersionId,
    },
    #[at("/:name/:old/:new/*path")]
    SingleSourceFile {
        name: String,
        old: VersionId,
        new: VersionId,
        path: String,
    },
    #[at("/:src_name/:old/:dst_name/:new/*path")]
    File {
        src_name: String,
        old: VersionId,
        dst_name: String,
        new: VersionId,
        path: String,
    },
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <Home /> },
        Route::About => html! { <About /> },
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

#[function_component]
pub fn App() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}
