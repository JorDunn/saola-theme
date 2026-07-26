//! Text input style: [`rest`].
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
use saola_tokens::{Surface, Theme};

use crate::convert::ColorExt;

/// A text field at rest — the only look a Saola text input has (there is no
/// "active/selected" text input, unlike buttons and toggles).
pub fn rest(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style {
    let radius = t.radii.pill;
    let on = *t.on(s);
    let accent = t.palette.accent.into_iced();

    // Same resting fill as `button::rest`: opaque ivory on ink, translucent
    // ink-fill on paper.
    let background = match s {
        Surface::Ink => t.palette.paper,
        Surface::Paper => t.on_paper.fill,
    }
    .into_iced();
    let value = match s {
        Surface::Ink => t.palette.ink,
        Surface::Paper => on.primary,
    }
    .into_iced();

    let icon = on.secondary.into_iced();
    let placeholder = on.quaternary.into_iced();
    let divider = on.divider.into_iced();
    let disabled_bg = on.fill_subtle.into_iced();
    let disabled_text = on.disabled.into_iced();

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
