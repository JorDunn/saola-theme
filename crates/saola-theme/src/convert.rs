//! Bridges from `saola_tokens`' pure-data types to iced's types.
//!
//! Everything iced-specific about the token crate lives here:
//!
//! - [`ColorExt`] — `saola_tokens::Color` (u8 channels) → `iced::Color`
//!   (f32 channels). This is an extension trait rather than a `From` impl
//!   because of Rust's *orphan rule*: both `saola_tokens::Color` and
//!   `iced::Color` are foreign to this crate, and you may only implement a
//!   foreign trait (like `From`) when at least one of the types involved is
//!   your own. An extension trait *is* our own, so we may implement it for
//!   the foreign color type.
//! - [`ShadowExt`] — `saola_tokens::Shadow` → `iced::Shadow` (same orphan-rule
//!   story).
//! - [`to_iced_theme`] — builds an `iced::Theme` via `iced::Theme::custom`,
//!   so consumers keep full interop with third-party widgets and every
//!   `iced_layershell` example (a custom theme *type* would require ~20
//!   per-widget `Catalog` impls; Saola's identity comes from the style
//!   helpers in [`crate::style`], not from the theme type).
//! - [`leak_font_name`] + the [`ui_font`]/[`display_font`]/[`mono_font`]
//!   helpers — `iced::Font::with_name` demands a `&'static str`, but our
//!   font family names are `String`s loaded from theme data. The only safe
//!   way to make a runtime string live forever is to leak it. The leak is
//!   deliberate, tiny (a few bytes per theme load), and isolated to this
//!   module so nothing else in the crate needs to think about it.

use saola_tokens::Theme;

/// Conversion from `saola_tokens::Color` into `iced::Color`.
///
/// `saola_tokens::Color` stores channels as `u8` (0–255);
/// `iced::Color` stores them as `f32` (0.0–1.0).
pub trait ColorExt {
    /// Convert into an `iced::Color` by dividing every channel by 255.
    fn into_iced(self) -> iced::Color;
}

impl ColorExt for saola_tokens::Color {
    fn into_iced(self) -> iced::Color {
        iced::Color {
            r: f32::from(self.r) / 255.0,
            g: f32::from(self.g) / 255.0,
            b: f32::from(self.b) / 255.0,
            a: f32::from(self.a) / 255.0,
        }
    }
}

/// Conversion from `saola_tokens::Shadow` into `iced::Shadow`.
///
/// Saola shadows never have a horizontal offset, so `offset_y` maps to
/// `Vector::new(0.0, offset_y)`.
pub trait ShadowExt {
    /// Convert into an `iced::Shadow`.
    fn into_iced(self) -> iced::Shadow;
}

impl ShadowExt for saola_tokens::Shadow {
    fn into_iced(self) -> iced::Shadow {
        iced::Shadow {
            color: self.color.into_iced(),
            offset: iced::Vector::new(0.0, self.offset_y),
            blur_radius: self.blur,
        }
    }
}

/// Build an [`iced::Theme`] from a Saola [`Theme`] via `iced::Theme::custom`.
///
/// iced 0.14's `theme::Palette` has exactly six fields —
/// `background`, `text`, `primary`, `success`, `warning`, `danger` — and
/// Saola has exactly three colors. The mapping:
///
/// | iced field   | Saola token          | why                                |
/// |--------------|----------------------|------------------------------------|
/// | `background` | `palette.ink`        | every shell surface is ink         |
/// | `text`       | `on_ink.primary`     | ivory text on ink                  |
/// | `primary`    | `palette.accent`     | terracotta = on/selected/live      |
/// | `success`    | `palette.accent`     | three colors, never a fourth —     |
/// | `warning`    | `palette.accent`     | any built-in widget style that     |
/// | `danger`     | `palette.accent`     | reads these stays inside the trio  |
///
/// Widgets styled with [`crate::style`] helpers never read this palette;
/// it exists so *unstyled* widgets and third-party code still land inside
/// the Saola identity instead of iced's stock blue/green/red.
pub fn to_iced_theme(theme: &Theme) -> iced::Theme {
    iced::Theme::custom(
        theme.name.clone(),
        iced::theme::Palette {
            background: theme.palette.ink.into_iced(),
            text: theme.on_ink.primary.into_iced(),
            primary: theme.palette.accent.into_iced(),
            success: theme.palette.accent.into_iced(),
            warning: theme.palette.accent.into_iced(),
            danger: theme.palette.accent.into_iced(),
        },
    )
}

/// Turn a runtime font-family name into the `&'static str` that
/// `iced::Font::with_name` requires, by leaking it.
///
/// Each call leaks the string's bytes for the lifetime of the process —
/// call once per theme load (e.g. in `main`, or cache the resulting
/// `Font`s in your application state), not once per frame.
pub fn leak_font_name(name: &str) -> &'static str {
    Box::leak(name.to_owned().into_boxed_str())
}

