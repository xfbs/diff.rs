use crate::router::*;
use implicit_clone::unsync::{IArray, IString};
use log::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon as YewIcon, IconId};
use yewprint::*;

mod navigation;
use navigation::*;

#[derive(Properties, PartialEq)]
pub struct ProgressProps {
    pub progress: f64,
    pub status: String,
}

#[function_component]
pub fn Progress(props: &ProgressProps) -> Html {
    html! {
        <>
        <div class="bp3-progress-bar">
            <div class="bp3-progress-meter" style={format!("width: {}%", (100.0 * props.progress) as u8)}></div>
        </div>
        { &props.status }
        </>
    }
}

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
        <div style="height: 50px;"></div>
        <Center>
            {
                match &*state {
                    CrateState::Initial => html!{
                        <Progress progress={0.0} status={"Initial"} />
                    },
                    CrateState::Loading => html! {
                        <Progress progress={0.1} status={"Loading crate information"} />
                    },
                    CrateState::NotExists => html! {
                        <Progress progress={0.2} status={"Error: crate does not exist"} />
                    },
                    CrateState::Error(error) => html!{
                        <Progress progress={0.2} status={error.clone()} />
                    },
                    CrateState::Version(left, right) => html!{
                        <Redirect<Route> to={Route::Diff {
                            krate: props.name.clone(),
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
                DiffState::Versions(versions) => html!{
                    <div class="bp3-non-ideal-state">
                      <div class="bp3-non-ideal-state-visual" style="font-size: 48px; line-height: 48px;">
                        <Spinner size={48.0} />
                      </div>
                      <div class="bp3-non-ideal-state-text">
                        <h4 class="bp3-heading">{ "This folder is empty" }</h4>
                        <div>{ "Create a new file to populate the folder." }</div>
                      </div>
                    </div>
                },
            }
        }
        </>
    }
}
