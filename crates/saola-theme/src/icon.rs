//! SVG icon infrastructure: Lucide icons at stroke-width 2.75
//! ([`saola_tokens::Sizes::icon_stroke`]), embedded at compile time and
//! tinted at *view* time from theme colors.
//!
//! This module is the shared home of what used to be two hand-synced copies
//! (saola-panel's `src/icons.rs` and saola-files' copy of it, with a third
//! mandated for saola-capture): one `Icon` enum over one bundled asset set,
//! one [`icon`] constructor, and the leveled glyph ladders
//! ([`battery_icon`], [`wifi_icon`], [`volume_icon`], [`brightness_icon`])
//! that keep a bar readout and its popover showing the same glyph for the
//! same state.
//!
//! The style guide (§4 Geometry) requires every icon in the shell to be a
//! [Lucide](https://lucide.dev) icon at stroke-width 2.75, at every size —
//! never a size- or state-dependent stroke. Rather than trust every future
//! call site to remember that, the stroke width is baked into each `.svg`
//! asset once, at authoring time (see `assets/icons/*.svg`), and the test
//! module below asserts it stayed that way — against the
//! [`saola_tokens::Sizes::icon_stroke`] token itself, so the assertion and
//! the token can't drift apart. Size and color always come from the theme;
//! nothing here hardcodes either.
//!
//! Teaching note (`include_bytes!` and static handles): `include_bytes!`
//! runs at *compile* time — it reads the file at the given path (relative
//! to this source file) and embeds its bytes directly into the compiled
//! binary as a `&'static [u8]`. There is no filesystem access whatsoever at
//! runtime, and a missing or renamed asset is a compile error, not something
//! that can fail while a consumer is running. Each `Icon` variant maps to
//! exactly one embedded byte string; `Icon::handle` wraps those bytes in
//! an [`iced::widget::svg::Handle`] (`Handle::from_memory`), which the
//! renderer parses and rasterizes lazily, the first time it's actually drawn.
//!
//! Teaching note (tinting mechanics — where the color actually comes from):
//! every asset's own `stroke="currentColor"` is cosmetic and never what
//! appears on screen. `iced::widget::svg::Style { color }` (set by
//! [`icon`]'s `.style()` closure below) is read by the `iced_tiny_skia`
//! renderer's vector cache, which — when a `color` is present — rewrites
//! *every non-transparent pixel* of the rasterized icon to that color,
//! ignoring whatever the source SVG drew (see
//! `iced_tiny_skia::vector::Cache::draw`, the `if let Some([r, g, b, _]) =
//! key.color` branch, for the exact mechanism). That recolor is also part of
//! the rasterization cache key (`RasterKey { id, color, size }`), so drawing
//! the same icon in two different tints (e.g. ivory at rest vs. terracotta
//! live) rasterizes and caches each tint independently rather than fighting
//! over one cached bitmap. Concretely: call sites never set a color inside
//! an SVG file — they always pass a `theme.on(Surface)....` role (or
//! `theme.palette.accent`), converted with
//! [`crate::convert::ColorExt::into_iced`], as this module's `color`
//! argument.
//!
//! What's bundled: the union of the generic Lucide glyphs the panel and the
//! file manager grew independently, the greeter's power cluster and
//! no-avatar glyphs, plus the two Saola marks (style guide §8 — the mark is
//! design-system property, so its assets live here).
//! App-specific brand icons (the panel's `anthropic`/`claude-code` pair)
//! stay in the app that reports on that product — an app that needs a brand
//! glyph keeps a tiny local icons module for it and uses this one for
//! everything generic.
//!
//! One design rule worth restating for the `File*` family: **mimetype
//! differentiation is glyph *shape* only, never hue** (style guide §1). A
//! directory row tints `FileImage` with the same theme role a plain `File`
//! glyph gets — never a per-mimetype color.

use iced::widget::svg::{Handle, Style};
use iced::widget::{svg, Svg};
use iced::Color;

