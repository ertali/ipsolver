use wasm_bindgen::prelude::*;
use yew::Renderer;

mod components;
mod interior;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    Renderer::<components::App>::new().render();
    Ok(())
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    Renderer::<components::App>::new().render();
}
