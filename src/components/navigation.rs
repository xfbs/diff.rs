use crate::{components::Search, data::CrateResponse, *};
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
        <div class="flex flex-row flex-wrap sm:flex-nowrap gap-6">
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
                    <Link to={Route::Home} classes="flex flex-row items-center">
                        <YewIcon height={"1.5ex"} icon_id={IconId::LucideFileDiff} />
                        { "diff.rs" }
                    </Link>
                </NavbarHeading>
                <NavbarItem>
                    <Link to={Route::About}>
                        {"About"}
                    </Link>
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
    pub src_name: String,
    pub dst_name: String,
    pub old: Version,
    pub new: Version,
    pub src_info: Arc<CrateResponse>,
    pub dst_info: Arc<CrateResponse>,
    #[prop_or_default]
    pub onchange: Callback<((String, Version), (String, Version))>,
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
    let prepare_versions = |versions: &[crate::data::VersionInfo]| {
        versions
            .iter()
            .map(|version| {
                let num = IString::from(version.num.to_string());
                if version.yanked {
                    (num.clone(), format!("{num} (yanked)").into())
                } else {
                    (num.clone(), num.clone())
                }
            })
            .collect()
    };
    let switched = use_state(|| false);

    let (src_name, dst_name, old, new, src_info, dst_info) = if *switched {
        (
            &props.dst_name,
            &props.src_name,
            &props.new,
            &props.old,
            &props.dst_info,
            &props.src_info,
        )
    } else {
        (
            &props.src_name,
            &props.dst_name,
            &props.old,
            &props.new,
            &props.src_info,
            &props.dst_info,
        )
    };

    let src_versions: IndexMap<IString, IString> = prepare_versions(&src_info.versions);
    let dst_versions: IndexMap<IString, IString> = prepare_versions(&dst_info.versions);

    let switch = {
        let onchange = props.onchange.clone();
        let new_switched = !*switched;
        let versions = if new_switched {
            (
                (dst_name.clone(), new.clone()),
                (src_name.clone(), old.clone()),
            )
        } else {
            (
                (src_name.clone(), old.clone()),
                (dst_name.clone(), new.clone()),
            )
        };
        Callback::from(move |_| {
            switched.set(new_switched);
            onchange.emit(versions.clone());
        })
    };

    html! {
        <Navbar>
            <NavbarGroup>
                <NavbarHeading>
                    <Link to={Route::Home} classes="flex flex-row items-center"><YewIcon height={"1.5ex"} icon_id={IconId::LucideFileDiff} /><span>{ "diff.rs" }</span></Link></NavbarHeading>
                <NavbarDivider />
                <NavbarGroup>
                    <NavbarItem>
                        <a href={format!("https://crates.io/crates/{}", src_name)} class="flex flex-row items-center">
                            <YewIcon height={"1.5ex"} icon_id={IconId::LucideBox} />
                        </a>
                        { src_name.clone() }
                    </NavbarItem>
                    <NavbarItem>
                        <Select
                            values={src_versions.clone()}
                            selected={Some(old.to_string().into()) as Option<IString>}
                            onchange={
                                let onchange = props.onchange.clone();
                                let src_name = src_name.clone();
                                let dst_name = dst_name.clone();
                                let new = new.clone();
                                move |old: IString| {
                                    let old: Version = old.parse().unwrap();
                                    onchange.emit(((src_name.clone(), old.clone()), (dst_name.clone(), new.clone())))
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
                        <a href={format!("https://crates.io/crates/{}", dst_name)} class="flex flex-row items-center">
                            <YewIcon height={"1.5ex"} icon_id={IconId::LucideBox} />
                        </a>
                        { dst_name.clone() }
                    </NavbarItem>
                    <NavbarItem>
                        <Select
                            values={dst_versions}
                            selected={Some(new.to_string().into()) as Option<IString>}
                            onchange={
                                let onchange = props.onchange.clone();
                                let src_name = src_name.clone();
                                let dst_name = dst_name.clone();
                                let old = old.clone();
                                move |new: IString| {
                                    let new: Version = new.parse().unwrap();
                                    onchange.emit(((src_name.clone(), old.clone()), (dst_name.clone(), new.clone())))
                                }
                            }
                        />
                    </NavbarItem>
                    <NavbarDivider />
                </NavbarGroup>
            </NavbarGroup>
            <NavbarGroup>
                <Search />
            </NavbarGroup>
        </Navbar>
    }
}
