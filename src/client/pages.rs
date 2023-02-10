#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_router::{Redirect, Route, Router};

mod home;
mod layout;
mod user;

pub fn app(cx: Scope) -> Element {
    cx.render(rsx! {
        Router {
            // layout::Layout {
                Route {
                    to: "/",
                    home::Home {}
                }
                Route {
                    to: "/@/:id",
                    user::UserPage {}
                }
                // Route {
                //     to: "/@/:id/:erdos_num",
                //     user::UserPage {}
                // }
                // Route {
                //     to: "/@/:id/:erdos_num/:path_id",
                //     user::UserPage {}
                // }
                Redirect {
                    from: ""
                    to: "/"
                }
            // }
        }
    })
}
