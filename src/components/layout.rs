use yew::prelude::*;

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
