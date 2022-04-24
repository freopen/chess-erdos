use dioxus::prelude::*;

use crate::{client::uno::UnoAttributes, data::ErdosLink};

#[inline_props]
pub fn ErdosChainList<'a>(cx: Scope<'a>, id: &'a str, chain: &'a Vec<ErdosLink>) -> Element {
    let mut winner: &str = id;
    cx.render(rsx! (
        div {
            u_divide: "y",
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

#[inline_props]
fn ErdosLinkCard<'a>(cx: Scope<'a>, winner: &'a str, link: &'a ErdosLink) -> Element {
    cx.render(rsx! (
        div {
            u_w: "full",
            u_grid: "~ cols-2 gap-2",
            u_m: "2",
            u_p: "2",
            div {
                u_grid: "col-span-2",
                "time"
            }
            div {
                UserLabel {
                    id: winner,
                    title: &link.winner_info.title,
                }
                div {
                    RatingLabel {
                        rating: link.winner_info.rating,
                        rating_change: link.winner_info.rating_change,
                        is_winner: true,
                    }
                }
            }
            div {
                u_text: "right",
                UserLabel {
                    id: &link.loser_id,
                    title: &link.loser_info.title,
                }
                div {
                    RatingLabel {
                        rating: link.loser_info.rating,
                        rating_change: link.loser_info.rating_change,
                        is_winner: false,
                    }
                }
            }
        }
    ))
}

#[inline_props]
fn UserLabel<'a>(cx: Scope<'a>, id: &'a str, title: &'a str) -> Element {
    cx.render(rsx!("{id}"))
}

#[inline_props]
fn RatingLabel(cx: Scope, rating: u32, rating_change: i32, is_winner: bool) -> Element {
    cx.render(if *is_winner {
        rsx!("{rating}+{rating_change}")
    } else {
        let abs_rating_change = -rating_change;
        rsx!("{rating}-{abs_rating_change}")
    })
}
