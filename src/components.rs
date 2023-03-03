use crate::router::*;
use implicit_clone::unsync::{IArray, IString};
use log::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon as YewIcon, IconId};
use yewprint::*;

mod navigation;
use navigation::*;

#[function_component]
pub fn Home() -> Html {
    html! {
        <>
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
        </>
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
            match crate::crates::crate_info(&props.name).await {
                Ok(info) => state.set(CrateState::Version(
                    info.krate.max_version,
                    info.krate.max_stable_version,
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
        <div style="height: 50;"></div>
        {
            match &*state {
                CrateState::Initial => html!{ {"Initial"} },
                CrateState::Loading => html! { {"Loading"} },
                CrateState::NotExists => html! { {"Not exists"} },
                CrateState::Error(error) => html!{ {format!("Error: {error}")} },
                CrateState::Version(left, right) => html!{
                    <Redirect<Route> to={Route::Diff {
                        krate: props.name.clone(),
                        left: left.clone(),
                        right: right.clone(),
                    }} />
                },
            }
        }
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

#[derive(Clone, PartialEq, Eq, Default)]
pub enum DiffState {
    #[default]
    Initial,
    Loading,
    Versions(Vec<String>),
    Error(String),
    NotExists,
}

impl DiffState {
    fn is_versions(&self) -> bool {
        matches!(self, DiffState::Versions(_))
    }
}

#[function_component]
pub fn Diff(props: &DiffProps) -> Html {
    let state = use_state(|| DiffState::Initial);
    let navigator = use_navigator().unwrap();

    // load crate versions
    if *state == DiffState::Initial {
        let state = state.clone();
        let props = props.clone();
        spawn_local(async move {
            state.set(DiffState::Loading);
            match crate::crates::crate_info(&props.name).await {
                Ok(info) => state.set(DiffState::Versions(
                    info.versions
                        .into_iter()
                        .map(|version| version.num)
                        .collect(),
                )),
                Err(error) => state.set(DiffState::Error(error.to_string())),
            }
        });
    }

    let have_versions = matches!(&*state, DiffState::Versions(_));
    let versions: IArray<(IString, AttrValue)> = match &*state {
        DiffState::Versions(versions) => versions
            .iter()
            .map(|version| (version.clone().into(), version.clone().into()))
            .collect(),
        _ => [&props.left, &props.right]
            .iter()
            .map(|version| (version.to_string().into(), version.to_string().into()))
            .collect(),
    };

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
                                    krate: props.name.clone(),
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
                                    krate: props.name.clone(),
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
                DiffState::Initial => html!{ {"Initial"} },
                DiffState::Loading => html! { {"Loading"} },
                DiffState::NotExists => html! { {"Not exists"} },
                DiffState::Error(error) => html!{ {format!("Error: {error}")} },
                DiffState::Versions(versions) => html!{ {"Version"} },
            }
        }
        </>
    }
}
