use super::*;
use yew_router::prelude::*;

#[function_component]
fn Footer() -> Html {
    html! {
        <div class="text-center py-4">
            <a href="https://github.com/xfbs/diff.rs">{"diff.rs"}</a>
            {" build "}
            <a class="font-mono" href={concat!("https://github.com/xfbs/diff.rs/commit/", env!("VERGEN_GIT_SHA"))}>{&env!("VERGEN_GIT_SHA")[0..8]}</a>
            {", made with ❤️ by "}
            <a href="https://github.com/xfbs">{"xfbs"}</a>
        </div>
    }
}

#[function_component]
fn Logo() -> Html {
    html! {
        <h1 class="text-center text-3xl font-bold my-12">{ "diff.rs" }</h1>
    }
}

#[function_component]
pub fn Home() -> Html {
    let navigator = use_navigator().unwrap();
    let onchange = move |input: String| {
        if !input.is_empty() {
            navigator.push(&Route::Search { krate: input });
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
                </Content>
            </div>
            <Footer />
        </div>
    }
}
