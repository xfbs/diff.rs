mod components;
mod crates;
mod router;

fn main() {
    wasm_logger::init(Default::default());
    yew::Renderer::<router::App>::new().render();
}
