//! Pick list style: [`field`] for the closed control, [`menu`] for its
//! dropdown overlay (`.style(...)` and `.menu_style(...)` on the widget,
//! respectively — two separate `Catalog`s in iced 0.14).
//!
//! The field follows the same rest/active shape as
//! [`crate::style::text_input::rest`]: ivory on ink / fill on paper at rest,
//! with the 2px accent ring in `Status::Opened` doubling as the field's
//! focus indicator (`pick_list::Status` has no separate `Focused` — `Opened`
//! covers it, since the menu can only be open while focused). The menu's
//! hovered-option highlight is where the design language's "selection
//! highlights use accent at the selection radius" is expressed literally:
//! `overlay::menu::Style::selected_background` is accent, and
//! `border.radius` (which the menu also uses to round each highlighted row,
//! not just the menu's own outline) is `radii.selection`.

use iced::overlay::menu;
use iced::widget::pick_list::{Status, Style};
use iced::{Background, Border, Color};
use saola_tokens::{Surface, Theme};

use crate::convert::{ColorExt, ShadowExt};

/// The closed pick-list field.
pub fn field(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style {
    let radius = t.radii.pill;
    let on = *t.on(s);
    let accent = t.palette.accent.into_iced();

    let background = match s {
        Surface::Ink => t.palette.paper,
        Surface::Paper => t.on_paper.fill,
    }
    .into_iced();
    let text_color = match s {
        Surface::Ink => t.palette.ink,
        Surface::Paper => on.primary,
    }
    .into_iced();
    let handle_color = on.secondary.into_iced();
    let placeholder_color = on.quaternary.into_iced();
    let divider = on.divider.into_iced();

    move |_, status| {
        let border = |color: Color, width: f32| Border {
            color,
            width,
            radius: radius.into(),
        };
        let border = match status {
            Status::Active => border(Color::TRANSPARENT, 0.0),
            Status::Hovered => border(divider, 1.0),
            // The menu can only be open while the field is effectively
            // focused, so this doubles as the focus ring.
            Status::Opened { .. } => border(accent, 2.0),
        };
        Style {
            text_color,
            placeholder_color,
            handle_color,
            background: Background::Color(background),
            border,
        }
    }
}

/// The dropdown menu's own style: background, outline, and the
/// accent/`radii.selection` hovered-option highlight.
///
/// A menu is a popover, not a page surface — it reads as an ivory card on
/// either surface, the same way [`crate::style::container::card`] stays
/// paper-based on ink.
pub fn menu(t: &Theme, s: Surface) -> impl Fn(&iced::Theme) -> menu::Style {
    let radius = t.radii.selection;
    let on = *t.on(s);
    let accent = t.palette.accent;

    let background = t.palette.paper.into_iced();
    let text_color = t.palette.ink.into_iced();
    let border_color = on.divider.into_iced();
    let selected_background = accent.into_iced();
    let selected_text_color = t.palette.paper.into_iced();
    let shadow = t.shadows.popover.into_iced();

    move |_| menu::Style {
        background: Background::Color(background),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: radius.into(),
        },
        text_color,
        selected_text_color,
        selected_background: Background::Color(selected_background),
        shadow,
    }
}
