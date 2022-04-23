use dioxus::prelude::*;

use crate::{client::components::ErdosLinkCard, data::ErdosLink};

#[inline_props]
pub fn ErdosChainList<'a>(cx: Scope<'a>, id: &'a str, chain: &'a Vec<ErdosLink>) -> Element {
    let mut winner: &str = id;
    cx.render(rsx! (
        div {
            p { "{id}" }
            chain.iter().map(|link| {
                let winner = std::mem::replace(&mut winner, &link.loser_id);
                let key = link.erdos_number;
                rsx!(
                    ErdosLinkCard {
                        key: "{key}",
                        winner: winner,
                        link: link,
                    }
                )
            })
        }
    ))
}
