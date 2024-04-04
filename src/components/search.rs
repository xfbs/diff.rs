use crate::app::Route;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component]
fn SearchGlass() -> Html {
    html! {
        <svg class="w-4 h-4 text-gray-500 dark:text-gray-400" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 20">
            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m19 19-4-4m0-7A7 7 0 1 1 1 8a7 7 0 0 1 14 0Z"/>
        </svg>
    }
}

#[function_component]
pub fn Search() -> Html {
    let state = use_state(|| "".to_string());
    let navigator = use_navigator().unwrap();

    let oninput = {
        let state = state.clone();
        move |event: InputEvent| {
            state.set(event.target_unchecked_into::<HtmlInputElement>().value());
        }
    };

    let onkeydown = {
        let state = state.clone();
        move |event: KeyboardEvent| {
            if event.key() == "Enter" {
                navigator.push(&Route::Crate {
                    name: state.to_string(),
                });
            }
        }
    };

    html! {
        <div class="relative">
            <div class="absolute inset-y-0 start-0 flex items-center ps-3 pointer-events-none">
                <SearchGlass />
            </div>
            <input type="search" id="default-search" class="block w-full p-2 ps-10 text-sm text-gray-900 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-800 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="Search crates..." value={state.to_string()} {oninput} {onkeydown} />
        </div>
    }
}
