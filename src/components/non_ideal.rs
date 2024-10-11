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
        <div class="m-auto text-center">
            <div class="" style="font-size: 48px; line-height: 48px;">
                <Icon icon={Icon::Error} intent={Intent::Danger} size={48} />
            </div>
            <div class="">
                <h4 class="font-bold">{ &props.title }</h4>
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
        <div class="m-auto text-center">
            <div class="" style="font-size: 48px; line-height: 48px;">
                <Spinner size={48.0} />
            </div>
            <div class="">
                <h4 class="font-bold">{ &props.title }</h4>
                <div>{ &props.status }</div>
            </div>
        </div>
    }
}
