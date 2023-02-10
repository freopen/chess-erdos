use dioxus::prelude::*;
use dioxus_router::use_route;

use crate::{
    client::{
        components::{
            time::{Time, WC_TIME},
            user_card::UserCard,
            wcn::WCN,
        },
        API_URL,
    },
    data::User,
    util::ERDOS_ID,
};

fn WCErdosChains(cx: Scope) -> Element {
    cx.render(rsx!(
        div {
            div {
                WCN {}
                "0"
            }
            div {
                "from: "
                Time {
                    time: &WC_TIME,
                }
            }
            div {
                "to: now"
            }
            a {
                href: "https://en.wikipedia.org/wiki/World_Chess_Championship_2013",
                "WCC 2013"
            }
        }
    ))
}

pub fn UserPage(cx: Scope) -> Element {
    let route = use_route(cx);
    let id: String = route.parse_segment("id").unwrap().unwrap();
    if id.to_lowercase() == ERDOS_ID.to_lowercase() {
        return cx.render(rsx!(WCErdosChains {}));
    }
    let user = use_future(cx, (&id,), |(id,)| async move {
        reqwest::get(format!("{API_URL}/user/{id}"))
            .await
            .unwrap()
            .json::<Option<User>>()
            .await
            .unwrap()
    });

    let Some(user) = user.value() else {
        return cx.render(
            rsx! (
                div {
                    "Loading..."
                }
            )
        );
    };
    let Some(user) = user else {
        return cx.render(
            rsx! (
                div {
                    "User not found"
                }
            )
        );
    };
    if user.erdos_link_meta.is_empty() {
        return cx.render(rsx! (
            div {
                "User found, but they has no " WCN{} " yet."
            }
        ));
    }
    cx.render(rsx!(UserCard { user: user }))
}
