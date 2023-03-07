use yew::prelude::*;

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
