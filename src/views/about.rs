use crate::components::{Content, Footer, SimpleNavbar};
use yew::prelude::*;

#[function_component]
fn AboutText() -> Html {
    html! {
        <div class="prose prose-slate dark:prose-invert max-w-2xl m-auto p-4 pt-12">
            <h2>{"About"}</h2>
            <p>
                {"Web-based tool to view the differences between Rust crate versions. "}
                {"Enter a crate name such as "}<code>{"serde"}</code>
                {" in the search field in the top-right corner to get started. " }
                {"The tool allows you to see what changed between different versions "}
                {"of the same crate, or different versions of different crates. "}
                {"This allows you to see, for example, what changed between a crate "}
                {"and its fork."}
            </p>
            <p>
                {"It is implemented as a web application written in Rust and compiled to WebAssembly. "}
                {"It uses the "}
                <a href="https://crates.io/">{"crates.io"}</a>
                {" API to fetch crate metadata. To diff two crate versions, it downloads them, "}
                {"validates their hash sums, decompresses them and extracts their source files "}
                {"in-memory. It runs a diff algorithm over the source files and renders it with "}
                {"syntax highlighting in you browser. "}
                {"Source code for this application is available at "}
                <a href="https://github.com/xfbs/diff.rs">{"github.com/xfbs/diff.rs"}</a>
                {"."}
            </p>
            <h3>{"Acknowledgements"}</h3>
            <p>
                {"This project was made possible thanks to contributions made by the "}
                {"following individuals:"}
                <ul>
                    <li>
                        <a href="https://github.com/Alphare">{"Alphare"}</a>
                        {": contributed support for diffing across crates"}
                    </li>
                    <li>
                        <a href="https://github.com/eth3lbert">{"eth3lbert"}</a>
                        {": contributed folding for section which did not change"}
                    </li>
                    <li>
                        <a href="https://github.com/mystor">{"mystor"}</a>
                        {": contributed syntax highlighting and browsing files of a single crate version"}
                    </li>
                    <li>
                        <a href="https://github.com/SwishSwushPow">{"SwishSwushPow"}</a>
                        {": contributed hiding of unchanged files and folders"}
                    </li>
                    <li>
                        <a href="https://github.com/j-mahapatra">{"j-mahapatra"}</a>
                        {": helped migrate legacy CSS rules to Tailwind CSS"}
                    </li>
                    <li>
                        <a href="https://github.com/HWienhold">{"HWienhold"}</a>
                        {": added file filtering and summary page"}
                    </li>
                    <li>
                        <a href="https://github.com/tverghis">{"tverghis"}</a>
                        {": added folder expansion and collapsing"}
                    </li>
                </ul>
                {"This list is not exhaustive, check the repository for a full "}
                <a href="https://github.com/xfbs/diff.rs/graphs/contributors">{"list of contributors"}</a>
                {". "}
            </p>
            <p>
                {"Additionally, this tool builds on work done by the Rust ecosystem. It would "}
                {"not be possible without the following crates:"}
                <ul>
                    <li>
                        <a href="https://crates.io/crates/yew">{"Yew"}</a>
                        {": responsive frontend web framework for Rust,"}
                    </li>
                    <li>
                        <a href="https://crates.io/crates/syntect">{"Syntect"}</a>
                        {": syntax highlighter,"}
                    </li>
                    <li>
                        <a href="https://crates.io/crates/similar">{"Similar"}</a>
                        {": diff algorithm implementation,"}
                    </li>
                    <li>
                        <a href="https://crates.io/crates/tar">{"Tar"}</a>
                        {": reading of Tar archives,"}
                    </li>
                    <li>
                        <a href="https://crates.io/crates/flate2">{"Flate2"}</a>
                        {": DEFLATE decompression in pure Rust."}
                    </li>
                </ul>
                {"This list is not exhaustive, check the repository for a full "}
                <a href="https://github.com/xfbs/diff.rs/blob/master/Cargo.toml">{"list of crates used"}</a>
                {"."}
            </p>
            <h3>{"License"}</h3>
            <p>
                {"Licensed under the "}
                <a href="https://github.com/xfbs/diff.rs/blob/master/LICENSE.md">{"MIT"}</a>
                {" license."}
            </p>
            <h3>{"Privacy"}</h3>
            <p>
                {"This tool runs entirely in your browser and has no backend. "}
                {"As such, no data is logged by the tool itself. "}
            </p>
            <p>
                {"To view and diff crates, it makes requests to the crates.io API, "}
                {"which may log requests according to its "}
                <a href="https://foundation.rust-lang.org/policies/privacy-policy/">{"Privacy Policy"}</a>
                {"."}
            </p>
            <p>
                {"In addition, diff.rs contains some analytics that is used to measure "}
                {"how many active users it has. This service stores some data in anonymized "}
                {"fashion, according to the "}
                <a href="https://plausible.io/data-policy">{"Data Policy"}</a>
                {". "}
                {"If you use an adblocker, then analytics will likely be blocked. "}
                {"It does not use cookies, fingerprinting or any other invasive means "}
                {"to track visitors. You can view the collected data "}
                <a href="https://counter.dev/dashboard.html?user=xfbs&token=4kPlix1Li7w%3D">{"here"}</a>
                {"."}
            </p>
        </div>
    }
}

/// About page, showing background information on this project.
#[function_component]
pub fn About() -> Html {
    html! {
        <div class="flex flex-col min-h-screen">
            <div class="flex-1">
                <SimpleNavbar />
                <Content>
                    <AboutText />
                </Content>
            </div>

            <Footer />
        </div>
    }
}
