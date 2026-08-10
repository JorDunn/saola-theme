//! Notification toast internals (style guide §6): the two pieces
//! [`crate::style::container::notification_card`] doesn't style — the
//! bottom-edge lifetime countdown ([`life_rule`]) and the leading icon tile
//! ([`icon_tile`]).
//!
//! Both are ink-only, no `Surface` parameter — like
//! [`notification_card`](crate::style::container::notification_card), a
//! toast lives on the shell layer and is always drawn on ink regardless of
//! what surface the rest of a consumer's UI is in.

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
///     .style(style::notification::life_rule(&t))
///     .into();
/// ```
///
/// The urgent variant ([`crate::style::container::card_urgent`]) never gets
/// this rule at all (style guide 10b: "a terracotta ring and no life rule")
/// — don't compose the two.
pub fn life_rule(t: &Theme) -> impl Fn(&iced::Theme) -> progress_bar::Style + Clone {
    let track = t.on_ink.fill_subtle.into_iced();
    let accent = t.palette.accent.into_iced();

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
pub fn icon_tile(t: &Theme) -> impl Fn(&iced::Theme) -> container::Style + Clone {
    let background = t.on_ink.fill_subtle.into_iced();
    let text = t.on_ink.primary.into_iced();
    let radius = t.radii.tile;

    move |_| container::Style {
        text_color: Some(text),
        background: Some(Background::Color(background)),
        border: super::border_none(radius),
        ..container::Style::default()
    }
}
