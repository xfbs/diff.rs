use super::*;

/// About page, showing background information on this project.
#[function_component]
pub fn About() -> Html {
    html! {
        <div class="flex flex-col min-h-screen">
            <div class="flex-1">
                <SimpleNavbar />
                <Content>
                    <div class="max-w-2xl m-auto p-4 pt-12">
                        <p>{ "View the differences between Rust crate versions. Enter a crate name such as "}<Link to={Route::Crate { krate: "serde".into()}}>{"serde"}</Link>{" in the search field in the top-right corner to get started." }</p>
                        <p>{ "This is a WebAssembly-based web application written in Rust with "}<a href="https://docs.rs/yew">{"Yew"}</a>{". It uses the "}<a href="https://crates.io/">{"crates.io"}</a>{" API to fetch crate metadata, downloads and parses the crate sources in-memory and renders a diff in, all in the browser." }</p>
                        <p>{"Source code for this application is available at "}<a href="https://github.com/xfbs/diff.rs">{"github.com/xfbs/diff.rs"}</a>{"."}</p>
                    </div>
                </Content>
            </div>

            <div class="text-center py-4">
                <a href="https://github.com/xfbs/diff.rs">{"diff.rs"}</a>
                {" build "}
                <a class="font-mono" href={concat!("https://github.com/xfbs/diff.rs/commit/", env!("VERGEN_GIT_SHA"))}>{&env!("VERGEN_GIT_SHA")[0..8]}</a>
                {", made with ❤️ by "}
                <a href="https://github.com/xfbs">{"xfbs"}</a>
            </div>
        </div>
    }
}
