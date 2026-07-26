//! Text input styles: [`rest`] and [`rejected`].
//!
//! A text input is a control at rest, so it follows the same "ivory on ink /
//! fill on paper" shape as [`crate::style::button::rest`]: a solid ivory
//! field on an ink surface, a translucent ink-fill field on a paper window.
//! Unlike a button it has a real focus state (`text_input::Status::Focused`
//! *does* carry a payload in iced 0.14, unlike `button::Status`), so this is
//! the first style module that draws the 2 px terracotta ring inline instead
//! of leaving it to [`crate::style::focus_border`].
//!
//! Hover only nudges the border (a field's fill doesn't need to visibly
//! "press"); focus is what earns the accent ring. Disabled follows the same
//! `on(s).fill_subtle` background / `on(s).disabled` text rule as buttons.

use iced::widget::text_input::{Status, Style};
use iced::{Background, Border, Color};
use saola_tokens::{Color as TokenColor, Surface, Theme};

use crate::convert::ColorExt;

/// The fill/value/icon/placeholder recipe shared by [`rest`] and
/// [`rejected`] — both use the exact same "ivory on ink / fill on paper"
/// resting look, they only differ in what's drawn for the border.
struct Fill {
    background: TokenColor,
    value: TokenColor,
    icon: TokenColor,
    placeholder: TokenColor,
    disabled_bg: TokenColor,
    disabled_text: TokenColor,
}

fn fill(t: &Theme, s: Surface) -> Fill {
    let on = *t.on(s);
    Fill {
        // Same resting fill as `button::rest`: opaque ivory on ink,
        // translucent ink-fill on paper.
        background: match s {
            Surface::Ink => t.palette.paper,
            Surface::Paper => t.on_paper.fill,
        },
        value: match s {
            Surface::Ink => t.palette.ink,
            Surface::Paper => on.primary,
        },
        icon: on.secondary,
        placeholder: on.quaternary,
        disabled_bg: on.fill_subtle,
        disabled_text: on.disabled,
    }
}

/// A text field at rest — the default look a Saola text input has (there is
/// no "active/selected" text input, unlike buttons and toggles).
pub fn rest(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style {
    let radius = t.radii.pill;
    let f = fill(t, s);
    let accent = t.palette.accent.into_iced();
    let background = f.background.into_iced();
    let value = f.value.into_iced();
    let icon = f.icon.into_iced();
    let placeholder = f.placeholder.into_iced();
    let divider = t.on(s).divider.into_iced();
    let disabled_bg = f.disabled_bg.into_iced();
    let disabled_text = f.disabled_text.into_iced();

    move |_, status| {
        let border = |color: Color, width: f32| Border {
            color,
            width,
            radius: radius.into(),
        };
        match status {
            Status::Active => Style {
                background: Background::Color(background),
                border: border(Color::TRANSPARENT, 0.0),
                icon,
                placeholder,
                value,
                selection: accent,
            },
            Status::Hovered => Style {
                background: Background::Color(background),
                border: border(divider, 1.0),
                icon,
                placeholder,
                value,
                selection: accent,
            },
            // The one place in this crate the 2px accent ring is drawn
            // inline: `text_input::Status` carries a real focus state.
            Status::Focused { .. } => Style {
                background: Background::Color(background),
                border: border(accent, 2.0),
                icon,
                placeholder,
                value,
                selection: accent,
            },
            Status::Disabled => Style {
                background: Background::Color(disabled_bg),
                border: border(Color::TRANSPARENT, 0.0),
                icon: disabled_text,
                placeholder: disabled_text,
                value: disabled_text,
                selection: accent,
            },
        }
    }
}

/// A rejected text field — the lock screen's third field state (concept
/// 5c: "wrong password", after `rest` and `Status::Focused`). Same fills
/// and geometry as [`rest`]; the difference is entirely in the border: the
/// 2 px ring is drawn in *every* state (not just `Focused`), tinted with the
/// accent **text** color for the surface (`accent_light` on ink,
/// `accent_dark` on paper) rather than the pure-terracotta focus ring — a
/// tint, not a fourth color, and visibly distinct from `rest`'s focus ring
/// even while this field is also focused. Hint text ("Wrong password…") is
/// the consumer's job; this helper only owns the field chrome.
pub fn rejected(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style {
    let radius = t.radii.pill;
    let f = fill(t, s);
    let accent = t.palette.accent.into_iced();
    let background = f.background.into_iced();
    let value = f.value.into_iced();
    let icon = f.icon.into_iced();
    let placeholder = f.placeholder.into_iced();
    let disabled_bg = f.disabled_bg.into_iced();
    let disabled_text = f.disabled_text.into_iced();
    // The tint: accent-colored text for this surface, per the Architecture
    // mapping (`accent_light` is accent text on ink only, `accent_dark` is
    // accent text on paper only) — never the raw `palette.accent` used by
    // the plain focus ring.
    let tint = match s {
        Surface::Ink => t.palette.accent_light,
        Surface::Paper => t.palette.accent_dark,
    }
    .into_iced();

    move |_, status| {
        let ring = Border {
            color: tint,
            width: 2.0,
            radius: radius.into(),
        };
        match status {
            Status::Active => Style {
                background: Background::Color(background),
                border: ring,
                icon,
                placeholder,
                value,
                selection: accent,
            },
            Status::Hovered => Style {
                background: Background::Color(background),
                border: ring,
                icon,
                placeholder,
                value,
                selection: accent,
            },
            // The tint persists through focus too — that's what keeps this
            // state legible as "rejected" rather than collapsing back into
            // an ordinary focused `rest` field.
            Status::Focused { .. } => Style {
                background: Background::Color(background),
                border: ring,
                icon,
                placeholder,
                value,
                selection: accent,
            },
            Status::Disabled => Style {
                background: Background::Color(disabled_bg),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: radius.into(),
                },
                icon: disabled_text,
                placeholder: disabled_text,
                value: disabled_text,
                selection: accent,
            },
        }
    }
}
