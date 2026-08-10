//! The indeterminate progress rule — style guide §7's boot-menu countdown
//! ("the progress rule as the countdown") and the splash's "one
//! indeterminate rule". iced's `progress_bar` widget can only paint a
//! *determinate* fill anchored to the start edge
//! (`iced_widget::progress_bar::Style` is a background/bar color pair with
//! no way to detach the bar from `x = 0`), so a segment that sweeps along a
//! track, connected to neither edge, has to be hand-drawn.
//!
//! # `canvas::Program`, not a custom `Widget`
//!
//! [`crate::marquee::Marquee`] is this crate's other custom-drawn widget,
//! and it earns a full `Widget` impl because its content (shaped text) can
//! only be measured through a renderer's own paragraph cache. This rule has
//! no such need — it is two rounded rectangles, exactly the geometry
//! [`crate::canvas`] (the selection-chrome token bridge) already draws with
//! a `Path::rounded_rectangle` + `Frame::fill` pair on every frame — so
//! [`IndeterminateRule`] is a `canvas::Program`, not a `Widget`: less code,
//! and the crate already carries the `"canvas"` iced feature for
//! `crate::canvas` and the gallery's own selection-chrome demo
//! (`SelectionDemo`, in `examples/gallery/main.rs`) to depend on.
//!
//! # The sweep: ping-pong, no dwell
//!
//! The plan left two shapes open: a segment that **wraps** (reaches the far
//! edge, vanishes, reappears at the near edge) or one that **ping-pongs**
//! (reaches the far edge, reverses). This picks ping-pong, to match style
//! guide §5's marquee feel ([`crate::marquee`]) — the only other sweeping
//! motion in Saola — rather than introduce a second sweep grammar the
//! design language doesn't otherwise have.
//!
//! Unlike the marquee, there is **no dwell** at either end: the marquee's
//! dwell exists to give a reader time to *start* a line of text before it
//! moves; a content-free loading rule has no line to start, and reads as
//! "still working" most convincingly in continuous motion. The reversal
//! itself is a triangle wave built directly from [`crate::motion::fraction`]
//! — see [`ping_pong`], the whole animation in one pure function, unit
//! tested below without a renderer.
//!
//! # Tokens
//!
//! - `sizes.progress_girth` — thickness. [`crate::widget::progress_rule`] is
//!   the determinate sibling wired to the same token, so a consumer can
//!   swap between the two rules without either one picking a different
//!   height.
//! - `radii.pill` — both the track's and the segment's corner radius, the
//!   same pairing [`crate::style::progress::bar`] uses for the determinate
//!   bar.
//! - `motion.marquee_speed` — the segment's travel rate in px/s, **reused**
//!   rather than minting a dedicated indeterminate-speed token: no concepts
//!   mockup captions a distinct rate for this rule (unlike the marquee,
//!   which the concepts HTML does caption), and 24 px/s is already Saola's
//!   one "how fast does something glide" constant. Flag for a future
//!   concepts-measurement pass if a dedicated rate is ever specified.
//! - the segment's own length, [`SEGMENT_FRACTION`] (30% of the track), is
//!   a plain constant, not a token — nothing sources a ratio for it either;
//!   picked to read clearly as "a moving piece" rather than a near-full or
//!   near-empty bar.
//!
//! # Driving `elapsed`
//!
//! Same contract as [`crate::marquee`]: [`indeterminate_rule`] is a pure
//! function of "how long has this run been going," and the caller owns the
//! timer — a gated `iced::time::every` subscription while the rule is
//! actually on screen, an epoch established on the run's first tick, and
//! `now.saturating_duration_since(epoch)` recomputed every tick (never
//! accumulated, so one late tick costs one frame instead of banking drift
//! forever). There is no "gate" analogous to the marquee's overflow check —
//! an indeterminate rule is either shown or it isn't; whatever decides that
//! is the caller's concern, not this module's.

use std::time::Duration;

use iced::widget::canvas;
use iced::widget::Canvas;
use iced::{mouse, Fill, Point, Rectangle, Size};
use saola_tokens::{Surface, Theme};

use crate::convert::ColorExt;
use crate::motion;

/// The moving segment's length as a fraction of the track's own length. Not
/// a token — see the module docs.
const SEGMENT_FRACTION: f32 = 0.3;

