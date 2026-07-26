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

pub mod button;
pub mod container;
pub mod pick_list;
pub mod progress;
pub mod rule;
pub mod scrollable;
pub mod slider;
pub mod text_input;
pub mod toggles;

use crate::convert::ColorExt;
use saola_tokens::Theme;

/// The keyboard-focus ring: a 2 px terracotta border at the given radius.
/// Never a platform default.
///
/// iced 0.14's per-widget `Status` enums do not (yet) carry a focus state
/// for buttons, so this is a building block for consumers that track focus
/// themselves (and for the widgets whose `Status` *does* include focus,
/// like `text_input`, in later style modules).
pub fn focus_border(theme: &Theme, radius: f32) -> iced::Border {
    iced::Border {
        color: theme.palette.accent.into_iced(),
        width: 2.0,
        radius: radius.into(),
    }
}
