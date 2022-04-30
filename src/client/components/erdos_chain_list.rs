use chrono::{DateTime, Utc};
use dioxus::prelude::*;

use crate::{
    client::{components::{Time, WCN}, uno::UnoAttributes},
    data::{ErdosLink, PlayerInfo, Termination, TimeControl, TimeControlType},
};

#[inline_props]
pub fn ErdosChainList<'a>(
    cx: Scope<'a>,
    id: &'a str,
    chain: &'a Vec<ErdosLink>,
    to: Option<&'a DateTime<Utc>>,
) -> Element {
    let mut winner: &str = id;
    let erdos = chain[0].erdos_number;
    cx.render(rsx! (
        div {
            class: "snap-start last:snap-end",
            u_display: "inline-block",
            u_align: "top",
            u_flex: "shrink-0",
            u_m: "4",
            div {
                u_text: "center 5xl",
                u_font: "black",
                WCN {}
                "{erdos}"
            }
            div {
                u_text: "center",
                "from: "
                Time {
                    time: &chain[0].time,
                }
            }
            div {
                u_text: "center",
                u_m: "b-8",
                "to: "
                to.as_ref().map_or(rsx!("now"), |to| rsx!(
                    Time {
                        time: to,
                    }
                ))
            }
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
    let winner_color = if link.winner_is_white {
        "i-fa-regular:circle"
    } else {
        "i-fa-solid:circle"
    };
    let loser_color = if !link.winner_is_white {
        "i-fa-regular:circle"
    } else {
        "i-fa-solid:circle"
    };
    cx.render(rsx! (
        a {
            href: "https://lichess.org/{link.game_id}",
            div {
                u_w: "full",
                u_bg: "hover:sky-100",
                u_border: "rounded",
                u_transition: "~ all duration-300",
                div {
                    u_p: "1",
                    Time {
                        time: &link.time,
                    }
                    TimeControlLabel {
                        time_control: &link.time_control,
                    }
                    GameResultLabel {
                        move_count: link.move_count,
                        termination: &link.termination,
                    }
                }
                div {
                    u_m: "l-12",
                    span {
                        class: "{winner_color}",
                    }
                    PlayerLabel {
                        id: winner,
                        info: &link.winner_info,
                        erdos: link.erdos_number,
                    }
                }
                div {
                    u_m: "l-12",
                    span {
                        class: "{loser_color}",
                    }
                    PlayerLabel {
                        id: &link.loser_id,
                        info: &link.loser_info,
                        erdos: link.erdos_number - 1,
                    }
                }
            }
        }
    ))
}

#[inline_props]
fn PlayerLabel<'a>(cx: Scope<'a>, id: &'a str, info: &'a PlayerInfo, erdos: u32) -> Element {
    let title = if info.title.is_empty() {
        None
    } else {
        Some(rsx!(
            span {
                u_p: "l-0.5",
                u_font: "bold",
                u_text: "lg amber-600",
                "{info.title}",
            }
        ))
    };
    let rating_change = format!("{:+}", info.rating_change);
    cx.render(rsx!(
        Link {
            to: "/@/{id}",
            span {
                u_p: "0.5",
                u_text: "xs fuchsia-900",
                u_font: "black",
                u_bg: "hover:sky-300",
                u_border: "rounded",
                u_transition: "~ all duration-300",
                span {
                    class: "i-fa6-solid:chess-king",
                }
                span {
                    class: "i-fa6-solid:hashtag",
                }
                "{erdos}"
            }
        }
        title
        a {
            href: "https://lichess.org/@/{id}",
            u_font: "bold",
            u_text: "lg",
            u_bg: "hover:sky-300",
            u_border: "rounded",
            u_transition: "~ all duration-300",
            u_p: "1",
            "{id}",
        }
        span {
            "({info.rating})",
        }
        if info.rating_change >= 0 {
            rsx!(span{
                u_p: "1",
                u_text: "green-600",
                "{rating_change}"
            })
        } else {
            rsx!(span{
                u_p: "1",
                u_text: "red-600",
                "{rating_change}"
            })
        }
    ))
}

#[inline_props]
fn TimeControlLabel<'a>(cx: Scope<'a>, time_control: &'a TimeControl) -> Element {
    let time_control_icon = match time_control.game_type {
        TimeControlType::Blitz => "i-mdi:fire",
        TimeControlType::Rapid => "i-mdi:rabbit",
        TimeControlType::Classical => "i-mdi:turtle",
    };
    let time_control_sig = format!("{}+{}", time_control.main / 60, time_control.increment);
    let hint = format!(
        "{} game: {} minutes of main time plus {} seconds of increment each turn",
        match time_control.game_type {
            TimeControlType::Blitz => "Blitz",
            TimeControlType::Rapid => "Rapid",
            TimeControlType::Classical => "Classical",
        },
        time_control.main / 60,
        time_control.increment
    );
    cx.render(rsx!(
        span {
            u_p: "2",
            title: "{hint}",
            span {
                class: "{time_control_icon}",
            }
            "{time_control_sig}"
        }
    ))
}

#[inline_props]
fn GameResultLabel<'a>(cx: Scope<'a>, move_count: u32, termination: &'a Termination) -> Element {
    let moves_str = if move_count % 2 == 1 {
        format!("{}.5", move_count / 2)
    } else {
        format!("{}", move_count / 2)
    };
    let termination_icon = match termination {
        Termination::Checkmate => "i-fa6-solid:hashtag",
        Termination::Resign => "i-fa6-regular:flag",
        Termination::Time => "i-fa6-regular:clock",
    };
    let hint = format!(
        "Game ended after {} moves by {}",
        moves_str,
        match termination {
            Termination::Checkmate => "checkmate",
            Termination::Resign => "resignation",
            Termination::Time => "timeout",
        }
    );
    cx.render(rsx!(
        span {
            title: "{hint}",
            span {
                class: "i-fa-solid:mouse",
            }
            "{moves_str}"
            span {
                class: "i-fa-solid:arrow-right",
            }
            span {
                class: "{termination_icon}",
            }
        }
    ))
}
