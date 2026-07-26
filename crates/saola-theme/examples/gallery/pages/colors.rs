//! Colors page: every color token in the theme, as labeled swatches.
//!
//! Four sections:
//! 1. The identity trio (ink, paper, accent, plus the two accent text
//!    variants) — the three colors Saola is built from.
//! 2. The alpha-stepped on-surface role steps, *each rendered on its own
//!    surface* (on-ink roles sit on a real ink background, on-paper roles
//!    on a real paper background) — that's the only way a translucent color
//!    actually shows what it looks like.
//! 3. Wallpaper scrims.
//! 4. The terminal's 16 ANSI colors.

use iced::widget::{column, container, row, text, Space};
use iced::{Background, Border, Color as IcedColor, Element, Fill};
use saola_theme::tokens::{Color, OnSurface};
use saola_theme::{convert, style, ColorExt, Surface, Theme};

use crate::Message;

/// The Colors page. `primary` is which surface's role-step section is shown
/// first — it's `self.surface` from the sidebar toggle, so flipping the
/// toggle visibly reorders this page (the clearest place in the gallery to
/// see the surface axis affect layout, since both role-step sets stay on
/// screen together for comparison — that's the whole point of this page).
pub fn view(t: &Theme, primary: Surface) -> Element<'static, Message> {
    let heading_size = t.typography.size.section_heading;
    let label_size = t.typography.size.secondary;
    // Everything on this page that isn't explicitly wrapped in its own
    // surface container sits directly on the app shell, which is always
    // ink (per Architecture: "ink: every shell surface") — so secondary
    // labels default to the on-ink role.
    let ambient_label = t.on_ink.secondary.into_iced();

    let (first, second) = match primary {
        Surface::Ink => (
            role_steps_section(t, Surface::Ink),
            role_steps_section(t, Surface::Paper),
        ),
        Surface::Paper => (
            role_steps_section(t, Surface::Paper),
            role_steps_section(t, Surface::Ink),
        ),
    };

    column![
        text("Colors").size(heading_size),
        text("Identity — the three colors everything else derives from")
            .size(label_size)
            .color(ambient_label),
        identity_section(t),
        text("On-surface role steps — rendered on their own surface")
            .size(label_size)
            .color(ambient_label),
        first,
        second,
        text("Scrims").size(label_size).color(ambient_label),
        scrims_section(t),
        text("Terminal — the 16 ANSI colors")
            .size(label_size)
            .color(ambient_label),
        terminal_section(t),
    ]
    .spacing(20)
    .width(Fill)
    .into()
}

/// One named color swatch: a rounded tile filled with `color`, its token
/// name, and its literal hex value (including alpha, for the translucent
/// on-surface roles and scrims — that's the real token value, not what it
/// looks like once composited onto a surface).
fn swatch(
    t: &Theme,
    color: Color,
    name: &'static str,
    label_color: IcedColor,
) -> Element<'static, Message> {
    let tile_radius = t.radii.tile;
    let iced_color = color.into_iced();
    let tile = container(Space::new())
        .width(88)
        .height(40)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(iced_color)),
            border: Border {
                color: IcedColor::TRANSPARENT,
                width: 0.0,
                radius: tile_radius.into(),
            },
            ..Default::default()
        });

    column![
        tile,
        text(name).size(t.typography.size.label).color(label_color),
        text(color.to_string())
            .size(t.typography.size.label)
            .font(convert::mono_font(t))
            .color(label_color),
    ]
    .spacing(4)
    .width(88)
    .into()
}

/// A grid of swatches, wrapped into rows of `per_row` — `row!`/`column!`
/// don't wrap on their own, so long swatch lists (role steps, terminal
/// colors) are chunked by hand into several `row`s stacked in a `column`.
fn swatch_grid(
    t: &Theme,
    entries: &[(&'static str, Color)],
    label_color: IcedColor,
    per_row: usize,
) -> Element<'static, Message> {
    let rows: Vec<Element<'static, Message>> = entries
        .chunks(per_row)
        .map(|chunk| {
            let swatches: Vec<Element<'static, Message>> = chunk
                .iter()
                .map(|(name, color)| swatch(t, *color, name, label_color))
                .collect();
            row(swatches).spacing(12).into()
        })
        .collect();
    column(rows).spacing(12).into()
}

/// Ink, paper, accent, accent_light, accent_dark — the three-color identity
/// plus its two accent-as-text variants. Shown directly on the ink shell.
fn identity_section(t: &Theme) -> Element<'static, Message> {
    let p = t.palette;
    let entries = [
        ("ink", p.ink),
        ("paper", p.paper),
        ("accent", p.accent),
        ("accent_light", p.accent_light),
        ("accent_dark", p.accent_dark),
    ];
    swatch_grid(t, &entries, t.on_ink.secondary.into_iced(), 5)
}

