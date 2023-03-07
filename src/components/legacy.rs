use crate::components::navigation::*;
use crate::crates::{CrateInfo, CrateResponse, CrateSource, VersionInfo};
use crate::router::*;
use implicit_clone::unsync::{IArray, IString};
use log::*;
use similar::{ChangeTag, TextDiff};
use std::collections::BTreeSet;
use std::sync::Arc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::suspense::*;
use yew_icons::{Icon as YewIcon, IconId};
use yewprint::*;

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

#[derive(Clone, PartialEq, Eq, Default)]
pub enum CrateState {
    #[default]
    Initial,
    Loading,
    Version(String, String),
    Error(String),
    NotExists,
}

#[function_component]
pub fn Crate(props: &CrateProps) -> Html {
    let state = use_state(|| CrateState::Initial);

    // fetch crate info
    if *state == CrateState::Initial {
        let state = state.clone();
        let props = props.clone();
        spawn_local(async move {
            state.set(CrateState::Loading);
            match CrateInfo::fetch_cached(&props.name).await {
                Ok(info) => state.set(CrateState::Version(
                    info.krate.max_version.clone(),
                    info.krate.max_stable_version.clone(),
                )),
                Err(error) => state.set(CrateState::Error(error.to_string())),
            }
        });
    }

    html! {
        <>
        <Navbar>
            <NavbarGroup>
                <NavbarHeading><Link<Route> to={Route::Home}><YewIcon height={"1.5ex"} icon_id={IconId::LucideFileDiff} /> { "diff.rs" }</Link<Route>></NavbarHeading>
                <NavbarDivider />
                <NavbarHeading>
                    <a href={format!("https://crates.io/crates/{}", props.name)}>
                        <YewIcon height={"1.5ex"} icon_id={IconId::LucideBox} /> { &props.name }
                    </a>
                </NavbarHeading>
                <NavbarHeading>
                    <HtmlSelect<IString> minimal={true} disabled={true} options={[
                        ("left".into(), "left".into()),
                    ].into_iter().collect::<IArray<_>>()
                    } />
                </NavbarHeading>
                <NavbarHeading>{ "diff" }</NavbarHeading>
                <NavbarHeading>
                    <HtmlSelect<IString> minimal={true} disabled={true} options={[
                        ("right".into(), "right".into()),
                    ].into_iter().collect::<IArray<_>>()
                    } />
                </NavbarHeading>
                <NavbarDivider />
            </NavbarGroup>
            <div class="bp3-navbar-group bp3-align-right">
                <div class="bp3-navbar-heading bp3-fill">
                    <InputGroup placeholder="Search crates..." fill={true} left_icon={Icon::Search} />
                </div>
            </div>
        </Navbar>
        <div style="height: 50px;"></div>
        <Center>
            {
                match &*state {
                    CrateState::Initial => html!{
                        <Loading title={"Loading crate"} status={""} />
                    },
                    CrateState::Loading => html! {
                        <Loading title={"Loading crate"} status={"Loading crate information"} />
                    },
                    CrateState::NotExists => html! {
                        <Error title={"Loading crate"} status={"The crate does not exist"} />
                    },
                    CrateState::Error(error) => html!{
                        <Error title={"Loading crate"} status={error.to_string()} />
                    },
                    CrateState::Version(left, right) => html!{
                        <Redirect<Route> to={Route::Diff {
                            name: props.name.clone(),
                            left: left.clone(),
                            right: right.clone(),
                        }} />
                    },
                }
            }
        </Center>
        </>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct DiffProps {
    pub name: String,
    pub left: String,
    pub right: String,
    pub path: Option<String>,
}

// instead of using state here, use the use_future thingy.
#[derive(Clone, PartialEq, Eq, Default)]
pub enum DiffState {
    #[default]
    Loading,
    CrateInfo(Arc<CrateResponse>),
    CrateSource(Arc<CrateResponse>, Arc<CrateSource>, Arc<CrateSource>),
    Error(String),
}

#[function_component]
pub fn Diff(props: &DiffProps) -> Html {
    info!("Instantiating diff");
    let state = use_state(|| DiffState::Loading);
    let navigator = use_navigator().unwrap();

    // load crate versions
    if *state == DiffState::Loading {
        let state = state.clone();
        let props = props.clone();
        spawn_local(async move {
            match CrateInfo::fetch_cached(&props.name).await {
                Ok(info) => {
                    if !info.versions.iter().any(|v| v.num == props.left) {
                        state.set(DiffState::Error(format!(
                            "Version {} not found",
                            props.left
                        )));
                    } else if !info.versions.iter().any(|v| v.num == props.right) {
                        state.set(DiffState::Error(format!(
                            "Version {} not found",
                            props.right
                        )));
                    } else {
                        state.set(DiffState::CrateInfo(info));
                    }
                }
                Err(error) => state.set(DiffState::Error(error.to_string())),
            }
        });
    }

    let (have_versions, versions): (bool, IArray<(IString, AttrValue)>) = match &*state {
        DiffState::CrateInfo(info) | DiffState::CrateSource(info, _, _) => (
            true,
            info.versions
                .iter()
                .map(|version| (version.num.clone().into(), version.num.clone().into()))
                .collect(),
        ),
        _ => (
            false,
            [&props.left, &props.right]
                .iter()
                .map(|version| (version.to_string().into(), version.to_string().into()))
                .collect(),
        ),
    };

    if let DiffState::CrateInfo(crate_info) = &*state {
        let left = crate_info
            .versions
            .iter()
            .find(|v| v.num == props.left)
            .unwrap()
            .clone();
        let right = crate_info
            .versions
            .iter()
            .find(|v| v.num == props.right)
            .unwrap()
            .clone();
        let state = state.clone();
        let props = props.clone();
        let crate_info = crate_info.clone();
        spawn_local(async move {
            let left = match left.fetch_cached().await {
                Ok(source) => source,
                Err(error) => {
                    state.set(DiffState::Error(error.to_string()));
                    return;
                }
            };
            let right = match right.fetch_cached().await {
                Ok(source) => source,
                Err(error) => {
                    state.set(DiffState::Error(error.to_string()));
                    return;
                }
            };
            state.set(DiffState::CrateSource(crate_info, left, right));
        });
    }

    html! {
        <>
        <Navbar>
            <NavbarGroup>
                <NavbarHeading><Link<Route> to={Route::Home}><YewIcon height={"1.5ex"} icon_id={IconId::LucideFileDiff} /> { "diff.rs" }</Link<Route>></NavbarHeading>
                <NavbarDivider />
                <NavbarHeading>
                    <a href={format!("https://crates.io/crates/{}", props.name)}>
                        <YewIcon height={"1.5ex"} icon_id={IconId::LucideBox} /> { &props.name }
                    </a>
                </NavbarHeading>
                <NavbarHeading>
                    <HtmlSelect<IString>
                        minimal={true}
                        options={versions.clone()}
                        disabled={!have_versions}
                        value={Some(props.left.clone().into()) as Option<IString>}
                        onchange={
                            let navigator = navigator.clone();
                            let props = props.clone();
                            Callback::from(move |version: IString| {
                                navigator.push(&Route::Diff {
                                    name: props.name.clone(),
                                    left: version.to_string(),
                                    right: props.right.clone(),
                                });
                            })
                        }
                    />
                </NavbarHeading>
                <NavbarHeading>{ "diff" }</NavbarHeading>
                <NavbarHeading>
                    <HtmlSelect<IString>
                        minimal={true}
                        options={versions.clone()}
                        disabled={!have_versions}
                        value={Some(props.right.clone().into()) as Option<IString>}
                        onchange={
                            let navigator = navigator.clone();
                            let props = props.clone();
                            Callback::from(move |version: IString| {
                                navigator.push(&Route::Diff {
                                    name: props.name.clone(),
                                    right: version.to_string(),
                                    left: props.left.clone(),
                                });
                            })
                        }
                    />
                </NavbarHeading>
                <NavbarDivider />
            </NavbarGroup>
            <div class="bp3-navbar-group bp3-align-right">
                <div class="bp3-navbar-heading bp3-fill">
                    <InputGroup placeholder="Search crates..." fill={true} left_icon={Icon::Search} />
                </div>
            </div>
        </Navbar>
        <div style="height: 50px;"></div>
        {
            match &*state {
                DiffState::Loading => html! {
                    <Center>
                    <Loading title={"Loading crate"} status={"Loading crate version information"} />
                    </Center>
                },
                DiffState::Error(error) => html!{
                    <Center>
                    <Error title={"Error loading crate"} status={error.clone()} />
                    </Center>
                },
                DiffState::CrateInfo(_) => html!{
                    <Center>
                    <Loading title={"Loading crate"} status={"Loading crate source"} />
                    </Center>
                },
                DiffState::CrateSource(_, left, _) if props.path.is_none() => html!{
                    <Redirect<Route> to={Route::File {
                        name: props.name.clone(),
                        left: props.left.clone(),
                        right: props.right.clone(),
                        path: "Cargo.toml".into(),
                    }} />
                },
                DiffState::CrateSource(_, left, _) => html!{
                    <p> {format!("{:?}", left)} </p>
                },
            }
        }
        </>
    }
}
