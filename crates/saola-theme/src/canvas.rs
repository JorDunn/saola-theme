//! Canvas-side token bridge: selection chrome for `canvas::Program` impls.
//!
//! iced's `canvas::Program::draw` receives an `&iced::Theme`, and Saola's
//! tokens don't survive the trip through [`crate::to_iced_theme`] — so every
//! canvas program that wants token-faithful chrome has to pre-extract the
//! values it needs into plain `Copy` fields *before* drawing. That
//! pre-extraction is exactly what [`SelectionChrome`] is: the token values a
//! rectangular-selection surface draws with (scrim, accent edge, resize
//! handles), copied out of a [`Theme`] once, so a `canvas::Program` can hold
//! it by value and stay `'static` (letting `view` return
//! `Element<'static, _>`).
//!
//! The three drawing operations live here because none of them is
//! expressible as a widget style:
//!
//! - [`SelectionChrome::fill_scrim_around`] — a scrim with a hole in it. A
//!   container can't have a hole, so the scrim is painted as four bands
//!   *around* the selection ([`scrim_bands`] is the band math, pure and
//!   unit-tested).
//! - [`SelectionChrome::stroke_dashed_edge`] — a dashed accent edge on the
//!   (rounded) selection rectangle. No iced border style is dashed.
//! - [`SelectionChrome::fill_handles`] — eight round accent handles whose
//!   centers sit *on* the selection edge, half outside the rectangle they
//!   belong to.
//!
//! Dashed terracotta is Saola's "this is the thing selected" language; the
//! scrim is ink-tinted like every Saola scrim, so the dimmed surround reads
//! as shell chrome and the un-dimmed hole reads as live content.

use iced::widget::canvas::{Frame, LineCap, LineDash, LineJoin, Path, Stroke, Style};
use iced::{Point, Rectangle};

use crate::convert::ColorExt;
use crate::Theme;

// The scrim vocabulary is shared with the container helper — one enum for
// both "scrim as a full-bleed container" and "scrim as canvas bands".
pub use crate::style::container::ScrimKind;

/// Resolves a [`ScrimKind`] to the flat color a canvas band is filled with.
///
/// A `canvas::Frame` band is one fill, so the one gradient scrim
/// ([`ScrimKind::LockRest`]) resolves to its *strongest stop* — the nearest
/// flat equivalent. Surfaces that want the real gradient paint it with
/// [`crate::style::container::scrim`] and draw only the hole on canvas.
fn flat_scrim_color(theme: &Theme, kind: ScrimKind) -> iced::Color {
    let scrim = &theme.scrim;
    match kind {
        ScrimKind::Boot => scrim.boot,
        ScrimKind::Shutdown => scrim.shutdown,
        ScrimKind::LockAwake => scrim.lock_awake,
        ScrimKind::LockRest => {
            let strongest = scrim
                .lock_rest
                .stops
                .iter()
                .max_by(|a, b| a.color.a.cmp(&b.color.a))
                .expect("a ScrimGradient always has three stops");
            strongest.color
        }
        ScrimKind::Launcher => scrim.launcher,
        ScrimKind::Overview => scrim.overview,
        ScrimKind::Capture => scrim.capture,
        ScrimKind::Modal => scrim.modal,
        ScrimKind::TranslucentPanel => scrim.translucent_panel,
        ScrimKind::Canvas => scrim.canvas,
    }
    .into_iced()
}

/// The pre-extracted token values a rectangular-selection canvas draws with.
///
/// Every field is a plain `Copy` value rather than a `&Theme` — that is the
/// point of the struct (see the module docs): a `canvas::Program` holds a
/// `SelectionChrome` by value and stays `'static`. Fields are public so a
/// consumer with a non-standard surface can adjust one value instead of
/// abandoning the bridge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionChrome {
    /// The ink-tinted dimming painted around the selection.
    pub scrim: iced::Color,
    /// Terracotta: the dashed edge and the handles.
    pub accent: iced::Color,
    /// Corner radius of the selection edge (`radii.selection`; override
    /// with [`SelectionChrome::with_radius`] — `0.0` gives square corners).
    pub radius: f32,
    /// Stroke width of the dashed edge (`sizes.window_border`).
    pub edge_width: f32,
    /// Dash pattern of the edge (`sizes.selection_dash_fill` px of accent,
    /// `sizes.selection_dash_gap` px of gap).
    pub dash_segments: [f32; 2],
    /// Drawn radius of the eight handles (`sizes.handle_radius`). Hit radii
    /// are deliberately *not* here — how far a press can miss a handle is
    /// input policy, which belongs to the consumer, not the design system.
    pub handle_radius: f32,
}

