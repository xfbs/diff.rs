use crate::{
    data::{CrateDetail, SearchResponse},
    Link, Route,
};
use gloo_net::http::Request;
use implicit_clone::unsync::IString;
use web_sys::HtmlInputElement;
use yew::{prelude::*, suspense::use_future_with};
use yew_hooks::prelude::*;
use yew_router::prelude::*;

#[function_component]
pub(super) fn SearchGlass() -> Html {
    html! {
        <svg class="w-4 h-4 text-gray-500 dark:text-gray-400" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 20">
            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m19 19-4-4m0-7A7 7 0 1 1 1 8a7 7 0 0 1 14 0Z"/>
        </svg>
    }
}

#[derive(Properties, PartialEq)]
pub struct SearchBarProps {
    pub value: IString,
    #[prop_or_default]
    pub onchange: Callback<String>,
}

#[function_component]
pub fn SearchBar(props: &SearchBarProps) -> Html {
    let oninput = {
        let onchange = props.onchange.clone();
        move |event: InputEvent| {
            let target: HtmlInputElement = event.target_dyn_into().unwrap();
            onchange.emit(target.value());
        }
    };

    // prevent default action on form submission (page reload)
    let onsubmit = |event: SubmitEvent| {
        event.prevent_default();
    };

    // set focus
    let input = use_node_ref();
    use_effect_once({
        let input = input.clone();
        move || {
            if let Some(input) = input.cast::<HtmlInputElement>() {
                let _ = input.focus();
            }
            || {}
        }
    });

    html! {
        <form class="max-w-xl mx-auto p-4" {onsubmit}>
            <label for="default-search" class="mb-2 text-sm font-medium text-gray-900 sr-only dark:text-white">{"Search"}</label>
            <div class="relative">
                <div class="absolute inset-y-0 start-0 flex items-center ps-3 pointer-events-none">
                    <SearchGlass />
                </div>
                <input ref={input} type="search" id="default-search" class="block w-full p-4 ps-10 text-sm text-gray-900 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="Search for crates" required=true value={props.value.clone()} {oninput} />
                <button type="submit" class="text-white absolute end-2.5 bottom-2.5 bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-4 py-2 dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800">{"Search"}</button>
            </div>
        </form>
    }
}

#[function_component]
pub fn Search() -> Html {
    let state = use_state(|| "".to_string());
    let navigator = use_navigator().unwrap();

    let oninput = {
        let state = state.clone();
        move |event: InputEvent| {
            state.set(event.target_unchecked_into::<HtmlInputElement>().value());
        }
    };

    let onkeydown = {
        let state = state.clone();
        move |event: KeyboardEvent| {
            if event.key() == "Enter" {
                navigator.push(&Route::Crate {
                    krate: state.to_string(),
                });
            }
        }
    };

    html! {
        <div class="relative w-full">
            <div class="absolute inset-y-0 start-0 flex items-center ps-3 pointer-events-none">
                <SearchGlass />
            </div>
            <input type="search" id="default-search" class="block w-full p-2 ps-10 text-sm text-gray-900 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-800 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="Jump to crate..." value={state.to_string()} {oninput} {onkeydown} />
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SearchResultsProps {
    pub query: String,
}

#[function_component]
pub fn SearchResults(props: &SearchResultsProps) -> Html {
    let fallback = html! {};
    html! {
        <Suspense {fallback}>
            if props.query.is_empty() {
            } else {
            <SearchResultsLoader query={props.query.clone()} />
            }
        </Suspense>
    }
}

#[function_component]
pub fn SearchResultsLoader(props: &SearchResultsProps) -> HtmlResult {
    let info = use_future_with(props.query.clone(), |name| async move {
        let response = Request::get("https://crates.io/api/v1/crates")
            .query([("q", name.as_str())])
            .build()?
            .send()
            .await?;
        let text = response.json::<SearchResponse>().await?;
        Ok(text) as anyhow::Result<SearchResponse>
    })?;

    let html = match &*info {
        Ok(response) => html! {
            <div class="flex flex-col gap-2 my-4">
            { for response.crates.iter().map(|c| html! {<Card details={c.clone()} /> }) }
            </div>
        },
        Err(error) => html! {
            <>
                {"Error: "}
                {format!("{error:?}")}
            </>
        },
    };

    Ok(html)
}

#[derive(Properties, PartialEq)]
struct CardProps {
    pub details: CrateDetail,
}

#[function_component]
fn Card(props: &CardProps) -> Html {
    html! {
        <Link to={Route::Crate { krate: props.details.id.clone() }} classes="block p-6 bg-white border border-gray-200 rounded-lg shadow hover:bg-gray-100 dark:bg-gray-800 dark:border-gray-700 dark:hover:bg-gray-700">
            <h5 class="mb-2 mt-0 text-2xl font-bold tracking-tight text-gray-900 dark:text-white">{&props.details.id}{" "}<span class="text-gray-600">{"v"}{&props.details.max_version}</span></h5>
            <p class="font-normal text-gray-700 dark:text-gray-400">{&props.details.description}</p>
        </Link>
    }
}
