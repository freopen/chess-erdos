#![allow(non_snake_case)]
use dioxus::prelude::*;

mod layout;
mod home;
mod erdos_chains;

pub fn app(cx: Scope) -> Element {
    cx.render(
        rsx! {
            Router {
                layout::Layout {
                    Route {
                        to: "/",
                        home::Home {}
                    }
                    Route {
                        to: "/user/:id",
                        erdos_chains::ErdosChains {}
                    }
                    Redirect {
                        from: ""
                        to: "/"
                    }
                }            
            }
        }
    )
}

