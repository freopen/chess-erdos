use dioxus::prelude::*;

pub fn Home(cx: Scope) -> Element {
    cx.render(rsx! (
        div {
            p {"This website computes World Chess Champion number." }

        }
    ))
}
