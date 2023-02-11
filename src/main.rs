use implicit_clone::unsync::{IArray, IString};
use yew::prelude::*;
use yew_router::prelude::{Switch, *};
use yewprint::*;

#[derive(Clone, Routable, PartialEq)]
enum Route {
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
        _ => html! { <Crate name={"wireguard_keys"} /> },
    }
}

#[function_component]
fn App() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

#[function_component]
fn Home() -> Html {
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    html! {
        <>
            <div class="bp3-navbar bp3-fixed-top">
                <div class="bp3-navbar-group bp3-align-left">
                    <div class="bp3-navbar-heading"><a href="/">{ "diff.rs" }</a></div>
                    <div class="bp3-navbar-divider"></div>
                    <div class="bp3-navbar-heading bp3-fill">
                        <InputGroup placeholder="Search crates..." fill={true} left_icon={Icon::Search} />
                    </div>
                </div>
            </div>
        <div>
            <button {onclick}>{ "+1" }</button>
            <p>{ *counter }</p>
        </div>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct CrateProps {
    pub name: String,
}

#[function_component]
fn Crate(props: &CrateProps) -> Html {
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    html! {
        <>
            <div class="bp3-navbar bp3-fixed-top">
                <div class="bp3-navbar-group bp3-align-left">
                    <div class="bp3-navbar-heading"><a href="/">{ "diff.rs" }</a></div>
                    <div class="bp3-navbar-divider"></div>
                    <div class="bp3-navbar-heading">{ &props.name }</div>
                    <div class="bp3-navbar-heading">
                        <HtmlSelect<IString> options={[
                            ("0.1.1".into(), "0.1.1".into()),
                            ("0.1.0".into(), "0.1.0".into()),
                        ].into_iter().collect::<IArray<_>>()
                        } />
                    </div>
                    <div class="bp3-navbar-heading">{ "diff" }</div>
                    <div class="bp3-navbar-heading">
                        <HtmlSelect<IString> options={[
                            ("0.1.0".into(), "0.1.0".into()),
                            ("0.1.1".into(), "0.1.1".into()),
                        ].into_iter().collect::<IArray<_>>()
                        } />
                    </div>
                    <div class="bp3-navbar-divider"></div>
                </div>
                <div class="bp3-navbar-group bp3-align-right">
                    <div class="bp3-navbar-heading bp3-fill">
                        <InputGroup placeholder="Search crates..." fill={true} left_icon={Icon::Search} />
                    </div>
                </div>
            </div>
        <div>
            <button {onclick}>{ "+1" }</button>
            <p>{ *counter }</p>
        </div>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
