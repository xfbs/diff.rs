use crate::{app::*, components::Search, data::CrateResponse};
use implicit_clone::unsync::IString;
use indexmap::IndexMap;
use semver::Version;
use std::sync::Arc;
use web_sys::HtmlSelectElement;
use yew::prelude::*;
use yew_icons::{Icon as YewIcon, IconId};

#[derive(Properties, PartialEq)]
pub struct NavbarProps {
    pub children: Children,
}

#[function_component]
pub fn Navbar(props: &NavbarProps) -> Html {
    html! {
        <nav id="navbar" class="bg-[#f6f8fa] dark:bg-[#010409] sticky w-full z-20 top-0 start-0 border-b border-gray-200 dark:border-gray-600 dark:text-gray-300" aria-label="Main">
            <div class="max-w-screen-xl flex flex-wrap items-center justify-between mx-auto p-4 flex-col sm:flex-row gap-4">
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
        <div class="text-xl font-bold text-nowrap flex flex-row items-center">
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
pub struct SelectProps {
    #[prop_or_default]
    values: IndexMap<IString, IString>,
    #[prop_or_default]
    selected: Option<IString>,
    #[prop_or_default]
    onchange: Callback<IString>,
}

#[function_component]
pub fn Select(props: &SelectProps) -> Html {
    let onchange = {
        let onchange = props.onchange.clone();
        move |event: Event| {
            let target = event.target_dyn_into::<HtmlSelectElement>().unwrap();
            let value = target.value();
            onchange.emit(value.into());
        }
    };
    html! {
        <select class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-1.5 dark:bg-gray-800 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" {onchange}>
        {
            for props
                .values
                .iter()
                .map(|(key, value)| {
                    let selected = props
                        .selected
                        .as_ref()
                        .map(|k| k == key)
                        .unwrap_or(false);
                    html! {
                        <option {selected} value={key}>{value}</option>
                    }
                })
        }
        </select>
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
fn SwitchIcon() -> Html {
    html! {
        <svg class="h-4 w-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
            <path fill="currentColor" d="M7.72 21.78a.75.75 0 0 0 1.06-1.06L5.56 17.5h14.69a.75.75 0 0 0 0-1.5H5.56l3.22-3.22a.75.75 0 1 0-1.06-1.06l-4.5 4.5a.75.75 0 0 0 0 1.06l4.5 4.5Zm8.56-9.5a.75.75 0 1 1-1.06-1.06L18.44 8H3.75a.75.75 0 0 1 0-1.5h14.69l-3.22-3.22a.75.75 0 0 1 1.06-1.06l4.5 4.5a.75.75 0 0 1 0 1.06l-4.5 4.5Z">
            </path>
        </svg>
    }
}

#[function_component]
pub fn ComplexNavbar(props: &ComplexNavbarProps) -> Html {
    let versions: IndexMap<IString, IString> = props
        .info
        .versions
        .iter()
        .map(|version| {
            let num = IString::from(version.num.to_string());
            if version.yanked {
                (num.clone(), format!("{num} (yanked)").into())
            } else {
                (num.clone(), num.clone())
            }
        })
        .collect();

    let switch = {
        let onchange = props.onchange.clone();
        let versions = (props.new.clone(), props.old.clone());
        move |_| {
            onchange.emit(versions.clone());
        }
    };

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
                    <Select
                        values={versions.clone()}
                        selected={Some(props.old.to_string().into()) as Option<IString>}
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
                <NavbarItem>
                    <span class="cursor-pointer hover:rotate-180 transition delay-150 duration-300 ease-in-out" onclick={switch}>
                        <SwitchIcon />
                    </span>
                </NavbarItem>
                <NavbarItem>
                    <Select
                        values={versions}
                        selected={Some(props.new.to_string().into()) as Option<IString>}
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
