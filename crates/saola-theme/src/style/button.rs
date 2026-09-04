//! Button styles: `rest`, `active`, `emphasis`, `muted`, `bare`,
//! `list_row`, `selection_tile`, `menu_row`, `breadcrumb`, `bare_icon`.
//!
//! The one rule, applied to buttons:
//!
//! - [`rest`] — a control at rest: a solid **ivory** pill on ink (ink label)
//!   in shell chrome, a translucent ivory **fill** pill on ink inside an app
//!   window, an ink **fill** pill on paper (ink label) in both.
//! - [`active`] — on / selected / live: a **terracotta** pill with an ivory
//!   label, identical on both surfaces.
//! - [`emphasis`] — [`rest`] or [`active`] behind one closure type, picked
//!   by a `bool`, for consumers that flip a button between the two.
//! - [`rest_faded`]/[`emphasis_faded`] — [`rest`]/[`emphasis`] at an `alpha`
//!   opacity, for a button drawn inside a fading toast or card (iced 0.14 has
//!   no subtree opacity, so the fade has to reach every painted color).
//!   `rest`/`emphasis` are thin `alpha: 1.0` wrappers over these.
//! - [`muted`] — muted / off-ish: a **subtle-fill** pill with a
//!   secondary-emphasis label, quieter than `rest`.
//! - [`bare`] — label only; hover/press surface it through the fill steps.
//! - [`list_row`] — a content row (file listing, sidebar place): [`bare`]'s
//!   progression at rest, [`active`]'s terracotta when `selected`.
//! - [`selection_tile`] — [`list_row`]'s exact recipe at `radii.tile`, for
//!   grid-view tiles.
//! - [`menu_row`] — a menu option: quiet at rest, terracotta the moment it
//!   is hovered ("hover is the selection preview").
//! - [`breadcrumb`] — a file-picker path segment (style guide §7): quiet
//!   (`secondary` label, `fill_subtle` hover) for a segment you can still
//!   navigate to; the current segment ignores `Status` entirely and always
//!   draws [`active`]'s terracotta "on" look.
//! - [`bare_icon`] — a click target with **no background at any state**
//!   (style guide §6's power/boot menu: "icons directly on the surface").
//!   All of the state change rides the glyph's own tint, which a button
//!   style closure can't reach — see
//!   [`crate::widget::bare_icon_item`].
//!
//! There is deliberately no `danger` variant: Saola has three colors, never
//! a fourth. Destructive confirmation is a consumer *pattern* (wording,
//! placement, a second step), not a palette entry.
//!
//! ## How hover/press are derived
//!
//! State layering moves through the alpha fill steps, never new colors. A
//! button has a single flat `background`, so there are two cases:
//!
//! - The button's resting fill is **translucent** (`bare`, `rest` on paper,
//!   `rest` on ink in [`Chrome::Window`]): we just pick a deeper fill step
//!   and let iced blend it over whatever surface is behind the button.
//! - The button's resting fill is **opaque** (ivory `rest` on ink in
//!   [`Chrome::Shell`], terracotta `active`): the fill step must layer over
//!   the button's *own*
//!   fill, so we pre-composite in token space with
//!   `saola_tokens::Color::over` (opaque base ⇒ opaque result). An ivory
//!   pill is a tiny paper surface, so it hovers through the *on-paper*
//!   steps; a terracotta pill takes ivory content, so it hovers through the
//!   *on-ink* (ivory) steps.
//!
//! ## Focus
//!
//! iced 0.14's `button::Status` is exactly `Active | Hovered | Pressed |
//! Disabled` — there is no focus variant, so the 2 px terracotta focus ring
//! cannot be expressed here. Use [`crate::style::focus_border`] from
//! consumer code that tracks keyboard focus itself.

use iced::widget::button::{Status, Style};
use iced::Background;
use saola_tokens::{Chrome, Surface, Theme};

use crate::convert::ColorExt;

/// Assemble a pill-shaped [`Style`] from an optional background and a label
/// color. `..Style::default()` keeps iced's own defaults for shadow and
/// pixel snapping.
fn pill(background: Option<iced::Color>, text_color: iced::Color, radius: f32) -> Style {
    Style {
        background: background.map(Background::Color),
        text_color,
        border: super::border_none(radius),
        ..Style::default()
    }
}

