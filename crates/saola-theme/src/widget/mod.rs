//! Bundled widgets: constructors that pair an iced widget with its Saola
//! style *and* the size token the style implies, so consumers can't get one
//! half right and the other wrong.
//!
//! The style helpers in [`crate::style`] deliberately stop at styling —
//! but a hairline's thickness is a *size argument* to `rule::horizontal`/
//! `rule::vertical`, not part of `rule::Style`, so a style helper alone
//! can't guarantee a 1 px rule. [`hairline`] and [`vertical_hairline`]
//! close that gap by constructing the `Rule` themselves — and the rest of
//! this module extends the same principle to the composite shapes every
//! consumer was assembling by hand (a pill button is a style *plus* a
//! height token *plus* a padding token *plus* a centering container).
//!
//! Beyond pairing tokens with styles, these constructors absorb the three
//! iced 0.14 workarounds the consumers had each re-documented inline:
//!
//! 1. **Buttons don't center their content.** `iced_core::layout::padded`
//!    places a button's content flush at the padding's top-left corner and
//!    never aligns it — a shrink-height label inside a fixed-height button
//!    floats to the top. The fix is always the same sandwich, promoted here
//!    as [`centered`] (saola-panel `popovers/mod.rs` wrote it once and used
//!    it seven times; saola-capture and saola-files each carried inline
//!    copies of the same comment).
//! 2. **Hover paint order.** iced paints a container's background first and
//!    its content on top — so a hover fill can only be *seen* if the
//!    hover-carrying button sits **inside** the ink-drawing container, never
//!    the other way around. [`hover_pill`] owns that layering (saola-panel
//!    `main.rs`'s island pill, with a 16-line teaching comment and an
//!    acknowledged copy at its `claude_island_pill`).
//! 3. **An `Svg` icon's tint is fixed at build time** — the icon's style
//!    closure never sees the surrounding button's `Status`, so hover/disabled
//!    tints cannot ride the button style. Constructors that place an icon
//!    ([`icon_button`], [`menu_row`]) therefore take the tint from the
//!    caller, who knows the app state (saola-files `header.rs::nav_button`
//!    and `sidebar.rs::row_button` document the constraint; [`role`] picks
//!    the tint for the common cases).
//!
//! Text roles (pre-sized, pre-fonted, pre-colored `Text` constructors) live
//! in the [`text`] submodule.

pub mod text;

use iced::widget::text as text_widget;
use iced::widget::{button, container, rule, Container, Row, Rule, Space};
use iced::{Center, Element, Fill};
use saola_tokens::{Surface, Theme};

use crate::convert::{ui_font, ui_font_regular, ColorExt};
use crate::icon::{icon, Icon};
use crate::style;

// ---------------------------------------------------------------------------
// Rules
// ---------------------------------------------------------------------------

/// A horizontal hairline rule: `sizes.hairline` (1 px) thick, in the
/// surface's `divider` role ([`style::rule::rest`]). It fills the available
/// width, like any horizontal `Rule`.
///
/// ```no_run
/// use saola_theme::{widget, Surface, Theme};
///
/// let t = Theme::saola();
/// let divider: iced::widget::Rule<'_> = widget::hairline(&t, Surface::Paper);
/// ```
pub fn hairline<'a>(t: &Theme, s: Surface) -> Rule<'a> {
    rule::horizontal(t.sizes.hairline).style(style::rule::rest(t, s))
}

/// The vertical counterpart of [`hairline`]: `sizes.hairline` (1 px) wide,
/// filling the available height. iced names the two orientations as
/// separate constructors (`rule::horizontal` / `rule::vertical`), so this
/// crate does too.
pub fn vertical_hairline<'a>(t: &Theme, s: Surface) -> Rule<'a> {
    rule::vertical(t.sizes.hairline).style(style::rule::rest(t, s))
}

/// A [`hairline`] wrapped in `sizes.popover_separator_gap` of vertical
/// padding — the divider row a menu or popover puts between sections.
/// Full width.
///
/// Promoted from saola-panel, where `tray_menu::separator` and
/// `claude_usage::separator` were byte-identical (both reserving
/// `island_gap / 2` around the rule — the flagged gap that became the
/// `popover_separator_gap` token).
pub fn separator<'a, M: 'a>(t: &Theme, s: Surface) -> Element<'a, M> {
    container(hairline(t, s))
        .padding(iced::padding::vertical(t.sizes.popover_separator_gap))
        .width(Fill)
        .into()
}

