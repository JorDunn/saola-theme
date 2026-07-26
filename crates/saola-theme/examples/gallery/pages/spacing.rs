//! Spacing page: Radii and Sizes visualizers.
//!
//! Like the Typography page, radii and sizes don't have a "surface"-specific
//! *shape* — a 24px window radius is a 24px window radius on either
//! surface — so `surface` here again controls where the whole page is
//! drawn (directly on the ink shell, or inside a `paper_window` card),
//! which is what makes toggling it visibly re-render this page too.

use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Background, Border, Center, Element, Fill};
use saola_theme::tokens::OnSurface;
use saola_theme::{style, ColorExt, Surface, Theme};

use crate::Message;

/// The Spacing page.
pub fn view(t: &Theme, surface: Surface) -> Element<'static, Message> {
    let on: OnSurface = *t.on(surface);
    let heading_size = t.typography.size.section_heading;
    let label_size = t.typography.size.secondary;

    let content = column![
        text("Spacing")
            .size(heading_size)
            .color(on.primary.into_iced()),
        text("Radii — every corner radius in the theme")
            .size(label_size)
            .color(on.secondary.into_iced()),
        radii_section(t, &on),
        text("Hit targets — minimum tappable area")
            .size(label_size)
            .color(on.secondary.into_iced()),
        hit_targets_section(t, &on),
        text("Icon sizes")
            .size(label_size)
            .color(on.secondary.into_iced()),
        icon_sizes_section(t, &on),
    ]
    .spacing(20)
    .width(Fill);

    // Nine radius tiles plus two visualizer rows can run taller than the
    // gallery window, so this page scrolls like the Typography page does.
    let scrolled = scrollable(content)
        .style(style::scrollable::rest(t, surface))
        .height(Fill);

    match surface {
        Surface::Ink => scrolled.into(),
        Surface::Paper => container(scrolled)
            .style(style::container::paper_window(t))
            .padding(24)
            .width(Fill)
            .height(Fill)
            .into(),
    }
}

/// A fixed-size (72×72) box with a *varying* corner radius — the shape that
/// actually demonstrates what a radius token looks like. `radius.pill`
/// (999) renders as a full circle here, because a radius bigger than half
/// the box gets clamped by the renderer — exactly how the pill shape works
/// everywhere else in Saola (buttons, chips, inputs).
fn radius_tile(
    t: &Theme,
    on: &OnSurface,
    name: &'static str,
    radius: f32,
) -> Element<'static, Message> {
    let fill = on.fill.into_iced();
    let border_color = on.divider.into_iced();
    let tile = container(Space::new())
        .width(72)
        .height(72)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(fill)),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: radius.into(),
            },
            ..Default::default()
        });

    column![
        tile,
        text(format!("{name} · {radius:.0}px"))
            .size(t.typography.size.label)
            .color(on.secondary.into_iced()),
    ]
    .spacing(6)
    .align_x(Center)
    .into()
}

/// A box sized to the token's actual pixel value, with a small constant
/// corner radius (borrowed from `radii.selection` purely as "a modest
/// rounding," not to imply a selected state) — the shape that demonstrates
/// *relative scale* between tokens, as opposed to `radius_tile`'s fixed
/// box / varying radius.
fn size_tile(
    t: &Theme,
    on: &OnSurface,
    name: &'static str,
    size: f32,
) -> Element<'static, Message> {
    let fill = on.fill.into_iced();
    let border_color = on.divider.into_iced();
    let radius = t.radii.selection;
    let tile = container(Space::new())
        .width(size)
        .height(size)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(fill)),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: radius.into(),
            },
            ..Default::default()
        });

    column![
        tile,
        text(format!("{name} · {size:.0}px"))
            .size(t.typography.size.label)
            .color(on.secondary.into_iced()),
    ]
    .spacing(6)
    .align_x(Center)
    .into()
}

/// Every corner radius in the theme. Pill/window/card/checkbox lead the
/// row (the four the Architecture section calls out by name — "everything
/// is a pill ... or an over-rounded rectangle"), then the rest of the scale.
fn radii_section(t: &Theme, on: &OnSurface) -> Element<'static, Message> {
    let r = t.radii;
    let entries: [(&str, f32); 9] = [
        ("pill", r.pill),
        ("window", r.window),
        ("card", r.card),
        ("checkbox", r.checkbox),
        ("popover", r.popover),
        ("popover_wide", r.popover_wide),
        ("inset", r.inset),
        ("tile", r.tile),
        ("selection", r.selection),
    ];
    let tiles: Vec<Element<'static, Message>> = entries
        .into_iter()
        .map(|(name, radius)| radius_tile(t, on, name, radius))
        .collect();
    wrap_rows(tiles, 5)
}

/// The bar and touch hit-target minimums, drawn at actual size.
fn hit_targets_section(t: &Theme, on: &OnSurface) -> Element<'static, Message> {
    let s = t.sizes;
    let entries: [(&str, f32); 2] = [
        ("hit_target_bar", s.hit_target_bar),
        ("hit_target_touch", s.hit_target_touch),
    ];
    let tiles: Vec<Element<'static, Message>> = entries
        .into_iter()
        .map(|(name, size)| size_tile(t, on, name, size))
        .collect();
    wrap_rows(tiles, 4)
}

/// The four icon sizes, drawn at actual size, plus the constant stroke
/// width they all share (a line thickness, not a box — shown as text).
fn icon_sizes_section(t: &Theme, on: &OnSurface) -> Element<'static, Message> {
    let s = t.sizes;
    let entries: [(&str, f32); 4] = [
        ("icon_bar", s.icon_bar),
        ("icon_row", s.icon_row),
        ("icon_menu", s.icon_menu),
        ("icon_bare", s.icon_bare),
    ];
    let mut tiles: Vec<Element<'static, Message>> = entries
        .into_iter()
        .map(|(name, size)| size_tile(t, on, name, size))
        .collect();
    tiles.push(
        text(format!(
            "icon_stroke · {:.2}px (constant at every icon size)",
            s.icon_stroke
        ))
        .size(t.typography.size.label)
        .color(on.secondary.into_iced())
        .into(),
    );
    wrap_rows(tiles, 4)
}

/// Lay out already-built elements `per_row` at a time.
///
/// This chunks *elements*, not raw data (contrast with the Colors page's
/// `swatch_grid`, which chunks a data slice and builds each swatch inside
/// the loop) — `icon_sizes_section` needs to mix a plain text row in after
/// its tiles, so it has to build its `Vec<Element>` first. `Element` isn't
/// `Clone`, so chunking it has to consume the `Vec` with `into_iter()`
/// rather than borrowing slices the way `swatch_grid` does.
fn wrap_rows(
    elements: Vec<Element<'static, Message>>,
    per_row: usize,
) -> Element<'static, Message> {
    let mut rows = Vec::new();
    let mut current = Vec::new();
    for element in elements {
        current.push(element);
        if current.len() == per_row {
            rows.push(row(std::mem::take(&mut current)).spacing(16).into());
        }
    }
    if !current.is_empty() {
        rows.push(row(current).spacing(16).into());
    }
    column(rows).spacing(16).into()
}
