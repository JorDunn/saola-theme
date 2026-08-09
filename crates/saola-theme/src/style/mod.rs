//! Style helpers — the heart of the crate.
//!
//! Every helper takes `(&saola_tokens::Theme, Surface)` (or just the theme,
//! when the look is surface-independent), copies the handful of `Copy`
//! token values it needs, and returns a `'static` closure in the exact shape
//! iced's `.style(...)` methods want. Copying up front is what makes the
//! closure `'static`: it owns plain colors and numbers instead of borrowing
//! the theme, so the returned widget can outlive the `&Theme` borrow.
//!
//! The design language in one sentence: **ivory fill = a control at rest
//! (ink text on it); terracotta fill = on/selected/live (ivory text on it);
//! hover and press move through the alpha fill steps, never new colors;
//! everything is a pill or an over-rounded rectangle; keyboard focus is a
//! 2 px terracotta ring.**
//!
//! Every helper's return type is `impl Fn(...) -> Style + Clone`: the
//! closures capture only `Copy` token values, so the `Clone` bound is free —
//! and without it, a consumer building a list could not reuse one closure
//! across rows (`.style(...)` takes it by value), forcing a rebuild per row
//! (saola-panel documented that cost three separate times before the bound
//! existed).

pub mod button;
pub mod container;
pub mod pick_list;
pub mod progress;
pub mod radio;
pub mod rule;
pub mod scrollable;
pub mod segmented;
pub mod slider;
pub mod text_input;
pub mod toggles;

use crate::convert::ColorExt;
use saola_tokens::Theme;

/// The transparent no-op border every borderless Saola shape carries: iced
/// styles have no "no border" — the closest thing is a zero-width
/// transparent border that still owns the shape's corner `radius`. This
/// eight-line literal was re-derived in at least three consumers (and by
/// the private `pill` assembler in [`button`]) before it was a helper.
pub fn border_none(radius: f32) -> iced::Border {
    iced::Border {
        color: iced::Color::TRANSPARENT,
        width: 0.0,
        radius: radius.into(),
    }
}

/// The accent ring: a `sizes.ring` (2 px) terracotta border at the given
/// radius — the one emphasis stroke in the design language, whether it
/// means keyboard focus ([`focus_border`]), urgency
/// ([`container::card_urgent`]), or needs-attention (the panel's tray badge
/// ring). Consumers that need the ring as a raw [`iced::Border`] (e.g. to
/// place on a widget whose style helper can't know about the state) take it
/// from here instead of re-deriving `accent` + a hardcoded `2.0`.
pub fn accent_ring(theme: &Theme, radius: f32) -> iced::Border {
    iced::Border {
        color: theme.palette.accent.into_iced(),
        width: theme.sizes.ring,
        radius: radius.into(),
    }
}

/// The keyboard-focus ring: a `sizes.ring` (2 px) terracotta border at the
/// given radius. Never a platform default.
///
/// This is [`accent_ring`] under its keyboard-focus name: iced 0.14's
/// per-widget `Status` enums do not (yet) carry a focus state for buttons,
/// so this is a building block for consumers that track focus themselves
/// (and for the widgets whose `Status` *does* include focus, like
/// `text_input`, in later style modules).
pub fn focus_border(theme: &Theme, radius: f32) -> iced::Border {
    accent_ring(theme, radius)
}

#[cfg(test)]
mod tests {
    use saola_tokens::{Surface, Theme};

    /// The whole point of the `+ Clone` bound on every helper's return
    /// type: one closure, built once, reused across a list of rows. This
    /// compiles only while the bound holds, and exercises one helper of
    /// each closure shape (with-`Status` and without).
    #[test]
    fn style_closures_are_clone() {
        let t = Theme::saola();

        let row = super::button::list_row(&t, Surface::Paper, false, false);
        let clones = [row.clone(), row.clone(), row];
        assert_eq!(clones.len(), 3);

        let tile = super::container::tile(&t, Surface::Ink);
        let _reused = tile.clone();

        let field = super::text_input::rest(&t, Surface::Ink);
        let _reused = field.clone();
    }
}
