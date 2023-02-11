use dioxus::prelude::*;
use dioxus_router::Link;

#[inline_props]
pub fn Layout<'a>(cx: Scope<'a>, children: Element<'a>) -> Element {
    let user_id = use_state(cx, || "".to_string());
    cx.render(rsx! (
        div {
            header {
                Link {
                    id: "home",
                    to: "/",
                    span {
                        class: "i-fa6-solid-house",
                    }
                }
                input {
                    id: "username_input",
                    "type": "text",
                    placeholder: "Enter lichess username",
                    oninput: move |e| {
                        user_id.set(e.value.clone());
                    },
                }
            }
            main {
                children
            }
        }
    ))
}
