use crate::client::uno::UnoAttributes;
use dioxus::prelude::*;

#[inline_props]
pub fn Layout<'a>(cx: Scope<'a>, children: Element<'a>) -> Element {
    let router = use_router(&cx);
    let user_id = use_state(&cx, || "".to_string());
    cx.render(rsx! (
        div {
            header {
                h1 {
                    u_text: "3xl center",
                    u_p: "6",
                    "World Chess Champion number"
                }
                div {
                    u_flex: "~",
                    u_justify: "center",
                    Link {
                        to: "/",
                        div {
                            u_display: "inline-block",
                            u_p: "r-8",
                            class: "i-fa6-solid:house",
                        }
                    }
                    input {
                        "type": "text",
                        placeholder: "Enter lichess username",
                        oninput: move |e| {
                            user_id.set(e.value.clone());
                        },
                        onkeyup: move |e| {
                            if e.key == "Enter" {
                                router.push_route(&format!("/user/{user_id}"), None, None);
                            }
                        },
                    }
                }
            }
            main {
                u_flex: "~",
                u_justify: "center",
                children
            }
        }
    ))
}