impl SelectionChrome {
    /// Extracts the selection-chrome values from `theme`, dimming the
    /// surround with the given scrim.
    pub fn new(theme: &Theme, scrim: ScrimKind) -> Self {
        SelectionChrome {
            scrim: flat_scrim_color(theme, scrim),
            accent: theme.palette.accent.into_iced(),
            radius: theme.radii.selection,
            edge_width: theme.sizes.window_border,
            dash_segments: [
                theme.sizes.selection_dash_fill,
                theme.sizes.selection_dash_gap,
            ],
            handle_radius: theme.sizes.handle_radius,
        }
    }

    /// Replaces the edge's corner radius (e.g. `0.0` for a square-cornered
    /// highlight around content that is itself square).
    #[must_use]
    pub fn with_radius(self, radius: f32) -> Self {
        SelectionChrome { radius, ..self }
    }

    /// Dims all of `outer` — the "nothing selected yet" state, before the
    /// first drag opens a hole.
    pub fn fill_scrim(&self, frame: &mut Frame, outer: Rectangle) {
        fill_band(frame, outer, self.scrim);
    }

    /// Dims `outer` everywhere *except* `hole`: four scrim bands around the
    /// selection, so the content behind the hole shows through at full
    /// strength. A hole flush against an edge of `outer` produces a
    /// zero-area band (skipped); a hole outside `outer` dims everything.
    pub fn fill_scrim_around(&self, frame: &mut Frame, outer: Rectangle, hole: Rectangle) {
        for band in scrim_bands(outer, hole) {
            fill_band(frame, band, self.scrim);
        }
    }

    /// Strokes the dashed accent edge on `rect`, rounded by `self.radius`.
    pub fn stroke_dashed_edge(&self, frame: &mut Frame, rect: Rectangle) {
        let outline = Path::rounded_rectangle(rect.position(), rect.size(), self.radius.into());
        frame.stroke(
            &outline,
            Stroke {
                style: Style::Solid(self.accent),
                width: self.edge_width,
                line_cap: LineCap::Butt,
                line_join: LineJoin::Round,
                line_dash: LineDash {
                    segments: &self.dash_segments,
                    offset: 0,
                },
            },
        );
    }

    /// Fills the eight round accent handles: one per corner, one per edge
    /// midpoint, each centered on the edge so it sits half outside `rect`.
    pub fn fill_handles(&self, frame: &mut Frame, rect: Rectangle) {
        for center in handle_centers(rect) {
            frame.fill(&Path::circle(center, self.handle_radius), self.accent);
        }
    }
}

/// Fills `band` if it has positive area — a zero-area band is the normal
/// result of a hole flush against an edge, and painting a negative-size
/// rectangle would be a silent painting bug.
fn fill_band(frame: &mut Frame, band: Rectangle, color: iced::Color) {
    if band.width > 0.0 && band.height > 0.0 {
        frame.fill_rectangle(band.position(), band.size(), color);
    }
}

