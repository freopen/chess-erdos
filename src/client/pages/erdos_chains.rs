use dioxus::prelude::*;

use crate::{data::ErdosChains, client::{components::ErdosChainList, uno::UnoAttributes}};

pub fn ErdosChains(cx: Scope) -> Element {
    let route = use_route(&cx);
    let id = route.segment("id").unwrap();
    let erdos_chains = {
        let id = id.to_string();
        use_future(&cx, (), |_| async move {
            pot::from_slice::<ErdosChains>(
                &reqwest::get(format!("http://192.168.1.98:3000/api/erdos_chains/{id}"))
                    .await
                    .unwrap()
                    .bytes()
                    .await
                    .unwrap(),
            )
            .unwrap()
        })
    };
    if let Some(erdos_chains) = erdos_chains.value() {
        let mut to = None;
        cx.render(rsx! (
            div {
                class: "snap-x",
                u_flex: "~ nowrap",
                u_overflow: "x-scroll",
                erdos_chains.erdos_chains.iter().map(|chain| {
                    let prev_to = to.replace(&chain[0].time);
                    rsx!(
                        ErdosChainList {
                            id: id,
                            chain: chain,
                            to: prev_to,
                        }
                    )
                })
            }
        ))
    } else {
        cx.render(rsx! (
            div {
                "loading"
            }
        ))
    }
}