/// Map a CSS-numeric font weight (100..=900) onto iced's nearest named
/// [`iced::font::Weight`] variant.
fn iced_weight(weight: u16) -> iced::font::Weight {
    use iced::font::Weight;
    match weight {
        ..150 => Weight::Thin,
        150..250 => Weight::ExtraLight,
        250..350 => Weight::Light,
        350..450 => Weight::Normal,
        450..550 => Weight::Medium,
        550..650 => Weight::Semibold,
        650..750 => Weight::Bold,
        750..850 => Weight::ExtraBold,
        850.. => Weight::Black,
    }
}

/// The UI face (`typography.family_ui`, IBM Plex Sans in the built-in theme)
/// at `weight.medium` — the default for everything you scan: panel/bar text,
/// body, row titles (the style guide's hard rule is Sans **500** for all of
/// these; use [`ui_font_regular`] only for the explicitly-400 roles). Leaks
/// the family name — see [`leak_font_name`].
pub fn ui_font(theme: &Theme) -> iced::Font {
    iced::Font {
        weight: iced_weight(theme.typography.weight.medium),
        ..iced::Font::with_name(leak_font_name(&theme.typography.family_ui))
    }
}

/// The UI face at `weight.regular` — secondary row text, metadata, and the
/// launcher input, per the style guide's scale. Leaks the family name — see
/// [`leak_font_name`].
pub fn ui_font_regular(theme: &Theme) -> iced::Font {
    iced::Font {
        weight: iced_weight(theme.typography.weight.regular),
        ..iced::Font::with_name(leak_font_name(&theme.typography.family_ui))
    }
}

/// The display face (`typography.family_display`, IBM Plex Serif) at
/// `weight.display` as an `iced::Font`. Leaks the family name — see
/// [`leak_font_name`].
pub fn display_font(theme: &Theme) -> iced::Font {
    iced::Font {
        weight: iced_weight(theme.typography.weight.display),
        ..iced::Font::with_name(leak_font_name(&theme.typography.family_display))
    }
}

/// The mono face (`typography.family_mono`, IBM Plex Mono) at
/// `weight.regular` — file paths, hex values, terminal-style text. Keycaps
/// and uppercase section labels are Mono 500: use [`mono_font_medium`].
/// Leaks the family name — see [`leak_font_name`].
pub fn mono_font(theme: &Theme) -> iced::Font {
    iced::Font {
        weight: iced_weight(theme.typography.weight.regular),
        ..iced::Font::with_name(leak_font_name(&theme.typography.family_mono))
    }
}

/// The mono face at `weight.medium` — keycaps and uppercase section labels,
/// per the style guide's scale. Leaks the family name — see
/// [`leak_font_name`].
pub fn mono_font_medium(theme: &Theme) -> iced::Font {
    iced::Font {
        weight: iced_weight(theme.typography.weight.medium),
        ..iced::Font::with_name(leak_font_name(&theme.typography.family_mono))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saola_tokens::Color;

    #[test]
    fn weight_maps_to_nearest_variant() {
        use iced::font::Weight;
        assert_eq!(iced_weight(400), Weight::Normal);
        assert_eq!(iced_weight(500), Weight::Medium);
        assert_eq!(iced_weight(100), Weight::Thin);
        assert_eq!(iced_weight(900), Weight::Black);
        // Off-scale values snap to the nearest hundred's variant.
        assert_eq!(iced_weight(449), Weight::Normal);
        assert_eq!(iced_weight(450), Weight::Medium);
    }

    #[test]
    fn into_iced_scales_channels() {
        let c = Color {
            r: 255,
            g: 0,
            b: 51,
            a: 128,
        }
        .into_iced();
        assert_eq!(c.r, 1.0);
        assert_eq!(c.g, 0.0);
        assert!((c.b - 0.2).abs() < 1e-6);
        assert!((c.a - 128.0 / 255.0).abs() < 1e-6);
    }

    #[test]
    fn to_iced_theme_maps_ink_paper_accent() {
        let t = Theme::saola();
        let iced_theme = to_iced_theme(&t);
        let p = iced_theme.palette();
        assert_eq!(p.background, t.palette.ink.into_iced());
        assert_eq!(p.text, t.on_ink.primary.into_iced());
        assert_eq!(p.primary, t.palette.accent.into_iced());
        // Three colors, never a fourth: the status colors are all accent.
        assert_eq!(p.success, t.palette.accent.into_iced());
        assert_eq!(p.warning, t.palette.accent.into_iced());
        assert_eq!(p.danger, t.palette.accent.into_iced());
    }

    #[test]
    fn leaked_font_name_is_static_and_equal() {
        let name: &'static str = leak_font_name("IBM Plex Sans");
        assert_eq!(name, "IBM Plex Sans");
    }
}
