//! Saola design tokens for Iced.
//!
//! Generated from SAOLA-STYLE-GUIDE.md / saola-tokens.json. Keep the three in step.
//!
//! The one rule: ivory fill = a control at rest, terracotta fill = on/selected/live.
//! Ivory takes ink text; terracotta takes ivory text. Never introduce a third colour.

use iced::{Border, Color, Font, Shadow, Vector};

// ── Colour ──────────────────────────────────────────────────────────────────

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a,
    }
}

pub mod color {
    use super::*;

    /// Every shell surface.
    pub const INK: Color = rgb(0x0C, 0x0A, 0x00);
    /// A control at rest — off, unselected, available.
    pub const PAPER: Color = rgb(0xFF, 0xFF, 0xF0);
    /// On, selected, focused, live.
    pub const ACCENT: Color = rgb(0xC6, 0x71, 0x39);
    /// Accent-coloured *text on ink* only.
    pub const ACCENT_LIGHT: Color = rgb(0xF6, 0xA0, 0x6B);
    /// Accent-coloured *text on ivory* only.
    pub const ACCENT_DARK: Color = rgb(0x8C, 0x49, 0x1A);

    /// Ivory stepped with alpha, for use on ink surfaces.
    pub mod on_ink {
        use super::*;
        pub const PRIMARY: Color = PAPER;
        pub const SECONDARY: Color = rgba(0xFF, 0xFF, 0xF0, 0.72);
        pub const TERTIARY: Color = rgba(0xFF, 0xFF, 0xF0, 0.55);
        pub const QUATERNARY: Color = rgba(0xFF, 0xFF, 0xF0, 0.40);
        pub const DISABLED: Color = rgba(0xFF, 0xFF, 0xF0, 0.35);
        pub const DIVIDER: Color = rgba(0xFF, 0xFF, 0xF0, 0.12);
        pub const FILL_SUBTLE: Color = rgba(0xFF, 0xFF, 0xF0, 0.07);
        pub const FILL: Color = rgba(0xFF, 0xFF, 0xF0, 0.12);
        pub const FILL_STRONG: Color = rgba(0xFF, 0xFF, 0xF0, 0.16);
    }

    /// Ink stepped with alpha, for use on ivory windows.
    pub mod on_paper {
        use super::*;
        pub const PRIMARY: Color = INK;
        pub const SECONDARY: Color = rgba(0x0C, 0x0A, 0x00, 0.70);
        pub const TERTIARY: Color = rgba(0x0C, 0x0A, 0x00, 0.55);
        pub const QUATERNARY: Color = rgba(0x0C, 0x0A, 0x00, 0.45);
        pub const DISABLED: Color = rgba(0x0C, 0x0A, 0x00, 0.35);
        pub const DIVIDER: Color = rgba(0x0C, 0x0A, 0x00, 0.10);
        pub const FILL_SUBTLE: Color = rgba(0x0C, 0x0A, 0x00, 0.04);
        pub const FILL: Color = rgba(0x0C, 0x0A, 0x00, 0.08);
        pub const TRACK: Color = rgba(0x0C, 0x0A, 0x00, 0.14);
    }

    /// Wallpaper scrims. The image never changes between states — only how much shows.
    pub mod scrim {
        use super::*;
        pub const BOOT: Color = rgba(0x0C, 0x0A, 0x00, 0.78);
        pub const SHUTDOWN: Color = rgba(0x0C, 0x0A, 0x00, 0.88);
        pub const LOCK_AWAKE: Color = rgba(0x0C, 0x0A, 0x00, 0.62);
        pub const LAUNCHER: Color = rgba(0x0C, 0x0A, 0x00, 0.52);
        pub const OVERVIEW: Color = rgba(0x0C, 0x0A, 0x00, 0.55);
        pub const CAPTURE: Color = rgba(0x0C, 0x0A, 0x00, 0.62);
        pub const MODAL: Color = rgba(0x0C, 0x0A, 0x00, 0.62);
        pub const TRANSLUCENT_PANEL: Color = rgba(0x0C, 0x0A, 0x00, 0.60);
    }
}

// ── Type ────────────────────────────────────────────────────────────────────

pub mod font {
    use super::*;

    /// All interface text — everything you scan.
    pub const UI: Font = Font::with_name("IBM Plex Sans");
    /// Display only: wordmark, clock, panel headings, dialog titles. Never in the bar.
    pub const DISPLAY: Font = Font::with_name("IBM Plex Serif");
    /// Keycaps, paths, hex, terminal, D-Bus names.
    pub const MONO: Font = Font::with_name("IBM Plex Mono");