/// A control at rest — off, unselected, available.
///
/// On ink the recipe depends on `c`. In [`Chrome::Shell`] (panel, popover,
/// launcher) it is the style guide §6 secondary pill: a solid ivory fill with
/// an ink label. In [`Chrome::Window`] — an ink app window — a full-opacity
/// ivory pill out-shouts the one terracotta control it sits beside, so rest
/// recedes into the `on_ink` fill ladder instead (`fill` → `fill_strong` →
/// `track`) with an `on_ink.primary` label.
///
/// On paper the two chromes are **identical**: a control at rest is already a
/// translucent ink fill there, so there is nothing louder to step back from.
///
/// Hover and press step through the fill roles. (On paper, and on ink in
/// window context, `fill_strong` and `track` share a value by construction,
/// so press reads one step past hover only where the tokens provide one.)
pub fn rest(t: &Theme, s: Surface, c: Chrome) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    rest_faded(t, s, c, 1.0)
}

/// [`rest`] at `alpha` opacity: the identical recipe, with every arm's
/// background *and* label color — [`Status::Disabled`] included — scaled by
/// [`crate::convert::ColorExt::with_opacity`]. [`rest`] is the thin
/// `alpha: 1.0` wrapper over this function (not a parallel copy), so the two
/// recipes cannot drift apart.
///
/// Exists for a control drawn inside a fading toast or card: iced 0.14 has
/// no subtree opacity, so a view fading the card around a button must also
/// fade the button itself, the same reason
/// [`crate::style::container::notification_card`] takes `alpha`.
///
/// `alpha` is clamped to `0.0..=1.0` (a non-finite value reads as `1.0`),
/// same as `notification_card`'s.
pub fn rest_faded(
    t: &Theme,
    s: Surface,
    c: Chrome,
    alpha: f32,
) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    // Copy the Copy token values out of the theme so the closure is 'static.
    let alpha = if alpha.is_finite() {
        alpha.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let radius = t.radii.pill;
    let on = *t.on(s);
    let (rest_bg, hover_bg, press_bg, label) = match (s, c) {
        // Solid ivory pill: its own surface is paper, so hover/press are the
        // on-paper fill steps composited over paper (opaque results).
        (Surface::Ink, Chrome::Shell) => (
            t.palette.paper,
            t.on_paper.fill_subtle.over(t.palette.paper),
            t.on_paper.fill.over(t.palette.paper),
            t.palette.ink,
        ),
        // Translucent ivory-fill pill inside an ink window: iced blends the
        // deeper steps over the ink ground, so there is no `Color::over`
        // pre-compositing here (that is only for opaque fills).
        (Surface::Ink, Chrome::Window) => (
            t.on_ink.fill,
            t.on_ink.fill_strong,
            t.on_ink.track,
            t.on_ink.primary,
        ),
        // Translucent ink-fill pill: iced blends deeper steps over the paper
        // window behind it. Same in both chromes.
        (Surface::Paper, _) => (
            t.on_paper.fill,
            t.on_paper.fill_strong,
            t.on_paper.track,
            t.on_paper.primary,
        ),
    };
    move |_, status| match status {
        Status::Active => pill(
            Some(rest_bg.with_opacity(alpha)),
            label.with_opacity(alpha),
            radius,
        ),
        Status::Hovered => pill(
            Some(hover_bg.with_opacity(alpha)),
            label.with_opacity(alpha),
            radius,
        ),
        Status::Pressed => pill(
            Some(press_bg.with_opacity(alpha)),
            label.with_opacity(alpha),
            radius,
        ),
        Status::Disabled => pill(
            Some(on.fill_subtle.with_opacity(alpha)),
            on.disabled.with_opacity(alpha),
            radius,
        ),
    }
}

/// On, selected, live — a terracotta pill with an ivory label, the same on
/// both surfaces. The surface only decides the disabled treatment.
pub fn active(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    let radius = t.radii.pill;
    let on = *t.on(s);
    let accent = t.palette.accent;
    let label = t.palette.paper;
    // Terracotta takes ivory content, so its hover/press layer the ivory
    // (on-ink) fill steps over the accent (opaque results).
    let hover_bg = t.on_ink.fill_subtle.over(accent);
    let press_bg = t.on_ink.fill.over(accent);
    move |_, status| match status {
        Status::Active => pill(Some(accent.into_iced()), label.into_iced(), radius),
        Status::Hovered => pill(Some(hover_bg.into_iced()), label.into_iced(), radius),
        Status::Pressed => pill(Some(press_bg.into_iced()), label.into_iced(), radius),
        Status::Disabled => pill(
            Some(on.fill_subtle.into_iced()),
            on.disabled.into_iced(),
            radius,
        ),
    }
}

