//! Renders a [`saola_tokens::Theme`]'s terminal palette as an Alacritty
//! `colors.*` TOML config.
//!
//! This crate depends on `saola-tokens` only (plus `serde`/`toml` for the
//! TOML I/O) — it physically cannot see `iced`. That's the point: it's the
//! first of what should become a family of exporters (GRUB, Plymouth, other
//! terminal emulators) that all read the same pure-data tokens without
//! pulling in a GUI toolkit. `cargo tree -p saola-export-alacritty -e normal`
//! is the proof.
//!
//! # The alpha problem
//!
//! Alacritty's color fields are plain `#rrggbb` strings — no alpha channel.
//! Saola's terminal selection token is translucent by design
//! (`terminal.selection` is terracotta at 30%, `#c671394d`), so before it can
//! be emitted here it has to be pre-composited with [`saola_tokens::Color::over`]
//! onto `terminal.background` (see [`selection_background`]). Ink at the
//! bottom, so `#c671394d` over `#0c0a00` becomes the opaque `#442911` that
//! `design/saola-alacritty.toml` already documents as the golden value.
//!
//! Selection *text* and cursor *text* aren't part of the `Terminal` token
//! group at all (Alacritty needs them, the token model doesn't model a
//! separate "text on selection" role) — both follow the one design rule
//! documented in `CLAUDE.md`: terracotta/selection fill takes ivory text, so
//! selection text is `theme.palette.paper`. Cursor text is exactly
//! `terminal.cursor_text` (already ink), taken straight from the token.

use saola_tokens::{AnsiColors, Color, Theme};
use serde::{Deserialize, Serialize};

/// `[colors.primary]`: the terminal's default background/foreground pair.
///
/// Derives `Deserialize` too (not just `Serialize`) so the golden test below
/// can parse `design/saola-alacritty.toml` through these exact same types —
/// extra fields the golden file carries but this crate doesn't render
/// (`dim_foreground` here, `vi_mode_cursor`/`search`/`footer_bar`/`hints` on
/// [`Colors`], `[font]`/`[window]` at the document root) are simply ignored
/// by serde, since none of these types use `deny_unknown_fields`.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Primary {
    background: Color,
    foreground: Color,
}

/// `[colors.cursor]`: the block cursor and the text drawn underneath it.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct CursorColors {
    cursor: Color,
    text: Color,
}

/// `[colors.selection]`: the highlighted-text background/foreground pair.
/// `background` is the pre-composited (opaque) selection color — see the
/// module docs and [`selection_background`].
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct SelectionColors {
    background: Color,
    text: Color,
}

/// The `[colors]` table: everything Alacritty needs, nested under `colors`.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Colors {
    primary: Primary,
    cursor: CursorColors,
    selection: SelectionColors,
    normal: AnsiColors,
    bright: AnsiColors,
}

/// The whole document: just the one top-level `colors` table.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct AlacrittyConfig {
    colors: Colors,
}

/// Pre-composite a translucent token color onto `background` for a no-alpha
/// export target. Public so other no-alpha exporters (and this crate's own
/// tests) can reuse the exact compositing rule instead of re-deriving it.
pub fn composite_over_background(color: Color, background: Color) -> Color {
    color.over(background)
}

/// The selection background Alacritty actually needs: `theme.terminal.selection`
/// (translucent terracotta) composited onto `theme.terminal.background` (opaque
/// ink), since Alacritty's colors have no alpha channel.
fn selection_background(theme: &Theme) -> Color {
    composite_over_background(theme.terminal.selection, theme.terminal.background)
}

/// Render `theme`'s terminal palette as an Alacritty colors TOML document
/// (`[colors.primary]`, `cursor`, `selection`, `normal`, `bright`).
///
/// # Panics
///
/// Panics if the underlying TOML serialization fails. `AlacrittyConfig` is a
/// plain tree of strings/tables, so this can't actually happen for any
/// `Theme` value — there's no way to construct a `Theme` whose fields would
/// make serialization fail (colors always serialize to a hex string).
pub fn render(theme: &Theme) -> String {
    let t = &theme.terminal;
    let config = AlacrittyConfig {
        colors: Colors {
            primary: Primary {
                background: t.background,
                foreground: t.foreground,
            },
            cursor: CursorColors {
                cursor: t.cursor,
                text: t.cursor_text,
            },
            selection: SelectionColors {
                background: selection_background(theme),
                text: theme.palette.paper,
            },
            normal: t.normal,
            bright: t.bright,
        },
    };
    toml::to_string_pretty(&config).expect("AlacrittyConfig always serializes to valid TOML")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn composites_translucent_selection_over_ink() {
        let theme = Theme::saola();
        let composited =
            composite_over_background(theme.terminal.selection, theme.terminal.background);
        // Terracotta at 30% (`#c671394d`) over ink (`#0c0a00`) — the value
        // `design/saola-alacritty.toml` documents (decided 2026-07-26).
        assert_eq!(composited, Color::rgb(0x44, 0x29, 0x11));
    }

    #[test]
    fn render_matches_golden_alacritty_file() {
        let rendered = render(&Theme::saola());
        let rendered_parsed: AlacrittyConfig =
            toml::from_str(&rendered).expect("render() always produces valid TOML");

        let golden_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../design/saola-alacritty.toml");
        let golden_str =
            std::fs::read_to_string(&golden_path).expect("design/saola-alacritty.toml exists");
        let golden_parsed: AlacrittyConfig =
            toml::from_str(&golden_str).expect("design/saola-alacritty.toml is valid TOML");

        // Typed comparison rather than raw `toml::Value` equality: it's
        // robust to key order and comments (as a `toml::Value` diff would
        // be), *and* to hex-string casing (`Color`'s deserializer parses hex
        // case-insensitively — the golden file is uppercase, `render`'s
        // output is lowercase) and to the golden file's Alacritty-only extra
        // fields (ignored, see the doc comment on `Primary`). Both sides go
        // through the exact same typed shape, so this is genuine semantic
        // equality, not a byte/string diff.
        assert_eq!(rendered_parsed, golden_parsed);
    }
}