/// One embedded icon asset. Each variant is one 24×24 Lucide SVG (or, for
/// the two `Mark*` variants, the Saola mark from style guide §8) living
/// under `assets/icons/`.
///
/// Adding an icon: drop the `.svg` file in `assets/icons/` with
/// `stroke-width="2.75"` baked in, add a variant here, add its `bytes()`
/// arm, and add it to `tests::ALL` so the asset tests cover it automatically.
/// (A deliberately solid asset — rare; see [`Icon::Play`] — also goes in
/// `tests::SOLID`.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    // ── Volume ladder ────────────────────────────────────────────────────
    Volume2,
    Volume1,
    /// The wave-less speaker — the bottom rung of the volume ladder (audible
    /// but effectively silent), below `Volume1`'s single wave.
    Volume,
    VolumeX,

    // ── Media transport ──────────────────────────────────────────────────
    /// Solid-filled, per style guide §4: "No filled icons except
    /// play/next/record/stop, which are solid." The one bundled asset with
    /// no stroke at all — see `tests::SOLID`.
    Play,
    Pause,
    SkipBack,
    /// "Next". Its Lucide glyph is a filled triangle *plus* a genuinely
    /// unfillable straight-line bar — that bar keeps a real
    /// `stroke-width="2.75"` stroke, so the asset satisfies the stroke
    /// invariant even though the triangle is solid.
    SkipForward,

    // ── Wi-Fi strength ladder ────────────────────────────────────────────
    /// Full-strength Wi-Fi (all arcs) — the top of the strength ladder;
    /// `WifiHigh`/`WifiLow`/`WifiZero` are the rungs below it, `WifiOff`
    /// the disconnected state.
    Wifi,
    WifiHigh,
    WifiLow,
    WifiZero,
    WifiOff,

    // ── Battery charge ladder ────────────────────────────────────────────
    /// The empty battery outline — the bottom of the charge ladder;
    /// `BatteryLow`/`BatteryMedium`/`BatteryFull` add fill bars,
    /// `BatteryCharging` is the bolt shown while on AC.
    Battery,
    BatteryLow,
    BatteryMedium,
    BatteryFull,
    BatteryCharging,

    // ── Brightness ladder ────────────────────────────────────────────────
    /// The brightness ladder: `SunDim` → `SunMedium` → `Sun`, dimmest to
    /// brightest.
    Sun,
    SunMedium,
    SunDim,

    // ── Bluetooth states ─────────────────────────────────────────────────
    /// Bluetooth states: the bare rune (powered, idle), `Connected` (a
    /// device is attached), `Off` (adapter soft-off).
    Bluetooth,
    BluetoothConnected,
    BluetoothOff,

    // ── The Saola marks (style guide §8) ─────────────────────────────────
    /// The default Saola mark: two splaying strokes ("horns"), style guide
    /// §8. Design-system property, so the asset lives here rather than in
    /// any one app.
    MarkHorns,
    /// The alternative mark: a broken ring with a dot at the break, style
    /// guide §8. Its dot is a small *filled* circle inside an otherwise
    /// stroked asset — see the asset-invariant tests for how that's covered.
    MarkNotch,

    // ── Places (sidebars) ────────────────────────────────────────────────
    House,
    Download,
    Image,
    Music,
    Film,
    HardDrive,
    Usb,
    Trash2,
    Bookmark,
    Monitor,
    /// A saved remote server entry in a places sidebar.
    Server,
    /// "Connect" for a saved server entry.
    PlugZap,

    // ── File-kind glyphs — shape carries mimetype, never hue ────────────
    Folder,
    FolderOpen,
    /// The generic file glyph.
    File,
    FileText,
    FileImage,
    FileAudio,
    FileVideo,
    FileCode,
    FileArchive,
    /// A `.desktop`/config-shaped file.
    FileCog,
    /// Unrecognized mimetype fallback, distinct from the generic
    /// [`File`][Icon::File].
    FileQuestion,
    /// The symlink emblem.
    Link,
    /// A permission-denied/inaccessible entry.
    Lock,

    // ── Navigation chrome ────────────────────────────────────────────────
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    /// Breadcrumb segment separator.
    ChevronRight,
    List,
    LayoutGrid,
    Eye,
    EyeOff,
    /// Ascending name/date/size sort indicator.
    ArrowDownAZ,
    /// Descending name/date/size sort indicator.
    ArrowUpAZ,
    /// The overflow-menu trigger.
    Ellipsis,
    /// The window close pill.
    X,
    /// Rename.
    Pencil,
    /// Manual refresh.
    RefreshCw,

    // ── Clipboard/file actions ──────────────────────────────────────────
    Copy,
    /// Cut.
    Scissors,
    ClipboardPaste,
    FolderPlus,
    FilePlus,
    /// Undo.
    RotateCcw,
    /// "Open in terminal".
    Terminal,
    /// Properties/details.
    Info,
    /// "Open with…" / open in an external app.
    ExternalLink,
    /// A completed/selected affordance (e.g. the active entry in a popover).
    Check,
    /// A network/remote-scheme location.
    Globe,

    // ── Session / power cluster ─────────────────────────────────────────
    /// Shut down.
    Power,
    /// Reboot. Clockwise on purpose: `RotateCcw` is the undo glyph, and a
    /// reboot is a fresh start, not a step back.
    RotateCw,
    /// Suspend.
    Moon,
    /// A person with no avatar — the greeter's "Not listed?" tile, and any
    /// user row that has no picture to show.
    UserRound,
}