/// Builds the indeterminate rule: a `sizes.progress_girth`-thick canvas
/// that fills the available width, sweeping a terracotta segment
/// ([`SEGMENT_FRACTION`] of the track) back and forth along a
/// `track`-role background. See the module docs for the sweep and token
/// choices, and for the `elapsed` contract.
///
/// ```no_run
/// use saola_theme::{indeterminate::indeterminate_rule, Surface, Theme};
/// use std::time::Duration;
///
/// let t = Theme::saola();
/// let _rule: iced::Element<'_, ()> =
///     indeterminate_rule(&t, Surface::Paper, Duration::from_millis(400)).into();
/// ```
pub fn indeterminate_rule<Message>(
    t: &Theme,
    s: Surface,
    elapsed: Duration,
) -> Canvas<IndeterminateRule, Message> {
    canvas(program(t, s, elapsed))
        .width(Fill)
        .height(t.sizes.progress_girth)
}

/// Pre-extracts the token values [`IndeterminateRule::draw`] paints with —
/// split out from [`indeterminate_rule`] so the tests below can inspect the
/// struct directly without reaching into a `Canvas`, which keeps its
/// `program` field private.
fn program(t: &Theme, s: Surface, elapsed: Duration) -> IndeterminateRule {
    IndeterminateRule {
        track: t.on(s).track.into_iced(),
        accent: t.palette.accent.into_iced(),
        radius: t.radii.pill,
        speed: t.motion.marquee_speed,
        elapsed,
    }
}

/// The indeterminate rule's `canvas::Program`. Holds only pre-extracted,
/// `Copy` token values — same "extract once, stay `'static`" shape as
/// [`crate::canvas::SelectionChrome`] — plus `elapsed`, the one value that
/// changes frame to frame. Construct it through [`indeterminate_rule`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IndeterminateRule {
    /// The track's fill (the surface's `track` role).
    track: iced::Color,
    /// The moving segment's fill (`palette.accent`).
    accent: iced::Color,
    /// Corner radius of both the track and the segment (`radii.pill`,
    /// auto-clamped by `Path::rounded_rectangle` to half the girth, so it
    /// always reads as fully rounded regardless of thickness).
    radius: f32,
    /// The segment's travel rate in px/s (`motion.marquee_speed`).
    speed: f32,
    /// How long the current run has been going.
    elapsed: Duration,
}

impl<Message> canvas::Program<Message> for IndeterminateRule {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        // Frame coordinates are local to the frame's own origin, not
        // window space — same note `SelectionDemo` in the gallery makes.
        let size = frame.size();

        let track = canvas::Path::rounded_rectangle(Point::ORIGIN, size, self.radius.into());
        frame.fill(&track, self.track);

        let segment_width = (size.width * SEGMENT_FRACTION).min(size.width);
        let travel = (size.width - segment_width).max(0.0);
        let x = ping_pong(self.elapsed, travel, self.speed) * travel;

        let segment = canvas::Path::rounded_rectangle(
            Point::new(x, 0.0),
            Size::new(segment_width, size.height),
            self.radius.into(),
        );
        frame.fill(&segment, self.accent);

        vec![frame.into_geometry()]
    }
}

