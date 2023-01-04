#![allow(non_snake_case)]
use dioxus::prelude::*;

mod erdos_chains;
mod home;
mod layout;

pub fn app(cx: Scope) -> Element {
    cx.render(rsx! {
        Router {
            layout::Layout {
                Route {
                    to: "/",
                    home::Home {}
                }
                Route {
                    to: "/@/:id",
                    erdos_chains::ErdosChains {}
                }
                Redirect {
                    from: ""
                    to: "/"
                }
            }
        }
    })
}
