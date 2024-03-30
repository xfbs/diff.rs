use super::*;

#[function_component]
pub fn Home() -> Html {
    html! {
        <>
            <SimpleNavbar />
            <Content>
                <div style="width: 700px; margin: auto; padding-top: 20px;">
                    <h1>{ "diff.rs" }</h1>
                    <p>{ "View the differences between Rust crate versions. Enter a crate name such as "}<a href="/serde/1.0.153/1.0.153">{"serde"}</a>{" in the search field in the top-right corner to get started." }</p>
                    <p>{ "This is a WebAssembly-based web application written in Rust with "}<a href="https://docs.rs/yew">{"Yew"}</a>{". It uses the "}<a href="https://crates.io/">{"crates.io"}</a>{" API to fetch crate metadata, downloads and parses the crate sources in-memory and renders a diff in, all in the browser." }</p>
                    <p>{"Source code for this application is available at "}<a href="https://github.com/xfbs/diff.rs">{"github.com/xfbs/diff.rs"}</a>{"."}</p>
                </div>
            </Content>
        </>
    }
}
