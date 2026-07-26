//! Toggle styles: [`checkbox`] and [`toggler`].
//!
//! Both follow the one rule literally: **off = ivory/fill (rest)**, the same
//! opaque-ivory-on-ink / translucent-ink-fill-on-paper split as
//! [`crate::style::button::rest`]; **on = terracotta (active)**, identical on
//! both surfaces, the same as [`crate::style::button::active`]. Neither
//! widget's `Status` has a `Pressed`/`Dragged` variant (only
//! `Active`/`Hovered`/`Disabled`, each carrying the checked/toggled flag), so
//! there is one hover step instead of button's hover-then-press pair.
//!
//! Checkbox is the one deliberately-not-round shape: `radii.checkbox` (7px),
//! not a pill. The toggler stays a perfect pill by leaving `border_radius` as
//! `None` (iced draws a fully round toggler when it's unset).
//!
//! The "content on a fill" half of the one rule governs the small elements
//! each widget draws on top of its fill: a checkbox's check-mark glyph and a
//! toggler's knob take ink on an ivory/fill background, ivory on a terracotta
//! one — exactly like a button's label.

// `iced::widget::checkbox`/`toggler` name both a module (Status/Style live
// there) *and*, via iced's helper functions, a value in the widget
// constructor's namespace — importing either with `use` collides with the
// `pub fn checkbox`/`toggler` this module defines below, so the widget
// modules are referred to by their full path instead.
use iced::widget::checkbox as checkbox_widget;
use iced::widget::toggler as toggler_widget;
use iced::{Background, Border, Color};
use saola_tokens::{Color as TokenColor, Surface, Theme};

use crate::convert::ColorExt;

/// Off/on backgrounds (with one hover step each) plus the "content" color
/// that sits on each, shared by checkbox and toggler since both follow the
/// same off = ivory/fill, on = terracotta split. `pub(crate)` so
/// [`crate::style::radio`] can reuse the exact same recipe — a radio button
/// is the same off/on split again, just drawn circular.
pub(crate) struct ToggleColors {
    pub(crate) off: TokenColor,
    pub(crate) off_hovered: TokenColor,
    pub(crate) off_content: TokenColor,
    pub(crate) on: TokenColor,
    pub(crate) on_hovered: TokenColor,
    pub(crate) on_content: TokenColor,
    /// `on(s).divider` — the off-state outline.
    pub(crate) off_border: TokenColor,
    pub(crate) disabled_bg: TokenColor,
    pub(crate) disabled_content: TokenColor,
}

pub(crate) fn toggle_colors(t: &Theme, s: Surface) -> ToggleColors {
    let on = *t.on(s);
    let accent = t.palette.accent;
    // Off = ivory/fill (rest): opaque ivory pill over ink (hover steps
    // through the on-paper fills, since an ivory control is a tiny paper
    // surface), translucent ink-fill on paper (hover blends one step deeper).
    let (off, off_hovered, off_content) = match s {
        Surface::Ink => (
            t.palette.paper,
            t.on_paper.fill_subtle.over(t.palette.paper),
            t.palette.ink,
        ),
        Surface::Paper => (t.on_paper.fill, t.on_paper.fill_strong, t.on_paper.primary),
    };
    ToggleColors {
        off,
        off_hovered,
        off_content,
        // On = terracotta (active), identical on both surfaces; hover steps
        // through the on-ink (ivory) fills over accent, same as
        // `button::active`.
        on: accent,
        on_hovered: t.on_ink.fill_subtle.over(accent),
        on_content: t.palette.paper,
        off_border: on.divider,
        disabled_bg: on.fill_subtle,
        disabled_content: on.disabled,
    }
}

/// A checkbox. Radius is `radii.checkbox` (7px) — the one deliberately
/// not-round shape in Saola.
pub fn checkbox(
    t: &Theme,
    s: Surface,
) -> impl Fn(&iced::Theme, checkbox_widget::Status) -> checkbox_widget::Style {
    let radius = t.radii.checkbox;
    let colors = toggle_colors(t, s);
    let on = *t.on(s);

    move |_, status| {
        use checkbox_widget::Status;
        let (background, icon_color, border_color, text_color) = match status {
            Status::Active { is_checked } => {
                if is_checked {
                    (colors.on, colors.on_content, colors.on, on.primary)
                } else {
                    (
                        colors.off,
                        colors.off_content,
                        colors.off_border,
                        on.primary,
                    )
                }
            }
            Status::Hovered { is_checked } => {
                if is_checked {
                    (
                        colors.on_hovered,
                        colors.on_content,
                        colors.on_hovered,
                        on.primary,
                    )
                } else {
                    (
                        colors.off_hovered,
                        colors.off_content,
                        colors.off_border,
                        on.primary,
                    )
                }
            }
            Status::Disabled { .. } => (
                colors.disabled_bg,
                colors.disabled_content,
                colors.disabled_bg,
                colors.disabled_content,
            ),
        };
        checkbox_widget::Style {
            background: Background::Color(background.into_iced()),
            icon_color: icon_color.into_iced(),
            border: Border {
                color: border_color.into_iced(),
                width: 1.0,
                radius: radius.into(),
            },
            text_color: Some(text_color.into_iced()),
        }
    }
}

/// A toggler (switch). Left as a perfect pill (`border_radius: None`).
pub fn toggler(
    t: &Theme,
    s: Surface,
) -> impl Fn(&iced::Theme, toggler_widget::Status) -> toggler_widget::Style {
    let colors = toggle_colors(t, s);
    let on = *t.on(s);

    move |_, status| {
        use toggler_widget::Status;
        let (track, knob, text_color) = match status {
            Status::Active { is_toggled } => {
                if is_toggled {
                    (colors.on, colors.on_content, on.primary)
                } else {
                    (colors.off, colors.off_content, on.primary)
                }
            }
            Status::Hovered { is_toggled } => {
                if is_toggled {
                    (colors.on_hovered, colors.on_content, on.primary)
                } else {
                    (colors.off_hovered, colors.off_content, on.primary)
                }
            }
            Status::Disabled { .. } => (
                colors.disabled_bg,
                colors.disabled_content,
                colors.disabled_content,
            ),
        };
        toggler_widget::Style {
            background: Background::Color(track.into_iced()),
            background_border_width: 0.0,
            background_border_color: Color::TRANSPARENT,
            foreground: Background::Color(knob.into_iced()),
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
            text_color: Some(text_color.into_iced()),
            // `None` draws a perfectly round toggler — everything is a pill.
            border_radius: None,
            padding_ratio: 0.1,
        }
    }
}
