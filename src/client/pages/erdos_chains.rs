use dioxus::prelude::*;

use crate::{data::ErdosChains, client::components::ErdosChainList};

pub fn ErdosChains(cx: Scope) -> Element {
    let route = use_route(&cx);
    let id = route.segment("id").unwrap();
    let erdos_chains = {
        let id = id.to_string();
        use_future(&cx, (), |_| async move {
            pot::from_slice::<ErdosChains>(
                &reqwest::get(format!("http://localhost:3000/api/erdos_chains/{id}"))
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
        cx.render(rsx! (
            ErdosChainList {
                id: id,
                chain: &erdos_chains.erdos_chains[0],
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