/// Muted / off-ish — a quiet pill for states like volume muted or Wi-Fi
/// offline: same geometry as [`rest`], but a subtle translucent fill with a
/// secondary-emphasis label. Hover and press step the fill deeper
/// (`fill_subtle → fill → fill_strong`); every fill here is translucent, so
/// iced blends it over whatever surface is behind the button.
pub fn muted(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    let radius = t.radii.pill;
    let on = *t.on(s);
    move |_, status| match status {
        Status::Active => pill(
            Some(on.fill_subtle.into_iced()),
            on.secondary.into_iced(),
            radius,
        ),
        Status::Hovered => pill(Some(on.fill.into_iced()), on.secondary.into_iced(), radius),
        Status::Pressed => pill(
            Some(on.fill_strong.into_iced()),
            on.secondary.into_iced(),
            radius,
        ),
        Status::Disabled => pill(
            Some(on.fill_subtle.into_iced()),
            on.disabled.into_iced(),
            radius,
        ),
    }
}

/// A list/sidebar row — the style guide's §6 "List row" treatment, promoted
/// from the three identical local derivations in saola-files
/// (`dirview::list::row_style`, `dirview::grid::tile_style`,
/// `sidebar::row_style`): a pill-radius row that is **transparent at rest**
/// (the row is content sitting directly on its surface, not a control), and
/// surfaces `fill_subtle` on hover, `fill` on press — [`bare`]'s
/// progression, minus its label-only framing. Text at rest is the surface's
/// primary role.
///
/// With `selected`, the one rule takes over: a terracotta fill with an
/// ivory label across every state. Selected hover/press pre-composite the
/// *surface's own* fill steps over the accent (`Color::over`; opaque base ⇒
/// opaque result): on paper that is `on_paper.fill_subtle`/`fill` over
/// accent — exactly the saola-files recipe — and on ink the ivory steps,
/// which coincides with [`active`]'s treatment. In other words: hover on a
/// selected row deepens through the same steps an unselected row uses, just
/// pre-composited because the accent fill is opaque.
///
/// Rows are content, not controls, so `Status::Disabled` (which is also
/// what a row without `.on_press` reports) draws exactly the rest state —
/// matching the saola-files derivations' `_ =>` arms — rather than the
/// grayed treatment the control helpers above use.
///
/// `focused` is the keyboard cursor: iced buttons have no `Status::Focused`,
/// so the consumer tracks the cursor itself and passes it here, and the row
/// draws the [`crate::style::focus_border`] ring (accent at `sizes.ring`)
/// in place of its transparent border. It has to be a parameter rather than
/// an overlay because `button::Style` has exactly one `border` field —
/// saola-files (`dirview::list`) tried to compose the ring *around* an
/// upstream row style and couldn't.
pub fn list_row(
    t: &Theme,
    s: Surface,
    selected: bool,
    focused: bool,
) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    selectable(t, s, selected, focused, t.radii.pill)
}

/// A grid-view tile — [`list_row`]'s exact recipe (transparent rest,
/// `fill_subtle`/`fill` hover/press, terracotta when `selected`, focus ring
/// when `focused`) at `radii.tile` instead of `radii.pill`: a grid tile is
/// the two-dimensional analogue of a list row (promoted from saola-files'
/// `dirview::grid::tile_style`, which documented the recipe as
/// "deliberately identical, just at `radii.tile`"). Pair it with the
/// `sizes.grid_tile*` tokens for geometry.
pub fn selection_tile(
    t: &Theme,
    s: Surface,
    selected: bool,
    focused: bool,
) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    selectable(t, s, selected, focused, t.radii.tile)
}

/// The shared body of [`list_row`] and [`selection_tile`] — one recipe, two
/// radii.
fn selectable(
    t: &Theme,
    s: Surface,
    selected: bool,
    focused: bool,
    radius: f32,
) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    let on = *t.on(s);
    let accent = t.palette.accent;
    let ivory = t.palette.paper;
    let selected_hover = on.fill_subtle.over(accent);
    let selected_press = on.fill.over(accent);
    let border = if focused {
        super::focus_border(t, radius)
    } else {
        super::border_none(radius)
    };
    move |_, status| {
        let style = if selected {
            let background = match status {
                Status::Hovered => selected_hover,
                Status::Pressed => selected_press,
                Status::Active | Status::Disabled => accent,
            };
            pill(Some(background.into_iced()), ivory.into_iced(), radius)
        } else {
            let background = match status {
                Status::Hovered => Some(on.fill_subtle.into_iced()),
                Status::Pressed => Some(on.fill.into_iced()),
                Status::Active | Status::Disabled => None,
            };
            pill(background, on.primary.into_iced(), radius)
        };
        Style { border, ..style }
    }
}

