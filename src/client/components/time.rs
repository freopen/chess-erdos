use chrono::{DateTime, Utc, TimeZone};
use dioxus::prelude::*;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref LOCAL_TZ: chrono::FixedOffset =
        chrono::FixedOffset::west((js_sys::Date::new_0().get_timezone_offset() * 60.) as i32);
    pub static ref WC_TIME: DateTime<Utc> = Utc.ymd(2013, 11, 22).and_hms(0, 0, 0);
}

#[inline_props]
pub fn Time<'a>(cx: Scope<'a>, time: &'a DateTime<Utc>) -> Element {
    let duration = chrono_humanize::HumanTime::from(**time);
    let full_time = time
        .with_timezone(&*LOCAL_TZ)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    cx.render(rsx! (
        span {
            title: "{full_time}",
            "{duration}"
        }
    ))
}
