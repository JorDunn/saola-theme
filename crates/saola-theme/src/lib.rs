//! # saola-theme
//!
//! The iced integration layer of the Saola design system: style helpers
//! that read [`saola_tokens::Theme`] and produce iced widget styles.
//!
//! The design language in one sentence: **three colors, never a fourth** —
//! ink is every shell surface, ivory fill is a control at rest (ink text on
//! it), terracotta fill is on/selected/live (ivory text on it). Hover and
//! press move through the alpha fill steps; everything is a pill or an
//! over-rounded rectangle; keyboard focus is a 2 px terracotta ring.
//!
//! There is exactly one documented exception to "never a fourth": the
//! session-status semaphore dots
//! ([`style::container::status_dot`], [`saola_tokens::Palette`]'s `status_*`
//! fields), five hues used only as small status marks on the panel and
//! never as a control's fill.
//!
//! ```no_run
//! use saola_theme::{style, Surface, Theme};
//! use iced::widget::button;
//!
//! let theme = Theme::saola();
//! let _wifi: iced::widget::Button<'_, ()> =
//!     button("Wi-Fi").style(style::button::rest(&theme, Surface::Ink));
//! ```
//!
//! Layering: [`saola_tokens`] is pure data (no GUI dependencies); this crate
//! is the only place tokens meet iced. The bridge lives in [`convert`], the
//! per-widget styles in [`style`], the few bundled widget constructors
//! (where a token is a *size argument*, not a style field) in [`widget`],
//! the pure animation math over the `motion.*` tokens (progress
//! fractions, the toast envelope, the breathing curve) in [`motion`], the
//! shared Lucide icon set (assets, [`icon::Icon`], the glyph ladders) in
//! [`icon`], and the token bridge for `canvas::Program` drawing (where
//! iced styles can't reach at all) in [`canvas`].

pub mod avatar;
pub mod canvas;
pub mod convert;
pub mod icon;
pub mod motion;
pub mod style;
pub mod widget;

pub use convert::{to_iced_theme, ColorExt, GradientExt, ShadowExt};
// Like `iced::widget::svg`, `icon` is both this module and its constructor
// function — modules and functions live in different namespaces, so the
// common call `saola_theme::icon(...)` and the qualified
// `saola_theme::icon::battery_icon(...)` both resolve.
pub use icon::{icon, Icon};
pub use saola_tokens::{Surface, Theme};

/// The token crate, re-exported so consumers only depend on `saola-theme`.
pub use saola_tokens as tokens;
