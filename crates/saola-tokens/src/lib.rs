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
}
