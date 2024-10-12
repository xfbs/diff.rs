use crate::{components::*, Route};
use yew::prelude::*;
use yew_hooks::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SearchProps {
    pub search: String,
}

#[function_component]
fn Logo() -> Html {
    html! {
        <h1 class="text-center text-3xl font-bold my-12 dark:text-white">{ "diff.rs" }</h1>
    }
}

/// Search view, shows search results.
#[function_component]
pub fn Search(props: &SearchProps) -> Html {
    let state = use_debounce_state(String::new, 500);
    state.set(props.search.clone());
    let navigator = use_navigator().unwrap();
    let onchange = move |input: String| {
        if input.is_empty() {
            navigator.push(&Route::Home);
        } else {
            navigator.push(&Route::Search { query: input });
        }
    };

    html! {
        <div class="flex flex-col min-h-screen">
            <div class="flex-1">
                <SimpleNavbar />
                <Content>
                    <div class="max-w-3xl m-auto">
                        <Logo />
                        <SearchBar value={props.search.to_string()} {onchange} />
                        <div class="my-6">
                            <SearchResults query={state.to_string()} />
                        </div>
                    </div>
                </Content>
            </div>
        </div>
    }
}