// ---------------------------------------------------------------------------
// Centering & envelopes
// ---------------------------------------------------------------------------

/// Vertically center a widget inside a fixed-height parent — the antidote
/// to iced 0.14 workaround #1 (see the module docs): a `button` lays its
/// content out at the padding origin and never aligns it, so a
/// shrink-height child of any fixed-height button needs this sandwich or it
/// pins to the top edge.
///
/// Width is deliberately left alone: a `Container` adopts its content's
/// width hint, so a `Fill`-wide row still fills and a bare label still
/// shrinks; callers who also want horizontal centering chain
/// `.width(Fill).align_x(Center)` themselves. (Verbatim from saola-panel
/// `popovers/mod.rs::centered`, used seven-plus times there and copied
/// inline by saola-capture and saola-files.)
pub fn centered<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Container<'a, M> {
    container(content).height(Fill).align_y(Center)
}

/// The list-row envelope: a `sizes.list_row`-tall container that vertically
/// centers its content — the shape every fixed-rhythm row in a popover or
/// sidebar sits in. Returned as a `Container` so callers can keep chaining
/// (`.width(Fill)`, padding).
///
/// Named `list_row_container` (not `list_row`) to stay clear of
/// [`style::button::list_row`], which styles the *interactive* row this
/// envelope often wraps. Takes no `Surface`: the envelope is pure geometry —
/// height and centering — and paints nothing surface-dependent.
pub fn list_row_container<'a, M: 'a>(
    t: &Theme,
    content: impl Into<Element<'a, M>>,
) -> Container<'a, M> {
    container(content).height(t.sizes.list_row).align_y(Center)
}

