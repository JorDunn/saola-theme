//! RGBA color type with a hex-string wire format.
//!
//! Theme files store colors as `"#RRGGBB"` / `"#RRGGBBAA"` strings — the
//! same shape used by CSS, GTK, and most other theme formats — rather than
//! as TOML tables like `{ r = 255, g = 0, b = 0 }`. That means `Color` needs
//! *hand-written* `Serialize`/`Deserialize` impls instead of `#[derive]`:
//! derive only knows how to map struct fields to format fields 1:1, it can't
//! encode "pack these four numbers into one string."

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// An RGBA color: four 8-bit channels, red/green/blue/alpha.
///
/// Fields are `pub`. In many languages you'd hide them behind getters "just
/// in case", but in Rust that convention only pays off when there's an
/// invariant to protect (e.g. "these two fields must always sum to 100").
/// Here every `(r, g, b, a)` combination the type system allows (any four
/// `u8`s) is already a valid color, so a constructor-only API would just be
/// ceremony. `parse_hex` exists because *that* input (an arbitrary string)
/// does need validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Why `Color` derives `Copy`: it's four bytes, the same size as a `u32`.
/// Types that small should be `Copy` so callers can pass them around by
/// value without thinking about borrowing — `fn lighten(self, ...)` below
/// takes `self` (not `&self`) for the same reason: consuming and returning
/// a new `Color` is exactly as cheap as mutating in place, and it reads
/// better at call sites (`color.lighten(0.2)` rather than needing `mut`).
impl Color {
    /// Fully opaque black.
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    /// Fully opaque white.
    pub const WHITE: Color = Color::rgb(255, 255, 255);