    pub const LOCK_CLOCK: f32 = 168.0;
    pub const SCREEN_TITLE: f32 = 44.0;
    pub const PANEL_HEADING: f32 = 22.0;
    pub const DIALOG_TITLE: f32 = 24.0;
    pub const SECTION_HEADING: f32 = 20.0;
    pub const LAUNCHER_INPUT: f32 = 22.0;
    pub const BODY: f32 = 13.5;
    /// Hard minimum for anything in the panel.
    pub const BAR: f32 = 13.0;
    pub const SECONDARY: f32 = 12.5;
    pub const META: f32 = 12.0;
    pub const LABEL: f32 = 11.0;
    pub const KEYCAP: f32 = 11.0;
}

// ── Geometry ────────────────────────────────────────────────────────────────

pub mod radius {
    /// Buttons, chips, inputs, toggles, list rows.
    pub const PILL: f32 = 999.0;
    pub const POPOVER: f32 = 30.0;
    pub const POPOVER_WIDE: f32 = 34.0;
    pub const WINDOW: f32 = 24.0;
    pub const CARD: f32 = 26.0;
    pub const INSET: f32 = 20.0;
    pub const TILE: f32 = 13.0;
    pub const CHECKBOX: f32 = 7.0;
    pub const SELECTION: f32 = 6.0;
}

pub mod size {
    pub const PANEL_PILL: f32 = 40.0;
    pub const PANEL_BAR: f32 = 48.0;
    pub const PANEL_MARGIN_ISLANDS: f32 = 26.0;
    pub const PANEL_MARGIN_LEDGER: f32 = 20.0;
    pub const ISLAND_GAP: f32 = 10.0;
    pub const POPOVER_TOP: f32 = 72.0;
    pub const POPOVER_WIDTH: f32 = 440.0;
    pub const NOTIFICATION_CENTRE_WIDTH: f32 = 460.0;
    pub const LAUNCHER_WIDTH: f32 = 640.0;
    pub const NOTIFICATION_CARD_WIDTH: f32 = 440.0;
    pub const HIT_TARGET_BAR: f32 = 40.0;
    pub const HIT_TARGET_TOUCH: f32 = 44.0;
    pub const ICON_BAR: f32 = 15.0;
    pub const ICON_ROW: f32 = 16.0;
    pub const ICON_MENU: f32 = 19.0;
    /// Lucide, at this stroke width, at every size.
    pub const ICON_STROKE: f32 = 2.75;
    pub const WINDOW_BORDER: f32 = 2.0;
    pub const WINDOW_HEADER: f32 = 46.0;
}

pub mod shadow {
    use super::*;

    pub fn popover() -> Shadow {
        Shadow {
            color: rgba(0x0C, 0x0A, 0x00, 0.50),
            offset: Vector::new(0.0, 18.0),
            blur_radius: 48.0,
        }
    }

    pub fn window() -> Shadow {
        Shadow {
            color: rgba(0x0C, 0x0A, 0x00, 0.50),
            offset: Vector::new(0.0, 24.0),
            blur_radius: 64.0,
        }
    }
}

// ── Motion (milliseconds) ───────────────────────────────────────────────────

pub mod motion {
    use std::time::Duration;

    pub const HOVER: Duration = Duration::from_millis(140);
    pub const POPOVER: Duration = Duration::from_millis(160);
    pub const WAKE: Duration = Duration::from_millis(450);

    /// Notification popup: 0.35s slide in, 5s at rest, 1s fade with no movement.
    pub const TOAST_IN: Duration = Duration::from_millis(350);
    pub const TOAST_IDLE: Duration = Duration::from_millis(5000);
    pub const TOAST_OUT: Duration = Duration::from_millis(1000);
    pub const TOAST_TOTAL: Duration = Duration::from_millis(6350);
    pub const TOAST_MAX_STACK: usize = 3;
}

// ── Control states ──────────────────────────────────────────────────────────

/// The two states every interactive element has. There is no third.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Off, unselected, available.
    Rest,
    /// On, selected, focused, live.
    Active,
}

impl State {
    /// Fill for a control sitting on an ink surface.
    pub fn fill_on_ink(self) -> Color {
        match self {
            State::Rest => color::PAPER,
            State::Active => color::ACCENT,
        }
    }

    /// Fill for a control sitting on an ivory window.
    pub fn fill_on_paper(self) -> Color {
        match self {
            State::Rest => color::on_paper::FILL,
            State::Active => color::ACCENT,
        }
    }

    /// Label colour. Ivory fill takes ink text; terracotta fill takes ivory text.
    pub fn label_on_ink(self) -> Color {
        match self {
            State::Rest => color::INK,
            State::Active => color::PAPER,
        }
    }

    pub fn label_on_paper(self) -> Color {
        match self {
            State::Rest => color::INK,
            State::Active => color::PAPER,
        }
    }
}

/// A pill. Everything in Saola is one of these or an over-rounded rectangle.
pub fn pill_border() -> Border {
    Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: radius::PILL.into(),
    }
}

/// Keyboard focus: a 2px terracotta ring, inset. Never a platform default.
pub fn focus_border(r: f32) -> Border {
    Border {
        color: color::ACCENT,
        width: 2.0,
        radius: r.into(),
    }
}