impl Icon {
    /// The embedded SVG source bytes for this icon. `include_bytes!` reads
    /// the file at compile time (path relative to *this* source file) —
    /// see the module doc comment for why that means no runtime I/O at all.
    fn bytes(self) -> &'static [u8] {
        match self {
            Icon::Volume2 => include_bytes!("../assets/icons/volume-2.svg"),
            Icon::Volume1 => include_bytes!("../assets/icons/volume-1.svg"),
            Icon::Volume => include_bytes!("../assets/icons/volume.svg"),
            Icon::VolumeX => include_bytes!("../assets/icons/volume-x.svg"),
            Icon::Play => include_bytes!("../assets/icons/play.svg"),
            Icon::Pause => include_bytes!("../assets/icons/pause.svg"),
            Icon::SkipBack => include_bytes!("../assets/icons/skip-back.svg"),
            Icon::SkipForward => include_bytes!("../assets/icons/skip-forward.svg"),
            Icon::Wifi => include_bytes!("../assets/icons/wifi.svg"),
            Icon::WifiHigh => include_bytes!("../assets/icons/wifi-high.svg"),
            Icon::WifiLow => include_bytes!("../assets/icons/wifi-low.svg"),
            Icon::WifiZero => include_bytes!("../assets/icons/wifi-zero.svg"),
            Icon::WifiOff => include_bytes!("../assets/icons/wifi-off.svg"),
            Icon::Battery => include_bytes!("../assets/icons/battery.svg"),
            Icon::BatteryLow => include_bytes!("../assets/icons/battery-low.svg"),
            Icon::BatteryMedium => include_bytes!("../assets/icons/battery-medium.svg"),
            Icon::BatteryFull => include_bytes!("../assets/icons/battery-full.svg"),
            Icon::BatteryCharging => include_bytes!("../assets/icons/battery-charging.svg"),
            Icon::Sun => include_bytes!("../assets/icons/sun.svg"),
            Icon::SunMedium => include_bytes!("../assets/icons/sun-medium.svg"),
            Icon::SunDim => include_bytes!("../assets/icons/sun-dim.svg"),
            Icon::Bluetooth => include_bytes!("../assets/icons/bluetooth.svg"),
            Icon::BluetoothConnected => include_bytes!("../assets/icons/bluetooth-connected.svg"),
            Icon::BluetoothOff => include_bytes!("../assets/icons/bluetooth-off.svg"),
            Icon::MarkHorns => include_bytes!("../assets/icons/mark-horns.svg"),
            Icon::MarkNotch => include_bytes!("../assets/icons/mark-notch.svg"),
            Icon::House => include_bytes!("../assets/icons/house.svg"),
            Icon::Download => include_bytes!("../assets/icons/download.svg"),
            Icon::Image => include_bytes!("../assets/icons/image.svg"),
            Icon::Music => include_bytes!("../assets/icons/music.svg"),
            Icon::Film => include_bytes!("../assets/icons/film.svg"),
            Icon::HardDrive => include_bytes!("../assets/icons/hard-drive.svg"),
            Icon::Usb => include_bytes!("../assets/icons/usb.svg"),
            Icon::Trash2 => include_bytes!("../assets/icons/trash-2.svg"),
            Icon::Bookmark => include_bytes!("../assets/icons/bookmark.svg"),
            Icon::Monitor => include_bytes!("../assets/icons/monitor.svg"),
            Icon::Server => include_bytes!("../assets/icons/server.svg"),
            Icon::PlugZap => include_bytes!("../assets/icons/plug-zap.svg"),
            Icon::Folder => include_bytes!("../assets/icons/folder.svg"),
            Icon::FolderOpen => include_bytes!("../assets/icons/folder-open.svg"),
            Icon::File => include_bytes!("../assets/icons/file.svg"),
            Icon::FileText => include_bytes!("../assets/icons/file-text.svg"),
            Icon::FileImage => include_bytes!("../assets/icons/file-image.svg"),
            Icon::FileAudio => include_bytes!("../assets/icons/file-audio.svg"),
            Icon::FileVideo => include_bytes!("../assets/icons/file-video.svg"),
            Icon::FileCode => include_bytes!("../assets/icons/file-code.svg"),
            Icon::FileArchive => include_bytes!("../assets/icons/file-archive.svg"),
            Icon::FileCog => include_bytes!("../assets/icons/file-cog.svg"),
            Icon::FileQuestion => include_bytes!("../assets/icons/file-question.svg"),
            Icon::Link => include_bytes!("../assets/icons/link.svg"),
            Icon::Lock => include_bytes!("../assets/icons/lock.svg"),
            Icon::ArrowLeft => include_bytes!("../assets/icons/arrow-left.svg"),
            Icon::ArrowRight => include_bytes!("../assets/icons/arrow-right.svg"),
            Icon::ArrowUp => include_bytes!("../assets/icons/arrow-up.svg"),
            Icon::ChevronRight => include_bytes!("../assets/icons/chevron-right.svg"),
            Icon::List => include_bytes!("../assets/icons/list.svg"),
            Icon::LayoutGrid => include_bytes!("../assets/icons/layout-grid.svg"),
            Icon::Eye => include_bytes!("../assets/icons/eye.svg"),
            Icon::EyeOff => include_bytes!("../assets/icons/eye-off.svg"),
            Icon::ArrowDownAZ => include_bytes!("../assets/icons/arrow-down-a-z.svg"),
            Icon::ArrowUpAZ => include_bytes!("../assets/icons/arrow-up-a-z.svg"),
            Icon::Ellipsis => include_bytes!("../assets/icons/ellipsis.svg"),
            Icon::X => include_bytes!("../assets/icons/x.svg"),
            Icon::Pencil => include_bytes!("../assets/icons/pencil.svg"),
            Icon::RefreshCw => include_bytes!("../assets/icons/refresh-cw.svg"),
            Icon::Copy => include_bytes!("../assets/icons/copy.svg"),
            Icon::Scissors => include_bytes!("../assets/icons/scissors.svg"),
            Icon::ClipboardPaste => include_bytes!("../assets/icons/clipboard-paste.svg"),
            Icon::FolderPlus => include_bytes!("../assets/icons/folder-plus.svg"),
            Icon::FilePlus => include_bytes!("../assets/icons/file-plus.svg"),
            Icon::RotateCcw => include_bytes!("../assets/icons/rotate-ccw.svg"),
            Icon::Terminal => include_bytes!("../assets/icons/terminal.svg"),
            Icon::Info => include_bytes!("../assets/icons/info.svg"),
            Icon::ExternalLink => include_bytes!("../assets/icons/external-link.svg"),
            Icon::Check => include_bytes!("../assets/icons/check.svg"),
            Icon::Globe => include_bytes!("../assets/icons/globe.svg"),
            Icon::Power => include_bytes!("../assets/icons/power.svg"),
            Icon::RotateCw => include_bytes!("../assets/icons/rotate-cw.svg"),
            Icon::Moon => include_bytes!("../assets/icons/moon.svg"),
            Icon::UserRound => include_bytes!("../assets/icons/user-round.svg"),
        }
    }

    /// An SVG handle for this icon.
    ///
    /// `Handle::from_memory` hashes the bytes it's given to build the
    /// handle's id (see `iced_core::svg::Handle::from_data`), and that id is
    /// what the renderer's rasterization cache keys on — so calling this
    /// repeatedly for the same variant (once per `view`, say) is cheap: the
    /// same static bytes hash the same way every time, and the renderer
    /// recognizes the cache hit instead of re-parsing the SVG each frame.
    fn handle(self) -> Handle {
        Handle::from_memory(self.bytes())
    }
}

