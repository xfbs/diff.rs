use crate::components::*;
use implicit_clone::unsync::{IArray, IString};
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
            <DiffViewer {name} />
        },
        Route::Diff { name, left, right } => html! {
            <DiffViewer {name} {left} {right} />
        },
        Route::File { name, left, right, path } => html! {
            <DiffViewer {name} {left} {right} {path} />
        },
        Route::NotFound => html! { <NotFound /> },
        _ => html! { <Crate name={"wireguard_keys"} /> },
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
