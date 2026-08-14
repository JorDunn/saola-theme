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
use iced::widget::text::IntoFragment;
use iced::widget::{
    button, container, progress_bar, rule, Column, Container, ProgressBar, Row, Rule, Space,
};
use iced::{Center, Element, Fill};
use saola_tokens::{Chrome, Surface, Theme};

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
// Progress
// ---------------------------------------------------------------------------

/// The determinate progress rule at the canonical `sizes.progress_girth`
/// thickness: fills the available width, styled [`style::progress::bar`].
/// Exists so a consumer never hand-picks a girth for an ordinary
/// `progress_bar` the way [`hairline`] exists so nobody hand-picks a
/// hairline's thickness.
///
/// `value` is already-normalized `0.0..=1.0` progress — matching
/// [`crate::motion::life_fraction`]'s convention — rather than
/// `progress_bar`'s own doc example range of `0.0..=100.0`.
///
/// Pairs with [`crate::indeterminate::indeterminate_rule`], the
/// §7 indeterminate sibling wired to the same `sizes.progress_girth` token,
/// so a boot menu can swap between the determinate and indeterminate rule
/// without either one picking a different height.
///
/// ```no_run
/// use saola_theme::{widget, Surface, Theme};
///
/// let t = Theme::saola();
/// let _rule: iced::widget::ProgressBar<'_> = widget::progress_rule(&t, Surface::Paper, 0.4);
/// ```
pub fn progress_rule<'a>(t: &Theme, s: Surface, value: f32) -> ProgressBar<'a> {
    progress_bar(0.0..=1.0, value)
        .length(Fill)
        .girth(t.sizes.progress_girth)
        .style(style::progress::bar(t, s))
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
/// `emphasized`, the rest recipe for `s`/`c` otherwise) at fixed
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
/// use saola_theme::{widget, Chrome, Surface, Theme};
///
/// let t = Theme::saola();
/// let _save: iced::Element<'_, ()> =
///     widget::pill_button(&t, Surface::Paper, Chrome::Window, "Save", Some(()), true);
/// ```
pub fn pill_button<'a, M: Clone + 'a>(
    t: &Theme,
    s: Surface,
    c: Chrome,
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
        .style(style::button::emphasis(t, s, c, emphasized))
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
// Breadcrumb trail
// ---------------------------------------------------------------------------

/// A file-picker breadcrumb trail (style guide §7): one
/// [`style::button::breadcrumb`] pill per crumb with `paddings.breadcrumb`
/// applied — the padding a button *style* closure can't set, same gap this
/// crate's other token-plus-style pairings close (`life_rule`/`icon_tile`'s
/// split in Stage 13, `hairline`'s thickness above) — separated by a
/// `sizes.icon_bar`-sized [`Icon::ChevronRight`] glyph in the surface's
/// `quaternary` role.
///
/// The separator is traced from the concepts' file-picker row (`9f File
/// picker ink`'s `crumbsInk` list): each segment is followed by an `#i-chev`
/// glyph — Lucide's `chevron-right`, unrotated — stroked at `rgba(*, .35)`,
/// which lands almost exactly on `quaternary`'s alpha step (`.40` on ink,
/// `.45` on paper) rather than any other role on the ladder.
///
/// A crumb's `on_press` doubles as "is this the current folder?" — `None`
/// marks the current crumb, mirroring [`menu_row`]'s "enabled derived from
/// `on_press.is_some()`" convention: it renders emphasized via
/// [`style::button::breadcrumb`]'s `is_current` branch and captures no
/// clicks, matching the reality that navigating to the folder you're
/// already standing in should do nothing. Every other crumb is clickable
/// and quiet until hovered.
///
/// Labels are `impl IntoFragment<'a>` — `&str`, `String`, or `Cow<'a, str>`
/// — so per-frame owned path segments work (the trail a file manager builds
/// from a `PathBuf` each `view` call). `IntoFragment` consumes its value,
/// which is why crumbs come in by value (`impl IntoIterator`, typically an
/// array literal) rather than the slice the rest of this module favors. One
/// label type per call: `L` is monomorphic, so a trail mixing `&str` and
/// `String` converts everything to one of them first.
///
/// ```no_run
/// use saola_theme::{widget, Surface, Theme};
///
/// let t = Theme::saola();
/// let _trail: iced::Element<'_, ()> = widget::breadcrumb(
///     &t,
///     Surface::Paper,
///     [("Home", Some(())), ("Projects", Some(())), ("saola-theme", None)],
/// );
/// let name = String::from("saola-theme");
/// let _owned: iced::Element<'_, ()> = widget::breadcrumb(
///     &t,
///     Surface::Paper,
///     [(String::from("Home"), Some(())), (name, None)],
/// );
/// ```
pub fn breadcrumb<'a, L, M>(
    t: &Theme,
    s: Surface,
    crumbs: impl IntoIterator<Item = (L, Option<M>)>,
) -> Element<'a, M>
where
    L: IntoFragment<'a>,
    M: Clone + 'a,
{
    let separator_tint = t.on(s).quaternary.into_iced();
    let mut crumbs = crumbs.into_iter().peekable();
    let mut trail = Row::new().spacing(t.sizes.pill_gap).align_y(Center);
    while let Some((label, on_press)) = crumbs.next() {
        let is_current = on_press.is_none();
        trail = trail.push(
            button(
                text_widget(label)
                    .font(ui_font(t))
                    .size(t.typography.size.secondary),
            )
            .padding(t.paddings.breadcrumb)
            .style(style::button::breadcrumb(t, s, is_current))
            .on_press_maybe(on_press),
        );
        if crumbs.peek().is_some() {
            trail = trail.push(icon(Icon::ChevronRight, t.sizes.icon_bar, separator_tint));
        }
    }
    trail.into()
}

// ---------------------------------------------------------------------------
// Bare-icon menu (power / boot menu)
// ---------------------------------------------------------------------------

/// One item in a bare-icon menu — the power/boot menu's shape (style guide
/// §6, captioned "bare icons on the panel — ivory at 55% at rest, full
/// terracotta when hovered or selected"): the glyph at `sizes.icon_bare`
/// (wired here — it had no callers before this), tinted `on_ink.tertiary`
/// at rest (ivory's `.55`-alpha step — the concepts' "55%" verbatim) and
/// [`Emphasis::Live`]'s ink-context accent (`palette.accent_light`, the same
/// tint every other "on/selected" status glyph in this crate uses) when
/// `hovered`. Reusing [`role`]/[`Emphasis`] here — rather than a new tint
/// enum — is deliberate: the plan asked for it, and it is the same iced
/// 0.14 workaround #3 every icon-bearing constructor in this module already
/// documents (an `Svg`'s color is fixed at build time, so a button's live
/// `Status` can't drive it — the caller has to pick the tint up front,
/// which is exactly what `hovered` does here).
///
/// Below the glyph sits a `sizes.grid_tile_label`-tall reserved block —
/// borrowing the "label band under a square item" token
/// [`crate::style::button::selection_tile`]'s grid-tile geometry already
/// uses — holding the label only when `hovered`, empty otherwise. Every
/// item reserves the *same* height regardless of its own hover state, which
/// is the point: the concepts caption this "one shared label" specifically
/// so the row's height never shifts as different items hover in and out.
///
/// The outer button paints nothing at any state
/// ([`style::button::bare_icon`]) — "icons directly on the surface" means
/// no pill, no fill, ever; the glyph and its label carry the whole hover
/// signal. Ink-only, no `Surface` parameter: these menus live on shell
/// scrims (style guide §6), never a paper window.
pub fn bare_icon_item<'a, M: Clone + 'a>(
    t: &Theme,
    kind: Icon,
    label: &'a str,
    hovered: bool,
    on_press: Option<M>,
) -> Element<'a, M> {
    let tint = if hovered {
        role(t, Surface::Ink, Emphasis::Live)
    } else {
        t.on_ink.tertiary.into_iced()
    };
    let caption: Element<'a, M> = if hovered {
        text_widget(label)
            .font(ui_font_regular(t))
            .size(t.typography.size.secondary)
            .color(tint)
            .into()
    } else {
        Space::new().into()
    };
    let content = Column::new()
        .align_x(Center)
        .spacing(t.sizes.gap_tight)
        .push(icon(kind, t.sizes.icon_bare, tint))
        .push(
            container(caption)
                .width(Fill)
                .height(t.sizes.grid_tile_label)
                .align_x(Center)
                .align_y(Center),
        );
    button(content)
        .style(style::button::bare_icon(t))
        .on_press_maybe(on_press)
        .into()
}

