use crate::components::{Content, Footer, SimpleNavbar};
use yew::prelude::*;

const TEXT: &'static str = include_str!("about.md");

#[function_component]
fn AboutText() -> Html {
    let html = comrak::markdown_to_html(&TEXT, &Default::default());
    let parsed = Html::from_html_unchecked(AttrValue::from(html));
    html! {
        <div class="prose prose-slate dark:prose-invert max-w-2xl m-auto p-4 pt-12">
        { parsed }
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
