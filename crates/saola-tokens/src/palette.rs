//! The Saola identity: three colors (ink, paper, terracotta accent) plus the
//! alpha-stepped roles derived from them for use *on* each surface.
//!
//! Saola deliberately has no conventional dark/light theme pair. Instead,
//! text/divider/fill roles are the identity colors stepped down with alpha,
//! once for use on an ink surface and once for use on a paper surface. The
//! [`Surface`] enum is the axis that varies, not a `Theme` variant.

use crate::Color;
use serde::{Deserialize, Serialize};

/// The three-color identity, resolved to concrete, fully opaque colors.
///
/// `accent_light` and `accent_dark` are *not* independent brand colors —
/// they're accent-tinted text colors for use in one context each:
/// `accent_light` is accent-colored text *on ink*, `accent_dark` is
/// accent-colored text *on paper*. Never introduce a fourth color.
///
/// # The session-status exception
///
/// The `status_*` fields at the bottom are the **one documented exception**
/// to "three colors, never a fourth" (Jordan's decision, 2026-07-31). They
/// are a *semaphore* family: five hues that encode the state of a Claude
/// Code session at a glance, drawn as ~16 px round dots directly on ink in
/// the panel. They exist because that readout has five mutually exclusive
/// states which must be told apart *pre-attentively* — one accent and a set
/// of alpha steps can distinguish "live" from "not live", but not
/// "generating" from "needs you" from "done".
///
/// The scope of the exception is deliberately narrow, and nothing widens it
/// without another explicit decision:
///
/// - **Status semaphores only.** Never a control fill, never a pill, never
///   text, never a border, never a hover state. The one rule (ivory fill =
///   at rest, terracotta fill = on/selected/live) still governs every
///   interactive element in the system, unchanged.
/// - There is still **no danger/success/warning color**:
///   `status_attention` is red because it is one arm of a five-position
///   semaphore, not because red means "error". Severity elsewhere is carried
///   by wording, as before.
/// - Every value stays in the muted, warm register that sits next to ivory
///   and terracotta — none of these is a saturated "OS notification" hue.
///
/// The style helper that paints them is
/// `saola_theme::style::container::status_dot`, which re-states this
/// exception at the point of use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Palette {
    /// Every shell surface (background of ink-context UI).
    pub ink: Color,
    /// A control at rest — off, unselected, available; light window backgrounds.
    pub paper: Color,
    /// On, selected, focused, live.
    pub accent: Color,
    /// Accent-colored *text on ink* only.
    pub accent_light: Color,
    /// Accent-colored *text on paper* only.
    pub accent_dark: Color,
    /// Semaphore: the session is generating. Amber-gold, and deliberately
    /// pushed off `accent`/`accent_light`'s hue so a working dot is never
    /// mistaken for ordinary accent text on the bar. Breathes.
    pub status_working: Color,
    /// Semaphore: subagents are running under the session. Muted violet —
    /// the one cool-ish hue in the family, since "something else is running
    /// on your behalf" should not read as another shade of "you are busy".
    /// Breathes.
    pub status_subagents: Color,
    /// Semaphore: the session is blocked on Jordan (a prompt, a permission,
    /// a question). Muted brick red. Steady — the dot that wants attention
    /// does not move, so movement on the bar always means "still running".
    pub status_attention: Color,
    /// Semaphore: the session finished and its output is awaiting review.
    /// Muted steel blue. Steady.
    pub status_done: Color,
    /// Semaphore: a session is open with nothing happening in it. Sage
    /// green, the quietest of the five. Steady.
    pub status_idle: Color,
}

/// Hand-written rather than `#[derive(Default)]`: derive would give every
/// field `Color::default()` (transparent black), which is not a real Saola
/// color. `#[serde(default)]` on the struct (above) means "if a field is
/// missing from a partial TOML table, take it from `Palette::default()`" —
/// so this impl has to already hold the *real* values, not a placeholder.
impl Default for Palette {
    fn default() -> Self {
        Palette {
            ink: Color::rgb(0x0C, 0x0A, 0x00),
            paper: Color::rgb(0xFF, 0xFF, 0xF0),
            accent: Color::rgb(0xC6, 0x71, 0x39),
            accent_light: Color::rgb(0xF6, 0xA0, 0x6B),
            accent_dark: Color::rgb(0x8C, 0x49, 0x1A),
            // The session-status semaphore family (see the struct docs).
            // Each was picked to clear 3:1 contrast against ink as a small
            // non-text mark, to stay mutually distinguishable at 16 px
            // (CIELAB ΔE ≥ 33 between any two of them), and — for
            // `status_working` — to sit far enough from `accent`/
            // `accent_light` (ΔE ≥ 22) not to read as terracotta.
            status_working: Color::rgb(0xDD, 0xA2, 0x3F),
            status_subagents: Color::rgb(0xA4, 0x81, 0xC7),
            status_attention: Color::rgb(0xD0, 0x54, 0x4A),
            status_done: Color::rgb(0x6D, 0x9E, 0xC6),
            status_idle: Color::rgb(0x82, 0xA8, 0x78),
        }
    }
}

