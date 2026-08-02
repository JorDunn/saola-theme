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
//! per-widget styles in [`style`].

pub mod convert;
pub mod style;

pub use convert::{to_iced_theme, ColorExt, ShadowExt};
pub use saola_tokens::{Surface, Theme};

/// The token crate, re-exported so consumers only depend on `saola-theme`.
pub use saola_tokens as tokens;
