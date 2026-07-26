//! Radio button style: [`radio`].
//!
//! Follows the [`crate::style::toggles`] pattern literally — **off = ivory/
//! fill (rest)**, **on = terracotta (active), ivory dot** — reusing the
//! exact same [`super::toggles::toggle_colors`] recipe checkbox and toggler
//! use, since a radio button is the same off/on split again, just drawn
//! circular instead of square/pill.
//!
//! Circular is free: `radio::Style` in iced 0.14 has no `radius` field at
//! all — the widget always fills a circle sized to its own bounds (`size /
//! 2.0`), so there's nothing to set here for shape.
//!
//! **Surprise vs. checkbox/toggler**: `radio::Status` is *only* `Active {
//! is_selected } | Hovered { is_selected }` — there is no `Disabled` variant
//! whatsoever (checkbox/toggler both have one). A disabled-looking radio row
//! is therefore a consumer concern (e.g. not attaching `on_click`), not
//! something this style closure can special-case.

// `iced::widget::radio` names both a module (`Status`/`Style` live there)
// and, via iced's helper function, a value — the same collision as
// checkbox/toggler (see `toggles.rs`), so it's aliased on import.
use iced::widget::radio as radio_widget;
use iced::Background;
use saola_tokens::{Surface, Theme};

use super::toggles::toggle_colors;
use crate::convert::ColorExt;

/// A radio button — one option in a single-choice group (e.g. concept 9d's
/// Ascending/Descending rows). Unselected = the rest fill with no dot
/// (iced only draws the dot when `is_selected`, so `dot_color` is inert in
/// that state); selected = terracotta with an ivory dot. Hover steps
/// through the fill roles, exactly like checkbox/toggler.
pub fn radio(
    t: &Theme,
    s: Surface,
) -> impl Fn(&iced::Theme, radio_widget::Status) -> radio_widget::Style {
    let colors = toggle_colors(t, s);

    move |_, status| {
        use radio_widget::Status;
        let (background, dot_color, border_color) = match status {
            Status::Active { is_selected } => {
                if is_selected {
                    (colors.on, colors.on_content, colors.on)
                } else {
                    (colors.off, colors.off_content, colors.off_border)
                }
            }
            Status::Hovered { is_selected } => {
                if is_selected {
                    (colors.on_hovered, colors.on_content, colors.on_hovered)
                } else {
                    (colors.off_hovered, colors.off_content, colors.off_border)
                }
            }
        };
        radio_widget::Style {
            background: Background::Color(background.into_iced()),
            dot_color: dot_color.into_iced(),
            border_width: 1.0,
            border_color: border_color.into_iced(),
            text_color: None,
        }
    }
}
