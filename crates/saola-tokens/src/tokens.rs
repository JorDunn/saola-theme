//! Structured, non-color token groups: typography, geometry, shadows,
//! motion durations, and the terminal palette.
//!
//! Everything here is plain data with no CSS strings — `design/saola-tokens.json`
//! encodes shadows as CSS `box-shadow` strings and durations/easing as CSS
//! transition syntax, but this crate parses that once, by hand, into typed
//! fields (`offset_y: f32`, `duration: u32`, etc.) so consumers never touch
//! a string to read a number.

use crate::Color;
use serde::{Deserialize, Serialize};

/// Font weight, following the CSS `font-weight` numeric scale (400 =
/// regular, 500 = medium). `u16` rather than `u32`: weights only ever run
/// 100..=900, and a smaller type documents that range at a glance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FontWeights {
    pub regular: u16,
    pub medium: u16,
    pub display: u16,
}

impl Default for FontWeights {
    fn default() -> Self {
        FontWeights {
            regular: 400,
            medium: 500,
            display: 400,
        }
    }
}

/// The full type-size scale, in logical pixels (`f32`, since iced and most
/// GUI toolkits size text in fractional points/px, not whole units).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FontSizes {
    pub lock_clock: f32,
    pub screen_title: f32,
    pub panel_heading: f32,
    pub dialog_title: f32,
    pub section_heading: f32,
    pub launcher_input: f32,
    pub body: f32,
    pub bar: f32,
    pub secondary: f32,
    pub meta: f32,
    pub label: f32,
    pub keycap: f32,
}

impl Default for FontSizes {
    fn default() -> Self {
        FontSizes {
            lock_clock: 168.0,
            screen_title: 44.0,
            panel_heading: 22.0,
            dialog_title: 24.0,
            section_heading: 20.0,
            launcher_input: 22.0,
            body: 13.5,
            bar: 13.0,
            secondary: 12.5,
            meta: 12.0,
            label: 11.0,
            keycap: 11.0,
        }
    }
}

/// The three type families and the full size/weight scale built from them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Typography {
    /// All interface text — everything you scan. IBM Plex Sans.
    pub family_ui: String,
    /// Display only: wordmark, clock, panel headings, dialog titles. Never
    /// in the panel bar. IBM Plex Serif.
    pub family_display: String,
    /// Keycaps, paths, hex, terminal, D-Bus names. IBM Plex Mono.
    pub family_mono: String,
    pub weight: FontWeights,
    pub size: FontSizes,
    /// Hard floor: nothing in the panel may render smaller than this,
    /// regardless of which named size a given label happens to use.
    pub minimum_bar_size: f32,
}

impl Default for Typography {
    fn default() -> Self {
        Typography {
            family_ui: "IBM Plex Sans".to_string(),
            family_display: "IBM Plex Serif".to_string(),
            family_mono: "IBM Plex Mono".to_string(),
            weight: FontWeights::default(),
            size: FontSizes::default(),
            minimum_bar_size: 13.0,
        }
    }
}

/// Corner radii, in logical pixels. Saola shapes are always either a pill
/// (`radius.pill`) or an over-rounded rectangle — never a sharp corner.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Radii {
    /// Buttons, chips, inputs, toggles, list rows — fully round ends.
    pub pill: f32,
    pub popover: f32,
    pub popover_wide: f32,
    pub window: f32,
    pub card: f32,
    pub inset: f32,
    pub tile: f32,
    /// Checkboxes are the one deliberately-not-round shape (7px, not pill).
    pub checkbox: f32,
    pub selection: f32,
}

impl Default for Radii {
    fn default() -> Self {
        Radii {
            pill: 999.0,
            popover: 30.0,
            popover_wide: 34.0,
            window: 24.0,
            card: 26.0,
            inset: 20.0,
            tile: 13.0,
            checkbox: 7.0,
            selection: 6.0,
        }
    }
}