/// Builds a sized, tinted [`Svg`] widget for `icon`.
///
/// `size` is a theme size token, in logical pixels — e.g.
/// `theme.sizes.icon_bar` for a panel bar, `theme.sizes.icon_row` for a
/// list row, `theme.sizes.icon_bare` for a bare-icon menu — never a literal
/// number. `color` is likewise always a theme role — a
/// `theme.on(Surface).primary`/`.secondary`/etc, or `theme.palette.accent`
/// for the terracotta "live" treatment — converted to [`iced::Color`] with
/// [`crate::convert::ColorExt::into_iced`] before it reaches this function.
///
/// See the module doc comment for exactly how `color` ends up as the pixels
/// on screen (it is not the SVG's own stroke color).
pub fn icon<'a>(kind: Icon, size: f32, color: Color) -> Svg<'a> {
    svg(kind.handle())
        .width(size)
        .height(size)
        .style(move |_theme, _status| Style { color: Some(color) })
}

// ─── Leveled glyph ladders ───────────────────────────────────────────────
//
// Pure functions from readout state to glyph, shared so a bar readout and
// its popover row can never drift onto different mappings (that drift is
// exactly what happened when each app hand-copied its own icons module).
// Thresholds are coarse on purpose: the glyph answers "roughly how much /
// how good", the popover has the exact number.

