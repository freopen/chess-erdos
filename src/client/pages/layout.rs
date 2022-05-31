use dioxus::prelude::*;

#[inline_props]
pub fn Layout<'a>(cx: Scope<'a>, children: Element<'a>) -> Element {
    let router = use_router(&cx);
    let user_id = use_state(&cx, || "".to_string());
    cx.render(rsx! (
        div {
            class: "w-max-content",
            header {
                Link {
                    to: "/",
                    span {
                        class: "i-fa6-solid:chess-king",
                    }
                    span {
                        class: "i-fa6-solid:hashtag",
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
                            router.push_route(&format!("/@/{user_id}"), None, None);
                        }
                    },
                }
            }
            main {
                children
            }
        }
    ))
}
