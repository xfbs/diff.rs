use crate::cache::*;
use crate::data::{CrateResponse, VersionInfo};
use crate::router::*;
use implicit_clone::unsync::{IArray, IString};
use std::sync::Arc;
use yew::prelude::*;
use yew::suspense::*;
use yewprint::*;
mod navigation;
use navigation::*;
mod diff_view;
mod file_tree;
use diff_view::*;
use file_tree::*;
mod layout;
use layout::*;
mod non_ideal;
use non_ideal::*;
mod search;
use search::*;

#[function_component]
pub fn Home() -> Html {
    html! {
        <>
            <SimpleNavbar />
            <Content>
                <div style="width: 700px; margin: auto; padding-top: 20px;">
                    <Callout title={"Warning"} intent={Intent::Warning}>
                        <p>{"This application is experimental. Crates might intermittently not load correctly due to an issue with CORS headers from crates.io."}</p>
                    </Callout>
                    <h1>{ "diff.rs" }</h1>
                    <p>{ "View the differences between Rust crate versions. Enter a crate name such as "}<a href="/serde/1.0.153/1.0.153">{"serde"}</a>{" in the search field in the top-right corner to get started." }</p>
                    <p>{ "This is a WebAssembly-based web application written in Rust with "}<a href="https://docs.rs/yew">{"Yew"}</a>{". It uses the "}<a href="https://crates.io/">{"crates.io"}</a>{" API to fetch crate metadata, downloads and parses the crate sources in-memory and renders a diff in, all in the browser." }</p>
                    <p>{"Source code for this application is available at "}<a href="https://github.com/xfbs/diff.rs">{"github.com/xfbs/diff.rs"}</a>{"."}</p>
                </div>
            </Content>
        </>
    }
}

