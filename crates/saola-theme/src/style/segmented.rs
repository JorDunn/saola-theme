//! Segmented control styles: [`track`] and [`segment`].
//!
//! A segmented control (concept 9d's Files | Folders | All; 4b's workspace
//! chips — "ivory keys; the one you're on is lit") is built from existing
//! widgets, not a custom one: a `row` of buttons sitting inside a
//! container. [`track`] styles that container (the `track` role fill, pill
//! radius); [`segment`] styles each button, picking between the exact
//! [`crate::style::button::rest`] and [`crate::style::button::active`]
//! recipes depending on which key is lit.
//!
//! `segment`'s selected/unselected choice is a plain `bool` the consumer
//! already knows (which key is current), not something iced's `Status`
//! carries — `button::Status` has no notion of "this is the selected
//! segment", so the two recipes are inlined here rather than delegated to
//! `button::rest`/`button::active` directly: `-> impl Fn(...)` requires a
//! single concrete closure type per function, and an `if/else` returning
//! two different helpers' closures wouldn't unify.

use iced::widget::{button, container};
use iced::{Background, Border, Color};
use saola_tokens::{Surface, Theme};

use crate::convert::ColorExt;

/// The track a row of segments sits inside — the `track` role fill at pill
/// radius. Ink text (inherited as the default for any bare content, though
/// each segment sets its own via [`segment`]).
pub fn track(t: &Theme, s: Surface) -> impl Fn(&iced::Theme) -> container::Style {
    let radius = t.radii.pill;
    let on = *t.on(s);
    move |_| container::Style {
        text_color: Some(on.primary.into_iced()),
        background: Some(Background::Color(on.track.into_iced())),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: radius.into(),
        },
        ..container::Style::default()
    }
}

/// One segment (key) in the row. `is_selected` picks the look: the lit
/// segment is `button::active`'s terracotta pill with an ivory label,
/// identical on both surfaces; every other segment is `button::rest`'s
/// ivory-on-ink / fill-on-paper pill with an ink label. Hover/press step
/// through the same fill recipes those two helpers use.
pub fn segment(
    t: &Theme,
    s: Surface,
    is_selected: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    let radius = t.radii.pill;
    let on = *t.on(s);
    let accent = t.palette.accent;

    // `button::rest`'s recipe: opaque ivory pill on ink (its own surface is
    // paper, so hover/press are the on-paper fill steps composited over
    // paper), translucent ink-fill pill on paper.
    let (rest_bg, rest_hover, rest_press, rest_label) = match s {
        Surface::Ink => (
            t.palette.paper,
            t.on_paper.fill_subtle.over(t.palette.paper),
            t.on_paper.fill.over(t.palette.paper),
            t.palette.ink,
        ),
        Surface::Paper => (
            t.on_paper.fill,
            t.on_paper.fill_strong,
            t.on_paper.track,
            t.on_paper.primary,
        ),
    };
    // `button::active`'s recipe: terracotta, ivory label, same on both
    // surfaces; hover/press layer the on-ink (ivory) fill steps over accent.
    let selected_bg = accent;
    let selected_hover = t.on_ink.fill_subtle.over(accent);
    let selected_press = t.on_ink.fill.over(accent);
    let selected_label = t.palette.paper;

    move |_, status| {
        let (background, label) = if is_selected {
            match status {
                button::Status::Active => (selected_bg, selected_label),
                button::Status::Hovered => (selected_hover, selected_label),
                button::Status::Pressed => (selected_press, selected_label),
                button::Status::Disabled => (on.fill_subtle, on.disabled),
            }
        } else {
            match status {
                button::Status::Active => (rest_bg, rest_label),
                button::Status::Hovered => (rest_hover, rest_label),
                button::Status::Pressed => (rest_press, rest_label),
                button::Status::Disabled => (on.fill_subtle, on.disabled),
            }
        };
        button::Style {
            background: Some(Background::Color(background.into_iced())),
            text_color: label.into_iced(),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: radius.into(),
            },
            ..button::Style::default()
        }
    }
}
