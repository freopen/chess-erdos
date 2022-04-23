use crate::client::pages::app;

mod components;
mod pages;
mod uno;

pub fn launch() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    dioxus::web::launch(app);
}
