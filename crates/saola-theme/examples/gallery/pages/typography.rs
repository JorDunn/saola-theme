//! Typography page: the three IBM Plex families, and the full size scale,
//! as live specimens.
//!
//! Unlike the Colors page (which *must* show both surfaces side by side to
//! compare role steps), a font doesn't have a "surface"-specific look — so
//! this page instead uses `surface` as *where the whole page is drawn*.
//! On ink it sits directly on the shell; on paper it renders inside a real
//! `paper_window` card. That's a second, independent way the surface toggle
//! proves itself at runtime: instead of reordering sections like the Colors
//! page, this page's entire background and text color flip.

use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Fill};
use saola_theme::tokens::OnSurface;
use saola_theme::{convert, style, ColorExt, Surface, Theme};

use crate::Message;

/// The Typography page.
pub fn view(t: &Theme, surface: Surface) -> Element<'static, Message> {
    let on: OnSurface = *t.on(surface);
    let heading_size = t.typography.size.section_heading;

    let content = column![
        text("Typography")
            .size(heading_size)
            .color(on.primary.into_iced()),
        families_section(t, &on),
        text("Tabular numerals")
            .size(t.typography.size.secondary)
            .color(on.secondary.into_iced()),
        tabular_numerals_section(t, surface, &on),
        text("Size scale")
            .size(t.typography.size.secondary)
            .color(on.secondary.into_iced()),
        size_scale_section(t, &on),
    ]
    .spacing(20)
    .width(Fill);

    // The size scale runs from 11px labels up to a 168px lock-screen
    // clock — taller than the gallery window — so this page scrolls.
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

/// One specimen per type family: its name, itself set in its own face at a
/// readable size, and a short pangram-ish sample line at body size.
fn family_specimen(
    t: &Theme,
    on: &OnSurface,
    label: &str,
    family_name: &str,
    font: iced::Font,
) -> Element<'static, Message> {
    column![
        text(label.to_string())
            .size(t.typography.size.label)
            .color(on.secondary.into_iced()),
        text(family_name.to_string())
            .font(font)
            .size(t.typography.size.dialog_title)
            .color(on.primary.into_iced()),
        text("The quick brown fox jumps over 0123456789")
            .font(font)
            .size(t.typography.size.body)
            .color(on.primary.into_iced()),
    ]
    .spacing(4)
    .into()
}

/// The three IBM Plex families: Sans (UI, everything you scan), Serif
/// (display only — wordmark, clock, headings, never the panel bar), Mono
/// (keycaps, paths, hex, terminal).
fn families_section(t: &Theme, on: &OnSurface) -> Element<'static, Message> {
    column![
        family_specimen(
            t,
            on,
            "family_ui — IBM Plex Sans",
            &t.typography.family_ui,
            convert::ui_font(t),
        ),
        family_specimen(
            t,
            on,
            "family_display — IBM Plex Serif",
            &t.typography.family_display,
            convert::display_font(t),
        ),
        family_specimen(
            t,
            on,
            "family_mono — IBM Plex Mono",
            &t.typography.family_mono,
            convert::mono_font(t),
        ),
    ]
    .spacing(16)
    .into()
}

/// Tabular numerals: IBM Plex ships tabular lining figures as its *default*
/// (and only) figure widths — every digit is 600/1000 em in every family
/// and weight we use — so same-slot numeric strings (a clock, a battery
/// percentage) never reflow as their digits change. iced 0.14 exposes no
/// OpenType feature API (no `tnum`), and none is needed; the full
/// verification lives in `docs/decisions/tabular-numerals.md`.
///
/// The proof specimen: each string sits in its own shrink-width keycap
/// chip. Because every digit occupies the same advance width, the chips in
/// each column stack flush — a proportional figure set would leave the
/// columns ragged (`7%` is deliberately one digit shorter: exactly one
/// slot narrower, not "a bit" narrower).
fn tabular_numerals_section(
    t: &Theme,
    surface: Surface,
    on: &OnSurface,
) -> Element<'static, Message> {
    let ui_font = convert::ui_font(t);
    let size = t.typography.size.panel_heading;

    let chip_column = |strings: &[&str]| -> Element<'static, Message> {
        let chips: Vec<Element<'static, Message>> = strings
            .iter()
            .map(|s| {
                container(
                    text(s.to_string())
                        .font(ui_font)
                        .size(size)
                        .color(on.primary.into_iced()),
                )
                .style(style::container::keycap(t, surface))
                .padding([2, 10])
                .into()
            })
            .collect();
        column(chips).spacing(4).into()
    };

    column![
        row![
            chip_column(&["09:41", "11:11", "23:59", "00:00"]),
            chip_column(&["78%", "17%", "41%", "7%"]),
        ]
        .spacing(24),
        text("Plex figures are tabular by default — equal-slot strings stay flush; the clock never reflows.")
            .size(t.typography.size.label)
            .color(on.tertiary.into_iced()),
    ]
    .spacing(8)
    .into()
}

/// Every named size in `Typography::size`, each specimen set in the UI face
/// at its own size, largest first (the order they read best in, and the
/// order `design/saola-tokens.json`'s `font.size` object lists them in).
fn size_scale_section(t: &Theme, on: &OnSurface) -> Element<'static, Message> {
    let sizes = t.typography.size;
    let ui_font = convert::ui_font(t);
    let entries: [(&str, f32); 12] = [
        ("lock_clock", sizes.lock_clock),
        ("screen_title", sizes.screen_title),
        ("panel_heading", sizes.panel_heading),
        ("dialog_title", sizes.dialog_title),
        ("section_heading", sizes.section_heading),
        ("launcher_input", sizes.launcher_input),
        ("body", sizes.body),
        ("bar", sizes.bar),
        ("secondary", sizes.secondary),
        ("meta", sizes.meta),
        ("label", sizes.label),
        ("keycap", sizes.keycap),
    ];

    let rows: Vec<Element<'static, Message>> = entries
        .into_iter()
        .map(|(name, size)| {
            text(format!("{name} · {size}px — Saola"))
                .font(ui_font)
                .size(size)
                .color(on.primary.into_iced())
                .into()
        })
        .collect();

    column(rows).spacing(10).into()
}