/// The alpha-stepped roles read by style helpers: text emphasis levels,
/// dividers, and control-fill steps.
///
/// This struct is the *union* of the JSON's `onInk` and `onPaper` field
/// sets. The two contexts don't define exactly the same fields (`onInk` has
/// no `track`, `onPaper` has no `fillStrong`) — but every style helper needs
/// to be able to ask *any* role of *either* surface, so both
/// [`OnSurface::on_ink`] and [`OnSurface::on_paper`] fill every field here.
/// The one field missing from each source is filled from its nearest
/// neighbor: `on_ink.track` reuses `on_ink.fill_strong` (its strongest
/// fill), and `on_paper.fill_strong` reuses `on_paper.track` (its track
/// value).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct OnSurface {
    /// Full-emphasis text/icons. Opaque — this *is* the opposite identity color.
    pub primary: Color,
    /// Most body text.
    pub secondary: Color,
    /// De-emphasized text, captions.
    pub tertiary: Color,
    /// Placeholder-level text.
    pub quaternary: Color,
    /// Disabled controls' text/icons.
    pub disabled: Color,
    /// Hairline dividers.
    pub divider: Color,
    /// Lightest control fill — hover state on an otherwise bare element.
    pub fill_subtle: Color,
    /// A control at rest (on paper) or a low fill step (on ink).
    pub fill: Color,
    /// The strongest non-accent fill step (pressed states, etc).
    pub fill_strong: Color,
    /// Track background for sliders/progress bars.
    pub track: Color,
}

impl OnSurface {
    /// Ivory (`paper`) stepped with alpha, for use on ink surfaces.
    pub fn on_ink() -> Self {
        // Alpha channels below are `round(cssAlpha * 255)` from
        // `design/saola-tokens.json`'s `color.onInk` object.
        let step = |a: u8| Color::rgba(0xFF, 0xFF, 0xF0, a);
        OnSurface {
            primary: Color::rgb(0xFF, 0xFF, 0xF0), // rgba(..., 1.00) -> opaque
            secondary: step(184),                  // 0.72
            tertiary: step(140),                   // 0.55
            quaternary: step(102),                 // 0.40
            disabled: step(89),                    // 0.35
            divider: step(31),                     // 0.12
            fill_subtle: step(18),                 // 0.07
            fill: step(31),                        // 0.12
            fill_strong: step(41),                 // 0.16
            // The JSON's onInk object has no `track` — onInk is the
            // strongest fill it defines, per Architecture.
            track: step(41),
        }
    }

    /// Ink stepped with alpha, for use on paper (ivory window) surfaces.
    pub fn on_paper() -> Self {
        // Alpha channels below are `round(cssAlpha * 255)` from the JSON's
        // `color.onPaper` object.
        let step = |a: u8| Color::rgba(0x0C, 0x0A, 0x00, a);
        OnSurface {
            primary: Color::rgb(0x0C, 0x0A, 0x00), // rgba(..., 1.00) -> opaque
            secondary: step(179),                  // 0.70
            tertiary: step(140),                   // 0.55
            quaternary: step(115),                 // 0.45
            disabled: step(89),                    // 0.35
            divider: step(26),                     // 0.10
            fill_subtle: step(10),                 // 0.04
            fill: step(20),                        // 0.08
            track: step(36),                       // 0.14
            // The JSON's onPaper object has no `fillStrong` — onPaper's
            // track *is* its strongest fill, per Architecture.
            fill_strong: step(36),
        }
    }
}

/// `OnSurface`'s own container-level `#[serde(default)]` (above) needs a
/// single fallback for use when a *partial* `[on_ink]` or `[on_paper]` TOML
/// table is missing individual fields. There's no context-free "correct"
/// choice — this picks `on_ink`'s values arbitrarily as a baseline. It only
/// matters for hand-edited partial theme files; `Theme::saola()` always
/// produces a fully-populated struct via `on_ink()`/`on_paper()` directly,
/// so this default is never consulted by the built-in theme.
impl Default for OnSurface {
    fn default() -> Self {
        OnSurface::on_ink()
    }
}

/// Wallpaper scrims: how much of the wallpaper shows through in each shell
/// state. The image itself never changes — only the ink overlay's opacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Scrim {
    pub boot: Color,
    pub shutdown: Color,
    pub lock_awake: Color,
    pub launcher: Color,
    pub overview: Color,
    pub capture: Color,
    pub modal: Color,
    pub translucent_panel: Color,
}

impl Default for Scrim {
    fn default() -> Self {
        // Alpha channels are `round(cssAlpha * 255)` from the JSON's
        // `color.scrim` object; all scrims are ink-tinted.
        let step = |a: u8| Color::rgba(0x0C, 0x0A, 0x00, a);
        Scrim {
            boot: step(199),              // 0.78
            shutdown: step(224),          // 0.88
            lock_awake: step(158),        // 0.62
            launcher: step(133),          // 0.52
            overview: step(140),          // 0.55
            capture: step(158),           // 0.62
            modal: step(158),             // 0.62
            translucent_panel: step(153), // 0.60
        }
    }
}

/// The surface context a style helper is rendering onto. This is the axis
/// that varies in Saola instead of a dark/light theme choice — every style
/// helper takes a `&Theme` *and* a `Surface` so it knows which `OnSurface`
/// role set to read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Surface {
    /// Rendering onto an ink background (shell chrome: panel, launcher, lock).
    Ink,
    /// Rendering onto a paper background (light window content, cards).
    Paper,
}