/// A quiet single-line placeholder holding a [`list_row_container`] slot —
/// the "backend absent" line that keeps a popover's fixed layout when a
/// section has nothing to say. `sizes.bar`-sized text in the `secondary`
/// role.
///
/// Promoted from saola-panel, where `quick_settings::quiet_row` and
/// `claude_usage::quiet_line` were the same function written twice.
pub fn quiet_row<'a, M: 'a>(t: &Theme, s: Surface, label: &'a str) -> Element<'a, M> {
    list_row_container(
        t,
        text_widget(label)
            .size(t.typography.size.bar)
            .font(ui_font_regular(t))
            .color(t.on(s).secondary.into_iced()),
    )
    .into()
}

/// A full-bleed centered "nothing here" message: `sizes.secondary`-sized
/// regular text in the `tertiary` role, centered both ways in all the
/// space the parent gives it.
///
/// Promoted from saola-files, where `dirview::list`, `dirview::grid`, and
/// `trashview` carried three byte-identical copies.
pub fn empty_state<'a, M: 'a>(t: &Theme, s: Surface, message: &'a str) -> Element<'a, M> {
    container(
        text_widget(message)
            .size(t.typography.size.secondary)
            .font(ui_font_regular(t))
            .color(t.on(s).tertiary.into_iced()),
    )
    .width(Fill)
    .height(Fill)
    .align_x(Center)
    .align_y(Center)
    .into()
}

/// A section heading for a sidebar or settings group: `sizes.label`-sized
/// mono-medium text in the `tertiary` role ([`text::label`]), padded
/// `sizes.pill_gap` all round so it holds its own band in a list.
///
/// iced has no letter-spacing, and this crate applies no case transform —
/// the uppercase look the style guide asks for is the caller passing an
/// uppercase string (saola-files `sidebar.rs::section_label`, the source of
/// this shape, does exactly that).
pub fn section_label<'a, M: 'a>(t: &Theme, s: Surface, label: &'a str) -> Element<'a, M> {
    let gap = t.sizes.pill_gap;
    container(text::label(t, s, label))
        .padding([gap, gap])
        .into()
}

/// The footer band a window docks its transient chrome into (a progress
/// strip, an undo toast): a [`style::container::card`] at fixed
/// `sizes.ops_strip` height, full width, `popover_padding / 2` horizontal
/// padding, content vertically centered (the strip has no vertical padding,
/// so without the centering its contents render flush against the card's
/// top edge).
///
/// Promoted from saola-files, where `dialogs::progress` and
/// `dialogs::undo_toast` built identical strips and documented that the two
/// "occupy the identical footer band and must read as one continuous piece
/// of chrome when one replaces the other".
pub fn footer_strip<'a, M: 'a>(
    t: &Theme,
    s: Surface,
    content: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    container(content)
        .style(style::container::card(t, s))
        .width(Fill)
        .height(t.sizes.ops_strip)
        .padding([0.0, t.sizes.popover_padding / 2.0])
        .align_y(Center)
        .into()
}

// ---------------------------------------------------------------------------
// Rectangles ("draw me a shape")
// ---------------------------------------------------------------------------

/// A bare painted rectangle: a `Container` whose only child is a zero-area
/// `Space`, sized explicitly and handed a container style to draw with. A
/// `container` draws its background across its own bounds, so the swatch
/// *is* the container — the `Space` is just the child it needs to have.
///
/// This is the idiom behind every non-text mark in the shell — saola-panel's
/// column dashes (`modules::columns`) and session-status dots
/// (`modules::claude`) both spell it out, and the
/// [`style::container::dash`]/[`style::container::status_dot`] docs teach
/// the trick, which is the tell that it should be a constructor. Takes no
/// `&Theme`: the style closure carries every color, and the dimensions are
/// the caller's token to choose (`sizes.dash_height`, a width from state…).
pub fn swatch<'a, M: 'a>(
    width: f32,
    height: f32,
    style: impl Fn(&iced::Theme) -> container::Style + 'a,
) -> Element<'a, M> {
    container(Space::new())
        .width(width)
        .height(height)
        .style(style)
        .into()
}

/// A square [`swatch`] — the status-dot shape. Pair it with
/// [`style::container::status_dot`] (which supplies `radii.pill`, so any
/// square swatch it paints is a disc) and `sizes.dash_height`.
pub fn dot<'a, M: 'a>(
    size: f32,
    style: impl Fn(&iced::Theme) -> container::Style + 'a,
) -> Element<'a, M> {
    swatch(size, size, style)
}

// ---------------------------------------------------------------------------
// Buttons
// ---------------------------------------------------------------------------

/// A text pill button: [`style::button::emphasis`] (terracotta when
/// `emphasized`, the ivory/fill rest recipe otherwise) at fixed
/// `sizes.hit_target_bar` height, label centered via [`centered`],
/// horizontal padding from `paddings.pill_button`. Vertical padding is zero
/// — under a fixed height the centering sandwich replaces it (the pattern
/// every consumer copy used: saola-capture `overlay.rs`/`editor.rs`/
/// `app.rs`, saola-files ×8).
///
/// `on_press: None` renders the disabled look and captures nothing — the
/// iced 0.14 rule that a button without `.on_press` reports
/// `Status::Disabled` is a feature here, not a gotcha.
///
/// ```no_run
/// use saola_theme::{widget, Surface, Theme};
///
/// let t = Theme::saola();
/// let _save: iced::Element<'_, ()> =
///     widget::pill_button(&t, Surface::Paper, "Save", Some(()), true);
/// ```
pub fn pill_button<'a, M: Clone + 'a>(
    t: &Theme,
    s: Surface,
    label: &'a str,
    on_press: Option<M>,
    emphasized: bool,
) -> Element<'a, M> {
    let content = centered(
        text_widget(label)
            .font(ui_font(t))
            .size(t.typography.size.body),
    );
    button(content)
        .height(t.sizes.hit_target_bar)
        .padding([0.0, t.paddings.pill_button[1]])
        .style(style::button::emphasis(t, s, emphasized))
        .on_press_maybe(on_press)
        .into()
}

/// An icon pill button, optionally with a trailing label: the glyph at
/// `sizes.icon_row` and the label a `sizes.pill_gap` apart, centered inside
/// a quiet [`style::button::bare`] pill at `sizes.hit_target_bar` height.
/// Horizontal padding is `paddings.pill_button` with a label,
/// `paddings.icon_button` without.
///
/// **The caller supplies `tint`** — iced 0.14 workaround #3 (module docs):
/// an `Svg`'s color closure is baked in at build time and never re-evaluated
/// per `button::Status`, so the button style cannot dim or highlight the
/// glyph; only the caller knows whether the icon should read enabled,
/// selected, or disabled (saola-files `header.rs::nav_button` picks
/// `on_paper.disabled` itself when the button has no `on_press`, and
/// [`role`] exists to make that choice a one-liner).
pub fn icon_button<'a, M: Clone + 'a>(
    t: &Theme,
    s: Surface,
    kind: Icon,
    label: Option<&'a str>,
    tint: iced::Color,
    on_press: Option<M>,
) -> Element<'a, M> {
    let glyph = icon(kind, t.sizes.icon_row, tint);
    let (content, horizontal_padding): (Element<'a, M>, f32) = match label {
        Some(label) => (
            Row::new()
                .push(glyph)
                .push(
                    text_widget(label)
                        .font(ui_font(t))
                        .size(t.typography.size.body),
                )
                .spacing(t.sizes.pill_gap)
                .align_y(Center)
                .into(),
            t.paddings.pill_button[1],
        ),
        None => (glyph.into(), t.paddings.icon_button[1]),
    };
    button(centered(content))
        .height(t.sizes.hit_target_bar)
        .padding([0.0, horizontal_padding])
        .style(style::button::bare(t, s))
        .on_press_maybe(on_press)
        .into()
}