/// The ping-pong position of the sweep at `elapsed`: `0.0` at the near end
/// of `travel_px`, `1.0` at the far end, linear in between, reversing at
/// each end — a triangle wave built from two mirrored calls to
/// [`crate::motion::fraction`] (one per half of the cycle), rather than a
/// dwelling state machine like [`crate::marquee`]'s `phase_at` (see the
/// module docs for why the shapes differ).
///
/// Degenerate inputs (no travel, a non-finite or non-positive speed) return
/// `0.0` — parked at the near end — rather than dividing by zero or
/// propagating a NaN, the same "nothing to sweep" answer
/// [`crate::marquee`]'s `phase_at` gives for its own degenerate cases.
fn ping_pong(elapsed: Duration, travel_px: f32, speed: f32) -> f32 {
    if !travel_px.is_finite() || travel_px <= 0.0 || !speed.is_finite() || speed <= 0.0 {
        return 0.0;
    }

    // One-way sweep time, in whole milliseconds — `motion::fraction` takes
    // its duration as `u32` ms, so the rate-to-duration conversion happens
    // once here rather than inside the hot loop.
    let half_cycle_secs = f64::from(travel_px) / f64::from(speed);
    let half_cycle_ms = ((half_cycle_secs * 1000.0).round() as u64).clamp(1, u64::from(u32::MAX));
    let half_cycle_ms = half_cycle_ms as u32;
    let half_cycle = Duration::from_millis(u64::from(half_cycle_ms));
    let cycle_ms = u128::from(half_cycle_ms) * 2;

    let wrapped_ms = elapsed.as_millis() % cycle_ms;
    let wrapped = Duration::from_millis(wrapped_ms as u64);

    if wrapped < half_cycle {
        motion::fraction(wrapped, half_cycle_ms)
    } else {
        1.0 - motion::fraction(wrapped - half_cycle, half_cycle_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRAVEL: f32 = 100.0;
    const SPEED: f32 = 24.0;

    /// One-way sweep time for [`TRAVEL`] at [`SPEED`] — every timeline in
    /// this test module is stated in terms of it.
    fn half_cycle_ms() -> u64 {
        ((f64::from(TRAVEL) / f64::from(SPEED)) * 1000.0).round() as u64
    }

    #[test]
    fn parks_at_the_near_edge_at_the_start() {
        assert_eq!(ping_pong(Duration::ZERO, TRAVEL, SPEED), 0.0);
    }

    #[test]
    fn reaches_the_far_edge_at_one_half_cycle() {
        let value = ping_pong(Duration::from_millis(half_cycle_ms()), TRAVEL, SPEED);
        assert!((value - 1.0).abs() < 1e-3, "{value}");
    }

    #[test]
    fn returns_to_the_near_edge_at_a_full_cycle() {
        let value = ping_pong(Duration::from_millis(half_cycle_ms() * 2), TRAVEL, SPEED);
        assert!(value.abs() < 1e-3, "{value}");
    }

    #[test]
    fn the_sweep_is_linear_within_each_leg() {
        let a = ping_pong(Duration::from_millis(half_cycle_ms() / 4), TRAVEL, SPEED);
        let b = ping_pong(Duration::from_millis(half_cycle_ms() / 2), TRAVEL, SPEED);
        let c = ping_pong(
            Duration::from_millis(half_cycle_ms() * 3 / 4),
            TRAVEL,
            SPEED,
        );
        assert!((a - 0.25).abs() < 1e-2, "{a}");
        assert!((b - 0.5).abs() < 1e-2, "{b}");
        assert!((c - 0.75).abs() < 1e-2, "{c}");
    }

    #[test]
    fn the_reversal_at_the_far_edge_is_symmetric() {
        let half = half_cycle_ms();
        let before = ping_pong(Duration::from_millis(half - 5), TRAVEL, SPEED);
        let after = ping_pong(Duration::from_millis(half + 5), TRAVEL, SPEED);
        assert!((before - after).abs() < 0.02, "{before} vs {after}");
    }

    #[test]
    fn the_cycle_repeats_indefinitely() {
        let half = half_cycle_ms();
        let sample_at = half / 3;
        let one = ping_pong(Duration::from_millis(sample_at), TRAVEL, SPEED);
        // Many cycles later, at the same phase within a cycle.
        let later = ping_pong(
            Duration::from_millis(sample_at + half * 2 * 5000),
            TRAVEL,
            SPEED,
        );
        assert!((one - later).abs() < 1e-3, "{one} vs {later}");
    }

    #[test]
    fn degenerate_input_parks_rather_than_dividing_by_it() {
        for (travel, speed) in [
            (0.0, SPEED),
            (-10.0, SPEED),
            (f32::NAN, SPEED),
            (TRAVEL, 0.0),
            (TRAVEL, -1.0),
            (TRAVEL, f32::NAN),
        ] {
            assert_eq!(
                ping_pong(Duration::from_secs(3), travel, speed),
                0.0,
                "travel={travel} speed={speed}"
            );
        }
    }

    #[test]
    fn the_program_reads_tokens_from_the_theme() {
        let t = Theme::saola();
        for s in [Surface::Ink, Surface::Paper] {
            let rule = program(&t, s, Duration::from_millis(500));
            assert_eq!(rule.track, t.on(s).track.into_iced());
            assert_eq!(rule.accent, t.palette.accent.into_iced());
            assert_eq!(rule.radius, t.radii.pill);
            assert_eq!(rule.speed, t.motion.marquee_speed);
            assert_eq!(rule.elapsed, Duration::from_millis(500));
        }
    }

    /// A smoke test that the generics line up: [`indeterminate_rule`]
    /// builds a real `Element` without a renderer.
    #[test]
    fn constructor_builds() {
        let t = Theme::saola();
        let _: iced::Element<'_, ()> =
            indeterminate_rule(&t, Surface::Ink, Duration::from_millis(0)).into();
    }
}
