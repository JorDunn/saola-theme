//! Text role constructors: pre-sized, pre-fonted, pre-colored [`Text`]
//! widgets, one per named role in the design language.
//!
//! Every consumer was writing the same font + size + color triple by hand
//! (~38 in saola-files, ~40 in saola-panel, a handful in saola-lockscreen)
//! — three chances per call site to drift from the style guide. Each
//! constructor here returns the plain `iced::widget::Text`, so call sites
//! keep chaining (`.wrapping(...)`, `.width(...)`) as usual.
//!
//! Naming note: this module is `widget::text` while iced's own is
//! `iced::widget::text` — inside this crate the import is aliased
//! (`use iced::widget::text as text_widget;`), the same treatment
//! CLAUDE.md prescribes for the `checkbox` module/function clash.
//!
//! Roles deliberately *not* here: display/heading text (one-off enough
//! that the triple is the clearer spelling), and anything colored by a
//! wrapping button's style — a label inside a [`crate::style::button`]
//! helper must **not** carry its own `.color(...)`, or the style's
//! per-status text color (hover swaps, disabled dimming) can never apply.

use iced::widget::text as text_widget;
use iced::widget::text::IntoFragment;
use iced::widget::Text;
use saola_tokens::{Surface, Theme};

use crate::convert::{mono_font_medium, ui_font_regular, ColorExt};

/// Body copy: `size.body` regular UI text in the full-emphasis `primary`
/// role.
pub fn body<'a>(t: &Theme, s: Surface, content: impl IntoFragment<'a>) -> Text<'a> {
    text_widget(content)
        .size(t.typography.size.body)
        .font(ui_font_regular(t))
        .color(t.on(s).primary.into_iced())
}

/// Supporting copy: `size.secondary` regular UI text in the `secondary`
/// role — the detail line under a title, a status readout.
pub fn secondary<'a>(t: &Theme, s: Surface, content: impl IntoFragment<'a>) -> Text<'a> {
    text_widget(content)
        .size(t.typography.size.secondary)
        .font(ui_font_regular(t))
        .color(t.on(s).secondary.into_iced())
}

/// A label/annotation: `size.label` mono-medium text in the `tertiary`
/// role — section headings ([`crate::widget::section_label`] wraps this),
/// keycap-adjacent captions, technical annotations. iced has no
/// letter-spacing; the uppercase treatment is the caller's string.
pub fn label<'a>(t: &Theme, s: Surface, content: impl IntoFragment<'a>) -> Text<'a> {
    text_widget(content)
        .size(t.typography.size.label)
        .font(mono_font_medium(t))
        .color(t.on(s).tertiary.into_iced())
}

/// A hint: `size.secondary` regular UI text in the placeholder-level
/// `quaternary` role — inline guidance quiet enough to ignore
/// (saola-lockscreen's input placeholder is the reference use of
/// `quaternary`).
pub fn hint<'a>(t: &Theme, s: Surface, content: impl IntoFragment<'a>) -> Text<'a> {
    text_widget(content)
        .size(t.typography.size.secondary)
        .font(ui_font_regular(t))
        .color(t.on(s).quaternary.into_iced())
}

/// Error copy: `size.body` regular UI text in the per-surface accent text
/// color — `accent_light` on ink, `accent_dark` on paper, per style guide
/// §1 ("accent-light — accent-coloured text on ink only: hints, error
/// copy, prompt highlights").
///
/// There is no danger color in this system: **severity is carried by the
/// wording**, not by a red (saola-lockscreen `reveal.rs`, verbatim). If the
/// message needs to read as more than emphasized text, change the sentence.
pub fn error<'a>(t: &Theme, s: Surface, content: impl IntoFragment<'a>) -> Text<'a> {
    let color = match s {
        Surface::Ink => t.palette.accent_light,
        Surface::Paper => t.palette.accent_dark,
    };
    text_widget(content)
        .size(t.typography.size.body)
        .font(ui_font_regular(t))
        .color(color.into_iced())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The role constructors accept every fragment shape iced's own `text`
    /// does — `&str`, `String`, and a displayable value via `.to_string()`.
    #[test]
    fn roles_accept_common_fragments() {
        let t = Theme::saola();
        let _: Text<'_> = body(&t, Surface::Ink, "borrowed");
        let _: Text<'_> = secondary(&t, Surface::Paper, String::from("owned"));
        let _: Text<'_> = label(&t, Surface::Paper, format!("{:>3}%", 87));
        let _: Text<'_> = hint(&t, Surface::Ink, "Press Enter");
        let _: Text<'_> = error(&t, Surface::Ink, "That password wasn't right");
    }
}
