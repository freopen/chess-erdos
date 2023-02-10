use crate::client::pages::app;

mod components;
mod pages;

pub const API_URL: &str = env!("API_URL");

pub fn launch() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    dioxus_web::launch(app);
}
