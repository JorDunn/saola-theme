//! Notification toast internals (style guide §6): the two pieces
//! [`crate::style::container::notification_card`] doesn't style — the
//! bottom-edge lifetime countdown ([`life_rule`]) and the leading icon tile
//! ([`icon_tile`], or [`icon_tile_colored`] when the tile *is* the payload —
//! a colour-picker swatch — rather than a recess for a glyph).
//!
//! Both are ink-only, no `Surface` parameter — like
//! [`notification_card`](crate::style::container::notification_card), a
//! toast lives on the shell layer and is always drawn on ink regardless of
//! what surface the rest of a consumer's UI is in.
//!
//! Both also take `alpha`, for the same reason
//! [`notification_card`](crate::style::container::notification_card) does:
//! iced 0.14 has no subtree opacity, so a toast fading in or out has to
//! scale the alpha of every color it paints — the track and bar, the tile
//! background and text — not just the card behind them.

use iced::widget::{container, progress_bar};
use iced::{Background, Border, Color};
use saola_tokens::Theme;

use crate::convert::ColorExt;

/// The card's bottom-edge lifetime rule: styles an iced `progress_bar` as a
/// `sizes.life_rule` (3 px) straight rule — `on_ink.fill_subtle` track,
/// terracotta bar, square corners (a rule, not a pill, matching
/// [`crate::style::rule::rest`]'s shape rather than [`super::progress::bar`]'s
/// pill).
///
/// A style closure can't express a *shrinking fraction* — that's the
/// `progress_bar` widget's own `value`, not a `Style` field — so pair this
/// with [`crate::motion::life_fraction`] as the value, and set the girth
/// from `sizes.life_rule` yourself:
///
/// ```no_run
/// use saola_theme::{style, Theme};
/// use std::time::Duration;
///
/// let t = Theme::saola();
/// let elapsed = Duration::from_millis(400);
/// let value = saola_theme::motion::life_fraction(&t, elapsed);
/// let _rule: iced::Element<'_, ()> = iced::widget::progress_bar(0.0..=1.0, value)
///     .length(iced::Fill)
///     .girth(t.sizes.life_rule)
///     .style(style::notification::life_rule(&t, 1.0))
///     .into();
/// ```
///
/// The urgent variant ([`crate::style::container::card_urgent`]) never gets
/// this rule at all (style guide 10b: "a terracotta ring and no life rule")
/// — don't compose the two.
///
/// `alpha` is clamped to `0.0..=1.0` (a non-finite value reads as `1.0`),
/// same as [`crate::style::container::notification_card`]'s `alpha`, and
/// scales both the track and the bar — the card behind this rule fades, so
/// the rule drawn on top of it has to fade at the same rate or it would
/// visibly outlive the card.
pub fn life_rule(t: &Theme, alpha: f32) -> impl Fn(&iced::Theme) -> progress_bar::Style + Clone {
    let alpha = if alpha.is_finite() {
        alpha.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let track = t.on_ink.fill_subtle.with_opacity(alpha);
    let accent = t.palette.accent.with_opacity(alpha);

    move |_| progress_bar::Style {
        background: Background::Color(track),
        bar: Background::Color(accent),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
    }
}

/// The toast's leading icon tile: a `sizes.icon_tile` (36 px) square at the
/// tile-tier radius (`radii.tile`), `on_ink.fill_subtle` fill — the same
/// recessed-fill recipe as [`crate::style::container::tile`], named for the
/// toast's leading icon rather than a general recess.
///
/// The square footprint is the caller's `.width(t.sizes.icon_tile)`
/// `.height(t.sizes.icon_tile)` (a container style closure can't set
/// geometry, same boundary [`life_rule`] hits above); the icon's own tint is
/// the caller's job too, same constraint every icon-bearing constructor in
/// [`crate::widget`] documents (an `Svg`'s color can't ride a container
/// style).
///
/// `alpha` is clamped to `0.0..=1.0` (a non-finite value reads as `1.0`),
/// same as [`life_rule`]'s — it fades both the tile's background and its
/// `text_color` (the color an icon glyph inherits when it doesn't set its
/// own), so the tile fades at the same rate as the card it sits on.
pub fn icon_tile(t: &Theme, alpha: f32) -> impl Fn(&iced::Theme) -> container::Style + Clone {
    let alpha = if alpha.is_finite() {
        alpha.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let background = t.on_ink.fill_subtle.with_opacity(alpha);
    let text = t.on_ink.primary.with_opacity(alpha);
    let radius = t.radii.tile;

    move |_| container::Style {
        text_color: Some(text),
        background: Some(Background::Color(background)),
        border: super::border_none(radius),
        ..container::Style::default()
    }
}

/// [`icon_tile`] painted with a caller-supplied `fill` instead of the
/// recessed `on_ink.fill_subtle` — for the one toast whose tile is a
/// *swatch*, not a recess behind a glyph: saola-capture's colour-picker
/// toast shows the picked pixel as the tile itself.
///
/// `fill` is in token space (`saola_tokens::Color`, 8-bit channels) so a
/// picked pixel maps straight onto it (`Color { r, g, b, a: 255 }`), and so
/// the fade below composes in the same space as every other alpha step in
/// this module. Any RGB is accepted — a swatch is the exception the design
/// language carves out for the *content* a toast reports on; it does not
/// make a fourth palette colour available to controls.
///
/// `text_color` follows the one rule (text is the opposite of its fill):
/// it is whichever of `palette.ink` / `palette.paper` has the higher WCAG
/// contrast against `fill`, so a glyph that inherits the tile's text colour
/// stays legible on a light pick and a dark pick alike. Same geometry and
/// radius as [`icon_tile`]; `alpha` is clamped the same way and scales both
/// the swatch and its text colour.
pub fn icon_tile_colored(
    t: &Theme,
    fill: saola_tokens::Color,
    alpha: f32,
) -> impl Fn(&iced::Theme) -> container::Style + Clone {
    let alpha = if alpha.is_finite() {
        alpha.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let background = fill.with_opacity(alpha);
    let text = text_on(t, fill).with_opacity(alpha);
    let radius = t.radii.tile;

    move |_| container::Style {
        text_color: Some(text),
        background: Some(Background::Color(background)),
        border: super::border_none(radius),
        ..container::Style::default()
    }
}

/// Ink or paper, whichever reads better on `fill` — the WCAG contrast
/// ratio `(L_lighter + 0.05) / (L_darker + 0.05)` computed against both
/// identity colours, higher wins. Ties go to ink (a control at rest takes
/// ink text).
fn text_on(t: &Theme, fill: saola_tokens::Color) -> saola_tokens::Color {
    let contrast = |a: saola_tokens::Color, b: saola_tokens::Color| {
        let (la, lb) = (a.relative_luminance(), b.relative_luminance());
        (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
    };
    let ink = t.palette.ink;
    let paper = t.palette.paper;
    if contrast(paper, fill) > contrast(ink, fill) {
        paper
    } else {
        ink
    }
}

#[cfg(test)]
mod tests {
    use iced::Background;
    use saola_tokens::{Color, Theme};

    use super::*;

    fn style(fill: Color, alpha: f32) -> container::Style {
        let t = Theme::saola();
        icon_tile_colored(&t, fill, alpha)(&iced::Theme::Dark)
    }

    fn background(style: &container::Style) -> iced::Color {
        match style.background {
            Some(Background::Color(c)) => c,
            other => panic!("expected a flat colour background, got {other:?}"),
        }
    }

    #[test]
    fn colored_tile_paints_the_supplied_fill() {
        let picked = Color {
            r: 40,
            g: 120,
            b: 200,
            a: 255,
        };
        let s = style(picked, 1.0);
        assert_eq!(background(&s), picked.into_iced());
    }

    #[test]
    fn colored_tile_text_is_ink_on_a_light_pick_and_paper_on_a_dark_one() {
        let t = Theme::saola();
        let light = Color {
            r: 250,
            g: 240,
            b: 200,
            a: 255,
        };
        let dark = Color {
            r: 30,
            g: 20,
            b: 60,
            a: 255,
        };
        assert_eq!(text_on(&t, light), t.palette.ink);
        assert_eq!(text_on(&t, dark), t.palette.paper);
        // The identity colours themselves resolve to their opposite.
        assert_eq!(text_on(&t, t.palette.paper), t.palette.ink);
        assert_eq!(text_on(&t, t.palette.ink), t.palette.paper);
    }

    #[test]
    fn colored_tile_alpha_scales_fill_and_text_and_is_clamped() {
        let picked = Color {
            r: 200,
            g: 100,
            b: 50,
            a: 255,
        };
        let half = style(picked, 0.5);
        assert!((background(&half).a - 0.5).abs() < 1e-6);
        assert!((half.text_color.unwrap().a - 0.5).abs() < 1e-6);

        let over = style(picked, 7.0);
        assert!((background(&over).a - 1.0).abs() < 1e-6);
        let under = style(picked, -3.0);
        assert!(background(&under).a.abs() < 1e-6);
        let nan = style(picked, f32::NAN);
        assert!((background(&nan).a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn colored_tile_matches_icon_tile_geometry() {
        let t = Theme::saola();
        let plain = icon_tile(&t, 1.0)(&iced::Theme::Dark);
        let s = style(t.palette.accent, 1.0);
        assert_eq!(s.border.radius, plain.border.radius);
        assert_eq!(s.border.width, plain.border.width);
    }
}
