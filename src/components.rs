use crate::crates::{CrateInfo, CrateResponse, VersionInfo};
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
mod legacy;
use diff_view::*;
use file_tree::*;
mod layout;
use layout::*;
mod non_ideal;
use non_ideal::*;

#[function_component]
pub fn Home() -> Html {
    html! {
        <>
            <SimpleNavbar />
            <Content>
                <div class="content">
                    <h1>{ "diff.rs" }</h1>
                    <p>{ "View the differences between crates." }</p>
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
        <Center>
            <Loading title={"Loading crate"} status={"Loading crate metadata"} />
        </Center>
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
        |name| async move { CrateInfo::fetch_cached(&name).await },
        props.name.clone(),
    )?;

    match &*info {
        Ok(info) => Ok(html! {
            <VersionResolver {info} left={props.left.clone()} right={props.right.clone()} path={props.path.clone()} />
        }),
        Err(_error) => todo!(),
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
    let (left, right) = match (&props.left, &props.right) {
        (Some(left), Some(right)) => (left, right),
        _ => {
            let left = props
                .left
                .as_ref()
                .unwrap_or(&props.info.krate.max_stable_version);
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
    let left = props.info.versions.iter().find(|v| &v.num == left);
    let right = props.info.versions.iter().find(|v| &v.num == right);

    match (left, right) {
        (Some(left), Some(right)) => html! {
            <SourceFetcher info={props.info.clone()} left={left.clone()} right={right.clone()} path={props.path.clone()} />
        },
        _ => html! {
            <p>{format!("Error: Version not found")}</p>
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
            versions={props.info.versions.iter().map(|v| v.num.clone()).collect::<Vec<_>>()}
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
        |version| async move { version.fetch_cached().await },
        props.left.clone(),
    )?;

    // fetch right version source
    let right = use_future_with_deps(
        |version| async move { version.fetch_cached().await },
        props.right.clone(),
    )?;

    let (left, right) = match (&*left, &*right) {
        (Ok(left), Ok(right)) => (left, right),
        (Err(error), _) | (_, Err(error)) => {
            return Ok(html! {<p>{format!("Error: {error}")}</p> })
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