    /// Build a fully opaque color (`a = 255`) from RGB channels.
    ///
    /// `const fn` so palette tables in later stages can define colors as
    /// `const` values evaluated at compile time, not just at runtime.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 255 }
    }

    /// Build a color from all four channels.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    /// Parse a hex color string: `"#RRGGBB"` (opaque) or `"#RRGGBBAA"`.
    ///
    /// Case-insensitive (`"#FF0000"` and `"#ff0000"` both work); the `#`
    /// prefix is required.
    pub fn parse_hex(s: &str) -> Result<Self, ColorParseError> {
        let hex = s.strip_prefix('#').ok_or(ColorParseError::MissingHash)?;

        // Validate *before* slicing by byte index below. `str` slicing
        // panics if a byte index doesn't land on a UTF-8 character
        // boundary (e.g. slicing into the middle of a multi-byte emoji).
        // Checking `is_ascii_hexdigit()` first for every byte guarantees
        // every char is one ASCII byte, so `hex.len()` is both the byte
        // count and the char count, and every 2-byte slice below is safe.
        if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(ColorParseError::InvalidDigits);
        }

        // Now infallible: every byte in `slice` was already checked above.
        let channel =
            |slice: &str| u8::from_str_radix(slice, 16).expect("validated ascii hex digit pair");

        match hex.len() {
            6 => Ok(Color::rgb(
                channel(&hex[0..2]),
                channel(&hex[2..4]),
                channel(&hex[4..6]),
            )),
            8 => Ok(Color::rgba(
                channel(&hex[0..2]),
                channel(&hex[2..4]),
                channel(&hex[4..6]),
                channel(&hex[6..8]),
            )),
            n => Err(ColorParseError::WrongLength(n)),
        }
    }

    /// Return a copy of this color with the alpha channel replaced.
    pub const fn with_alpha(self, a: u8) -> Self {
        Color { a, ..self }
    }

    /// Mix this color toward white by `amount` (clamped to `0.0..=1.0`).
    ///
    /// `amount = 0.0` returns the color unchanged; `amount = 1.0` returns
    /// white. Alpha is left untouched — lightening a color shouldn't make
    /// it more or less transparent.
    pub fn lighten(self, amount: f32) -> Self {
        self.mix_toward(255, amount)
    }

    /// Mix this color toward black by `amount` (clamped to `0.0..=1.0`).
    pub fn darken(self, amount: f32) -> Self {
        self.mix_toward(0, amount)
    }

    /// Linear-interpolate each of r/g/b toward `target` by `amount`.
    fn mix_toward(self, target: u8, amount: f32) -> Self {
        let amount = amount.clamp(0.0, 1.0);
        let mix = |channel: u8| -> u8 {
            let channel = channel as f32;
            let target = target as f32;
            (channel + (target - channel) * amount).round() as u8
        };
        Color {
            r: mix(self.r),
            g: mix(self.g),
            b: mix(self.b),
            a: self.a,
        }
    }

    /// Composite `self` *over* `base` using the standard "source-over"
    /// alpha-blending formula (the same operator CSS, SVG, and most 2D
    /// renderers call "over"). `self` is the translucent color on top;
    /// `base` is what's underneath.
    ///
    /// Why this exists: Saola's on-surface roles (e.g. `on_ink.secondary`)
    /// are translucent by design — they're `paper` or `ink` stepped down
    /// with alpha, so a single set of roles works on any tinted background.
    /// But you can't measure the contrast of a *translucent* color against
    /// anything meaningful; contrast (`relative_luminance`) only makes
    /// sense for solid colors. `over` answers "what solid color does this
    /// translucent role actually paint, once it's sitting on its surface?"
    /// so contrast tests can check *that* against the surface.
    ///
    /// When `base` is fully opaque (`base.a == 255`), the result is always
    /// fully opaque too — painting anything, however transparent, on top of
    /// a solid backdrop still yields a solid color underneath the brush.
    pub fn over(self, base: Color) -> Color {
        // Work in normalized 0.0..=1.0 space; alpha compositing math reads
        // far more naturally there than in 0..=255 integers.
        let src_a = self.a as f64 / 255.0;
        let dst_a = base.a as f64 / 255.0;
        let out_a = src_a + dst_a * (1.0 - src_a);

        // Each output channel is a weighted average of the source and
        // destination channels, weighted by how much each contributes to
        // the final (possibly still partial) coverage `out_a`. This is the
        // "un-premultiplied" form of the Porter-Duff `over` operator.
        let mix = |src_c: u8, dst_c: u8| -> u8 {
            if out_a <= 0.0 {
                // Fully transparent result (both colors were transparent) —
                // channel values are meaningless, so just report zero
                // rather than dividing by zero below.
                return 0;
            }
            let src_c = src_c as f64 / 255.0;
            let dst_c = dst_c as f64 / 255.0;
            let out_c = (src_c * src_a + dst_c * dst_a * (1.0 - src_a)) / out_a;
            (out_c * 255.0).round() as u8
        };

        Color {
            r: mix(self.r, base.r),
            g: mix(self.g, base.g),
            b: mix(self.b, base.b),
            a: (out_a * 255.0).round() as u8,
        }
    }

    /// WCAG 2.x relative luminance of this color, ignoring alpha.
    ///
    /// Used to compute contrast ratios (`(L1 + 0.05) / (L2 + 0.05)`) for
    /// accessibility checks in later stages. Formula from
    /// <https://www.w3.org/TR/WCAG21/#dfn-relative-luminance>.
    pub fn relative_luminance(&self) -> f64 {
        // Each channel is first normalized to 0.0..=1.0 ("gamma-encoded"
        // sRGB), then converted to linear light before weighting — sRGB
        // isn't a linear scale, so weighting the raw 0..=255 values would
        // give a physically wrong result.
        let linearize = |channel: u8| -> f64 {
            let c = channel as f64 / 255.0;
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let r = linearize(self.r);
        let g = linearize(self.g);
        let b = linearize(self.b);
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }
}

