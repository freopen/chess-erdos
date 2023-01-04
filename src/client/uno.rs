use dioxus_core::*;
use std::fmt::Arguments;

macro_rules! uno_attribute {
    (
        $(
            $(#[$attr:meta])*
            $name:ident: $lit:literal;
        )*
    ) => {
        $(
            $(#[$attr])*
            fn $name<'a>(&self, cx: NodeFactory<'a>, val: Arguments) -> Attribute<'a> {
                cx.attr($lit, val, None, false)
            }
        )*
    };
}

pub trait UnoAttributes {
    uno_attribute! {
        u_font: "u-font";
        u_text: "u-text";
        u_underline: "u-underline";
        u_list: "u-list";
        u_bg: "u-bg";
        u_gradient: "u-gradient";
        u_border: "u-border";
        u_divide: "u-divide";
        u_ring: "u-ring";
        u_icon: "u-icon";
        u_container: "u-container";
        u_p: "u-p";
        u_m: "u-m";
        u_space: "u-space";
        u_w: "u-w";
        u_min_w: "u-min-w";
        u_max_w: "u-max-w";
        u_h: "u-h";
        u_min_h: "u-min-h";
        u_max_h: "u-max-h";
        u_flex: "u-flex";
        u_grid: "u-grid";
        u_table: "u-table";
        u_order: "u-order";
        u_align: "u-align";
        u_justify: "u-justify";
        u_place: "u-place";
        u_display: "u-display";
        u_pos: "u-pos";
        u_box: "u-box";
        u_caret: "u-caret";
        u_isolation: "u-isolation";
        u_object: "u-object";
        u_overflow: "u-overflow";
        u_overscroll: "u-overscroll";
        u_z: "u-z";
        u_shadow: "u-shadow";
        u_opacity: "u-opacity";
        u_blend: "u-blend";
        u_filter: "u-filter";
        u_backdrop: "u-backdrop";
        u_transition: "u-transition";
        u_animate: "u-animate";
        u_transform: "u-transform";
        u_appearance: "u-appearance";
        u_cursor: "u-cursor";
        u_outline: "u-outline";
        u_pointer: "u-pointer";
        u_resize: "u-resize";
        u_select: "u-select";
        u_sr: "u-sr";
    }
}

impl<T: DioxusElement> UnoAttributes for T {}
