use yew::prelude::*;

#[function_component]
pub fn Footer() -> Html {
    html! {
        <div class="text-center py-4 text-gray-700 dark:text-gray-300">
            <a href="https://github.com/xfbs/diff.rs">{"diff.rs"}</a>
            {" build "}
            <a class="font-mono" href={concat!("https://github.com/xfbs/diff.rs/commit/", env!("VERGEN_GIT_SHA"))}>{&env!("VERGEN_GIT_SHA")[0..8]}</a>
            {", made with ❤️ by "}
            <a href="https://github.com/xfbs">{"xfbs"}</a>
        </div>
    }
}
