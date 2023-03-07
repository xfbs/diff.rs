use crate::crates::{CrateInfo, CrateResponse, CrateSource, VersionInfo};
use crate::router::*;
use implicit_clone::unsync::{IArray, IString};
use similar::{ChangeTag, TextDiff};
use std::sync::Arc;
use yew::prelude::*;
use yew::suspense::*;
use yew_icons::{Icon as YewIcon, IconId};

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

#[derive(Properties, PartialEq)]
pub struct ContentProps {
    pub children: Children,
}

#[function_component]
pub fn Content(props: &ContentProps) -> Html {
    html! {
        <div style="margin-top: 50px;">
            { for props.children.iter() }
        </div>
    }
}