/// The panel's islands-shape trigger: an outer container drawing the solid
/// ink pill ([`style::container::bar_pill`]) with **no padding**, and an
/// inner [`style::button::bare`] button filling it, carrying both the hover
/// fill and the pill's content padding.
///
/// The layering is iced 0.14 workaround #2 (module docs) and the order is
/// load-bearing: iced paints a container's background first and its content
/// on top, so the button's hover fill (`fill_subtle` at `radii.pill`, the
/// exact same rounded rect) lands *above* the ink and tints the whole pill.
/// Wrapping the ink pill in the button instead would paint the hover fill
/// underneath the ink, where it could never be seen (saola-panel `main.rs`'s
/// island pill, copied verbatim by its `claude_island_pill`).
///
/// The `[0, height / 2]` padding is the pill-padding law: iced clamps a
/// radius-999 pill's corner radius to `height / 2`, so `height / 2` is the
/// closest content can sit to the pill's end without leaving the rounded
/// fill. Nesting the content's own buttons inside this one is safe —
/// children update first, and a captured event stops there.
pub fn hover_pill<'a, M: Clone + 'a>(
    t: &Theme,
    height: f32,
    content: impl Into<Element<'a, M>>,
    on_press: M,
) -> Element<'a, M> {
    container(
        button(centered(content))
            .height(Fill)
            .padding([0.0, height / 2.0])
            .style(style::button::bare(t, Surface::Ink))
            .on_press(on_press),
    )
    .height(height)
    .style(style::container::bar_pill(t))
    .into()
}

/// One row in a menu (tray menu, context menu): an optional leading glyph
/// at `sizes.icon_menu`, a body-sized label, `paddings.strip` padding, full
/// width, styled [`style::button::menu_row`] (quiet at rest, full terracotta
/// with an ivory label on hover — hover is the selection preview).
///
/// `enabled` is derived from `on_press.is_some()` — exactly the distinction
/// `style::button::menu_row` documents: a button without `.on_press` reports
/// `Status::Disabled`, so the style's `enabled` flag (not the status) picks
/// the resting label color. The **label** rides the button style's per-status
/// text color (rest → hover ivory swap included); the **glyph** cannot
/// (workaround #3), so the caller picks `tint` — [`role`]`(t, s,
/// Emphasis::Rest)` for an ordinary item, `Emphasis::Disabled` for a dead
/// one, matching what the label will do at rest. (Promoted from saola-files
/// `menus.rs`, two near-identical clones; saola-panel's tray rows.)
pub fn menu_row<'a, M: Clone + 'a>(
    t: &Theme,
    s: Surface,
    kind: Option<Icon>,
    label: &'a str,
    tint: iced::Color,
    on_press: Option<M>,
) -> Element<'a, M> {
    let mut content = Row::new().spacing(t.sizes.pill_gap).align_y(Center);
    if let Some(kind) = kind {
        content = content.push(icon(kind, t.sizes.icon_menu, tint));
    }
    content = content.push(
        text_widget(label)
            .font(ui_font(t))
            .size(t.typography.size.body),
    );

    let enabled = on_press.is_some();
    button(content)
        .style(style::button::menu_row(t, s, enabled))
        .width(Fill)
        .padding(t.paddings.strip)
        .on_press_maybe(on_press)
        .into()
}

// ---------------------------------------------------------------------------
// Segmented control
// ---------------------------------------------------------------------------

/// Height of one segment in [`segmented_row`]: the track is padded
/// `sizes.segment_inset` on every side, so this is what makes the assembled
/// control land at exactly `sizes.hit_target_bar` tall (asserted in the
/// tests below).
fn segment_height(t: &Theme) -> f32 {
    t.sizes.hit_target_bar - 2.0 * t.sizes.segment_inset
}

