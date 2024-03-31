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
    #[prop_or_default]
    pub children: Children,
}

#[function_component]
pub fn Content(props: &ContentProps) -> Html {
    html! {
        <div>
            { for props.children.iter() }
        </div>
    }
}

#[function_component]
pub fn Footer(props: &ContentProps) -> Html {
    html! {
        <footer style="text-align: center; position: absolute; bottom: 0; margin: auto;">
            { for props.children.iter() }
        </footer>
    }
}
