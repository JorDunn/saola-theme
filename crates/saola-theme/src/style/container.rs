//! Container styles: the surfaces everything else sits on.
//!
//! `container::Style` in iced 0.14 has no `Status` — a container is never
//! hovered or pressed — so these closures take only `&iced::Theme`.
//! Every helper sets `text_color`, which iced propagates as the default
//! text color for the container's descendants: put content on an ink
//! surface and it comes out ivory without further ceremony.

use iced::widget::container::Style;
use iced::{Background, Border, Color};
use saola_tokens::{Surface, Theme};

use crate::convert::{ColorExt, ShadowExt};

/// A borderless rounded rectangle with a background and inherited text color.
fn surface(background: iced::Color, text: iced::Color, radius: f32) -> Style {
    Style {
        text_color: Some(text),
        background: Some(Background::Color(background)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: radius.into(),
        },
        ..Style::default()
    }
}

/// A shell surface: solid ink, edge to edge (no rounding), ivory text.
pub fn ink_surface(t: &Theme) -> impl Fn(&iced::Theme) -> Style {
    let ink = t.palette.ink.into_iced();
    let text = t.on_ink.primary.into_iced();
    move |_| surface(ink, text, 0.0)
}

/// A light application window: solid paper at the window radius, with the
/// 2 px ink window border and the window shadow. Ink text.
pub fn paper_window(t: &Theme) -> impl Fn(&iced::Theme) -> Style {
    let paper = t.palette.paper.into_iced();
    let text = t.on_paper.primary.into_iced();
    let ink = t.palette.ink.into_iced();
    let radius = t.radii.window;
    let border_width = t.sizes.window_border;
    let shadow = t.shadows.window.into_iced();
    move |_| Style {
        border: Border {
            color: ink,
            width: border_width,
            radius: radius.into(),
        },
        shadow,
        ..surface(paper, text, radius)
    }
}

/// A card at the card radius. On ink it is a solid ivory card (ink text,
/// popover shadow — a notification card floating on the shell); on paper it
/// is a subtle ink-fill inset of the window (no shadow).
pub fn card(t: &Theme, s: Surface) -> impl Fn(&iced::Theme) -> Style {
    let radius = t.radii.card;
    let text = t.on_paper.primary.into_iced();
    let (background, shadow) = match s {
        Surface::Ink => (
            t.palette.paper.into_iced(),
            Some(t.shadows.popover.into_iced()),
        ),
        Surface::Paper => (t.on_paper.fill_subtle.into_iced(), None),
    };
    move |_| Style {
        shadow: shadow.unwrap_or_default(),
        ..surface(background, text, radius)
    }
}

/// The translucent panel scrim as a pill — the bar's islands. The wallpaper
/// shows through; text is ivory (the scrim is ink-tinted).
pub fn translucent_panel(t: &Theme) -> impl Fn(&iced::Theme) -> Style {
    let scrim = t.scrim.translucent_panel.into_iced();
    let text = t.on_ink.primary.into_iced();
    let radius = t.radii.pill;
    move |_| surface(scrim, text, radius)
}

/// A tooltip: solid ink at the tile radius with ivory text and the popover
/// shadow — readable on either surface.
pub fn tooltip(t: &Theme) -> impl Fn(&iced::Theme) -> Style {
    let ink = t.palette.ink.into_iced();
    let text = t.on_ink.primary.into_iced();
    let radius = t.radii.tile;
    let shadow = t.shadows.popover.into_iced();
    move |_| Style {
        shadow,
        ..surface(ink, text, radius)
    }
}
