//! Text input styles: [`rest`], [`rejected`], [`prompt`], and
//! [`prompt_rejected`].
//!
//! A text input is a control at rest, so it follows the same "ivory on ink /
//! fill on paper" shape as [`crate::style::button::rest`]: a solid ivory
//! field on an ink surface, a translucent ink-fill field on a paper window.
//! Unlike a button it has a real focus state (`text_input::Status::Focused`
//! *does* carry a payload in iced 0.14, unlike `button::Status`), so this is
//! the first style module that draws the `sizes.ring` terracotta ring inline
//! instead of leaving it to [`crate::style::focus_border`].
//!
//! Hover only nudges the border (a field's fill doesn't need to visibly
//! "press"); focus is what earns the accent ring. Disabled follows the same
//! `on(s).fill_subtle` background / `on(s).disabled` text rule as buttons.
//!
//! [`prompt`]/[`prompt_rejected`] are the *quiet* pair: the same borders and
//! states over a translucent `fill_subtle` field instead of a solid control
//! fill — for fields that sit on a scrim (the lock screen's password
//! prompt) rather than in a form.

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
pub fn rest(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    let radius = t.radii.pill;
    let f = fill(t, s);
    let ring_width = t.sizes.ring;
    let hairline = t.sizes.hairline;
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
                border: border(divider, hairline),
                icon,
                placeholder,
                value,
                selection: accent,
            },
            // Where the accent ring is drawn inline rather than via
            // `focus_border`: `text_input::Status` carries a real focus
            // state.
            Status::Focused { .. } => Style {
                background: Background::Color(background),
                border: border(accent, ring_width),
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
pub fn rejected(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    let radius = t.radii.pill;
    let f = fill(t, s);
    let ring_width = t.sizes.ring;
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
            width: ring_width,
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

/// The quiet prompt fill shared by [`prompt`] and [`prompt_rejected`]: the
/// surface's own `fill_subtle` with `primary` value text — a translucent
/// recess in the scrim, not the solid control fill [`fill`] gives `rest`.
/// The disabled background *is* the resting background: see [`prompt`]'s
/// no-jump rule.
fn prompt_fill(t: &Theme, s: Surface) -> Fill {
    let on = *t.on(s);
    Fill {
        background: on.fill_subtle,
        value: on.primary,
        icon: on.secondary,
        placeholder: on.quaternary,
        disabled_bg: on.fill_subtle,
        disabled_text: on.disabled,
    }
}

/// A prompt field — the lock screen's password input (style guide §7), and
/// any other lone field sitting on a scrim rather than in a form. Same
/// pill, borders, and state progression as [`rest`], over a *quiet* fill:
/// the surface's `fill_subtle` with `primary` value text, instead of
/// `rest`'s opaque control fill (a prompt on a scrim is a recess to type
/// into, not an ivory control). Promoted from saola-lockscreen's
/// `reveal::field_style`, which documented itself as exactly this
/// substitution.
///
/// The style closure has no height to set (that's the consumer's `.height`
/// call on the `text_input` widget itself) — the intended field height for
/// this look is `sizes.field_lock` (62 px, the spec's 60–64 range;
/// saola-lockscreen's shipped field is 68 px and should adopt this token).
///
/// **The no-jump disabled rule**: `Status::Disabled` keeps the resting
/// fill — only the text drops to the disabled step. The lock screen
/// disables the field while PAM verifies a password, and the field visibly
/// jumping to a different fill on every submit would read as flicker, not
/// state.
pub fn prompt(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    let radius = t.radii.pill;
    let f = prompt_fill(t, s);
    let ring_width = t.sizes.ring;
    let hairline = t.sizes.hairline;
    let accent = t.palette.accent.into_iced();
    let background = f.background.into_iced();
    let value = f.value.into_iced();
    let icon = f.icon.into_iced();
    let placeholder = f.placeholder.into_iced();
    let divider = t.on(s).divider.into_iced();
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
                border: border(divider, hairline),
                icon,
                placeholder,
                value,
                selection: accent,
            },
            Status::Focused { .. } => Style {
                background: Background::Color(background),
                border: border(accent, ring_width),
                icon,
                placeholder,
                value,
                selection: accent,
            },
            // The no-jump rule: the resting fill stays put while the
            // consumer works (PAM, a lookup); only the text dims.
            Status::Disabled => Style {
                background: Background::Color(background),
                border: border(Color::TRANSPARENT, 0.0),
                icon: disabled_text,
                placeholder: disabled_text,
                value: disabled_text,
                selection: accent,
            },
        }
    }
}

/// A rejected prompt field — [`prompt`] after a wrong password (concept
/// 5c). Exactly [`rejected`]'s relationship to [`rest`]: same fill and
/// geometry as [`prompt`], with the `sizes.ring` border drawn in every
/// interactive state and tinted with the surface's accent *text* color
/// (`accent_light` on ink, `accent_dark` on paper — a tint, not a fourth
/// color), so the field keeps saying "that was wrong" while the user
/// retypes, even while focused. `Status::Disabled` keeps [`prompt`]'s
/// no-jump fill and drops the ring — resubmitting is the moment the
/// verdict is void.
pub fn prompt_rejected(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    let radius = t.radii.pill;
    let f = prompt_fill(t, s);
    let ring_width = t.sizes.ring;
    let accent = t.palette.accent.into_iced();
    let background = f.background.into_iced();
    let value = f.value.into_iced();
    let icon = f.icon.into_iced();
    let placeholder = f.placeholder.into_iced();
    let disabled_text = f.disabled_text.into_iced();
    // The same surface-appropriate tint `rejected` uses (§1: accent_light
    // is accent text on ink only; accent_dark on paper only).
    let tint = match s {
        Surface::Ink => t.palette.accent_light,
        Surface::Paper => t.palette.accent_dark,
    }
    .into_iced();

    move |_, status| {
        let ring = Border {
            color: tint,
            width: ring_width,
            radius: radius.into(),
        };
        match status {
            Status::Active | Status::Hovered | Status::Focused { .. } => Style {
                background: Background::Color(background),
                border: ring,
                icon,
                placeholder,
                value,
                selection: accent,
            },
            Status::Disabled => Style {
                background: Background::Color(background),
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
