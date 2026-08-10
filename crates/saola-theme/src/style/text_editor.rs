//! Text editor style: [`rest`] — the multi-line counterpart of
//! [`crate::style::text_input::rest`], for notes/compose fields.
//!
//! `iced::widget::text_editor::Catalog` (`text_editor.rs`,
//! `iced_widget-0.14.2/src/text_editor.rs`) is its **own** trait
//! (`pub trait Catalog: theme::Base`) — *not* `text_input::Catalog`
//! composed the way `combo_box`'s is — so unlike
//! [`crate::style::combo_box`] this module can't just hand back
//! `text_input::rest`. The shapes are near-twins, verified against the
//! source:
//!
//! - `text_editor::Status` is `Active` / `Hovered` /
//!   `Focused { is_hovered: bool }` / `Disabled` — identical to
//!   `text_input::Status`, payload included.
//! - `text_editor::Style` is `text_input::Style` minus the `icon` field:
//!   `background: Background`, `border: Border`, `placeholder: Color`,
//!   `value: Color`, `selection: Color`.
//!
//! So [`rest`] mirrors `text_input::rest`'s *values* through the editor's
//! own types: the same resting fill (opaque ivory on ink, translucent
//! ink-fill on paper), the same border progression (none at rest, a
//! `sizes.hairline` divider stroke on hover, the `sizes.ring` terracotta
//! ring on focus — `Focused`'s `is_hovered` payload is ignored, exactly as
//! `text_input::rest` ignores its own), accent selection, and a
//! placeholder in the hint-role `quaternary` step.
//!
//! Two deliberate deviations from a literal field-for-field copy:
//!
//! - **Radius is `radii.inset` (20), not `radii.pill`.** A single-line
//!   field is a pill; a multi-line editor is the design language's *other*
//!   shape, the over-rounded rectangle. Pill radius on a box several lines
//!   tall clamps to `height / 2` and turns the sides into semicircles that
//!   eat the first and last lines' leading edges. `radii.inset` is the
//!   window-scale over-rounded step ([`crate::style::container::inset`]'s
//!   radius), which is what a notes/compose recess is.
//! - **The placeholder (and the hover-legibility roles generally) read
//!   from the field's own fill context, not blindly from `on(s)`.** On an
//!   ink surface the resting field is *opaque ivory* — a paper-like
//!   surface — so its placeholder is `on_paper.quaternary` (translucent
//!   ink). `on_ink.quaternary` would be translucent ivory composited over
//!   ivory: literally invisible. `text_input::rest`'s `fill()` reads
//!   icon/placeholder from the same on-paper ladder for the same reason —
//!   the two modules agree on both surfaces, not just paper.
//!
//! Disabled follows `text_input::rest`: the fill drops to the surface's
//! `fill_subtle` (a translucent recess in the *surface*, so its text roles
//! come from `on(s)`) and all text drops to the `disabled` step.
//!
//! Per the stage plan there is no `rejected` variant: the `Style` doesn't
//! make one free (it would be a full second closure, as in
//! `text_input::rejected`), and no consumer pattern needs a "rejected
//! notes field" yet.
//!
//! ```no_run
//! use iced::widget::text_editor::{self, TextEditor};
//! use saola_theme::{style, Surface, Theme};
//!
//! let t = Theme::saola();
//! let content = text_editor::Content::new();
//! let _editor: iced::Element<'_, text_editor::Action> = TextEditor::new(&content)
//!     .placeholder("Notes…")
//!     .style(style::text_editor::rest(&t, Surface::Ink))
//!     .on_action(|action| action)
//!     .into();
//! ```

use iced::widget::text_editor::{Status, Style};
use iced::{Background, Border, Color};
use saola_tokens::{Surface, Theme};

use crate::convert::ColorExt;

/// A multi-line text editor at rest — [`crate::style::text_input::rest`]'s
/// look adapted to the editor's over-rounded-rectangle shape. See the
/// module docs for the two deliberate deviations (radius, placeholder
/// role).
pub fn rest(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    let radius = t.radii.inset;
    let ring_width = t.sizes.ring;
    let hairline = t.sizes.hairline;
    let accent = t.palette.accent.into_iced();
    // Same resting fill as `text_input::rest` / `button::rest`: opaque
    // ivory on ink, translucent ink-fill on paper.
    let background = match s {
        Surface::Ink => t.palette.paper,
        Surface::Paper => t.on_paper.fill,
    }
    .into_iced();
    let value = match s {
        Surface::Ink => t.palette.ink,
        Surface::Paper => t.on_paper.primary,
    }
    .into_iced();
    // The field's own fill is paper-like in *both* surface contexts (opaque
    // ivory, or ink-fill over an ivory window), so the placeholder is the
    // on-paper hint role on both — see the module docs.
    let placeholder = t.on_paper.quaternary.into_iced();
    let divider = t.on(s).divider.into_iced();
    let disabled_bg = t.on(s).fill_subtle.into_iced();
    let disabled_text = t.on(s).disabled.into_iced();

    move |_, status| {
        let border = |color: Color, width: f32| Border {
            color,
            width,
            radius: radius.into(),
        };
        match status {
            Status::Active => Style {
                background: Background::Color(background),
                border: border(Color::TRANSPARENT, 0.0),
                placeholder,
                value,
                selection: accent,
            },
            Status::Hovered => Style {
                background: Background::Color(background),
                border: border(divider, hairline),
                placeholder,
                value,
                selection: accent,
            },
            // The 2 px terracotta ring, drawn inline for the same reason
            // `text_input::rest` draws it: this `Status` carries a real
            // focus state.
            Status::Focused { .. } => Style {
                background: Background::Color(background),
                border: border(accent, ring_width),
                placeholder,
                value,
                selection: accent,
            },
            Status::Disabled => Style {
                background: Background::Color(disabled_bg),
                border: border(Color::TRANSPARENT, 0.0),
                placeholder: disabled_text,
                value: disabled_text,
                selection: accent,
            },
        }
    }
}
