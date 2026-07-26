//! Rule (divider) style: [`rest`].
//!
//! `rule::Style` has no `Status`, like [`crate::style::container`] and
//! [`crate::style::progress`]. A rule is a hairline, so it reads the
//! surface's `divider` role at full length with a hairline radius.

use iced::widget::rule::{FillMode, Style};
use saola_tokens::{Surface, Theme};

use crate::convert::ColorExt;

pub fn rest(t: &Theme, s: Surface) -> impl Fn(&iced::Theme) -> Style {
    let color = t.on(s).divider.into_iced();

    move |_| Style {
        color,
        radius: 0.0.into(),
        fill_mode: FillMode::Full,
        snap: true,
    }
}
