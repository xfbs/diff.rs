use crate::{
    data::{CrateDetail, SearchResponse, SummaryCategory, SummaryResponse},
    Link, Route,
};
use gloo_net::http::Request;
use implicit_clone::unsync::IString;
use web_sys::HtmlInputElement;
use yew::{
    prelude::*,
    suspense::{use_future, use_future_with},
};
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

#[function_component]
fn GitIcon() -> Html {
    html! {
        <svg class="inline" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="1em" height="1em" fill="currentColor"><path d="M15 4.75a3.25 3.25 0 1 1 6.5 0 3.25 3.25 0 0 1-6.5 0ZM2.5 19.25a3.25 3.25 0 1 1 6.5 0 3.25 3.25 0 0 1-6.5 0Zm0-14.5a3.25 3.25 0 1 1 6.5 0 3.25 3.25 0 0 1-6.5 0ZM5.75 6.5a1.75 1.75 0 1 0-.001-3.501A1.75 1.75 0 0 0 5.75 6.5Zm0 14.5a1.75 1.75 0 1 0-.001-3.501A1.75 1.75 0 0 0 5.75 21Zm12.5-14.5a1.75 1.75 0 1 0-.001-3.501A1.75 1.75 0 0 0 18.25 6.5Z"></path><path d="M5.75 16.75A.75.75 0 0 1 5 16V8a.75.75 0 0 1 1.5 0v8a.75.75 0 0 1-.75.75Z"></path><path d="M17.5 8.75v-1H19v1a3.75 3.75 0 0 1-3.75 3.75h-7a1.75 1.75 0 0 0-1.75 1.75H5A3.25 3.25 0 0 1 8.25 11h7a2.25 2.25 0 0 0 2.25-2.25Z"></path></svg>
    }
}

#[function_component]
fn DocsRsIcon() -> Html {
    html! {
        <svg class="inline" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="1em" height="1em" fill="currentColor">
            <path d="M488.6 250.2L392 214V105.5c0-15-9.3-28.4-23.4-33.7l-100-37.5c-8.1-3.1-17.1-3.1-25.3 0l-100 37.5c-14.1 5.3-23.4 18.7-23.4 33.7V214l-96.6 36.2C9.3 255.5 0 268.9 0 283.9V394c0 13.6 7.7 26.1 19.9 32.2l100 50c10.1 5.1 22.1 5.1 32.2 0l103.9-52 103.9 52c10.1 5.1 22.1 5.1 32.2 0l100-50c12.2-6.1 19.9-18.6 19.9-32.2V283.9c0-15-9.3-28.4-23.4-33.7zM358 214.8l-85 31.9v-68.2l85-37v73.3zM154 104.1l102-38.2 102 38.2v.6l-102 41.4-102-41.4v-.6zm84 291.1l-85 42.5v-79.1l85-38.8v75.4zm0-112l-102 41.4-102-41.4v-.6l102-38.2 102 38.2v.6zm240 112l-85 42.5v-79.1l85-38.8v75.4zm0-112l-102 41.4-102-41.4v-.6l102-38.2 102 38.2v.6z"></path>
        </svg>
    }
}

#[derive(Properties, PartialEq)]
struct CardProps {
    pub details: CrateDetail,
}

#[function_component]
fn Card(props: &CardProps) -> Html {
    let link = Route::Crate {
        krate: props.details.id.clone(),
    };
    html! {
        <Link to={link} classes="card">
            <div class="header">
                <h3 class="name">{&props.details.id}</h3>
                <span class="version">{props.details.max_version.to_string()}</span>
                <span class="grow"></span>
                <a class="icon" href={format!("https://docs.rs/{}/{}", &props.details.id, &props.details.max_version)}><DocsRsIcon /></a>
                if let Some(url) = &props.details.repository {
                    <a class="icon" href={url.to_string()}><GitIcon /></a>
                }
            </div>
            <p class="description">{&props.details.description}</p>
        </Link>
    }
}

#[function_component]
pub fn SummaryLoader(summary: &StaticResultPropNew) -> HtmlResult {
    let info = use_future(|| async move {
        let response = Request::get("https://crates.io/api/v1/summary")
            .build()?
            .send()
            .await?;
        let text = response.json::<SummaryResponse>().await?;
        Ok(text) as anyhow::Result<SummaryResponse>
    })?;

    let html = match &*info {
        Ok(response) => html! {
            <section class="summary">
            {
                for summary.category.iter().copied().map(|category| html! {
                    <SummaryColumn {category} crates={response.get(category).clone()} />
                })
            }
            </section>
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
pub struct SummaryColumnProps {
    category: SummaryCategory,
    crates: Vec<CrateDetail>,
}

#[function_component]
fn SummaryColumn(props: &SummaryColumnProps) -> Html {
    if props.crates.is_empty() {
        return html! {};
    }

    html! {
        <div class="column">
            <h2 class="title">
                { props.category.title() }
            </h2>
            <section class="results">
                { for props.crates.iter().cloned().map(|c| html! {<Card details={c} /> }) }
            </section>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct StaticResultPropNew {
    pub category: Vec<SummaryCategory>,
}

#[function_component]
pub fn DefaultSummarySection() -> Html {
    let fallback = html! {
        {"Loading"}
    };
    html! {
        <Suspense {fallback}>
            <SummaryLoader category={ vec![
                SummaryCategory::MostRecent,
                SummaryCategory::MostDownloaded,
                SummaryCategory::JustUpdated,
            ]}/>
        </Suspense>
    }
}
