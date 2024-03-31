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
    #[at("/:name/:old/:new")]
    Diff {
        name: String,
        old: VersionId,
        new: VersionId,
    },
    #[at("/:name/:old/:new/*path")]
    File {
        name: String,
        old: VersionId,
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
                {name}
                old={VersionId::Named(VersionNamed::Previous)}
                new={VersionId::Named(VersionNamed::Latest)}
            />
        },
        Route::Diff { name, old, new } => html! {
            <Diff {name} {old} {new} />
        },
        Route::File {
            name,
            old,
            new,
            path,
        } => html! {
            <Diff {name} {old} {new} {path} />
        },
        Route::NotFound => html! { <NotFound /> },
        Route::Search { krate: _ } => html! { <p>{"Search"}</p> },
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