#[function_component]
pub fn NotFound() -> Html {
    html! {
        <>
            <SimpleNavbar />
            <Content>
                <Error title={"Not found"} status={"The URL was not found"} />
            </Content>
        </>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct CrateProps {
    pub name: String,
}

#[derive(Properties, PartialEq, Clone)]
pub struct DiffProps {
    pub name: String,
    pub left: Option<String>,
    pub right: Option<String>,
    pub path: Option<String>,
}

#[function_component]
pub fn Diff(props: &DiffProps) -> Html {
    let fallback = html! {
        <>
            <SimpleNavbar />
            <Content>
                <Center>
                    <Loading title={"Loading crate"} status={"Loading crate metadata"} />
                </Center>
            </Content>
        </>
    };
    html! {
        <Suspense {fallback}>
            <CrateFetcher
                name={props.name.clone()}
                left={props.left.clone()}
                right={props.right.clone()}
                path={props.path.clone()}
            />
        </Suspense>
    }
}

#[function_component]
pub fn CrateFetcher(props: &DiffProps) -> HtmlResult {
    let info = use_future_with_deps(
        |name| async move { CRATE_RESPONSE_CACHE.fetch_cached(&name).await },
        props.name.clone(),
    )?;

    match &*info {
        Ok(info) => Ok(html! {
            <VersionResolver {info} left={props.left.clone()} right={props.right.clone()} path={props.path.clone()} />
        }),
        Err(error) => Ok(html! {
            <>
                <SimpleNavbar />
                <Content>
                    <Center>
                        <Error title={"Loading crate"} status={format!("Error: {error}")} />
                    </Center>
                </Content>
            </>
        }),
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct VersionResolverProps {
    pub info: Arc<CrateResponse>,
    pub left: Option<String>,
    pub right: Option<String>,
    pub path: Option<String>,
}

#[function_component]
pub fn VersionResolver(props: &VersionResolverProps) -> Html {
    // redirect to latest versions if none specified
    let (left_str, right_str) = match (&props.left, &props.right) {
        (Some(left), Some(right)) => (left, right),
        _ => {
            let left = props
                .left
                .as_ref()
                .or(props.info.krate.max_stable_version.as_ref())
                .unwrap_or(&props.info.krate.max_version);
            let right = props
                .right
                .as_ref()
                .unwrap_or(&props.info.krate.max_version);
            return html! {
                <Redirect<Route> to={Route::Diff {
                    name: props.info.krate.id.clone(),
                    left: left.into(),
                    right: right.into(),
                }} />
            };
        }
    };

    // find krate version info
    let left = props.info.versions.iter().find(|v| &v.num == left_str);
    let right = props.info.versions.iter().find(|v| &v.num == right_str);

    match (left, right) {
        (Some(left), Some(right)) => html! {
            <SourceFetcher info={props.info.clone()} left={left.clone()} right={right.clone()} path={props.path.clone()} />
        },
        (None, _) => html! {
            <>
            <SimpleNavbar />
            <Content>
            <Center>
            <Error title={"Resolving version"} status={format!("Error: version {left_str} not found")} />
            </Center>
            </Content>
            </>
        },
        (_, None) => html! {
            <>
            <SimpleNavbar />
            <Content>
            <Center>
            <Error title={"Resolving version"} status={format!("Error: version {right_str} not found")} />
            </Center>
            </Content>
            </>
        },
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct SourceFetcherProps {
    pub info: Arc<CrateResponse>,
    pub left: VersionInfo,
    pub right: VersionInfo,
    pub path: Option<String>,
}

#[function_component]
pub fn SourceFetcher(props: &SourceFetcherProps) -> Html {
    let fallback = html! {
        <>
        <ComplexNavbar
            name={props.info.krate.id.clone()}
            left={props.left.num.clone()}
            right={props.right.num.clone()}
            info={props.info.clone()}
        />
        <Center>
            <Loading title={"Loading crate"} status={"Loading crate source"} />
        </Center>
        </>
    };
    html! {
        <Suspense {fallback}>
            <SourceFetcherInner
                info={props.info.clone()}
                left={props.left.clone()}
                right={props.right.clone()}
                path={props.path.clone()}
            />
        </Suspense>
    }
}

#[function_component]
pub fn SourceFetcherInner(props: &SourceFetcherProps) -> HtmlResult {
    // fetch left version source
    let left = use_future_with_deps(
        |version| async move { CRATE_SOURCE_CACHE.fetch_cached(&version).await },
        props.left.clone(),
    )?;

    // fetch right version source
    let right = use_future_with_deps(
        |version| async move { CRATE_SOURCE_CACHE.fetch_cached(&version).await },
        props.right.clone(),
    )?;

    let navigator = use_navigator().unwrap();
    let (left, right) = match (&*left, &*right) {
        (Ok(left), Ok(right)) => (left, right),
        (Err(error), _) | (_, Err(error)) => {
            return Ok(html! {
                <>
                <ComplexNavbar
                    name={props.info.krate.id.clone()}
                    left={props.left.num.clone()}
                    right={props.right.num.clone()}
                    info={props.info.clone()}
                    onchange={
                        let name = props.info.krate.id.clone();
                        let path = props.path.clone();
                        move |(left, right)| {
                            navigator.push(&Route::File {
                                name: name.clone(),
                                left: left,
                                right: right,
                                path: path.clone().unwrap_or_default(),
                            });
                        }
                    }
                />
                <Content>
                    <Center>
                        <Error title={"Loading crate"} status={format!("Error: {error}")} />
                    </Center>
                </Content>
                </>
            })
        }
    };

    let path = match &props.path {
        None => {
            return Ok(html! {
                <Redirect<Route> to={Route::File {
                    name: props.info.krate.id.clone(),
                    left: props.left.num.clone(),
                    right: props.right.num.clone(),
                    path: "Cargo.toml".into(),
                }} />
            })
        }
        Some(path) => path.clone(),
    };

    Ok(html! {
        <SourceView
            info={props.info.clone()}
            {left}
            {right}
            {path}
        />
    })
}
