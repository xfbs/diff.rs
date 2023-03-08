mod components;
mod crates;
mod router;
mod cache;

fn main() {
    wasm_logger::init(Default::default());
    yew::Renderer::<router::App>::new().render();
}
