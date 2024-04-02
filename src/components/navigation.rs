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
        <nav id="navbar" class="bg-[#f6f8fa] dark:bg-[#010409] sticky w-full z-20 top-0 start-0 border-b border-gray-200 dark:border-gray-600 dark:text-gray-300" aria-label="Main">
            <div class="max-w-screen-xl flex flex-wrap items-center justify-between mx-auto p-4">
                { for props.children.iter() }
            </div>
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
        <div class="flex flex-row flex-nowrap gap-6">
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
        <div class="text-xl font-bold text-nowrap">
        { for props.children.iter() }
        </div>
    }
}

#[function_component]
pub fn NavbarItem(props: &NavbarHeadingProps) -> Html {
    html! {
        <div class="text-lg text-nowrapf flex flex-row items-center">
        { for props.children.iter() }
        </div>
    }
}

#[function_component]
pub fn NavbarDivider() -> Html {
    html! {}
}

#[function_component]
pub fn SimpleNavbar() -> Html {
    html! {
        <Navbar>
            <NavbarGroup>
                <NavbarHeading>
                    <Link<Route> to={Route::Home} classes="flex flex-row items-center">
                        <YewIcon height={"1.5ex"} icon_id={IconId::LucideFileDiff} />
                        { "diff.rs" }
                    </Link<Route>>
                </NavbarHeading>
                <NavbarItem>
                    <Link<Route> to={Route::About}>
                        {"About"}
                    </Link<Route>>
                </NavbarItem>
            </NavbarGroup>
            <NavbarGroup>
                <Search />
            </NavbarGroup>
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
                <NavbarHeading>
                    <Link<Route> to={Route::Home} classes="flex flex-row items-center"><YewIcon height={"1.5ex"} icon_id={IconId::LucideFileDiff} /><span>{ "diff.rs" }</span></Link<Route>></NavbarHeading>
                <NavbarDivider />
                <NavbarItem>
                    <a href={format!("https://crates.io/crates/{}", props.name)} class="flex flex-row items-center">
                        <YewIcon height={"1.5ex"} icon_id={IconId::LucideBox} />
                    </a>
                    { props.name.clone() }
                </NavbarItem>
                <NavbarItem>
                    <HtmlSelect<IString>
                        minimal={true}
                        options={versions.clone()}
                        disabled={prop_versions.is_empty()}
                        value={Some(props.old.to_string().into()) as Option<IString>}
                        class="text-current dark:text-gray-200"
                        onchange={
                            let onchange = props.onchange.clone();
                            let new = props.new.clone();
                            move |old: IString| {
                                let old: Version = old.parse().unwrap();
                                onchange.emit((old.clone(), new.clone()))
                            }
                        }
                    />
                </NavbarItem>
                <NavbarItem>{ "diff" }</NavbarItem>
                <NavbarItem>
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
                </NavbarItem>
                <NavbarDivider />
            </NavbarGroup>
            <NavbarGroup>
                <Search />
            </NavbarGroup>
        </Navbar>
    }
}
