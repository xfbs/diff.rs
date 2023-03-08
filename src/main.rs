mod cache;
mod components;
mod data;
mod router;

fn main() {
    wasm_logger::init(Default::default());
    yew::Renderer::<router::App>::new().render();
}
