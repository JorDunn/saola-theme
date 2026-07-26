//! Slider style: [`rest`].
//!
//! `slider::Status` is `Active | Hovered | Dragged` — no `Disabled` variant
//! exists in iced 0.14 (a slider with no `on_change` still reports one of
//! these three). The rail is two backgrounds side by side: the filled
//! portion (how far the value has traveled — terracotta, "on/selected/live")
//! and the unfilled portion, which per the design language reads the
//! surface's `track` role. The handle is a small ivory disc with an accent
//! ring so it reads clearly on either fill.

use iced::widget::slider::{Handle, HandleShape, Rail, Status, Style};
use iced::{Background, Border, Color};
use saola_tokens::{Surface, Theme};

use crate::convert::ColorExt;

pub fn rest(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style {
    let track = t.on(s).track.into_iced();
    let accent = t.palette.accent;
    // The filled portion of the rail steps through the on-ink (ivory) fills
    // over accent on hover/drag, same recipe as `button::active`.
    let fill = accent.into_iced();
    let fill_hovered = t.on_ink.fill_subtle.over(accent).into_iced();
    let fill_dragged = t.on_ink.fill.over(accent).into_iced();
    let handle_bg = t.palette.paper.into_iced();
    let handle_border = accent.into_iced();

    move |_, status| {
        let fill = match status {
            Status::Active => fill,
            Status::Hovered => fill_hovered,
            Status::Dragged => fill_dragged,
        };
        Style {
            rail: Rail {
                backgrounds: (Background::Color(fill), Background::Color(track)),
                width: 4.0,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 2.0.into(),
                },
            },
            handle: Handle {
                shape: HandleShape::Circle { radius: 8.0 },
                background: Background::Color(handle_bg),
                border_width: 2.0,
                border_color: handle_border,
            },
        }
    }
}
