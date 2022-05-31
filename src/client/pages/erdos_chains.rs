use dioxus::prelude::*;
use reqwest::StatusCode;

use crate::{
    client::{components::{ErdosChainList, WCN, Time, WC_TIME}},
    data::ErdosChains,
    util::ERDOS_ID,
};

fn WCErdosChains(cx: Scope) -> Element {
    cx.render(rsx!(
        div {
            div {
                WCN{}
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

pub fn ErdosChains(cx: Scope) -> Element {
    let route = use_route(&cx);
    let id = route.segment("id").unwrap().to_string();
    if id.to_lowercase() == ERDOS_ID.to_lowercase() {
        return cx.render(rsx!(
            WCErdosChains{}
        ))
    }

    let erdos_chains = {
        use_future(&cx, (&id,), |(id,)| async move {
            let resp = reqwest::get(format!("https://freopen.org/api/erdos_chains/{id}"))
                .await
                .unwrap();
            if resp.status() == StatusCode::NOT_FOUND {
                None
            } else {
                assert!(resp.status().is_success());
                Some(rmp_serde::decode::from_slice::<ErdosChains>(&resp.bytes().await.unwrap()).unwrap())
            }
        })
    };

    cx.render(if let Some(erdos_chains) = erdos_chains.value() {
        if let Some(erdos_chains) = erdos_chains {
            if erdos_chains.erdos_chains.is_empty() {
                rsx! (
                    div {
                        "User found, but they has no " WCN{} " yet."
                    }
                )
            } else {
                let mut to = None;
                rsx! (
                    div {
                        erdos_chains.erdos_chains.iter().map(|chain| {
                            let key = chain[0].erdos_number;
                            if let Some(prev_to) = to.replace(&chain[0].time) {
                                rsx!(
                                    ErdosChainList {
                                        key: "{key}",
                                        id: &erdos_chains.id,
                                        chain: chain,
                                        to: prev_to,
                                    }
                                )
                            } else {
                                rsx!(
                                    ErdosChainList {
                                        key: "{key}",
                                        id: &erdos_chains.id,
                                        chain: chain,
                                    }
                                )
                            }
                        })
                    }
                )
            }
        } else {
            rsx! (
                div {
                    "User not found"
                }
            )
        }
    } else {
        rsx! (
            div {
                "Loading..."
            }
        )
    })
}