/// A closed-set selector: a row of pill segments over a
/// [`style::segmented::track`], one [`style::segmented::segment`] per
/// option, the selected one lit terracotta. `secondary`-sized labels,
/// centered per segment (the [`centered`]-style sandwich inlined, because
/// each segment also centers horizontally); `sizes.island_gap` horizontal
/// padding per segment; `sizes.segment_inset` as both the track's padding
/// and the gap between segments.
///
/// Ported from saola-capture, where `app::segmented_row` and
/// `editor::segmented_row` were near-identical twins (the second existing
/// only because the first hardcoded its module's `Message` — the generic
/// `on_select: Fn(T) -> M` here is the refactor its doc comment priced out).
/// One deliberate change from those twins: they gave each *segment*
/// `hit_target_bar` height, making the padded track 48 px tall; here the
/// track totals `hit_target_bar` exactly ([`segment_height`]), so the
/// control sits at standard control height next to a [`pill_button`].
///
/// `on_select` is a plain `Fn` called once per option to build that
/// option's press message — a tuple-variant constructor
/// (`Message::TargetSelected`) is itself a `Fn(T) -> M`, so call sites
/// usually pass the constructor bare.
pub fn segmented_row<'a, T, M>(
    t: &Theme,
    s: Surface,
    options: &[(T, &'a str)],
    selected: &T,
    on_select: impl Fn(T) -> M,
) -> Element<'a, M>
where
    T: Clone + PartialEq,
    M: Clone + 'a,
{
    let inset = t.sizes.segment_inset;
    let height = segment_height(t);
    let font = ui_font(t);
    let size = t.typography.size.secondary;

    let mut segments = Row::new().spacing(inset);
    for (value, label) in options {
        let is_selected = value == selected;
        // The centering sandwich, per segment: vertically load-bearing
        // (fixed-height button, zero vertical padding), horizontally the
        // segment hugs its label — `align_x` is set anyway so the intent
        // survives if a segment ever gets a fixed width (saola-capture's
        // note, kept).
        let content = container(text_widget(*label).font(font).size(size))
            .align_x(Center)
            .align_y(Center)
            .height(Fill);
        segments = segments.push(
            button(content)
                .height(height)
                .padding([0.0, t.sizes.island_gap])
                .style(style::segmented::segment(t, s, is_selected))
                .on_press(on_select(value.clone())),
        );
    }

    container(segments)
        .padding(inset)
        .style(style::segmented::track(t, s))
        .into()
}

// ---------------------------------------------------------------------------
// Status-mark roles
// ---------------------------------------------------------------------------

/// How strongly a status mark (a bare bar glyph, a readout icon) should
/// read — the four-way ternary saola-panel's status modules each wrote by
/// hand (`battery`, `network`, `bluetooth`, `volume`, `media`: five copies
/// of "accent when live, `primary` at rest, `secondary` when quiet,
/// `disabled` when dead"). Feed it to [`role`] for the color.
///
/// This lives in `widget` rather than `style` because it exists to pick the
/// **tint argument** the icon constructors take (workaround #3: an `Svg`'s
/// tint is the caller's job) — it is an input to widget assembly, not an
/// `.style(...)` closure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emphasis {
    /// On / charging / playing — the accent, in its per-surface *text* form
    /// (`accent_light` on ink, `accent_dark` on paper), because a status
    /// glyph is stroke-drawn like text. A *filled* shape (saola-panel's
    /// play glyph) takes raw `palette.accent` directly instead — see
    /// `modules::media::glyph_color`'s note.
    Live,
    /// Present and fine — full-emphasis `primary`. "Connected" is a resting
    /// state, never a terracotta one (saola-panel `bluetooth`/`network`
    /// module docs).
    Rest,
    /// Present but muted / off / idle — the `secondary` role.
    Quiet,
    /// Interactive but currently unavailable — the `disabled` role.
    Disabled,
}

