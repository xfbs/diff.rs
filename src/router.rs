use crate::components::*;
use implicit_clone::unsync::{IArray, IString};
use yew::prelude::*;
use yew_router::prelude::*;
pub use yew_router::prelude::{Link, Redirect};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/search/:krate")]
    Search { krate: String },
    #[at("/:krate/")]
    Crate { krate: String },
    #[at("/:krate/:left/:right")]
    Diff {
        krate: String,
        left: String,
        right: String,
    },
    #[at("/:krate/:left/:right/*path")]
    File {
        krate: String,
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
        Route::Crate { krate } => html! { <Crate name={krate} /> },
        Route::Diff { krate, left, right } => html! {
            <Diff
                name={krate}
                left={left}
                right={right}
                path={None as Option<String>}
            />
        },
        Route::File {
            krate,
            left,
            right,
            path,
        } => html! {
            <Diff
                name={krate}
                left={left}
                right={right}
                path={Some(path)}
            />
        },
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
