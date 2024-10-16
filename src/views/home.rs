use crate::{components::*, Route};
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component]
fn Logo() -> Html {
    html! {
        <h1 class="text-center text-3xl font-bold my-12 dark:text-white">{ "diff.rs" }</h1>
    }
}

/// Home page, shows search bar.
#[function_component]
pub fn Home() -> Html {
    let navigator = use_navigator().unwrap();
    let onchange = move |input: String| {
        if !input.is_empty() {
            navigator.push(&Route::Search { query: input });
        }
    };
    html! {
        <div class="flex flex-col min-h-screen">
            <div class="flex-1">
                <SimpleNavbar />
                <Content>
                    <Logo />
                    <div class="max-w-3xl m-auto">
                        <SearchBar value={""} {onchange} />
                    </div>
                    <DefaultSummarySection />
                </Content>
            </div>
            <Footer />
        </div>
    }
}
