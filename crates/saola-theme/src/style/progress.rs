//! Progress bar style: [`bar`].
//!
//! `progress_bar::Style` has no `Status` — a progress bar is never hovered
//! or pressed — so, like [`crate::style::container`], the closure takes only
//! `&iced::Theme`. Track background reads the surface's `track` role; the
//! filled portion is terracotta, same "track role + accent fill" pairing as
//! [`crate::style::slider`].

use iced::widget::progress_bar::Style;
use iced::{Background, Border, Color};
use saola_tokens::{Surface, Theme};

use crate::convert::ColorExt;

pub fn bar(t: &Theme, s: Surface) -> impl Fn(&iced::Theme) -> Style {
    let track = t.on(s).track.into_iced();
    let accent = t.palette.accent.into_iced();
    let radius = t.radii.pill;

    move |_| Style {
        background: Background::Color(track),
        bar: Background::Color(accent),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: radius.into(),
        },
    }
}
