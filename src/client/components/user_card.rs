use dioxus::prelude::*;

use crate::data::User;

#[inline_props]
pub fn UserCard<'a>(cx: Scope, user: &'a User) -> Element {
    cx.render(rsx!(
        div {
            "{user.id}"
        }
    ))
}
