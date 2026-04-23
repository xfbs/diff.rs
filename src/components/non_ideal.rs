use yew::prelude::*;

#[function_component]
fn ErrorIcon() -> Html {
    // from https://lucide.dev/icons/circle-x
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-12 h-12 text-red-500 dark:text-red-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10" />
            <path d="m15 9-6 6" />
            <path d="m9 9 6 6" />
        </svg>
    }
}

#[function_component]
fn Spinner() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" class="animate-spin w-12 h-12 text-blue-500 dark:text-blue-400" viewBox="0 0 24 24" fill="none">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 0 1 8-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 0 1 4 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
        </svg>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct ErrorProps {
    pub title: String,
    pub status: String,
}

#[function_component]
pub fn Error(props: &ErrorProps) -> Html {
    html! {
        <div class="m-auto text-center dark:text-gray-100">
            <div class="p-2 flex justify-center">
                <ErrorIcon />
            </div>
            <div>
                <h4 class="font-bold p-2">{ &props.title }</h4>
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
        <div class="m-auto text-center dark:text-gray-100">
            <div class="p-2 flex justify-center">
                <Spinner />
            </div>
            <div>
                <h4 class="font-bold p-2">{ &props.title }</h4>
                <div>{ &props.status }</div>
            </div>
        </div>
    }
}
