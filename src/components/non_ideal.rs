use yew::prelude::*;
use yewprint::*;

#[derive(Properties, PartialEq, Clone)]
pub struct ErrorProps {
    pub title: String,
    pub status: String,
}

#[function_component]
pub fn Error(props: &ErrorProps) -> Html {
    html! {
        <div class="bp3-non-ideal-state">
            <div class="bp3-non-ideal-state-visual" style="font-size: 48px; line-height: 48px;">
                <Icon icon={Icon::Error} intent={Intent::Danger} size={48} />
            </div>
            <div class="bp3-non-ideal-state-text">
                <h4 class="bp3-heading">{ &props.title }</h4>
                <div>{ &props.status }</div>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct LoadingProps {
    pub title: String,
    pub status: String,
}

#[function_component]
pub fn Loading(props: &LoadingProps) -> Html {
    html! {
        <div class="bp3-non-ideal-state">
            <div class="bp3-non-ideal-state-visual" style="font-size: 48px; line-height: 48px;">
                <Spinner size={48.0} />
            </div>
            <div class="bp3-non-ideal-state-text">
                <h4 class="bp3-heading">{ &props.title }</h4>
                <div>{ &props.status }</div>
            </div>
        </div>
    }
}
