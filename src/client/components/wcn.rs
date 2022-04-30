use dioxus::prelude::*;

pub fn WCN(cx: Scope) -> Element {
    cx.render(rsx!(
        span {
            class: "i-fa6-solid:chess-king",
        }
        span {
            class: "i-fa6-solid:hashtag",
        }
    ))
}