/// The color for a status mark at the given [`Emphasis`] on the given
/// surface. Returns an `iced::Color`, ready to hand to [`crate::icon::icon`]
/// or a text `.color(...)`.
pub fn role(t: &Theme, s: Surface, emphasis: Emphasis) -> iced::Color {
    match emphasis {
        Emphasis::Live => match s {
            Surface::Ink => t.palette.accent_light,
            Surface::Paper => t.palette.accent_dark,
        },
        Emphasis::Rest => t.on(s).primary,
        Emphasis::Quiet => t.on(s).secondary,
        Emphasis::Disabled => t.on(s).disabled,
    }
    .into_iced()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_live_is_the_per_surface_accent_text() {
        let t = Theme::saola();
        assert_eq!(
            role(&t, Surface::Ink, Emphasis::Live),
            t.palette.accent_light.into_iced()
        );
        assert_eq!(
            role(&t, Surface::Paper, Emphasis::Live),
            t.palette.accent_dark.into_iced()
        );
    }

    #[test]
    fn role_maps_rest_quiet_disabled_to_the_on_surface_ladder() {
        let t = Theme::saola();
        for s in [Surface::Ink, Surface::Paper] {
            assert_eq!(role(&t, s, Emphasis::Rest), t.on(s).primary.into_iced());
            assert_eq!(role(&t, s, Emphasis::Quiet), t.on(s).secondary.into_iced());
            assert_eq!(
                role(&t, s, Emphasis::Disabled),
                t.on(s).disabled.into_iced()
            );
        }
    }

    /// The four emphases must be four *distinct* colors per surface, or the
    /// ladder stops carrying information.
    #[test]
    fn role_ladder_is_distinct_per_surface() {
        let t = Theme::saola();
        for s in [Surface::Ink, Surface::Paper] {
            let ladder = [
                role(&t, s, Emphasis::Live),
                role(&t, s, Emphasis::Rest),
                role(&t, s, Emphasis::Quiet),
                role(&t, s, Emphasis::Disabled),
            ];
            for (i, a) in ladder.iter().enumerate() {
                for b in ladder.iter().skip(i + 1) {
                    assert_ne!(a, b, "emphasis ladder collapsed on {s:?}");
                }
            }
        }
    }

    /// The design call [`segment_height`] encodes: track padding + segment
    /// height must total exactly `hit_target_bar`, and the segment must
    /// keep a real height.
    #[test]
    fn segmented_track_totals_hit_target_bar() {
        let t = Theme::saola();
        let total = segment_height(&t) + 2.0 * t.sizes.segment_inset;
        assert_eq!(total, t.sizes.hit_target_bar);
        assert!(segment_height(&t) > 0.0);
    }

    /// Every constructor builds without a renderer — a pure smoke test that
    /// the generics, lifetimes, and style plumbing line up.
    #[test]
    fn constructors_build() {
        let t = Theme::saola();
        let s = Surface::Ink;

        let _: Element<'_, ()> = separator(&t, s);
        let _: Element<'_, ()> = quiet_row(&t, s, "no backend");
        let _: Element<'_, ()> = empty_state(&t, s, "This folder is empty");
        let _: Element<'_, ()> = section_label(&t, s, "PLACES");
        let _: Element<'_, ()> = footer_strip(&t, s, hairline(&t, s));
        let _: Element<'_, ()> = swatch(24.0, 8.0, style::container::badge(&t));
        let _: Element<'_, ()> = dot(6.0, style::container::badge(&t));
        let _: Element<'_, ()> = pill_button(&t, s, "Save", Some(()), true);
        let _: Element<'_, ()> = pill_button(&t, s, "Save", None, false);
        let _: Element<'_, ()> = icon_button(
            &t,
            s,
            Icon::Check,
            Some("Apply"),
            role(&t, s, Emphasis::Rest),
            Some(()),
        );
        let _: Element<'_, ()> = icon_button(
            &t,
            s,
            Icon::Check,
            None,
            role(&t, s, Emphasis::Disabled),
            None,
        );
        let _: Element<'_, ()> = hover_pill(&t, 28.0, quiet_row(&t, s, "cluster"), ());
        let _: Element<'_, ()> = menu_row(
            &t,
            s,
            Some(Icon::Check),
            "Open",
            role(&t, s, Emphasis::Rest),
            Some(()),
        );
        let _: Element<'_, ()> = menu_row(
            &t,
            s,
            None,
            "Unavailable",
            role(&t, s, Emphasis::Disabled),
            None,
        );
        let _: Element<'_, ()> =
            segmented_row(&t, s, &[(0u8, "Files"), (1, "Folders")], &0u8, |_| ());
        let _: iced::widget::Text<'_> = text::body(&t, s, "body");
        let _: iced::widget::Text<'_> = text::error(&t, s, "wrong password");
    }
}