// ---------------------------------------------------------------------------
// Segmented control
// ---------------------------------------------------------------------------

/// Height of one segment in [`segmented_row`]: the track is padded
/// `sizes.track_inset` on every side, so this is what makes the assembled
/// control land at exactly `sizes.hit_target_bar` tall (asserted in the
/// tests below).
fn segment_height(t: &Theme) -> f32 {
    t.sizes.hit_target_bar - 2.0 * t.sizes.track_inset
}

/// A closed-set selector: a row of pill segments over a
/// [`style::segmented::track`], one [`style::segmented::segment`] per
/// option, the selected one lit terracotta. `secondary`-sized labels,
/// centered per segment (the [`centered`]-style sandwich inlined, because
/// each segment also centers horizontally); `sizes.island_gap` horizontal
/// padding per segment; `sizes.track_inset` as the track's own padding
/// (the "inset of a handle's travel inside its track" token, applied here
/// to the segment row's travel inside the track container) and
/// `sizes.segment_inset` as the gap between segments — two different roles
/// that happen to share a value (4.0) today.
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
    c: Chrome,
    options: &[(T, &'a str)],
    selected: &T,
    on_select: impl Fn(T) -> M,
) -> Element<'a, M>
where
    T: Clone + PartialEq,
    M: Clone + 'a,
{
    let track_inset = t.sizes.track_inset;
    let segment_gap = t.sizes.segment_inset;
    let height = segment_height(t);
    let font = ui_font(t);
    let size = t.typography.size.secondary;

    let mut segments = Row::new().spacing(segment_gap);
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
                .style(style::segmented::segment(t, s, c, is_selected))
                .on_press(on_select(value.clone())),
        );
    }

    container(segments)
        .padding(track_inset)
        .style(style::segmented::track(t, s))
        .into()
}

