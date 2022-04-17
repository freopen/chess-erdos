use wasm_bindgen::prelude::*;

mod client;
mod data;
mod uno;

#[wasm_bindgen(start)]
pub fn run() {
    client::launch();
}