/// The four rectangles that tile `outer` minus `hole`: full-width top and
/// bottom bands, and left/right bands spanning only the hole's own rows.
///
/// Pure geometry, factored out of the painting so it can be unit-tested
/// without a renderer. The hole is clamped to `outer` first, so the four
/// returned bands always tile `outer − hole` exactly — no overlap, no gap.
/// Degenerate cases fall out of the clamp: a hole that doesn't overlap
/// `outer` (or has no area) yields one band covering all of `outer` plus
/// three zero-area bands. Callers must skip zero-area bands when painting
/// (as [`SelectionChrome::fill_scrim_around`] does).
pub fn scrim_bands(outer: Rectangle, hole: Rectangle) -> [Rectangle; 4] {
    let empty = |x: f32, y: f32| Rectangle::new(Point::new(x, y), iced::Size::ZERO);

    let Some(hole) = outer.intersection(&hole) else {
        // No overlap: the whole surface dims.
        return [
            outer,
            empty(outer.x, outer.y),
            empty(outer.x, outer.y),
            empty(outer.x, outer.y),
        ];
    };

    let outer_right = outer.x + outer.width;
    let outer_bottom = outer.y + outer.height;
    let hole_right = hole.x + hole.width;
    let hole_bottom = hole.y + hole.height;

    let top = Rectangle::new(
        Point::new(outer.x, outer.y),
        iced::Size::new(outer.width, hole.y - outer.y),
    );
    let bottom = Rectangle::new(
        Point::new(outer.x, hole_bottom),
        iced::Size::new(outer.width, outer_bottom - hole_bottom),
    );
    let left = Rectangle::new(
        Point::new(outer.x, hole.y),
        iced::Size::new(hole.x - outer.x, hole.height),
    );
    let right = Rectangle::new(
        Point::new(hole_right, hole.y),
        iced::Size::new(outer_right - hole_right, hole.height),
    );

    [top, bottom, left, right]
}