/// A label-only button: transparent at rest, surfacing through
/// `fill_subtle` → `fill` on hover/press.
pub fn bare(t: &Theme, s: Surface) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    let radius = t.radii.pill;
    let on = *t.on(s);
    move |_, status| match status {
        Status::Active => pill(None, on.primary.into_iced(), radius),
        Status::Hovered => pill(
            Some(on.fill_subtle.into_iced()),
            on.primary.into_iced(),
            radius,
        ),
        Status::Pressed => pill(Some(on.fill.into_iced()), on.primary.into_iced(), radius),
        Status::Disabled => pill(None, on.disabled.into_iced(), radius),
    }
}

/// One option row in a menu (a tray menu, a context menu) — quiet at rest,
/// and a full terracotta fill with an ivory label the moment it is hovered
/// or pressed: **hover is the selection preview** in a menu, so the hover
/// treatment is [`active`]'s "selected" look rather than the subtle fill
/// step every other button uses. Radius is `radii.selection` — a menu row
/// highlight, not a pill. (Promoted from saola-panel's
/// `popovers::tray_menu::row_style`, which documented why neither [`bare`]
/// nor [`active`] expresses "quiet until hovered".)
///
/// `enabled` picks the resting label: `on(s).primary` when the item is
/// actionable, `on(s).disabled` when not. It is a parameter (not derived
/// from `Status::Disabled`) because a menu row is only clickable when
/// enabled — a button without `.on_press` reports `Status::Disabled`
/// unconditionally, so the status alone cannot distinguish "disabled item"
/// from "enabled item iced happens to call disabled"; and the enabled row's
/// resting look must not change either way.
pub fn menu_row(
    t: &Theme,
    s: Surface,
    enabled: bool,
) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    let radius = t.radii.selection;
    let accent = t.palette.accent;
    let on_accent = t.palette.paper;
    let rest_label = if enabled {
        t.on(s).primary
    } else {
        t.on(s).disabled
    };
    move |_, status| match status {
        Status::Hovered | Status::Pressed => {
            pill(Some(accent.into_iced()), on_accent.into_iced(), radius)
        }
        Status::Active | Status::Disabled => pill(None, rest_label.into_iced(), radius),
    }
}

/// [`rest`] or [`active`], picked by `emphasized`, behind **one** closure
/// type.
///
/// `rest` and `active` each return their own opaque `impl Fn` type, so an
/// `if`/`else` over the two can't unify on the style argument — consumers
/// flipping a button between the two states were duplicating the whole
/// builder chain per arm (saola-files' breadcrumbs and header, saola-capture's
/// editor, each with a comment explaining the constraint). This helper does
/// the branching *inside* a single closure instead: both recipes are
/// computed into locals up front, and `emphasized` selects between them per
/// status.
///
/// `c` reaches the un-emphasized branch only — it is [`rest`]'s recipe, so it
/// carries [`rest`]'s shell-versus-window split on ink. The emphasized branch
/// is [`active`]'s terracotta, identical in both chromes.
pub fn emphasis(
    t: &Theme,
    s: Surface,
    c: Chrome,
    emphasized: bool,
) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    emphasis_faded(t, s, c, emphasized, 1.0)
}

/// [`emphasis`] at `alpha` opacity: the identical branching, with every
/// arm's background *and* label color — [`Status::Disabled`] included —
/// scaled by [`crate::convert::ColorExt::with_opacity`]. [`emphasis`] is the
/// thin `alpha: 1.0` wrapper over this function (not a parallel copy), so
/// the two recipes cannot drift apart — the same relationship
/// [`rest_faded`] has to [`rest`].
///
/// `alpha` is clamped to `0.0..=1.0` (a non-finite value reads as `1.0`),
/// same as `rest_faded`'s.
pub fn emphasis_faded(
    t: &Theme,
    s: Surface,
    c: Chrome,
    emphasized: bool,
    alpha: f32,
) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    let alpha = if alpha.is_finite() {
        alpha.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let radius = t.radii.pill;
    let on = *t.on(s);
    // `rest`'s recipe — see [`rest`] for the reasoning per surface/chrome.
    let (rest_bg, rest_hover, rest_press, rest_label) = match (s, c) {
        (Surface::Ink, Chrome::Shell) => (
            t.palette.paper,
            t.on_paper.fill_subtle.over(t.palette.paper),
            t.on_paper.fill.over(t.palette.paper),
            t.palette.ink,
        ),
        (Surface::Ink, Chrome::Window) => (
            t.on_ink.fill,
            t.on_ink.fill_strong,
            t.on_ink.track,
            t.on_ink.primary,
        ),
        (Surface::Paper, _) => (
            t.on_paper.fill,
            t.on_paper.fill_strong,
            t.on_paper.track,
            t.on_paper.primary,
        ),
    };
    // `active`'s recipe — terracotta with an ivory label on both surfaces.
    let accent = t.palette.accent;
    let active_hover = t.on_ink.fill_subtle.over(accent);
    let active_press = t.on_ink.fill.over(accent);
    let active_label = t.palette.paper;
    move |_, status| {
        let (background, label) = if emphasized {
            match status {
                Status::Active => (accent, active_label),
                Status::Hovered => (active_hover, active_label),
                Status::Pressed => (active_press, active_label),
                Status::Disabled => (on.fill_subtle, on.disabled),
            }
        } else {
            match status {
                Status::Active => (rest_bg, rest_label),
                Status::Hovered => (rest_hover, rest_label),
                Status::Pressed => (rest_press, rest_label),
                Status::Disabled => (on.fill_subtle, on.disabled),
            }
        };
        pill(
            Some(background.with_opacity(alpha)),
            label.with_opacity(alpha),
            radius,
        )
    }
}

