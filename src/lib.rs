use wasm_bindgen::prelude::*;

mod client;
mod data;

#[wasm_bindgen(start)]
pub fn run() {
    client::launch();
}
