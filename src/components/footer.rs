use yew::prelude::*;

/// Git commit this was built from, if known (not available in builds without git, such as Nix).
const GIT_SHA: Option<&str> = option_env!("VERGEN_GIT_SHA");

#[function_component]
pub fn Footer() -> Html {
    html! {
        <div class="text-center py-4 text-gray-700 dark:text-gray-300">
            <a href="https://github.com/xfbs/diff.rs">{"diff.rs"}</a>
            if let Some(sha) = GIT_SHA {
                {" build "}
                <a class="font-mono" href={format!("https://github.com/xfbs/diff.rs/commit/{sha}")}>{sha.get(..8).unwrap_or(sha)}</a>
            }
            {", made with ❤️ by "}
            <a href="https://github.com/xfbs">{"xfbs"}</a>
        </div>
    }
}
