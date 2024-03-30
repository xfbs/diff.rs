use super::*;
use crate::data::CrateResponse;
use semver::Version;
use yew_icons::{Icon as YewIcon, IconId};

#[derive(Properties, PartialEq)]
pub struct NavbarProps {
    pub children: Children,
}

#[function_component]
pub fn Navbar(props: &NavbarProps) -> Html {
    html! {
        <nav id="navbar" class="bp3-navbar bp3-fixed-top" aria-label="Main">
        { for props.children.iter() }
        </nav>
    }
}

#[derive(Properties, PartialEq)]
pub struct NavbarGroupProps {
    pub children: Children,
}

#[function_component]
pub fn NavbarGroup(props: &NavbarGroupProps) -> Html {
    html! {
        <div class="bp3-navbar-group bp3-align-left">
        { for props.children.iter() }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct NavbarHeadingProps {
    pub children: Children,
}

#[function_component]
pub fn NavbarHeading(props: &NavbarHeadingProps) -> Html {
    html! {
        <div class="bp3-navbar-heading">
        { for props.children.iter() }
        </div>
    }
}

#[function_component]
pub fn NavbarDivider() -> Html {
    html! {
        <div class="bp3-navbar-divider"></div>
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
                    <Search />
                </div>
            </div>
        </Navbar>
    }
}

#[derive(Properties, PartialEq)]
pub struct ComplexNavbarProps {
    pub name: String,
    pub old: Version,
    pub new: Version,
    pub info: Arc<CrateResponse>,
    #[prop_or_default]
    pub onchange: Callback<(Version, Version)>,
}

#[function_component]
pub fn ComplexNavbar(props: &ComplexNavbarProps) -> Html {
    let prop_versions: Vec<_> = props.info.versions.iter().map(|v| v.num.clone()).collect();

    let versions: IArray<(IString, AttrValue)> = props
        .info
        .versions
        .iter()
        .map(|version| (version, IString::from(version.num.to_string())))
        .map(|(version, num)| {
            if version.yanked {
                (num.clone(), format!("{num} (yanked)").into())
            } else {
                (num.clone(), num)
            }
        })
        .collect();

    html! {
        <Navbar>
            <NavbarGroup>
                <NavbarHeading><Link<Route> to={Route::Home}><YewIcon height={"1.5ex"} icon_id={IconId::LucideFileDiff} /> { "diff.rs" }</Link<Route>></NavbarHeading>
                <NavbarDivider />
                <NavbarHeading>
                    { props.name.clone() }
                </NavbarHeading>
                <NavbarHeading>
                    <a href={format!("https://crates.io/crates/{}", props.name)}>
                        <YewIcon height={"1.5ex"} icon_id={IconId::LucideBox} /> { "crates.io" }
                    </a>
                </NavbarHeading>
                <NavbarHeading>
                    <HtmlSelect<IString>
                        minimal={true}
                        options={versions.clone()}
                        disabled={prop_versions.is_empty()}
                        value={Some(props.old.to_string().into()) as Option<IString>}
                        onchange={
                            let onchange = props.onchange.clone();
                            let new = props.new.clone();
                            move |old: IString| {
                                let old: Version = old.parse().unwrap();
                                onchange.emit((old.clone(), new.clone()))
                            }
                        }
                    />
                </NavbarHeading>
                <NavbarHeading>{ "diff" }</NavbarHeading>
                <NavbarHeading>
                    <HtmlSelect<IString>
                        minimal={true}
                        options={versions}
                        disabled={prop_versions.is_empty()}
                        value={Some(props.new.to_string().into()) as Option<IString>}
                        onchange={
                            let onchange = props.onchange.clone();
                            let old = props.old.clone();
                            move |new: IString| {
                                let new: Version = new.parse().unwrap();
                                onchange.emit((old.clone(), new.clone()))
                            }
                        }
                    />
                </NavbarHeading>
                <NavbarDivider />
            </NavbarGroup>
            <div class="bp3-navbar-group bp3-align-right">
                <div class="bp3-navbar-heading bp3-fill">
                    <Search />
                </div>
            </div>
        </Navbar>
    }
}