/// Formats as lowercase `"#rrggbb"` when fully opaque, `"#rrggbbaa"`
/// otherwise. This is also what `Serialize` uses (via `to_string` below),
/// so `Display`'s output and the TOML wire format are always identical.
impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == 255 {
            write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            write!(
                f,
                "#{:02x}{:02x}{:02x}{:02x}",
                self.r, self.g, self.b, self.a
            )
        }
    }
}

/// Reasons a hex string failed to parse into a [`Color`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ColorParseError {
    #[error("hex color must start with '#'")]
    MissingHash,
    #[error("hex color must have 6 or 8 hex digits after '#', got {0}")]
    WrongLength(usize),
    #[error("hex color contains a non-hex-digit character")]
    InvalidDigits,
}

/// Hand-written `Serialize`: write the color as its `Display` string
/// (`"#rrggbb"` / `"#rrggbbaa"`) rather than as a `{ r, g, b, a }` table.
///
/// `#[derive(Serialize)]` isn't an option here — derive only knows how to
/// map Rust fields onto format fields one-to-one; it has no way to express
/// "pack four numbers into one string". Implementing the trait by hand is
/// how you plug custom formats like this into serde.
impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Hand-written `Deserialize`: read a string and parse it with
/// [`Color::parse_hex`], turning parse errors into serde errors so callers
/// (e.g. `toml::from_str`) get a proper "line N: ..." style error message
/// instead of a panic.
///
/// We deserialize into an owned `String` rather than borrowing `&str` from
/// the input. Borrowing would be a little cheaper, but not every format can
/// hand back a borrowed slice — e.g. a TOML string containing an escape
/// sequence (`"\\n"`) has to be unescaped into a *new* buffer, so the
/// deserializer can't just point at the original bytes. Requiring `String`
/// keeps this impl correct for any serde format, not just the ones whose
/// input happens to need no unescaping.
impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Color::parse_hex(&s).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rrggbb_uppercase_and_lowercase() {
        assert_eq!(
            Color::parse_hex("#FF0080").unwrap(),
            Color::rgb(255, 0, 128)
        );
        assert_eq!(
            Color::parse_hex("#ff0080").unwrap(),
            Color::rgb(255, 0, 128)
        );
        assert_eq!(
            Color::parse_hex("#Ff0080").unwrap(),
            Color::rgb(255, 0, 128)
        );
    }

    #[test]
    fn parses_rrggbbaa() {
        assert_eq!(
            Color::parse_hex("#FF008080").unwrap(),
            Color::rgba(255, 0, 128, 128)
        );
        assert_eq!(
            Color::parse_hex("#00000000").unwrap(),
            Color::rgba(0, 0, 0, 0)
        );
    }

    #[test]
    fn rejects_missing_hash() {
        assert_eq!(
            Color::parse_hex("ff0080"),
            Err(ColorParseError::MissingHash)
        );
    }

    #[test]
    fn rejects_wrong_length() {
        assert_eq!(
            Color::parse_hex("#fff"),
            Err(ColorParseError::WrongLength(3))
        );
        assert_eq!(
            Color::parse_hex("#ff00800"),
            Err(ColorParseError::WrongLength(7))
        );
        assert_eq!(Color::parse_hex("#"), Err(ColorParseError::WrongLength(0)));
    }

    #[test]
    fn rejects_bad_characters() {
        assert_eq!(
            Color::parse_hex("#gg0080"),
            Err(ColorParseError::InvalidDigits)
        );
        assert_eq!(
            Color::parse_hex("#ff00 0"),
            Err(ColorParseError::InvalidDigits)
        );
        // Non-ASCII input must be rejected, not panic while slicing.
        assert_eq!(
            Color::parse_hex("#ffµµµµ"),
            Err(ColorParseError::InvalidDigits)
        );
    }

    #[test]
    fn display_emits_lowercase_and_omits_alpha_when_opaque() {
        assert_eq!(Color::rgb(255, 0, 128).to_string(), "#ff0080");
        assert_eq!(Color::rgba(255, 0, 128, 128).to_string(), "#ff008080");
    }

    #[test]
    fn parse_and_display_round_trip() {
        for s in ["#000000", "#ffffff", "#123456", "#abcdef01"] {
            let color = Color::parse_hex(s).unwrap();
            assert_eq!(color.to_string(), s);
        }
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Wrapper {
        color: Color,
    }

    #[test]
    fn serde_round_trips_through_toml() {
        let original = Wrapper {
            color: Color::rgba(255, 0, 128, 200),
        };
        let toml_str = toml::to_string(&original).unwrap();
        assert_eq!(toml_str, "color = \"#ff0080c8\"\n");

        let parsed: Wrapper = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn serde_round_trips_opaque_color_through_toml() {
        let original = Wrapper {
            color: Color::rgb(10, 20, 30),
        };
        let toml_str = toml::to_string(&original).unwrap();
        assert_eq!(toml_str, "color = \"#0a141e\"\n");

        let parsed: Wrapper = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn serde_rejects_invalid_hex_string() {
        let bad_toml = "color = \"not-a-color\"\n";
        let result: Result<Wrapper, _> = toml::from_str(bad_toml);
        assert!(result.is_err());
    }

    #[test]
    fn with_alpha_only_changes_alpha() {
        let color = Color::rgb(1, 2, 3).with_alpha(42);
        assert_eq!(color, Color::rgba(1, 2, 3, 42));
    }

    #[test]
    fn lighten_zero_is_identity_and_one_is_white() {
        let color = Color::rgb(10, 20, 30);
        assert_eq!(color.lighten(0.0), color);
        assert_eq!(color.lighten(1.0), Color::rgb(255, 255, 255));
    }

    #[test]
    fn darken_zero_is_identity_and_one_is_black() {
        let color = Color::rgb(10, 20, 30);
        assert_eq!(color.darken(0.0), color);
        assert_eq!(color.darken(1.0), Color::rgb(0, 0, 0));
    }

    #[test]
    fn lighten_and_darken_preserve_alpha() {
        let color = Color::rgba(10, 20, 30, 99);
        assert_eq!(color.lighten(0.5).a, 99);
        assert_eq!(color.darken(0.5).a, 99);
    }

    #[test]
    fn over_opaque_base_yields_opaque_result() {
        let translucent = Color::rgba(255, 255, 240, 184);
        let result = translucent.over(Color::BLACK);
        assert_eq!(result.a, 255);
    }

    #[test]
    fn over_fully_transparent_source_returns_base_unchanged() {
        let transparent = Color::rgba(10, 20, 30, 0);
        let base = Color::rgb(100, 150, 200);
        assert_eq!(transparent.over(base), base);
    }

    #[test]
    fn over_fully_opaque_source_returns_source_unchanged() {
        let opaque = Color::rgb(10, 20, 30);
        let base = Color::rgb(100, 150, 200);
        assert_eq!(opaque.over(base), opaque);
    }

    #[test]
    fn over_half_alpha_white_over_black_is_mid_gray() {
        let half_white = Color::WHITE.with_alpha(128);
        let result = half_white.over(Color::BLACK);
        assert_eq!(result.a, 255);
        // 128/255 is not exactly 0.5, so allow a 1-unit rounding tolerance.
        assert!((result.r as i16 - 128).abs() <= 1);
        assert_eq!(result.r, result.g);
        assert_eq!(result.g, result.b);
    }

    #[test]
    fn relative_luminance_bounds() {
        assert!((Color::WHITE.relative_luminance() - 1.0).abs() < 1e-9);
        assert!((Color::BLACK.relative_luminance() - 0.0).abs() < 1e-9);

        // Luminance should increase monotonically from black to white for a
        // simple gray ramp.
        let dark_gray = Color::rgb(64, 64, 64).relative_luminance();
        let light_gray = Color::rgb(192, 192, 192).relative_luminance();
        assert!(dark_gray < light_gray);
    }
}
