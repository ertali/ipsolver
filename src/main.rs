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
    // For local dev (wasm-pack test, etc.)
    wasm_logger::init(wasm_logger::Config::default());
    Renderer::<components::App>::new().render();
}
