use implicit_clone::unsync::{IArray, IString};
use yew::prelude::*;
use yew_router::prelude::{Switch, *};
use yewprint::*;

mod components;
mod crates;
mod router;

fn main() {
    wasm_logger::init(Default::default());
    yew::Renderer::<router::App>::new().render();
}
