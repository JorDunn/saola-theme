//! `saola-tokens`: pure-data design tokens for the Saola desktop environment.
//!
//! This crate has no GUI dependencies (no `iced`) — it's just serde-able
//! structs describing colors, typography, spacing, etc., so that future
//! non-iced consumers (GRUB/Plymouth/terminal theme exporters) can depend
//! on it too.
//!
//! # The one rule
//!
//! Saola's identity is three colors, never a fourth: ink (every shell
//! surface), paper/ivory (a control at rest), and a terracotta accent (on,
//! selected, focused, live). Ivory fill takes ink text; terracotta fill
//! takes ivory text. Everything else in this crate — the alpha-stepped
//! [`palette::OnSurface`] roles, scrims, terminal colors — is derived from
//! those three, stepped with alpha for use on one of two surface contexts
//! ([`palette::Surface::Ink`] or [`palette::Surface::Paper`]). There is no
//! dark/light theme variant: the surface context is the axis that varies.
//!
//! # Loading a theme
//!
//! ```
//! use saola_tokens::Theme;
//!
//! let theme = Theme::saola();
//! let toml = theme.to_toml_string().unwrap();
//! let parsed = Theme::from_toml_str(&toml).unwrap();
//! assert_eq!(theme, parsed);
//!
//! // A partial (or empty) TOML document is valid too — every field not
//! // present falls back to the built-in Saola theme's value.
//! assert_eq!(Theme::from_toml_str("").unwrap(), Theme::saola());
//! ```

mod color;
mod palette;
mod tokens;

use std::path::Path;

use serde::{Deserialize, Serialize};

pub use color::{Color, ColorParseError};
pub use palette::{OnSurface, Palette, Scrim, Surface};
pub use tokens::{
    AnsiColors, FontSizes, FontWeights, Motion, Radii, Shadow, Shadows, Sizes, Terminal, Typography,
};

/// A complete Saola theme: the color identity, both on-surface role sets,
/// scrims, and every other structured token group.
///
/// There is exactly one built-in constructor, [`Theme::saola`], and
/// `Theme::default()` is defined to equal it — alternate themes are meant
/// to arrive as TOML files (via [`Theme::from_path`] / [`Theme::from_toml_str`]),
/// not as additional Rust constructors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    pub name: String,
    pub palette: Palette,
    /// Alpha-stepped roles for use on an ink surface.
    pub on_ink: OnSurface,
    /// Alpha-stepped roles for use on a paper surface.
    pub on_paper: OnSurface,
    pub scrim: Scrim,
    pub typography: Typography,
    pub radii: Radii,
    pub sizes: Sizes,
    pub shadows: Shadows,
    pub motion: Motion,
    pub terminal: Terminal,
}

impl Theme {
    /// The one built-in theme, hand-translated from `design/saola-tokens.json`.
    pub fn saola() -> Self {
        Theme {
            name: "Saola".to_string(),
            palette: Palette::default(),
            on_ink: OnSurface::on_ink(),
            on_paper: OnSurface::on_paper(),
            scrim: Scrim::default(),
            typography: Typography::default(),
            radii: Radii::default(),
            sizes: Sizes::default(),
            shadows: Shadows::default(),
            motion: Motion::default(),
            terminal: Terminal::default(),
        }
    }

    /// The role set for a given surface context — the selector every style
    /// helper in `saola-theme` calls to find out which alpha-stepped colors
    /// to use.
    pub fn on(&self, surface: Surface) -> &OnSurface {
        match surface {
            Surface::Ink => &self.on_ink,
            Surface::Paper => &self.on_paper,
        }
    }

    /// Parse a theme from a TOML document. Any field (at any level) absent
    /// from the document falls back to [`Theme::saola`]'s value for that
    /// field — an empty string is a completely valid (if pointless) theme
    /// file, and a real partial file only needs to list the fields it
    /// overrides.
    pub fn from_toml_str(s: &str) -> Result<Self, ThemeError> {
        Ok(toml::from_str(s)?)
    }