/// The content color [`style::segmented::segment`] gives a segment's label
/// at rest/hover/press — computed up front because an `Svg`'s tint is baked
/// at build time (workaround #3 in the module docs). The three values are
/// copied verbatim from that style's `selected_label`/`rest_label` locals;
/// parity is asserted in [`tests::segment_tint_matches_the_segment_styles_label_color`].
fn segment_tint(t: &Theme, s: Surface, c: Chrome, is_selected: bool) -> iced::Color {
    if is_selected {
        t.palette.paper.into_iced()
    } else {
        match (s, c) {
            (Surface::Ink, Chrome::Shell) => t.palette.ink.into_iced(),
            (Surface::Ink, Chrome::Window) => t.on_ink.primary.into_iced(),
            (Surface::Paper, _) => t.on_paper.primary.into_iced(),
        }
    }
}

/// [`segmented_row`]'s icon-only sibling: the same track-and-segments
/// assembly (same [`segment_height`], `sizes.track_inset` padding,
/// `sizes.island_gap` per-segment padding, [`style::segmented`] styles)
/// with a `sizes.icon_row` glyph per segment instead of a label — the
/// list/grid view-switcher shape ([`Icon::List`] | [`Icon::LayoutGrid`]).
///
/// Unlike [`icon_button`], the tint is *not* the caller's job: a segment's
/// content color is fully determined by `is_selected`, the surface, and the
/// chrome, all of which this constructor already knows, so [`segment_tint`]
/// computes it internally. Baking the tint is safe here — workaround #3
/// (an `Svg` can't follow the button's live `Status`) costs nothing,
/// because [`style::segmented::segment`] keeps its label color constant
/// across `Active`/`Hovered`/`Pressed` (only `Disabled` differs, and no
/// segment is ever disabled: every one gets an `on_press`). A hover that
/// *did* recolor the label would drift from the glyph; the parity test
/// pins the recipe so that can't happen silently.
pub fn segmented_row_icons<'a, T, M>(
    t: &Theme,
    s: Surface,
    c: Chrome,
    options: &[(T, Icon)],
    selected: &T,
    on_select: impl Fn(T) -> M,
) -> Element<'a, M>
where
    T: Clone + PartialEq,
    M: Clone + 'a,
{
    let track_inset = t.sizes.track_inset;
    let segment_gap = t.sizes.segment_inset;
    let height = segment_height(t);

    let mut segments = Row::new().spacing(segment_gap);
    for (value, kind) in options {
        let is_selected = value == selected;
        // The same centering sandwich as `segmented_row`, glyph for label.
        let content = container(icon(
            *kind,
            t.sizes.icon_row,
            segment_tint(t, s, c, is_selected),
        ))
        .align_x(Center)
        .align_y(Center)
        .height(Fill);
        segments = segments.push(
            button(content)
                .height(height)
                .padding([0.0, t.sizes.island_gap])
                .style(style::segmented::segment(t, s, c, is_selected))
                .on_press(on_select(value.clone())),
        );
    }

    container(segments)
        .padding(track_inset)
        .style(style::segmented::track(t, s))
        .into()
}

