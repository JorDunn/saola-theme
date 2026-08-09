//! Bundled widgets: constructors that pair an iced widget with its Saola
//! style *and* the size token the style implies, so consumers can't get one
//! half right and the other wrong.
//!
//! The style helpers in [`crate::style`] deliberately stop at styling —
//! but a hairline's thickness is a *size argument* to `rule::horizontal`/
//! `rule::vertical`, not part of `rule::Style`, so a style helper alone
//! can't guarantee a 1 px rule. [`hairline`] and [`vertical_hairline`]
//! close that gap by constructing the `Rule` themselves.

use iced::widget::{rule, Rule};
use saola_tokens::{Surface, Theme};

use crate::style;

/// A horizontal hairline rule: `sizes.hairline` (1 px) thick, in the
/// surface's `divider` role ([`style::rule::rest`]). It fills the available
/// width, like any horizontal `Rule`.
///
/// ```no_run
/// use saola_theme::{widget, Surface, Theme};
///
/// let t = Theme::saola();
/// let divider: iced::widget::Rule<'_> = widget::hairline(&t, Surface::Paper);
/// ```
pub fn hairline<'a>(t: &Theme, s: Surface) -> Rule<'a> {
    rule::horizontal(t.sizes.hairline).style(style::rule::rest(t, s))
}

/// The vertical counterpart of [`hairline`]: `sizes.hairline` (1 px) wide,
/// filling the available height. iced names the two orientations as
/// separate constructors (`rule::horizontal` / `rule::vertical`), so this
/// crate does too.
pub fn vertical_hairline<'a>(t: &Theme, s: Surface) -> Rule<'a> {
    rule::vertical(t.sizes.hairline).style(style::rule::rest(t, s))
}
