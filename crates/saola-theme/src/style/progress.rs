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

pub fn bar(t: &Theme, s: Surface) -> impl Fn(&iced::Theme) -> Style + Clone {
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

/// A static capacity meter: the same track-and-fill shape as [`bar`], but
/// the fill is **neutral** (`fill_strong`) rather than terracotta.
///
/// Terracotta means on/selected/live (style guide §1); "how full is this
/// disk" is none of those, so a capacity readout must not borrow it. The
/// track steps down to `fill_subtle` so the neutral fill still separates
/// from it (`fill_strong` equals `track` on both surfaces, so the track
/// role itself would swallow the fill).
pub fn meter(t: &Theme, s: Surface) -> impl Fn(&iced::Theme) -> Style + Clone {
    let track = t.on(s).fill_subtle.into_iced();
    let fill = t.on(s).fill_strong.into_iced();
    let radius = t.radii.pill;

    move |_| Style {
        background: Background::Color(track),
        bar: Background::Color(fill),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: radius.into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meter_fill_is_neutral_not_accent() {
        let t = Theme::saola();
        let accent = t.palette.accent.into_iced();
        for s in [Surface::Ink, Surface::Paper] {
            let style = meter(&t, s)(&iced::Theme::Dark);
            assert_eq!(
                style.bar,
                Background::Color(t.on(s).fill_strong.into_iced())
            );
            assert_ne!(style.bar, Background::Color(accent));
            assert_eq!(
                style.background,
                Background::Color(t.on(s).fill_subtle.into_iced())
            );
        }
    }
}