/// A file-picker breadcrumb segment (style guide §7): quiet at rest — no
/// fill, `secondary`-role label — surfacing `fill_subtle`/`fill` on
/// hover/press exactly like [`bare`], but with a quieter resting label than
/// `bare`'s `primary` (a breadcrumb segment reads as a secondary wayfinding
/// element, not primary content). Pair with `paddings.breadcrumb` and a
/// [`crate::style::border_none`]-radius `radii.pill` — the geometry a button
/// style closure can't set — via [`crate::widget::breadcrumb`].
///
/// `is_current` is the crumb for the folder you're already in: it has no
/// `.on_press` (mirroring [`menu_row`]'s "enabled derived from
/// `on_press.is_some()`" convention, applied by
/// [`crate::widget::breadcrumb`]), so iced would otherwise report
/// `Status::Disabled` unconditionally and draw the grayed-out disabled
/// look — wrong for a segment that should read as emphasized, not dead.
/// `is_current` short-circuits `Status` entirely instead, always drawing
/// [`active`]'s terracotta pill with an ivory label (the exact "on"
/// treatment the design language's one rule gives every current/selected
/// state).
///
/// A non-current crumb is content sitting in a path, not a control with a
/// meaningfully different "off" state — [`list_row`]'s reasoning applies
/// here too, so its `Active` and `Disabled` arms both draw the same quiet
/// rest look (a crumb built with no `.on_press` at all, rather than via
/// `is_current`, still reads correctly).
pub fn breadcrumb(
    t: &Theme,
    s: Surface,
    is_current: bool,
) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    let radius = t.radii.pill;
    let on = *t.on(s);
    let accent = t.palette.accent;
    let ivory = t.palette.paper;
    move |_, status| {
        if is_current {
            return pill(Some(accent.into_iced()), ivory.into_iced(), radius);
        }
        match status {
            Status::Hovered => pill(
                Some(on.fill_subtle.into_iced()),
                on.secondary.into_iced(),
                radius,
            ),
            Status::Pressed => pill(Some(on.fill.into_iced()), on.secondary.into_iced(), radius),
            Status::Active | Status::Disabled => pill(None, on.secondary.into_iced(), radius),
        }
    }
}

/// A click target with **no background at any state** — the power/boot
/// menu's icon items (style guide §6: "icons directly on the surface...
/// ivory 55% at rest, full terracotta hovered"). Every other button style in
/// this module paints *some* fill on hover/press ([`bare`]'s
/// `fill_subtle`/`fill`, [`rest`]'s solid pill); this one paints nothing,
/// ever — the icon glyph itself carries the whole state change, via a
/// caller-supplied tint. That tint can't ride this style closure (an
/// `Svg`'s color is fixed at build time — the same iced 0.14 constraint
/// [`crate::widget::icon_button`] documents), so
/// [`crate::widget::bare_icon_item`] computes it from
/// [`crate::widget::role`]/[`crate::widget::Emphasis`] instead.
///
/// Ink-only, no `Surface` parameter: the power/boot menus this styles live
/// on the shell scrim layer (style guide §6), never on a paper window —
/// the same "always one look" shape as [`crate::style::notification`] and
/// `container::popover`/`badge`/`tooltip`/`dialog::surface`.
pub fn bare_icon(t: &Theme) -> impl Fn(&iced::Theme, Status) -> Style + Clone {
    let label = t.on_ink.primary.into_iced();
    move |_, _status| pill(None, label, 0.0)
}