/// Highest percent that still counts as "low" for [`volume_icon`] — i.e.
/// `Volume1` (one sound wave) covers 1–49%, `Volume2` (two waves) covers
/// 50% and up. A two-rung ladder's only sensible split point is its
/// midpoint. (0% gets the wave-less [`Icon::Volume`] and real mute gets
/// [`Icon::VolumeX`] — the two states look different as well as being
/// colored differently.)
const LOW_VOLUME_PERCENT_MAX: u32 = 49;

/// Percentage + charging → which battery glyph a readout shows. Charging
/// always wins (the bolt is the "live" state's own shape, matching its
/// accent tint); otherwise the charge ladder: full ≥ 75, medium ≥ 40,
/// low ≥ 15, and the bare outline below that. Thresholds sit between
/// Lucide's own three fill bars.
///
/// Pure function of its arguments (no D-Bus, no globals) — unit-tested
/// below.
pub fn battery_icon(percentage: f64, charging: bool) -> Icon {
    if charging {
        Icon::BatteryCharging
    } else if percentage >= 75.0 {
        Icon::BatteryFull
    } else if percentage >= 40.0 {
        Icon::BatteryMedium
    } else if percentage >= 15.0 {
        Icon::BatteryLow
    } else {
        Icon::Battery
    }
}

/// Connection state + strength → which Wi-Fi glyph a readout shows. Offline
/// is [`Icon::WifiOff`]; connected climbs the arc ladder — full
/// [`Icon::Wifi`] ≥ 75, [`Icon::WifiHigh`] ≥ 50, [`Icon::WifiLow`] ≥ 25,
/// [`Icon::WifiZero`] below that — and a connection whose strength is
/// unknown (no signal level has arrived yet) shows the full glyph rather
/// than pretending to know it's weak.
///
/// Pure function of its arguments — unit-tested below.
pub fn wifi_icon(connected: bool, strength_percent: Option<u8>) -> Icon {
    if !connected {
        return Icon::WifiOff;
    }
    match strength_percent {
        None => Icon::Wifi,
        Some(p) if p >= 75 => Icon::Wifi,
        Some(p) if p >= 50 => Icon::WifiHigh,
        Some(p) if p >= 25 => Icon::WifiLow,
        Some(_) => Icon::WifiZero,
    }
}

