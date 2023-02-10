use yew::prelude::*;
use yewprint::*;
use implicit_clone::unsync::{IArray, IString};

#[function_component]
fn App() -> Html {
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    html! {
        <>
            <div class="bp3-navbar bp3-fixed-top">
                <div class="bp3-navbar-group bp3-align-left">
                    <div class="bp3-navbar-heading">{ "diff.rs" }</div>
                    <div class="bp3-navbar-divider"></div>
                    <div class="bp3-navbar-heading">{ "wireguard-keys" }</div>
                    <div class="bp3-navbar-heading">
                        <HtmlSelect<IString> options={[
                            ("0.1.1".into(), "0.1.1".into()),
                            ("0.1.0".into(), "0.1.0".into()),
                        ].into_iter().collect::<IArray<_>>()
                        } />
                    </div>
                    <div class="bp3-navbar-heading">{ "diff" }</div>
                    <div class="bp3-navbar-heading">
                        <HtmlSelect<IString> options={[
                            ("0.1.0".into(), "0.1.0".into()),
                            ("0.1.1".into(), "0.1.1".into()),
                        ].into_iter().collect::<IArray<_>>()
                        } />
                    </div>
                    <div class="bp3-navbar-divider"></div>
                    <div class="bp3-navbar-heading bp3-fill">
                        <InputGroup placeholder="Search crates..." fill={true} left_icon={Icon::Search} />
                    </div>
                </div>
            </div>
        <div>
            <button {onclick}>{ "+1" }</button>
            <p>{ *counter }</p>
        </div>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
