use super::*;
use yew_icons::{Icon as YewIcon, IconId};

#[derive(Properties, PartialEq)]
pub struct NavbarProps {
    pub children: Children,
}

#[function_component]
pub fn Navbar(props: &NavbarProps) -> Html {
    html! {
        <div class="bp3-navbar bp3-fixed-top">
        { for props.children.iter() }
        </div>
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
    pub left: String,
    pub right: String,
    pub info: Arc<CrateResponse>,
    #[prop_or_default]
    pub onchange: Callback<(String, String)>,
}

#[function_component]
pub fn ComplexNavbar(props: &ComplexNavbarProps) -> Html {
    let disabled = vec![props.left.clone(), props.right.clone()];
    let prop_versions: Vec<_> = props.info.versions.iter().map(|v| v.num.clone()).collect();
    let versions = match prop_versions.is_empty() {
        true => &disabled,
        false => &prop_versions,
    };

    let versions: IArray<(IString, AttrValue)> = props
        .info
        .versions
        .iter()
        .map(|version| (version, IString::from(version.num.clone())))
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
                    { &props.name }
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
                        value={Some(props.left.clone().into()) as Option<IString>}
                        onchange={
                            let onchange = props.onchange.clone();
                            let right = props.right.clone();
                            move |left: IString| {
                                onchange.emit((left.to_string(), right.clone()))
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
                        value={Some(props.right.clone().into()) as Option<IString>}
                        onchange={
                            let onchange = props.onchange.clone();
                            let left = props.left.clone();
                            move |right: IString| {
                                onchange.emit((left.clone(), right.to_string()))
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