/// The 10 `OnSurface` role steps for one surface, rendered inside a real
/// container styled for that surface — so the translucent fill/divider
/// steps actually alpha-composite against the right background, instead of
/// being shown as flat swatches that wouldn't demonstrate what "stepped
/// with alpha" actually looks like.
///
/// The two match arms are intentionally not merged behind a boxed closure:
/// `style::container::ink_surface` and `style::container::paper_window`
/// return two *different* concrete (if opaque) closure types, so a shared
/// variable would need `Box<dyn Fn(..)>` — more machinery than a straight
/// if/match needs here.
fn role_steps_section(t: &Theme, surface: Surface) -> Element<'static, Message> {
    let on: OnSurface = *t.on(surface);
    let entries = [
        ("primary", on.primary),
        ("secondary", on.secondary),
        ("tertiary", on.tertiary),
        ("quaternary", on.quaternary),
        ("disabled", on.disabled),
        ("divider", on.divider),
        ("fill_subtle", on.fill_subtle),
        ("fill", on.fill),
        ("fill_strong", on.fill_strong),
        ("track", on.track),
    ];

    match surface {
        Surface::Ink => {
            let label_color = t.on_ink.secondary.into_iced();
            container(
                column![
                    text("On ink")
                        .size(t.typography.size.secondary)
                        .color(label_color),
                    swatch_grid(t, &entries, label_color, 5),
                ]
                .spacing(10),
            )
            .style(style::container::ink_surface(t))
            .padding(20)
            .width(Fill)
            .into()
        }
        Surface::Paper => {
            let label_color = t.on_paper.secondary.into_iced();
            container(
                column![
                    text("On paper")
                        .size(t.typography.size.secondary)
                        .color(label_color),
                    swatch_grid(t, &entries, label_color, 5),
                ]
                .spacing(10),
            )
            .style(style::container::paper_window(t))
            .padding(20)
            .width(Fill)
            .into()
        }
    }
}

/// The 8 wallpaper scrims — all ink-tinted, varying only in how much of the
/// wallpaper they let through. Shown as flat swatches (the gallery has no
/// live wallpaper to composite them onto); the hex alpha value is the
/// informative part.
fn scrims_section(t: &Theme) -> Element<'static, Message> {
    let s = t.scrim;
    let entries = [
        ("boot", s.boot),
        ("shutdown", s.shutdown),
        ("lock_awake", s.lock_awake),
        ("launcher", s.launcher),
        ("overview", s.overview),
        ("capture", s.capture),
        ("modal", s.modal),
        ("translucent_panel", s.translucent_panel),
    ];
    swatch_grid(t, &entries, t.on_ink.secondary.into_iced(), 4)
}

/// The 16 terminal ANSI colors: the 8 `normal` colors, then the 8 `bright`
/// variants — pure data feeding future terminal-config exporters, but worth
/// seeing alongside the rest of the palette since they're derived from the
/// same three-color identity.
fn terminal_section(t: &Theme) -> Element<'static, Message> {
    let term = t.terminal;
    let normal = [
        ("black", term.normal.black),
        ("red", term.normal.red),
        ("green", term.normal.green),
        ("yellow", term.normal.yellow),
        ("blue", term.normal.blue),
        ("magenta", term.normal.magenta),
        ("cyan", term.normal.cyan),
        ("white", term.normal.white),
    ];
    let bright = [
        ("bright_black", term.bright.black),
        ("bright_red", term.bright.red),
        ("bright_green", term.bright.green),
        ("bright_yellow", term.bright.yellow),
        ("bright_blue", term.bright.blue),
        ("bright_magenta", term.bright.magenta),
        ("bright_cyan", term.bright.cyan),
        ("bright_white", term.bright.white),
    ];
    let label_color = t.on_ink.secondary.into_iced();

    column![
        swatch_grid(t, &normal, label_color, 8),
        swatch_grid(t, &bright, label_color, 8),
    ]
    .spacing(12)
    .into()
}
