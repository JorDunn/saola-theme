//! Button styles: `rest`, `active`, `muted`, `bare`.
//!
//! The one rule, applied to buttons:
//!
//! - [`rest`] — a control at rest: a solid **ivory** pill on ink (ink label),
//!   an ink **fill** pill on paper (ink label).
//! - [`active`] — on / selected / live: a **terracotta** pill with an ivory
//!   label, identical on both surfaces.
//! - [`muted`] — muted / off-ish: a **subtle-fill** pill with a
//!   secondary-emphasis label, quieter than `rest`.
//! - [`bare`] — label only; hover/press surface it through the fill steps.
//!
//! There is deliberately no `danger` variant: Saola has three colors, never
//! a fourth. Destructive confirmation is a consumer *pattern* (wording,
//! placement, a second step), not a palette entry.
//!
//! ## How hover/press are derived
//!
//! State layering moves through the alpha fill steps, never new colors. A
//! button has a single flat `background`, so there are two cases:
//!
//! - The button's resting fill is **translucent** (`bare`, `rest` on paper):
//!   we just pick a deeper fill step and let iced blend it over whatever
//!   surface is behind the button.
//! - The button's resting fill is **opaque** (ivory `rest` on ink,
//!   terracotta `active`): the fill step must layer over the button's *own*
//!   fill, so we pre-composite in token space with
//!   `saola_tokens::Color::over` (opaque base ⇒ opaque result). An ivory
//!   pill is a tiny paper surface, so it hovers through the *on-paper*
//!   steps; a terracotta pill takes ivory content, so it hovers through the
//!   *on-ink* (ivory) steps.
//!
//! ## Focus
//!
//! iced 0.14's `button::Status` is exactly `Active | Hovered | Pressed |
//! Disabled` — there is no focus variant, so the 2 px terracotta focus ring
//! cannot be expressed here. Use [`crate::style::focus_border`] from
//! consumer code that tracks keyboard focus itself.

use iced::widget::button::{Status, Style};
use iced::{Background, Border, Color};
use saola_tokens::{Surface, Theme};

use crate::convert::ColorExt;

/// Assemble a pill-shaped [`Style`] from an optional background and a label
/// color. `..Style::default()` keeps iced's own defaults for shadow and
/// pixel snapping.
fn pill(background: Option<iced::Color>, text_color: iced::Color, radius: f32) -> Style {
    Style {
        background: background.map(Background::Color),
        text_color,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: radius.into(),
        },
        ..Style::default()
    }
}

/// A control at rest — off, unselected, available.
///
/// On ink: a solid ivory pill with an ink label. On paper: an ink-fill pill
/// with an ink label. Hover and press step through the fill roles.
pub fn rest(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style {
    // Copy the Copy token values out of the theme so the closure is 'static.
    let radius = t.radii.pill;
    let on = *t.on(s);
    let (rest_bg, hover_bg, press_bg, label) = match s {
        // Solid ivory pill: its own surface is paper, so hover/press are the
        // on-paper fill steps composited over paper (opaque results).
        Surface::Ink => (
            t.palette.paper,
            t.on_paper.fill_subtle.over(t.palette.paper),
            t.on_paper.fill.over(t.palette.paper),
            t.palette.ink,
        ),
        // Translucent ink-fill pill: iced blends deeper steps over the paper
        // window behind it. (On paper, `fill_strong` and `track` share a
        // value by construction, so press reads one step past hover only
        // where the tokens provide one.)
        Surface::Paper => (
            t.on_paper.fill,
            t.on_paper.fill_strong,
            t.on_paper.track,
            t.on_paper.primary,
        ),
    };
    move |_, status| match status {
        Status::Active => pill(Some(rest_bg.into_iced()), label.into_iced(), radius),
        Status::Hovered => pill(Some(hover_bg.into_iced()), label.into_iced(), radius),
        Status::Pressed => pill(Some(press_bg.into_iced()), label.into_iced(), radius),
        Status::Disabled => pill(
            Some(on.fill_subtle.into_iced()),
            on.disabled.into_iced(),
            radius,
        ),
    }
}

/// On, selected, live — a terracotta pill with an ivory label, the same on
/// both surfaces. The surface only decides the disabled treatment.
pub fn active(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style {
    let radius = t.radii.pill;
    let on = *t.on(s);
    let accent = t.palette.accent;
    let label = t.palette.paper;
    // Terracotta takes ivory content, so its hover/press layer the ivory
    // (on-ink) fill steps over the accent (opaque results).
    let hover_bg = t.on_ink.fill_subtle.over(accent);
    let press_bg = t.on_ink.fill.over(accent);
    move |_, status| match status {
        Status::Active => pill(Some(accent.into_iced()), label.into_iced(), radius),
        Status::Hovered => pill(Some(hover_bg.into_iced()), label.into_iced(), radius),
        Status::Pressed => pill(Some(press_bg.into_iced()), label.into_iced(), radius),
        Status::Disabled => pill(
            Some(on.fill_subtle.into_iced()),
            on.disabled.into_iced(),
            radius,
        ),
    }
}

/// Muted / off-ish — a quiet pill for states like volume muted or Wi-Fi
/// offline: same geometry as [`rest`], but a subtle translucent fill with a
/// secondary-emphasis label. Hover and press step the fill deeper
/// (`fill_subtle → fill → fill_strong`); every fill here is translucent, so
/// iced blends it over whatever surface is behind the button.
pub fn muted(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style {
    let radius = t.radii.pill;
    let on = *t.on(s);
    move |_, status| match status {
        Status::Active => pill(
            Some(on.fill_subtle.into_iced()),
            on.secondary.into_iced(),
            radius,
        ),
        Status::Hovered => pill(Some(on.fill.into_iced()), on.secondary.into_iced(), radius),
        Status::Pressed => pill(
            Some(on.fill_strong.into_iced()),
            on.secondary.into_iced(),
            radius,
        ),
        Status::Disabled => pill(
            Some(on.fill_subtle.into_iced()),
            on.disabled.into_iced(),
            radius,
        ),
    }
}

/// A label-only button: transparent at rest, surfacing through
/// `fill_subtle` → `fill` on hover/press.
pub fn bare(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style {
    let radius = t.radii.pill;
    let on = *t.on(s);
    move |_, status| match status {
        Status::Active => pill(None, on.primary.into_iced(), radius),
        Status::Hovered => pill(
            Some(on.fill_subtle.into_iced()),
            on.primary.into_iced(),
            radius,
        ),
        Status::Pressed => pill(Some(on.fill.into_iced()), on.primary.into_iced(), radius),
        Status::Disabled => pill(None, on.disabled.into_iced(), radius),
    }
}