/// Percent + mute → which volume glyph a readout shows. See
/// [`LOW_VOLUME_PERCENT_MAX`] for the threshold rationale; muted always
/// wins, and unmuted 0% gets the wave-less speaker rather than the mute
/// cross — the states are different and look different.
///
/// Pure function of its arguments — unit-tested below.
pub fn volume_icon(percent: u32, muted: bool) -> Icon {
    if muted {
        Icon::VolumeX
    } else if percent == 0 {
        Icon::Volume
    } else if percent <= LOW_VOLUME_PERCENT_MAX {
        Icon::Volume1
    } else {
        Icon::Volume2
    }
}

/// Brightness percent → which sun glyph a slider row shows:
/// [`Icon::SunDim`] below a third, [`Icon::SunMedium`] below two-thirds,
/// full [`Icon::Sun`] above.
///
/// Pure function of its argument — unit-tested below.
pub fn brightness_icon(percent: u8) -> Icon {
    if percent < 34 {
        Icon::SunDim
    } else if percent < 67 {
        Icon::SunMedium
    } else {
        Icon::Sun
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every embedded icon, so the tests below can walk the whole asset set
    /// without a second hand-maintained list of bytes.
    const ALL: [Icon; 80] = [
        Icon::Volume2,
        Icon::Volume1,
        Icon::Volume,
        Icon::VolumeX,
        Icon::Play,
        Icon::Pause,
        Icon::SkipBack,
        Icon::SkipForward,
        Icon::Wifi,
        Icon::WifiHigh,
        Icon::WifiLow,
        Icon::WifiZero,
        Icon::WifiOff,
        Icon::Battery,
        Icon::BatteryLow,
        Icon::BatteryMedium,
        Icon::BatteryFull,
        Icon::BatteryCharging,
        Icon::Sun,
        Icon::SunMedium,
        Icon::SunDim,
        Icon::Bluetooth,
        Icon::BluetoothConnected,
        Icon::BluetoothOff,
        Icon::MarkHorns,
        Icon::MarkNotch,
        Icon::House,
        Icon::Download,
        Icon::Image,
        Icon::Music,
        Icon::Film,
        Icon::HardDrive,
        Icon::Usb,
        Icon::Trash2,
        Icon::Bookmark,
        Icon::Monitor,
        Icon::Server,
        Icon::PlugZap,
        Icon::Folder,
        Icon::FolderOpen,
        Icon::File,
        Icon::FileText,
        Icon::FileImage,
        Icon::FileAudio,
        Icon::FileVideo,
        Icon::FileCode,
        Icon::FileArchive,
        Icon::FileCog,
        Icon::FileQuestion,
        Icon::Link,
        Icon::Lock,
        Icon::ArrowLeft,
        Icon::ArrowRight,
        Icon::ArrowUp,
        Icon::ChevronRight,
        Icon::List,
        Icon::LayoutGrid,
        Icon::Eye,
        Icon::EyeOff,
        Icon::ArrowDownAZ,
        Icon::ArrowUpAZ,
        Icon::Ellipsis,
        Icon::X,
        Icon::Pencil,
        Icon::RefreshCw,
        Icon::Copy,
        Icon::Scissors,
        Icon::ClipboardPaste,
        Icon::FolderPlus,
        Icon::FilePlus,
        Icon::RotateCcw,
        Icon::Terminal,
        Icon::Info,
        Icon::ExternalLink,
        Icon::Check,
        Icon::Globe,
        Icon::Power,
        Icon::RotateCw,
        Icon::Moon,
        Icon::UserRound,
    ];

    /// The solid-filled assets — exempt from the stroke-width and
    /// `fill="none"` invariants below. Style guide §4: "No filled icons
    /// except play/next/record/stop, which are solid." Only `Play` is fully
    /// solid here: a filled triangle has no stroke to bake a width into.
    /// `SkipForward` ("next") also went solid, but its glyph is a filled
    /// triangle *plus* a genuinely unfillable straight-line bar — that bar
    /// keeps a real `stroke-width="2.75"` stroke and the asset keeps its
    /// root-level `fill="none"`, so it satisfies both invariants and stays
    /// out of this list. `MarkNotch`'s dot is a filled sub-element inside an
    /// otherwise stroked, `fill="none"`-rooted asset, so it too stays out.
    /// (The panel's other solids, the `anthropic`/`claude-code` brand marks,
    /// are deliberately *not* bundled here — brand assets stay with the app
    /// that reports on that product.)
    const SOLID: [Icon; 1] = [Icon::Play];

    /// `ALL` minus [`SOLID`] — every asset that carries a real stroke,
    /// derived by filtering rather than hand-maintained so the two lists
    /// can't fall out of sync.
    fn stroke_only() -> impl Iterator<Item = Icon> {
        ALL.into_iter().filter(|icon| !SOLID.contains(icon))
    }

    /// Binding constraint (style guide §4): every embedded *stroke-based*
    /// asset must have the theme's stroke width baked in at authoring time —
    /// nothing at the call site enforces that, so this test is what actually
    /// guards the invariant. The needle is built from
    /// [`saola_tokens::Sizes::icon_stroke`] itself, so the token and the
    /// assets can't drift apart without this failing. The [`SOLID`] assets
    /// have no stroke at all and are deliberately not walked here.
    #[test]
    fn every_asset_bakes_in_the_theme_stroke_width() {
        let stroke = crate::Theme::saola().sizes.icon_stroke;
        let needle = format!("stroke-width=\"{stroke}\"");
        for icon in stroke_only() {
            let source = std::str::from_utf8(icon.bytes())
                .unwrap_or_else(|_| panic!("{icon:?}'s asset is not valid UTF-8"));
            assert!(
                source.contains(&needle),
                "{icon:?}'s asset is missing {needle}"
            );
        }
    }

    /// Every stroke-based asset is outline-style at the root
    /// (`fill="none"`), so a future accidental solid-fill asset shows up
    /// here rather than silently passing the stroke-width test above (which
    /// only checks the attribute is *present*, not that it's meaningful on a
    /// genuinely stroked path). `SkipForward` and `MarkNotch` pass because
    /// their filled shapes are sub-elements under a `fill="none"` root —
    /// see [`SOLID`]'s doc comment.
    #[test]
    fn every_stroked_asset_is_unfilled_outline_style() {
        for icon in stroke_only() {
            let source = std::str::from_utf8(icon.bytes()).unwrap();
            assert!(
                source.contains("fill=\"none\""),
                "{icon:?}'s asset should be an unfilled outline (fill=\"none\")"
            );
        }
    }

    /// The assets exempted above: confirms each is genuinely solid (a
    /// filled shape, no stroke at all) rather than just having drifted off
    /// the token width by accident.
    #[test]
    fn solid_icons_are_filled_with_no_stroke() {
        for icon in SOLID {
            let source = std::str::from_utf8(icon.bytes()).unwrap();
            assert!(
                source.contains("fill=\"currentColor\""),
                "{icon:?}'s asset should be filled solid"
            );
            assert!(
                !source.contains("stroke-width"),
                "{icon:?}'s asset should have no stroke at all"
            );
        }
    }

    /// Cheap well-formedness check: catches a mismatched/truncated asset
    /// (e.g. a copy-paste mistake while authoring one of these files) at
    /// `cargo test` time instead of leaving it for a runtime SVG-parse
    /// failure inside `resvg`, which `include_bytes!` itself can't catch
    /// (it embeds bytes, it doesn't parse them).
    #[test]
    fn every_asset_is_a_24x24_svg() {
        for icon in ALL {
            let source = std::str::from_utf8(icon.bytes()).unwrap();
            assert!(
                source.contains("viewBox=\"0 0 24 24\""),
                "{icon:?}'s asset is not a 24x24 viewBox"
            );
        }
    }

    // ── The glyph ladders (ported with the panel's test suites) ─────────

    #[test]
    fn charging_always_shows_the_bolt() {
        assert_eq!(battery_icon(3.0, true), Icon::BatteryCharging);
        assert_eq!(battery_icon(100.0, true), Icon::BatteryCharging);
    }

    #[test]
    fn the_charge_ladder_steps_at_its_documented_thresholds() {
        assert_eq!(battery_icon(100.0, false), Icon::BatteryFull);
        assert_eq!(battery_icon(75.0, false), Icon::BatteryFull);
        assert_eq!(battery_icon(74.9, false), Icon::BatteryMedium);
        assert_eq!(battery_icon(40.0, false), Icon::BatteryMedium);
        assert_eq!(battery_icon(39.9, false), Icon::BatteryLow);
        assert_eq!(battery_icon(15.0, false), Icon::BatteryLow);
        assert_eq!(battery_icon(14.9, false), Icon::Battery);
        assert_eq!(battery_icon(0.0, false), Icon::Battery);
    }

    #[test]
    fn the_arc_ladder_steps_at_its_documented_thresholds() {
        assert_eq!(wifi_icon(true, Some(100)), Icon::Wifi);
        assert_eq!(wifi_icon(true, Some(75)), Icon::Wifi);
        assert_eq!(wifi_icon(true, Some(74)), Icon::WifiHigh);
        assert_eq!(wifi_icon(true, Some(50)), Icon::WifiHigh);
        assert_eq!(wifi_icon(true, Some(49)), Icon::WifiLow);
        assert_eq!(wifi_icon(true, Some(25)), Icon::WifiLow);
        assert_eq!(wifi_icon(true, Some(24)), Icon::WifiZero);
        assert_eq!(wifi_icon(true, Some(10)), Icon::WifiZero);
    }

    #[test]
    fn unknown_strength_shows_the_full_glyph_rather_than_guessing_weak() {
        assert_eq!(wifi_icon(true, None), Icon::Wifi);
    }

    #[test]
    fn offline_is_wifi_off_regardless_of_a_stale_strength() {
        assert_eq!(wifi_icon(false, None), Icon::WifiOff);
        // Belt and braces: callers clear strength whenever the link isn't
        // settled, but the glyph must never show arcs while offline even if
        // some future caller forgets.
        assert_eq!(wifi_icon(false, Some(40)), Icon::WifiOff);
    }

    #[test]
    fn muted_always_shows_the_x_glyph() {
        assert_eq!(volume_icon(0, true), Icon::VolumeX);
        assert_eq!(volume_icon(30, true), Icon::VolumeX);
        assert_eq!(volume_icon(100, true), Icon::VolumeX);
    }

    #[test]
    fn unmuted_silence_shows_the_wave_less_speaker_not_the_mute_cross() {
        assert_eq!(volume_icon(0, false), Icon::Volume);
    }

    #[test]
    fn icon_steps_from_low_to_high_at_the_midpoint() {
        assert_eq!(volume_icon(1, false), Icon::Volume1);
        assert_eq!(volume_icon(LOW_VOLUME_PERCENT_MAX, false), Icon::Volume1);
        assert_eq!(
            volume_icon(LOW_VOLUME_PERCENT_MAX + 1, false),
            Icon::Volume2
        );
        assert_eq!(volume_icon(100, false), Icon::Volume2);
        // Above 100% is legitimate — audio servers permit software
        // amplification, and the top rung covers it.
        assert_eq!(volume_icon(150, false), Icon::Volume2);
    }

    #[test]
    fn the_sun_ladder_steps_at_its_documented_thresholds() {
        assert_eq!(brightness_icon(0), Icon::SunDim);
        assert_eq!(brightness_icon(33), Icon::SunDim);
        assert_eq!(brightness_icon(34), Icon::SunMedium);
        assert_eq!(brightness_icon(66), Icon::SunMedium);
        assert_eq!(brightness_icon(67), Icon::Sun);
        assert_eq!(brightness_icon(100), Icon::Sun);
    }
}