// ---------------------------------------------------------------------------
// Notification centre
// ---------------------------------------------------------------------------

/// The notification-centre group-header row (style guide §6: the centre is
/// `sizes.notification_centre_width` wide, grouped by app, collapsible
/// groups): the app name in the [`text::label`] role, an unread-count
/// [`style::container::chip`] (omitted when `count` is zero — a header with
/// nothing unread doesn't need an empty pill), and a disclosure glyph that
/// flips with `collapsed`.
///
/// Built over the same row mechanics [`quiet_row`]/[`list_row_container`]
/// use ([`centered`]'s vertical-centering sandwich, `sizes.list_row` height,
/// `paddings.strip` horizontal padding — [`menu_row`]'s exact geometry, since
/// a group header is a menu-row twin that happens to carry a chip and a
/// glyph instead of a leading icon) and [`style::button::list_row`] for its
/// rest/hover states — a header is a row you click to toggle, not a control
/// with a selected/focused identity, so `selected`/`focused` are both
/// `false`.
///
/// There is no `Icon::ChevronDown` asset — only [`Icon::ChevronRight`] — so
/// the flip is a `Svg::rotation` (`iced_widget::svg::Svg::rotation`,
/// `Rotation::Floating` via its `From<f32>` impl) rather than a second
/// glyph: `0.0` collapsed (pointing at the group, closed), a quarter turn
/// clockwise when expanded (pointing down at the revealed rows). Floating
/// rotation keeps the glyph's own layout box unrotated, so it doesn't
/// nudge the row's height or the chip's position as it turns.
///
/// Ink-only, no `Surface` parameter: the notification centre lives on the
/// shell layer (style guide §6), the same ink-only rule
/// [`crate::style::notification`]'s toast internals already follow.
///
/// `on_toggle: None` renders the disabled look and captures nothing — the
/// same `button` convention every other constructor in this module uses.
///
/// ```no_run
/// use saola_theme::{widget, Theme};
///
/// let t = Theme::saola();
/// let _closed: iced::Element<'_, ()> =
///     widget::group_header(&t, "Files", 3, true, Some(()));
/// let _open: iced::Element<'_, ()> =
///     widget::group_header(&t, "Files", 3, false, Some(()));
/// ```
pub fn group_header<'a, M: Clone + 'a>(
    t: &Theme,
    app_name: &'a str,
    count: usize,
    collapsed: bool,
    on_toggle: Option<M>,
) -> Element<'a, M> {
    let s = Surface::Ink;

    let mut row = Row::new()
        .spacing(t.sizes.pill_gap)
        .align_y(Center)
        .width(Fill)
        .push(text::label(t, s, app_name));

    if count > 0 {
        row = row.push(
            container(text_widget(count.to_string()).size(t.typography.size.meta))
                .style(style::container::chip(t, s))
                .padding([2.0, 8.0]),
        );
    }

    let chevron_tint = t.on(s).secondary.into_iced();
    let rotation: f32 = if collapsed {
        0.0
    } else {
        90.0_f32.to_radians()
    };
    row = row
        .push(Space::new().width(Fill))
        .push(icon(Icon::ChevronRight, t.sizes.icon_bar, chevron_tint).rotation(rotation));

    button(centered(row))
        .width(Fill)
        .height(t.sizes.list_row)
        .padding(t.paddings.strip)
        .style(style::button::list_row(t, s, false, false))
        .on_press_maybe(on_toggle)
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

    /// The design call [`segment_height`] encodes: track padding
    /// (`track_inset`) + segment height must total exactly `hit_target_bar`,
    /// and the segment must keep a real height.
    #[test]
    fn segmented_track_totals_hit_target_bar() {
        let t = Theme::saola();
        let total = segment_height(&t) + 2.0 * t.sizes.track_inset;
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
        let _: iced::widget::ProgressBar<'_> = progress_rule(&t, s, 0.4);
        let _: Element<'_, ()> = quiet_row(&t, s, "no backend");
        let _: Element<'_, ()> = empty_state(&t, s, "This folder is empty");
        let _: Element<'_, ()> = section_label(&t, s, "PLACES");
        let _: Element<'_, ()> = footer_strip(&t, s, hairline(&t, s));
        let _: Element<'_, ()> = swatch(24.0, 8.0, style::container::badge(&t));
        let _: Element<'_, ()> = dot(6.0, style::container::badge(&t));
        let _: Element<'_, ()> = pill_button(&t, s, Chrome::Shell, "Save", Some(()), true);
        let _: Element<'_, ()> = pill_button(&t, s, Chrome::Window, "Save", None, false);
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
        let _: Element<'_, ()> = segmented_row(
            &t,
            s,
            Chrome::Shell,
            &[(0u8, "Files"), (1, "Folders")],
            &0u8,
            |_| (),
        );
        let _: Element<'_, ()> = segmented_row_icons(
            &t,
            s,
            Chrome::Window,
            &[(0u8, Icon::List), (1, Icon::LayoutGrid)],
            &0u8,
            |_| (),
        );
        let _: Element<'_, ()> = breadcrumb(
            &t,
            s,
            [
                ("Home", Some(())),
                ("Projects", Some(())),
                ("saola-theme", None),
            ],
        );
        let _: Element<'_, ()> = breadcrumb(
            &t,
            s,
            [
                (String::from("Home"), Some(())),
                (String::from("src"), None),
            ],
        );
        let _: Element<'_, ()> = bare_icon_item(&t, Icon::Lock, "Lock", false, Some(()));
        let _: Element<'_, ()> = bare_icon_item(&t, Icon::Lock, "Lock", true, Some(()));
        let _: Element<'_, ()> = group_header(&t, "Files", 3, true, Some(()));
        let _: Element<'_, ()> = group_header(&t, "Files", 3, false, Some(()));
        let _: Element<'_, ()> = group_header(&t, "Settings", 0, true, None);
        let _: iced::widget::Text<'_> = text::body(&t, s, "body");
        let _: iced::widget::Text<'_> = text::error(&t, s, "wrong password");
    }

    /// A breadcrumb crumb with `on_press: None` is the current-folder crumb
    /// — [`breadcrumb`]'s documented convention, mirroring [`menu_row`]'s
    /// "enabled derived from `on_press.is_some()`". This only asserts the
    /// convention compiles and holds for a plain `Option`; the emphasized
    /// *rendering* itself is exercised by [`style::button::breadcrumb`]'s
    /// `is_current` branch, which has no `Style` output to assert against
    /// without a renderer.
    #[test]
    fn breadcrumb_current_crumb_has_no_on_press() {
        let crumbs: &[(&str, Option<()>)] = &[("Home", Some(())), ("saola-theme", None)];
        let is_current: Vec<bool> = crumbs.iter().map(|(_, m)| m.is_none()).collect();
        assert_eq!(is_current, vec![false, true]);
    }

    /// [`segment_tint`] is a verbatim copy of the label colors inside
    /// [`style::segmented::segment`] (a baked `Svg` tint can't read them
    /// through the style closure) — this pins the copy to the original so
    /// the two can't drift apart silently.
    #[test]
    fn segment_tint_matches_the_segment_styles_label_color() {
        let t = Theme::saola();
        for s in [Surface::Ink, Surface::Paper] {
            for c in [Chrome::Shell, Chrome::Window] {
                for is_selected in [false, true] {
                    // The closure ignores its `&iced::Theme` argument (every
                    // color was captured from the Saola theme), so any variant
                    // works here.
                    let style = style::segmented::segment(&t, s, c, is_selected)(
                        &iced::Theme::Light,
                        iced::widget::button::Status::Active,
                    );
                    assert_eq!(segment_tint(&t, s, c, is_selected), style.text_color);
                }
            }
        }
    }
}
