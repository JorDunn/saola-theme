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

/// The floating ledger bar: one solid-ink pill inset from the screen edge
/// (`sizes.panel_margin_ledger` / `panel_margin_ledger_top` — the concept's
/// ledger is a rounded pill, not an edge-to-edge strip), ivory text, no
/// border or shadow. Ink-only shell chrome like [`translucent_panel`]
/// (which is its translucent islands counterpart).
pub fn bar_pill(t: &Theme) -> impl Fn(&iced::Theme) -> Style {
    let ink = t.palette.ink.into_iced();
    let text = t.on_ink.primary.into_iced();
    let radius = t.radii.pill;
    move |_| surface(ink, text, radius)
}

/// A popover: opaque ink at the popover radius with ivory text and the
/// popover shadow. Popovers are shell chrome, so like [`ink_surface`] and
/// [`translucent_panel`] this is ink-only — there is no paper popover.
pub fn popover(t: &Theme) -> impl Fn(&iced::Theme) -> Style {
    let ink = t.palette.ink.into_iced();
    let text = t.on_ink.primary.into_iced();
    let radius = t.radii.popover;
    let shadow = t.shadows.popover.into_iced();
    move |_| Style {
        shadow,
        ..surface(ink, text, radius)
    }
}

/// The urgent notification card (concept 10b): [`card`] plus a 2 px accent
/// ring — "a terracotta ring and no life rule" (no fourth color, no
/// vibrating/pulsing animation; the ring alone is what signals urgency).
pub fn card_urgent(t: &Theme, s: Surface) -> impl Fn(&iced::Theme) -> Style {
    let radius = t.radii.card;
    let text = t.on_paper.primary.into_iced();
    let accent = t.palette.accent.into_iced();
    let (background, shadow) = match s {
        Surface::Ink => (
            t.palette.paper.into_iced(),
            Some(t.shadows.popover.into_iced()),
        ),
        Surface::Paper => (t.on_paper.fill_subtle.into_iced(), None),
    };
    move |_| Style {
        border: Border {
            color: accent,
            width: 2.0,
            radius: radius.into(),
        },
        shadow: shadow.unwrap_or_default(),
        ..surface(background, text, radius)
    }
}

/// A keycap chip — the `↵`/`⇥` hints shown on nearly every concept screen.
/// A small pill/tile at `radii.selection`: `fill_subtle` background with a
/// slightly stronger `fill` outline, reading as a quiet, key-like edge
/// rather than competing with surrounding content. The mono font at
/// `size.keycap` is the consumer's job (this helper only owns the chip's
/// chrome, not the glyph inside it).
pub fn keycap(t: &Theme, s: Surface) -> impl Fn(&iced::Theme) -> Style {
    let radius = t.radii.selection;
    let on = *t.on(s);
    move |_| Style {
        text_color: Some(on.secondary.into_iced()),
        background: Some(Background::Color(on.fill_subtle.into_iced())),
        border: Border {
            color: on.fill.into_iced(),
            width: 1.0,
            radius: radius.into(),
        },
        ..Style::default()
    }
}

/// A badge — unread counts and similar tiny indicators. An accent pill with
/// ivory text, identical on both surfaces (the same "terracotta = live"
/// recipe as [`crate::style::button::active`]), so this takes only the
/// theme.
pub fn badge(t: &Theme) -> impl Fn(&iced::Theme) -> Style {
    let radius = t.radii.pill;
    let accent = t.palette.accent.into_iced();
    let text = t.palette.paper.into_iced();
    move |_| surface(accent, text, radius)
}

/// The state of one minimap dash. The panel's centre module draws the niri
/// column strip as one dash per column; exactly one is [`DashState::Focused`]
/// — the one live element, per the one rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashState {
    /// An on-screen, unfocused column.
    Rest,
    /// The focused column — terracotta, and widened by the consumer
    /// (`sizes.dash_width_focused`).
    Focused,
    /// An off-screen column, shown as a stub at either end of the strip.
    Stub,
}

/// One minimap dash: a tiny pill sitting directly on the bar. Like the
/// bare-icon menu, rest and stub dashes are ivory stepped with alpha
/// (tertiary / quaternary emphasis); the focused dash is full terracotta.
/// The bar is a shell surface, so this is ink-only.
///
/// Geometry — height, per-state widths, and the gap between dashes — comes
/// from the `sizes.dash_*` tokens; the consumer sets them on the container.
/// A dash carries no text, so no `text_color` is set.
pub fn dash(t: &Theme, state: DashState) -> impl Fn(&iced::Theme) -> Style {
    let fill = match state {
        DashState::Rest => t.on_ink.tertiary.into_iced(),
        DashState::Focused => t.palette.accent.into_iced(),
        DashState::Stub => t.on_ink.quaternary.into_iced(),
    };
    let radius = t.radii.pill;
    move |_| Style {
        background: Some(Background::Color(fill)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: radius.into(),
        },
        ..Style::default()
    }
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
