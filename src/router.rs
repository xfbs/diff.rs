use crate::components::*;

use yew::prelude::*;
use yew_router::prelude::*;
pub use yew_router::prelude::{use_navigator, Link, Redirect};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/search/:krate")]
    Search { krate: String },
    #[at("/:name/")]
    Crate { name: String },
    #[at("/:name/:left/:right")]
    Diff {
        name: String,
        left: String,
        right: String,
    },
    #[at("/:name/:left/:right/*path")]
    File {
        name: String,
        left: String,
        right: String,
        path: String,
    },
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <Home /> },
        Route::Crate { name } => html! {
            <Diff {name} />
        },
        Route::Diff { name, left, right } => html! {
            <Diff {name} {left} {right} />
        },
        Route::File {
            name,
            left,
            right,
            path,
        } => html! {
            <Diff {name} {left} {right} {path} />
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
