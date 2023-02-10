use dioxus::prelude::*;

use crate::client::{components::wcn::WCN, API_URL};

#[inline_props]
fn Header<'a>(cx: Scope<'a>, children: Element<'a>) -> Element {
    cx.render(rsx!(h2 { children }))
}

#[inline_props]
fn Paragraph<'a>(cx: Scope<'a>, children: Element<'a>) -> Element {
    cx.render(rsx!(p { children }))
}

#[inline_props]
fn Link<'a>(cx: Scope<'a>, href: &'a str, children: Element<'a>) -> Element {
    cx.render(rsx!(a {
        href: "{href}",
        children
    }))
}

pub fn Home(cx: Scope) -> Element {
    let last_processed_future = {
        use_future(cx, (), |_| async move {
            reqwest::get(format!("{API_URL}/last_processed"))
                .await
                .unwrap()
                .text()
                .await
                .unwrap()
        })
    };
    let last_processed_block = last_processed_future.value().map(|last_processed| {
        rsx! (
            Paragraph {
                "Last processed game log archive: { last_processed }."
            }
        )
    });
    cx.render(rsx! (
        div {
            h1 {
                "World Chess Champion number" 
            }
            Paragraph {
                "This website calculates the World Chess Champion Number (" WCN{} ") for every "
                "Lichess user."
            }
            last_processed_block
            Header {
                "What is the World Chess Champion Number?"
            }
            Paragraph {
                "First we need to discuss "
                Link {
                    href: "https://en.wikipedia.org/wiki/Erd%C5%91s_number",
                    "the Erdős number"
                }
                ". Paul Erdős (1913–1996) was an influential Hungarian mathematician who in the "
                "latter part of his life spent a great deal of time writing papers with a large "
                "number of colleagues, working on solutions to outstanding mathematical problems. "
                "Paul Erdős has an Erdős number of zero. Any coauthor of Erdős has an Erdős "
                "number of 1. Any coauthor of a coauthor of Erdős has an Erdős number of 2, and "
                "so on."
            }
            Paragraph {
                "Similarly the current WC has " WCN{} "0. Anyone who wins a game against the WC "
                "has " WCN{} "1, anyone who wins against " WCN{} "1 has " WCN{} "2, and so on. To "
                "get any " WCN{} " a player has to (directly or indirectly) win against the current "
                "WC on Lichess."
            }
            Header {
                "But then almost nobody will have a " WCN{} " right?"
            }
            Paragraph {
                "Sven Magnus Øen Carlsen, a chess grandmaster and the five-time World Chess "
                "Champion, is considered to be a pretty good chess player. His Lichess account "
                Link {
                    href: "https://lichess.org/@/DrNykterstein",
                    "DrNykterstein"
                }
                " reports that he lost just 16% of his blitz games. Nevertheless sometimes he "
                "loses to a weaker player, then they lose to even weaker player until eventually "
                "nearly everyone who won a single rated game on Lichess has a " WCN{} " and "
                "therefore (indirectly) won against Magnus Carlsen. This works similarly to the "
                Link {
                    href: "https://en.wikipedia.org/wiki/Six_degrees_of_separation",
                    "Six Degrees of Separation"
                }
                " idea."
            }
            Header {
                "That sounds like an awful way of measuring the chess skill!"
            }
            Paragraph {
                "Correct! If you're skilled player, you will probably have a lower " WCN{} " but "
                "there will be lots of very unexpected numbers. I created this website just for "
                "fun. I will be glad if this website inspires someone to play more chess."
            }
            Header {
                "Okay, how exactly is " WCN{} " calculated?"
            }
            Paragraph {
                "First we assign DrNykterstein a " WCN{} "0. Then scan every single Lichess game "
                "in chronological order that matches the following criteria: "
                ul {
                    li { "Rated game" }
                    li { "Classical, Rapid or Blitz time control" }
                    li { "At least 10 moves were played" }
                    li { "Ends decisively (by checkmate, timeout or resignation)" }
                }
                "If a loser of a game has a " WCN{} "x, and the winner has no " WCN{} " or his "
                WCN{} " is bigger than x+1 - then the winner assigned " WCN{} "x+1."
            }
            Header {
                "Why this specific set of rules?"
            }
            Paragraph {
                "The eligible game rule set is selected to balance between giving " WCN{} " to "
                "as many Lichess users as possible and to avoid strange cases when someone wins a "
                "superbullet game against +1500 rated player and skew " WCN{} " for everyone. "
                "I decided to include Blitz games because lots of people never play longer "
                "controls on Lichess (including Magnus himself). After all blitz is good enough "
                "to be a World Chess Championship tiebreak, it's natural to include it here too."
            }
            Paragraph {
                "The chronological game processing means that if you want " WCN{} "x, you have to "
                "win against someone with " WCN{} "x-1 after they got that rating. I think it's "
                "fair: people usually get better at chess over time, if someone you won earlier "
                "earns better " WCN{} " it doesn't mean you're automatically entitled to a better "
                WCN{} " too." 
            }
            Header {
                "How is this website made?"
            }
            Paragraph {
                "The whole source code is hosted on "
                Link {
                    href: "https://github.com/freopen/chess-erdos",
                    "GitHub"
                }
                ". The backend is written in Rust, using "
                Link {
                    href: "https://github.com/rust-rocksdb/rust-rocksdb",
                    "RocksDB Rust wrapper"
                }
                " for database and "
                Link {
                    href: "https://github.com/tokio-rs/axum",
                    "Axum"
                }
                " for web server. The frontend is written in Rust compiled to WebAssembly, using "
                Link {
                    href: "https://dioxuslabs.com/",
                    "Dioxus"
                }
                " for UI library, and "
                Link {
                    href: "https://uno.antfu.me",
                    "UnoCSS"
                }
                " for styling. The data source is "
                Link {
                    href: "https://database.lichess.org/",
                    "Lichess Database"
                }
                "."
            }
        }
    ))
}