    /// Read and parse a theme TOML file from disk.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ThemeError> {
        let contents = std::fs::read_to_string(path)?;
        Self::from_toml_str(&contents)
    }

    /// Serialize this theme to a TOML document (e.g. to write out
    /// `themes/saola.toml`, or to let a user dump their current theme to
    /// start a custom one).
    pub fn to_toml_string(&self) -> Result<String, ThemeError> {
        Ok(toml::to_string_pretty(self)?)
    }
}

/// `Theme::default()` is defined to be the built-in Saola theme, not an
/// all-zeros/all-transparent struct — this is what makes `#[serde(default)]`
/// above do the right thing when a TOML document (or a table within it) is
/// missing fields.
impl Default for Theme {
    fn default() -> Self {
        Theme::saola()
    }
}

/// Errors from reading, parsing, or serializing a [`Theme`].
///
/// `#[from]` (via `thiserror`) generates the `From` impls that let `?`
/// convert a `std::io::Error` / `toml::de::Error` / `toml::ser::Error`
/// straight into a `ThemeError` at each `?` call site above — that's what
/// makes `from_path`'s single `?` after `read_to_string` and the `?`
/// implied by `Ok(toml::from_str(s)?)` type-check.
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("failed to read theme file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse theme TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("failed to serialize theme to TOML: {0}")]
    Serialize(#[from] toml::ser::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saola_round_trips_through_toml() {
        let theme = Theme::saola();
        let toml_str = theme.to_toml_string().unwrap();
        let parsed = Theme::from_toml_str(&toml_str).unwrap();
        assert_eq!(theme, parsed);
    }

    #[test]
    fn empty_toml_string_parses_to_saola_theme() {
        // Proves partial-file support: with every field's `#[serde(default)]`
        // wired to the real theme, an empty document is a valid (fully
        // populated) theme, not an error and not a zeroed-out struct.
        let parsed = Theme::from_toml_str("").unwrap();
        assert_eq!(parsed, Theme::saola());
    }

    #[test]
    fn default_equals_saola() {
        assert_eq!(Theme::default(), Theme::saola());
    }

    #[test]
    fn from_path_reads_and_parses() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("saola-theme-test-{}.toml", std::process::id()));
        std::fs::write(&path, Theme::saola().to_toml_string().unwrap()).unwrap();

        let parsed = Theme::from_path(&path).unwrap();
        assert_eq!(parsed, Theme::saola());

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn from_path_missing_file_is_an_io_error() {
        let result = Theme::from_path("/nonexistent/path/does-not-exist.toml");
        assert!(matches!(result, Err(ThemeError::Io(_))));
    }

    #[test]
    fn from_toml_str_rejects_garbage() {
        let result = Theme::from_toml_str("not = [valid = toml");
        assert!(matches!(result, Err(ThemeError::Parse(_))));
    }

    // ---- Contrast invariants -------------------------------------------
    //
    // WCAG-style contrast ratio: (lighter luminance + 0.05) / (darker
    // luminance + 0.05). Translucent on-surface roles are composited
    // (`Color::over`) onto their surface first, since contrast is only
    // meaningful for solid colors.

    fn contrast_ratio(a: Color, b: Color) -> f64 {
        let (la, lb) = (a.relative_luminance(), b.relative_luminance());
        let (lighter, darker) = if la > lb { (la, lb) } else { (lb, la) };
        (lighter + 0.05) / (darker + 0.05)
    }

    #[test]
    fn ink_and_paper_contrast_both_ways() {
        let theme = Theme::saola();
        let ratio_a = contrast_ratio(theme.palette.ink, theme.palette.paper);
        let ratio_b = contrast_ratio(theme.palette.paper, theme.palette.ink);
        assert_eq!(ratio_a, ratio_b); // the formula is symmetric by construction
        assert!(ratio_a >= 4.5, "ink/paper contrast was {ratio_a}");
    }

    #[test]
    fn on_ink_primary_over_ink_meets_body_text_contrast() {
        let theme = Theme::saola();
        let composited = theme.on_ink.primary.over(theme.palette.ink);
        let ratio = contrast_ratio(composited, theme.palette.ink);
        assert!(ratio >= 4.5, "on_ink.primary/ink contrast was {ratio}");
    }

    #[test]
    fn on_paper_primary_over_paper_meets_body_text_contrast() {
        let theme = Theme::saola();
        let composited = theme.on_paper.primary.over(theme.palette.paper);
        let ratio = contrast_ratio(composited, theme.palette.paper);
        assert!(ratio >= 4.5, "on_paper.primary/paper contrast was {ratio}");
    }

    #[test]
    fn on_ink_secondary_over_ink_meets_body_text_contrast() {
        let theme = Theme::saola();
        let composited = theme.on_ink.secondary.over(theme.palette.ink);
        let ratio = contrast_ratio(composited, theme.palette.ink);
        assert!(ratio >= 4.5, "on_ink.secondary/ink contrast was {ratio}");
    }

    #[test]
    fn paper_on_accent_meets_large_text_contrast() {
        let theme = Theme::saola();
        let ratio = contrast_ratio(theme.palette.paper, theme.palette.accent);
        assert!(ratio >= 3.0, "paper/accent contrast was {ratio}");
    }

    #[test]
    fn accent_light_on_ink_meets_large_text_contrast() {
        let theme = Theme::saola();
        let ratio = contrast_ratio(theme.palette.accent_light, theme.palette.ink);
        assert!(ratio >= 3.0, "accent_light/ink contrast was {ratio}");
    }

    #[test]
    fn accent_dark_on_paper_meets_large_text_contrast() {
        let theme = Theme::saola();
        let ratio = contrast_ratio(theme.palette.accent_dark, theme.palette.paper);
        assert!(ratio >= 3.0, "accent_dark/paper contrast was {ratio}");
    }

    // ---- The session-status semaphore family ---------------------------
    //
    // The documented exception to "three colors, never a fourth" (see
    // `Palette`'s docs). These five are drawn as ~16px dots on ink, so they
    // are *non-text* marks: WCAG's 3:1 non-text contrast bar applies, not
    // 4.5:1. They also have to stay apart from each other — a semaphore
    // nobody can decode is worse than no semaphore — which is what the
    // CIELAB ΔE test below pins down.

    fn status_colors(theme: &Theme) -> [(&'static str, Color); 5] {
        [
            ("status_working", theme.palette.status_working),
            ("status_subagents", theme.palette.status_subagents),
            ("status_attention", theme.palette.status_attention),
            ("status_done", theme.palette.status_done),
            ("status_idle", theme.palette.status_idle),
        ]
    }

    /// CIE L*a*b* coordinates (D65 white point) for a *fully opaque* color.
    /// Contrast ratio answers "can I see it?"; ΔE answers "can I tell these
    /// two apart?", which is the question a five-state semaphore actually
    /// asks. Written out here rather than pulled in as a dependency —
    /// `saola-tokens` deliberately depends on nothing but serde/toml/
    /// thiserror, and dev-dependencies would still show up in that tree.
    fn lab(c: Color) -> (f64, f64, f64) {
        fn to_linear(channel: u8) -> f64 {
            let c = f64::from(channel) / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        fn f(t: f64) -> f64 {
            if t > 0.008_856 {
                t.cbrt()
            } else {
                7.787 * t + 16.0 / 116.0
            }
        }
        let (r, g, b) = (to_linear(c.r), to_linear(c.g), to_linear(c.b));
        // sRGB -> XYZ, then normalized by the D65 white point.
        let x = (0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047;
        let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let z = (0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883;
        let (fx, fy, fz) = (f(x), f(y), f(z));
        (116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz))
    }

    fn delta_e(a: Color, b: Color) -> f64 {
        let (l1, a1, b1) = lab(a);
        let (l2, a2, b2) = lab(b);
        ((l1 - l2).powi(2) + (a1 - a2).powi(2) + (b1 - b2).powi(2)).sqrt()
    }

    #[test]
    fn every_status_color_reads_as_a_mark_on_ink() {
        let theme = Theme::saola();
        for (name, color) in status_colors(&theme) {
            let ratio = contrast_ratio(color, theme.palette.ink);
            assert!(ratio >= 3.0, "{name}/ink contrast was {ratio}");
        }
    }

    #[test]
    fn status_colors_are_mutually_distinguishable() {
        let theme = Theme::saola();
        let colors = status_colors(&theme);
        for (i, (name_a, a)) in colors.iter().enumerate() {
            for (name_b, b) in &colors[i + 1..] {
                let distance = delta_e(*a, *b);
                assert!(
                    distance >= 25.0,
                    "{name_a} and {name_b} are only ΔE {distance} apart"
                );
            }
        }
    }

    #[test]
    fn status_working_is_distinct_from_the_accent_ramp() {
        // The one status color that could plausibly be mistaken for the
        // accent: both are warm oranges, and `accent_light` is already used
        // as text on the same ink bar the dots sit on.
        let theme = Theme::saola();
        for (name, accent) in [
            ("accent", theme.palette.accent),
            ("accent_light", theme.palette.accent_light),
        ] {
            let distance = delta_e(theme.palette.status_working, accent);
            assert!(
                distance >= 20.0,
                "status_working and {name} are only ΔE {distance} apart"
            );
        }
    }

    #[test]
    fn a_dimmed_breath_still_scales_the_status_alpha() {
        // The panel breathes a dot by scaling its fill alpha between
        // `motion.breathe_min_opacity` and 1.0; at the dim end the dot must
        // still be visible against ink, which is why the floor isn't 0.
        let theme = Theme::saola();
        let floor = theme.motion.breathe_min_opacity;
        assert!((0.2..1.0).contains(&floor), "breathe floor was {floor}");

        let alpha = (255.0 * floor).round() as u8;
        for (name, color) in status_colors(&theme) {
            let dimmed = Color::rgba(color.r, color.g, color.b, alpha).over(theme.palette.ink);
            let ratio = contrast_ratio(dimmed, theme.palette.ink);
            assert!(ratio > 1.4, "{name} at the breath floor was {ratio} on ink");
        }
    }

    #[test]
    fn partial_toml_keeps_the_status_family_and_breathe_defaults() {
        // The serde-default guarantee for the new fields specifically: a
        // theme file written before they existed (here, one that overrides
        // only `palette.accent` and one motion duration) still parses, and
        // still comes out with the real semaphore colors.
        // `r##"…"##` rather than `r#"…"#`: the TOML below contains `"#`
        // (the closing quote of a hex color), which would end an `r#` raw
        // string early. Two hashes, and the delimiter is unambiguous again.
        let parsed = Theme::from_toml_str(
            r##"
            [palette]
            accent = "#C67139"

            [motion]
            hover = 200
            "##,
        )
        .unwrap();

        assert_eq!(parsed.palette, Theme::saola().palette);
        assert_eq!(parsed.motion.hover, 200);
        assert_eq!(parsed.motion.breathe, Theme::saola().motion.breathe);
        assert_eq!(
            parsed.motion.breathe_min_opacity,
            Theme::saola().motion.breathe_min_opacity
        );
    }

    #[test]
    fn status_family_survives_a_toml_round_trip() {
        let theme = Theme::saola();
        let parsed = Theme::from_toml_str(&theme.to_toml_string().unwrap()).unwrap();
        assert_eq!(parsed.palette.status_working, theme.palette.status_working);
        assert_eq!(
            parsed.palette.status_subagents,
            theme.palette.status_subagents
        );
        assert_eq!(
            parsed.palette.status_attention,
            theme.palette.status_attention
        );
        assert_eq!(parsed.palette.status_done, theme.palette.status_done);
        assert_eq!(parsed.palette.status_idle, theme.palette.status_idle);
        assert_eq!(parsed.motion.breathe, theme.motion.breathe);
        assert_eq!(
            parsed.motion.breathe_min_opacity,
            theme.motion.breathe_min_opacity
        );
    }
}