/// Layout dimensions, hit targets, and icon sizes, all in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sizes {
    pub panel_pill: f32,
    pub panel_bar: f32,
    pub panel_margin_islands: f32,
    pub panel_margin_ledger: f32,
    pub island_gap: f32,
    /// Gap between an icon and its label *inside* a pill (`island_gap` is
    /// the gap between pills).
    pub pill_gap: f32,
    /// Maximum width of a text-bearing bar pill (the media "title — artist"
    /// pill) before its label truncates.
    pub pill_max_width: f32,
    /// Minimap dash geometry: the panel's centre module maps the niri column
    /// strip as one dash per column. A rest dash is a **round dot** (width
    /// equals `dash_height`, so `radii.pill` closes it into a circle); the
    /// focused one keeps that height but is about twice as wide (per concept
    /// listing 2a — corrected from an earlier thin-rule geometry, Jordan's
    /// decision 2026-07-31); off-screen columns are narrower stubs at each
    /// end, reading as a dot squeezed in at the strip's edge.
    pub dash_height: f32,
    pub dash_width_rest: f32,
    pub dash_width_focused: f32,
    pub dash_width_stub: f32,
    pub dash_gap: f32,
    pub popover_top: f32,
    pub popover_width: f32,
    /// Content padding inside a popover panel (spec §6: "20–22px padding").
    pub popover_padding: f32,
    pub notification_centre_width: f32,
    pub launcher_width: f32,
    pub notification_card_width: f32,
    /// Menu/list row height (tray menus, popover lists).
    pub list_row: f32,
    pub hit_target_bar: f32,
    pub hit_target_touch: f32,
    pub icon_bar: f32,
    pub icon_row: f32,
    pub icon_menu: f32,
    pub icon_bare: f32,
    /// Lucide icon stroke width, held constant at every icon size.
    pub icon_stroke: f32,
    pub window_border: f32,
    pub window_header: f32,
    /// Height of the compact media pill *inside* the 48px ledger bar
    /// (smaller than `panel_pill`, which is a free-standing islands pill).
    pub panel_pill_media: f32,
    /// Height of the compact clock pill inside the ledger bar.
    pub panel_pill_clock: f32,
    /// Vertical inset of the floating ledger bar from the screen edge
    /// (`panel_margin_ledger` is the horizontal inset).
    pub panel_margin_ledger_top: f32,
    /// Gap between elements along the ledger bar.
    pub bar_element_gap: f32,
    /// Gap between readouts inside the bar's status cluster.
    pub bar_cluster_gap: f32,
    /// Gap between an icon and its value inside one status readout
    /// (tighter than `pill_gap`, which is the icon↔label gap in a pill).
    pub bar_icon_gap: f32,
    /// Maximum width of the media pill's title text before it truncates
    /// (`pill_max_width` caps the whole pill; this caps just the title).
    pub media_title_max_width: f32,
}

impl Default for Sizes {
    fn default() -> Self {
        Sizes {
            panel_pill: 40.0,
            panel_bar: 48.0,
            panel_margin_islands: 26.0,
            panel_margin_ledger: 20.0,
            island_gap: 10.0,
            pill_gap: 8.0,
            pill_max_width: 280.0,
            dash_height: 16.0,
            dash_width_rest: 16.0,
            dash_width_focused: 34.0,
            dash_width_stub: 10.0,
            dash_gap: 5.0,
            popover_top: 72.0,
            popover_width: 440.0,
            popover_padding: 20.0,
            notification_centre_width: 460.0,
            launcher_width: 640.0,
            notification_card_width: 440.0,
            list_row: 38.0,
            hit_target_bar: 40.0,
            hit_target_touch: 44.0,
            // 16 (bumped from 15, 2026-08-01): the bar readouts went
            // icon-only, so the glyph is the whole readout and earns the top
            // of the style guide's 15–16 range — the leveled details
            // (battery fill bars, Wi-Fi arcs) are what the extra pixel buys.
            icon_bar: 16.0,
            icon_row: 16.0,
            icon_menu: 19.0,
            icon_bare: 32.0,
            icon_stroke: 2.75,
            window_border: 2.0,
            window_header: 46.0,
            panel_pill_media: 30.0,
            panel_pill_clock: 32.0,
            panel_margin_ledger_top: 18.0,
            bar_element_gap: 14.0,
            bar_cluster_gap: 15.0,
            bar_icon_gap: 7.0,
            media_title_max_width: 190.0,
        }
    }
}

