//! The modal dialog kit: one container-shaped style helper, plus the
//! assembly recipe every consumer needs around it.
//!
//! `sizes.dialog_width`, `typography.size.dialog_title`, and
//! [`crate::style::container::ScrimKind::Modal`] have existed since earlier
//! stages, but nothing ever tied them together into an actual dialog — this
//! module is that assembly.
//!
//! ## Assembly recipe
//!
//! [`surface`] only styles the dialog card itself — `container::Style` has
//! no width, position, or blur fields, so it can't place the dialog on
//! screen or blur what's behind it. A consumer builds the full modal like
//! this:
//!
//! 1. **Scrim behind**: a full-bleed backdrop with
//!    [`crate::style::container::scrim`] and
//!    [`crate::style::container::ScrimKind::Modal`] (`scrim.modal`,
//!    `rgba(12,10,0,0.62)`) sized to the window. The style guide also calls
//!    for a 7px compositor blur behind it (`sizes.scrim_blur_modal`) — iced
//!    0.14 cannot blur content behind a window, so that value is data for
//!    the consumer's own compositor effect, not something either `scrim` or
//!    `surface` renders.
//! 2. **Dialog centered**: [`surface`] on a `container` sized
//!    `.width(t.sizes.dialog_width)`, laid over the scrim with
//!    `.width(Fill).height(Fill).align_x(Center).align_y(Center)` on the
//!    backdrop container that holds it.
//! 3. **Title**: a `text(...)` at `typography.size.dialog_title` (24px), set
//!    in [`crate::convert::display_font`] per the style guide's "dialog
//!    titles" rule for IBM Plex Serif, at the top of the dialog's content
//!    column.
//! 4. **Footer**: [`crate::widget::footer_strip`] holding the actions —
//!    [`crate::style::button::rest`] for the dismissing/secondary action,
//!    [`crate::style::button::active`] for the primary one. There is no
//!    `danger` button variant: destructive confirmation is a consumer
//!    pattern (wording, ordering, a second step), never a fourth palette
//!    entry.
//!
//! The gallery's "Dialog" section in the Widgets page renders exactly this
//! recipe as a static specimen.

use iced::widget::container::Style;
use saola_tokens::{Surface, Theme};

/// The dialog card: a paper-surface card at `radii.card` with the popover
/// shadow — the same recipe [`crate::style::container::card`] uses for
/// `Surface::Ink` (a solid ivory card floating on a dark ground, ink text,
/// no border). A dialog always uses that recipe, never `card`'s
/// `Surface::Paper` variant, because dialogs are light windows *by
/// definition* (the design language's "dialogs are light windows" rule) —
/// they always float over a scrim (always ink-tinted), regardless of what
/// ambient surface the rest of the consumer's UI happens to be in. That's
/// why this helper takes no [`Surface`] parameter at all.
pub fn surface(t: &Theme) -> impl Fn(&iced::Theme) -> Style + Clone {
    super::container::card(t, Surface::Ink)
}
