//! Scrollable style: [`rest`].
//!
//! `scrollable::Status` carries per-axis hover/drag flags rather than a
//! single state, but both axes always render identically here, so the
//! closure only branches on whether *either* axis is being interacted with.
//! The scrollbar rail reads the surface's `track` role (matching
//! [`crate::style::slider`] and [`crate::style::progress`]); the scroller
//! (the draggable thumb) is ivory/fill at rest and terracotta while
//! dragged — the familiar off/on split, expressed as hover/drag instead of a
//! boolean since a scrollbar has no "on" state of its own.

use iced::widget::container;
use iced::widget::scrollable::{AutoScroll, Rail, Scroller, Status, Style};
use iced::{Background, Border, Color, Shadow, Vector};
use saola_tokens::{Surface, Theme};

use crate::convert::ColorExt;

pub fn rest(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style {
    let on = *t.on(s);
    let accent = t.palette.accent;
    let track = on.track;
    // Scroller (the thumb): ivory/fill at rest, one fill step deeper on
    // hover, terracotta while actively dragged — "on" only for as long as
    // the drag lasts.
    let (scroller_rest, scroller_hover) = match s {
        Surface::Ink => (
            t.palette.paper,
            t.on_paper.fill_subtle.over(t.palette.paper),
        ),
        Surface::Paper => (t.on_paper.fill, t.on_paper.fill_strong),
    };
    let scroller_drag = accent;

    // Copied out up front (not read from `t` inside the closure) so the
    // returned closure stays `'static`, same discipline as every other
    // style helper in this crate.
    let auto_scroll_bg = t.palette.paper.into_iced();
    let auto_scroll_border = on.divider.into_iced();
    let auto_scroll_radius = t.radii.pill;
    let auto_scroll_shadow_color = t.shadows.popover.color.into_iced();
    let auto_scroll_shadow_blur = t.shadows.popover.blur;
    let auto_scroll_icon = t.palette.ink.into_iced();

    move |_, status| {
        let is_dragged = matches!(
            status,
            Status::Dragged {
                is_horizontal_scrollbar_dragged: true,
                ..
            } | Status::Dragged {
                is_vertical_scrollbar_dragged: true,
                ..
            }
        );
        let is_hovered = matches!(
            status,
            Status::Hovered {
                is_horizontal_scrollbar_hovered: true,
                ..
            } | Status::Hovered {
                is_vertical_scrollbar_hovered: true,
                ..
            }
        );
        let scroller_color = if is_dragged {
            scroller_drag
        } else if is_hovered {
            scroller_hover
        } else {
            scroller_rest
        }
        .into_iced();

        let rail = Rail {
            background: Some(Background::Color(track.into_iced())),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 2.0.into(),
            },
            scroller: Scroller {
                background: Background::Color(scroller_color),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 2.0.into(),
                },
            },
        };

        Style {
            container: container::Style::default(),
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
            auto_scroll: AutoScroll {
                background: Background::Color(auto_scroll_bg),
                border: Border {
                    color: auto_scroll_border,
                    width: 1.0,
                    radius: auto_scroll_radius.into(),
                },
                shadow: Shadow {
                    color: auto_scroll_shadow_color,
                    offset: Vector::ZERO,
                    blur_radius: auto_scroll_shadow_blur,
                },
                icon: auto_scroll_icon,
            },
        }
    }
}