/// One drop shadow: a vertical offset and blur radius (the horizontal
/// offset is always `0` in Saola, so it isn't a field), plus its color.
///
/// This is the parsed form of a CSS `box-shadow: 0 <offset_y>px <blur>px
/// <color>` string from `design/saola-tokens.json` — parsed once, by hand,
/// rather than carried around as a string every consumer has to re-parse.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Shadow {
    pub color: Color,
    pub offset_y: f32,
    pub blur: f32,
}

/// `#[serde(default)]` on a struct requires that struct to implement
/// `Default` (serde derive checks this at compile time), even though
/// `Shadow` only ever appears nested inside [`Shadows`] here — this default
/// is the `popover` shadow's values, an arbitrary-but-valid fallback for a
/// partial `[shadows.something]` table missing individual fields.
impl Default for Shadow {
    fn default() -> Self {
        Shadow {
            color: Color::rgba(0x0C, 0x0A, 0x00, 128), // 0.50
            offset_y: 18.0,
            blur: 48.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Shadows {
    pub popover: Shadow,
    pub window: Shadow,
    pub overlay: Shadow,
}

impl Default for Shadows {
    fn default() -> Self {
        // Colors are ink at `round(cssAlpha * 255)`, from the JSON's
        // `shadow.*` CSS box-shadow strings.
        Shadows {
            popover: Shadow {
                color: Color::rgba(0x0C, 0x0A, 0x00, 128), // 0.50
                offset_y: 18.0,
                blur: 48.0,
            },
            window: Shadow {
                color: Color::rgba(0x0C, 0x0A, 0x00, 128), // 0.50
                offset_y: 24.0,
                blur: 64.0,
            },
            overlay: Shadow {
                color: Color::rgba(0x0C, 0x0A, 0x00, 153), // 0.60
                offset_y: 24.0,
                blur: 64.0,
            },
        }
    }
}

/// Animation durations, in milliseconds. `u32` rather than `std::time::Duration`:
/// this crate is pure data with no GUI/runtime dependency, and a plain
/// integer round-trips through TOML directly, whereas `Duration` would need
/// its own serde adapter. Consumers convert to `Duration::from_millis(...)`
/// at the point of use.
///
/// Easing curves and multi-step transform choreography (the JSON's
/// `"easing"` and `"from"` string fields) are deliberately out of scope for
/// v0.1.
///
/// This struct derives `PartialEq` but not `Eq` (unlike most token groups,
/// like most of the *color* groups): `breathe_min_opacity` is an `f32`, and
/// floats have no total equality in Rust. [`Radii`] and [`Sizes`] are
/// `PartialEq`-only for the same reason.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Motion {
    pub hover: u32,
    pub popover: u32,
    pub wake: u32,
    pub toast_in: u32,
    pub toast_idle: u32,
    pub toast_out: u32,
    pub toast_total: u32,
    pub toast_max_stack: u8,
    /// One full breath of a session-status semaphore dot, in milliseconds:
    /// the time for a breathing dot to fade down and back up *once*
    /// (dim → bright → dim), not a half cycle. The panel drives it as a
    /// smooth loop for the `status_working` / `status_subagents` states and
    /// holds the other three steady.
    ///
    /// This is the only looping animation in Saola, and it is slow on
    /// purpose — 2.4 s reads as breathing rather than blinking, and stays
    /// below the flash thresholds that make motion an accessibility
    /// problem. The opacity range it sweeps
    /// (`breathe_min_opacity`..=1.0) lives beside it rather than being
    /// hardcoded by the consumer.
    pub breathe: u32,
    /// The dimmest point of a breath, as an opacity multiplier on the dot's
    /// fill (the brightest point is always 1.0). Not zero: a breathing dot
    /// must never vanish, or the readout would look like it is blinking out
    /// of existence rather than idling.
    pub breathe_min_opacity: f32,
}

impl Default for Motion {
    fn default() -> Self {
        Motion {
            hover: 140,
            popover: 160,
            wake: 450,
            toast_in: 350,
            toast_idle: 5000,
            toast_out: 1000,
            toast_total: 6350,
            toast_max_stack: 3,
            breathe: 2400,
            breathe_min_opacity: 0.45,
        }
    }
}

/// The 8 standard ANSI colors (used twice: once for `normal`, once for the
/// brighter `bright` variants).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AnsiColors {
    pub black: Color,
    pub red: Color,
    pub green: Color,
    pub yellow: Color,
    pub blue: Color,
    pub magenta: Color,
    pub cyan: Color,
    pub white: Color,
}

impl Default for AnsiColors {
    fn default() -> Self {
        // "normal" ANSI row. `bright` is a separate Default below.
        AnsiColors {
            black: Color::rgb(0x0C, 0x0A, 0x00),
            red: Color::rgb(0xC0, 0x5B, 0x3C),
            green: Color::rgb(0x7A, 0x8A, 0x5E),
            yellow: Color::rgb(0xC6, 0x91, 0x39),
            blue: Color::rgb(0x5E, 0x7A, 0x8A),
            magenta: Color::rgb(0x9E, 0x6B, 0x7A),
            cyan: Color::rgb(0x6E, 0x91, 0x88),
            white: Color::rgb(0xE8, 0xE0, 0xCE),
        }
    }
}

impl AnsiColors {
    fn bright() -> Self {
        AnsiColors {
            black: Color::rgb(0x3A, 0x34, 0x2A),
            red: Color::rgb(0xE0, 0x7A, 0x57),
            green: Color::rgb(0x9C, 0xB0, 0x77),
            yellow: Color::rgb(0xE0, 0xB2, 0x5C),
            blue: Color::rgb(0x7C, 0x9C, 0xAE),
            magenta: Color::rgb(0xC0, 0x8A, 0x99),
            cyan: Color::rgb(0x8F, 0xB3, 0xA8),
            white: Color::rgb(0xFF, 0xFF, 0xF0),
        }
    }
}

/// Terminal palette — pure data feeding future exporters (an Alacritty
/// config, other terminal emulators' theme formats, etc). Not consumed by
/// `saola-theme`'s iced widgets. `design/saola-alacritty.toml` shows the
/// target shape of one such exporter's output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Terminal {
    pub background: Color,
    pub foreground: Color,
    pub cursor: Color,
    pub cursor_text: Color,
    pub selection: Color,
    pub normal: AnsiColors,
    pub bright: AnsiColors,
}

impl Default for Terminal {
    fn default() -> Self {
        Terminal {
            background: Color::rgb(0x0C, 0x0A, 0x00),
            foreground: Color::rgb(0xE8, 0xE0, 0xCE),
            cursor: Color::rgb(0xFF, 0xFF, 0xF0),
            cursor_text: Color::rgb(0x0C, 0x0A, 0x00),
            // Terracotta at 30% — exporters targeting formats without alpha
            // (Alacritty) pre-composite this over `background` with
            // `Color::over`.
            selection: Color::rgba(0xC6, 0x71, 0x39, 77), // 0.30
            normal: AnsiColors::default(),
            bright: AnsiColors::bright(),
        }
    }
}
