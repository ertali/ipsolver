use wasm_bindgen::prelude::*;
use yew::Renderer;

pub mod components;
pub mod interior;

pub use components::App;

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    Renderer::<App>::new().render();
}
