use crate::crates::{CrateInfo, CrateResponse, CrateSource, VersionInfo};
use crate::router::*;
use implicit_clone::unsync::{IArray, IString};
use itertools::{Itertools, Position};
use log::*;
use similar::{ChangeTag, TextDiff};
use std::collections::BTreeSet;
use std::sync::Arc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::suspense::*;
use yew_icons::{Icon as YewIcon, IconId};
use yewprint::id_tree::{InsertBehavior, Node, NodeId, TreeBuilder};
use yewprint::*;

mod navigation;
use navigation::*;

mod legacy;
mod file_tree;
use file_tree::*;

#[derive(Properties, PartialEq)]
pub struct CenterProps {
    pub children: Children,
}

#[function_component]
pub fn Center(props: &CenterProps) -> Html {
    html! {
        <div style="position: absolute; top: 50%; width: 100%; text-align: center;">
        { for props.children.iter() }
        </div>
    }
}

#[function_component]
pub fn SimpleNavbar() -> Html {
    html! {
        <Navbar>
            <NavbarGroup>
                <NavbarHeading><Link<Route> to={Route::Home}><YewIcon height={"1.5ex"} icon_id={IconId::LucideFileDiff} /> { "diff.rs" }</Link<Route>></NavbarHeading>
                <NavbarDivider />
            </NavbarGroup>
            <div class="bp3-navbar-group bp3-align-right">
                <div class="bp3-navbar-heading bp3-fill">
                    <InputGroup placeholder="Search crates..." fill={true} left_icon={Icon::Search} />
                </div>
            </div>
        </Navbar>
    }
}

#[function_component]
pub fn Home() -> Html {
    html! {
        <>
            <SimpleNavbar />
            <div style="height: 50px;"></div>
            <div class="content">
                <h1>{ "diff.rs" }</h1>
                <p>{ "View the differences between crates." }</p>
            </div>
        </>
    }
}

#[function_component]
pub fn NotFound() -> Html {
    html! {
        <>
            <SimpleNavbar />
            <div style="height: 50px;"></div>
            <Error title={"Not found"} status={"The URL was not found"} />
        </>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct ErrorProps {
    pub title: String,
    pub status: String,
}

#[function_component]
pub fn Error(props: &ErrorProps) -> Html {
    html! {
        <div class="bp3-non-ideal-state">
            <div class="bp3-non-ideal-state-visual" style="font-size: 48px; line-height: 48px;">
                <Icon icon={Icon::Error} intent={Intent::Danger} size={48} />
            </div>
            <div class="bp3-non-ideal-state-text">
                <h4 class="bp3-heading">{ &props.title }</h4>
                <div>{ &props.status }</div>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct LoadingProps {
    pub title: String,
    pub status: String,
}

#[function_component]
pub fn Loading(props: &LoadingProps) -> Html {
    html! {
        <div class="bp3-non-ideal-state">
            <div class="bp3-non-ideal-state-visual" style="font-size: 48px; line-height: 48px;">
                <Spinner size={48.0} />
            </div>
            <div class="bp3-non-ideal-state-text">
                <h4 class="bp3-heading">{ &props.title }</h4>
                <div>{ &props.status }</div>
            </div>
        </div>
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
        Err(error) => todo!(),
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
        <Center>
            <Loading title={"Loading crate"} status={"Loading crate source"} />
        </Center>
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

#[derive(Properties, PartialEq, Clone)]
pub struct SourceViewProps {
    pub info: Arc<CrateResponse>,
    pub left: Arc<CrateSource>,
    pub right: Arc<CrateSource>,
    pub path: String,
}

#[function_component]
pub fn SourceView(props: &SourceViewProps) -> Html {
    let left = props
        .left
        .files
        .get(&props.path)
        .map(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let right = props
        .right
        .files
        .get(&props.path)
        .map(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    html! {
        <>
        <p>{"Hey"}</p>
        <FileView info={props.info.clone()} left={props.left.clone()} right={props.right.clone()} path={props.path.clone()} />
        <DiffView {left} {right} path={props.path.clone()} />
        </>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct DiffViewProps {
    pub path: String,
    pub left: String,
    pub right: String,
}

#[function_component]
pub fn DiffView(props: &DiffViewProps) -> Html {
    let diff = TextDiff::from_lines(&props.left, &props.right);
    html! {
        <>
        <p>{"Diff"}</p>
        {
            diff.iter_all_changes().map(|change| {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };
                html!{ <p>{ format!("{sign}{change}") } </p> }
            }).collect::<Html>()
        }
        </>
    }
}
