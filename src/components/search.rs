use super::*;
use web_sys::HtmlInputElement;

#[function_component]
pub fn Search() -> Html {
    let state = use_state(|| "".to_string());
    let navigator = use_navigator().unwrap();
    html! {
        <InputGroup
            placeholder="Search crates..."
            fill={true}
            left_icon={Icon::Search}
            value={state.to_string()}
            oninput={
                let state = state.clone();
                move |event: InputEvent| {
                    state.set(event.target_unchecked_into::<HtmlInputElement>().value());
                }
            }
            onkeydown={
                let state = state;
                move |event: KeyboardEvent| {
                    if event.key() == "Enter" {
                        navigator.push(&Route::Crate {
                            name: state.to_string(),
                        });
                    }
                }
            }
        />
    }
}