/// The centers of the eight resize handles of `rect`: the four corners
/// first, then the four edge midpoints (matching the corners-first priority
/// a consumer's hit-testing wants — on a small selection a corner press
/// should win over the neighbouring midpoints).
pub fn handle_centers(rect: Rectangle) -> [Point; 8] {
    let left = rect.x;
    let top = rect.y;
    let right = rect.x + rect.width;
    let bottom = rect.y + rect.height;
    let mid_x = rect.x + rect.width / 2.0;
    let mid_y = rect.y + rect.height / 2.0;

    [
        Point::new(left, top),
        Point::new(right, top),
        Point::new(right, bottom),
        Point::new(left, bottom),
        Point::new(mid_x, top),
        Point::new(right, mid_y),
        Point::new(mid_x, bottom),
        Point::new(left, mid_y),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Size;

    fn rect(x: f32, y: f32, width: f32, height: f32) -> Rectangle {
        Rectangle::new(Point::new(x, y), Size::new(width, height))
    }

    fn area(r: &Rectangle) -> f32 {
        r.width.max(0.0) * r.height.max(0.0)
    }

    /// The tiling invariants every `scrim_bands` result must satisfy:
    /// bands stay inside `outer`, never overlap each other, never overlap
    /// the (clamped) hole, and their areas sum to `outer − hole` exactly.
    fn assert_tiles(outer: Rectangle, hole: Rectangle) {
        let bands = scrim_bands(outer, hole);
        let clamped_hole_area = outer.intersection(&hole).as_ref().map_or(0.0, area);

        for band in &bands {
            assert!(
                band.width >= 0.0 && band.height >= 0.0,
                "negative-size band {band:?} for outer {outer:?}, hole {hole:?}"
            );
            if area(band) > 0.0 {
                assert!(
                    band.is_within(&outer),
                    "band {band:?} leaks outside outer {outer:?}"
                );
                assert!(
                    band.intersection(&hole).is_none(),
                    "band {band:?} overlaps hole {hole:?}"
                );
            }
        }
        for (i, a) in bands.iter().enumerate() {
            for b in &bands[i + 1..] {
                assert!(a.intersection(b).is_none(), "bands {a:?} and {b:?} overlap");
            }
        }

        let banded: f32 = bands.iter().map(area).sum();
        let uncovered = area(&outer) - clamped_hole_area;
        assert!(
            (banded - uncovered).abs() < f32::EPSILON * area(&outer).max(1.0),
            "bands cover {banded}, expected {uncovered} (outer {outer:?}, hole {hole:?})"
        );
    }

    #[test]
    fn bands_tile_interior_hole() {
        assert_tiles(
            rect(0.0, 0.0, 800.0, 600.0),
            rect(100.0, 50.0, 300.0, 200.0),
        );
    }

    #[test]
    fn bands_tile_with_offset_outer_origin() {
        // The editor draws in view-space, where `outer` doesn't start at 0.
        assert_tiles(
            rect(40.0, 30.0, 500.0, 400.0),
            rect(90.0, 60.0, 100.0, 80.0),
        );
    }

    #[test]
    fn hole_flush_against_edges_zeroes_those_bands() {
        let outer = rect(0.0, 0.0, 800.0, 600.0);

        // Flush top-left: top and left bands have zero area.
        let bands = scrim_bands(outer, rect(0.0, 0.0, 200.0, 100.0));
        assert_eq!(area(&bands[0]), 0.0, "top band should be empty");
        assert_eq!(area(&bands[2]), 0.0, "left band should be empty");
        assert!(area(&bands[1]) > 0.0 && area(&bands[3]) > 0.0);
        assert_tiles(outer, rect(0.0, 0.0, 200.0, 100.0));

        // Flush bottom-right: bottom and right bands have zero area.
        let bands = scrim_bands(outer, rect(600.0, 500.0, 200.0, 100.0));
        assert_eq!(area(&bands[1]), 0.0, "bottom band should be empty");
        assert_eq!(area(&bands[3]), 0.0, "right band should be empty");
        assert_tiles(outer, rect(600.0, 500.0, 200.0, 100.0));
    }

    #[test]
    fn hole_covering_outer_leaves_no_bands() {
        let outer = rect(0.0, 0.0, 800.0, 600.0);
        for hole in [outer, rect(-10.0, -10.0, 900.0, 700.0)] {
            let bands = scrim_bands(outer, hole);
            assert!(
                bands.iter().all(|band| area(band) == 0.0),
                "hole covering outer must leave no visible band, got {bands:?}"
            );
            assert_tiles(outer, hole);
        }
    }

    #[test]
    fn hole_outside_outer_dims_everything() {
        let outer = rect(0.0, 0.0, 800.0, 600.0);
        for hole in [
            rect(900.0, 0.0, 100.0, 100.0), // fully outside
            rect(0.0, 0.0, 0.0, 0.0),       // zero-size
            rect(100.0, 100.0, 0.0, 50.0),  // zero-width
            rect(100.0, 100.0, 50.0, 0.0),  // zero-height
            rect(-200.0, -200.0, 100.0, 100.0),
        ] {
            let bands = scrim_bands(outer, hole);
            let banded: f32 = bands.iter().map(area).sum();
            assert_eq!(
                banded,
                area(&outer),
                "degenerate hole {hole:?} must dim all of outer"
            );
            assert_tiles(outer, hole);
        }
    }

    #[test]
    fn hole_partially_outside_is_clamped() {
        let outer = rect(0.0, 0.0, 800.0, 600.0);
        for hole in [
            rect(-50.0, 100.0, 200.0, 200.0), // sticks out left
            rect(700.0, 100.0, 200.0, 200.0), // sticks out right
            rect(100.0, -50.0, 200.0, 200.0), // sticks out top
            rect(100.0, 500.0, 200.0, 200.0), // sticks out bottom
            rect(-50.0, -50.0, 200.0, 200.0), // sticks out both, corner
        ] {
            assert_tiles(outer, hole);
        }
    }

    #[test]
    fn handle_centers_are_corners_then_midpoints() {
        let centers = handle_centers(rect(10.0, 20.0, 100.0, 60.0));
        assert_eq!(
            centers,
            [
                Point::new(10.0, 20.0),  // top-left
                Point::new(110.0, 20.0), // top-right
                Point::new(110.0, 80.0), // bottom-right
                Point::new(10.0, 80.0),  // bottom-left
                Point::new(60.0, 20.0),  // top mid
                Point::new(110.0, 50.0), // right mid
                Point::new(60.0, 80.0),  // bottom mid
                Point::new(10.0, 50.0),  // left mid
            ]
        );
    }

    #[test]
    fn selection_chrome_reads_the_selected_scrim() {
        let theme = Theme::saola();
        let capture = SelectionChrome::new(&theme, ScrimKind::Capture);
        let modal = SelectionChrome::new(&theme, ScrimKind::Modal);
        let boot = SelectionChrome::new(&theme, ScrimKind::Boot);

        assert_eq!(capture.scrim, theme.scrim.capture.into_iced());
        assert_eq!(modal.scrim, theme.scrim.modal.into_iced());
        assert_eq!(boot.scrim, theme.scrim.boot.into_iced());
        assert_eq!(capture.accent, theme.palette.accent.into_iced());
        assert_eq!(capture.radius, theme.radii.selection);
        assert_eq!(capture.edge_width, theme.sizes.window_border);
        assert_eq!(capture.with_radius(0.0).radius, 0.0);
    }
}
