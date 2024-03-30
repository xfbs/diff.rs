fn main() {
    wasm_logger::init(Default::default());
    yew::Renderer::<diff_rs::App>::new().render();
}
