use dioxus::prelude::*;

use crate::data::ErdosLink;

#[inline_props]
pub fn ErdosLinkCard<'a>(cx: Scope<'a>, winner: &'a str, link: &'a ErdosLink) -> Element {
    cx.render(rsx! (
        div {
            p { "{winner}" }
            p { "{link:?}" }
        }
    ))
}
