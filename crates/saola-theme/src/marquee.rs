//! The ping-pong text marquee — style guide §5's opt-in overflow sweep.
//!
//! Ported from saola-panel's `modules/window_title.rs` (its `Marquee` widget
//! plus the pure `phase_at` / `offset_at` / `window_width` helpers), which
//! kept it downstream only because the marquee tokens did not exist in the
//! pinned saola-theme release. They do now — `motion.marquee_dwell` (2000 ms)
//! and `motion.marquee_speed` (24.0 px/s) — so [`marquee`] reads them from
//! [`Theme`] instead of the panel's local constants.
//!
//! §5's motion: dwell at the head, sweep left at the token rate until the
//! tail is fully visible, dwell, sweep back, repeat. Translation only — no
//! fade, no scale, no colour change, and the text never wraps.
//!
//! # Pure mechanism, no styling
//!
//! The widget draws *only* text, in a colour and size the consumer already
//! resolved from tokens (the panel passes `typography.size.bar` and
//! `on_ink.secondary` for the focused-window title; another module may
//! legitimately pass another role) — it introduces no styling of its own.
//!
//! # The animation is split in two
//!
//! - [`phase_at`] / [`offset_at`] are pure functions of elapsed time, the
//!   overflow distance in pixels, and the two motion tokens — the whole
//!   `DwellHead → SweepLeft → DwellTail → SweepRight → repeat` state
//!   machine, unit-tested below without a compositor, a renderer, or a
//!   running iced app.
//! - [`Marquee`] measures the text with the renderer's own text stack
//!   (`Plain<Paragraph>`, the same cache the stock `text` widget keeps),
//!   draws it translated by [`offset_at`]'s pixels, and clips to its own
//!   bounds. That is what makes the sweep *pixel*-true rather than a
//!   character-at-a-time slide: the speed token is a pixel rate, and the
//!   only place pixels for a given string and font exist is the renderer.
//!
//! # What stays with the consumer
//!
//! **Driving `elapsed`.** The widget is a pure function of "how long has
//! this run been going"; the consumer owns the timer and the epoch. The
//! panel's shape (see `window_title.rs`): a gated
//! `iced::time::every(MARQUEE_TICK)` subscription that exists only while an
//! overflowing text is actually on screen, a stored epoch established by the
//! run's first tick (`get_or_insert`), and
//! `now.saturating_duration_since(epoch)` as the elapsed value — recomputed
//! from the epoch, never accumulated, so a late tick costs one frame rather
//! than banking error forever.
//!
//! **`MARQUEE_TICK` itself.** The redraw interval is a *frame budget, not a
//! design token* — the panel documents the distinction at length (33 ms
//! keeps each step of a 24 px/s sweep under the ~1 px threshold where
//! quantized travel reads as stepping) — so it stays a consumer constant and
//! is deliberately not minted here.
//!
//! **The overflow gate.** Whether to marquee at all (vs. truncate), and the
//! character budget that defines "overflows", are consumer knobs
//! (`max-chars` in the panel's config); this module just takes the budget as
//! [`marquee`]'s `max_chars`.

use std::time::Duration;

use iced::advanced::text as advanced_text;
use iced::advanced::text::paragraph::Plain;
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{layout, mouse, renderer, Layout, Widget};
use iced::alignment;
use iced::widget::text::Wrapping;
use iced::{Color, Element, Length, Pixels, Point, Rectangle, Size};
use saola_tokens::Theme;

/// A marquee for `content`, `elapsed` into its current run.
///
/// Matches the panel's call site: the consumer resolves the text's size and
/// colour from tokens itself (`typography.size.bar` / `on_ink.secondary` for
/// the window title), owns the `max_chars` budget, and drives `elapsed` from
/// its own gated tick subscription (see the module docs). The two §5 motion
/// values — dwell and sweep rate — come from `t.motion.marquee_dwell` /
/// `t.motion.marquee_speed`, which is the one reason this constructor takes
/// a [`Theme`] at all.
///
/// Turn the result into an [`Element`] with `.into()`.
pub fn marquee<'a>(
    t: &Theme,
    content: &'a str,
    elapsed: Duration,
    max_chars: usize,
    size: f32,
    color: Color,
) -> Marquee<'a> {
    Marquee {
        content,
        max_chars,
        size,
        color,
        elapsed,
        dwell: Duration::from_millis(u64::from(t.motion.marquee_dwell)),
        speed: t.motion.marquee_speed,
    }
}

/// Where the ping-pong is, as of `elapsed` into the run — the whole state
/// machine of style guide §5, and a pure function of its arguments.
///
/// `overflow_px` is how far the text has to travel: the measured width of the
/// full text minus the width of the window it is being shown through. Zero
/// (or negative — text that fits) means there is nothing to sweep, and the
/// machine parks at [`Phase::DwellHead`] forever, which is what makes "the
/// loop stops the moment the text fits" fall out of the arithmetic rather
/// than needing a special case at the call site.
///
/// The cycle is `dwell + sweep + dwell + sweep`, where `sweep =
/// overflow_px / speed`, so longer text takes proportionally longer to cross
/// — the speed token is a *rate*, not a duration.
///
/// Teaching note (why `f64` inside, ported from the panel): the modulo below
/// is what keeps text that has been sweeping for an hour in the same step as
/// text that started a second ago. `f32` has ~7 significant digits, so at an
/// hour's elapsed seconds it can no longer resolve a frame; `f64` has ~16
/// and stays exact for far longer than any window keeps focus.
fn phase_at(elapsed: Duration, overflow_px: f32, dwell: Duration, speed: f32) -> Phase {
    // The `is_finite` guards are for degenerate inputs (a NaN width, a zero
    // or NaN rate): NaN fails every comparison, so without them the machine
    // would fall through to the sweep arms and propagate NaN into an offset;
    // a non-positive rate has no finite sweep at all. Parking is the right
    // answer for "no travel to do" however that arises.
    if !overflow_px.is_finite() || overflow_px <= 0.0 || !speed.is_finite() || speed <= 0.0 {
        return Phase::DwellHead;
    }

    let sweep = f64::from(overflow_px) / f64::from(speed);
    let dwell = dwell.as_secs_f64();
    let cycle = 2.0 * (dwell + sweep);
    let at = elapsed.as_secs_f64() % cycle;

    if at < dwell {
        Phase::DwellHead
    } else if at < dwell + sweep {
        Phase::SweepLeft {
            progress: ((at - dwell) / sweep) as f32,
        }
    } else if at < dwell + sweep + dwell {
        Phase::DwellTail
    } else {
        Phase::SweepRight {
            progress: ((at - (dwell + sweep + dwell)) / sweep) as f32,
        }
    }
}

/// How far left the text is drawn, in logical pixels, at `elapsed` into the
/// run — `0.0` at the head, `overflow_px` at the tail, linear in between.
///
/// This is the only number [`Marquee`]'s `draw` takes from the animation,
/// and the only thing the state machine is *for*: the sweep is a translation
/// and nothing else (style guide §5 — no fade, no scale, no colour change).
fn offset_at(elapsed: Duration, overflow_px: f32, dwell: Duration, speed: f32) -> f32 {
    match phase_at(elapsed, overflow_px, dwell, speed) {
        Phase::DwellHead => 0.0,
        Phase::SweepLeft { progress } => overflow_px * progress,
        Phase::DwellTail => overflow_px,
        Phase::SweepRight { progress } => overflow_px * (1.0 - progress),
    }
}

/// One step of the §5 ping-pong. `progress` runs `0.0..1.0` across a sweep
/// (never reaching 1.0 — that instant is the next phase's start), which is
/// what lets [`offset_at`] be a plain interpolation.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Phase {
    /// Parked at the head of the text, one dwell long.
    DwellHead,
    /// Travelling left, revealing the tail.
    SweepLeft { progress: f32 },
    /// Parked with the tail fully visible, one dwell long.
    DwellTail,
    /// Travelling back right, returning to the head.
    SweepRight { progress: f32 },
}

/// The width of the window the text is shown through: `max_chars`
/// characters, where "a character" is the measured average advance of *this
/// text in this font* (`full_width / chars`).
///
/// Teaching note (why an average, ported from the panel): a character budget
/// is a count, the sweep rate is pixels, and something has to translate
/// between them. Under a proportional face there is no single pixel width
/// for "50 characters" — `WWWWW` and `iiiii` differ by a factor of three —
/// so any translation is an approximation. Taking the average from the text
/// actually on screen keeps the approximation *self-consistent*: the window
/// is exactly `max_chars/chars` of the full width, so it is narrower than
/// the text if and only if the text is longer than the budget — precisely
/// the character-count gate consumers apply before constructing a marquee at
/// all. The alternative (measuring a reference glyph once) would let the two
/// disagree, and a timer ticking against text that turned out to fit is
/// exactly the standing-timer failure the gate exists to prevent.
///
/// Note this is the width of the *window*, not of any truncation: nothing is
/// cut in marquee mode, the full text is drawn and clipped.
fn window_width(full_width: f32, chars: usize, max_chars: usize) -> f32 {
    if chars == 0 {
        return 0.0;
    }
    let average_advance = full_width / chars as f32;
    (average_advance * max_chars as f32).min(full_width)
}

/// The marquee's rendering half: a fixed-width window onto text that is laid
/// out in full and drawn translated. Construct it with [`marquee`].
///
/// Teaching note (why a custom widget rather than composing stock ones):
/// §5's sweep is specified in pixels per second, so the animation needs the
/// text's width in pixels — and text has a width only once a renderer has
/// shaped it with a real font. iced exposes that measurement to widgets and
/// nowhere else: [`Plain`] is the same paragraph cache the stock `text`
/// widget keeps in its own tree state, `min_bounds()` is the measurement,
/// and `fill_paragraph`'s `clip_bounds` is the clipping. Everything this
/// widget adds on top is two lines — a narrower layout node, and an `x`
/// shifted by [`offset_at`].
pub struct Marquee<'a> {
    /// The whole text, uncut — the cap becomes the window's width, not a
    /// slice of the string.
    content: &'a str,
    /// The window's width, in characters (see [`window_width`]).
    max_chars: usize,
    /// The text size, passed in rather than read here: this widget resolves
    /// no tokens of its own (the panel passes `typography.size.bar`).
    size: f32,
    /// The text colour, already converted — same reasoning as `size` (the
    /// panel passes `on_ink.secondary`).
    color: Color,
    /// How long the current run has been going; turned into pixels by
    /// [`offset_at`] in `draw`, once the travel distance is known.
    elapsed: Duration,
    /// `motion.marquee_dwell`, resolved by [`marquee`].
    dwell: Duration,
    /// `motion.marquee_speed`, resolved by [`marquee`].
    speed: f32,
}

/// The widget's tree state: the renderer's laid-out copy of the text.
///
/// Cached in the tree (rather than re-shaped every frame) for the same
/// reason the stock `text` widget caches it — shaping is the expensive part,
/// and [`Plain::update`] re-shapes only when the content or format actually
/// changed. At a ~30 Hz sweep that difference is the whole cost of the
/// animation.
struct MarqueeState<P: advanced_text::Paragraph> {
    paragraph: Plain<P>,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Marquee<'_>
where
    Renderer: advanced_text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<MarqueeState<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(MarqueeState::<Renderer::Paragraph> {
            paragraph: Plain::default(),
        })
    }

    /// Shrink in both axes: the window is as wide as `max_chars` asks for and
    /// no wider, so the widget occupies the same kind of space a truncated
    /// text would and the row closes up around it.
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    /// Measures the text at its natural width, then hands back a node only
    /// as wide as the window.
    ///
    /// `Size::INFINITE` bounds with `Wrapping::None` is what "measure this
    /// string as one line, however long that is" looks like — the resulting
    /// `min_bounds().width` is the full pixel width the sweep travels across.
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree
            .state
            .downcast_mut::<MarqueeState<Renderer::Paragraph>>();

        let _ = state.paragraph.update(advanced_text::Text {
            content: self.content,
            bounds: Size::INFINITE,
            size: Pixels(self.size),
            line_height: advanced_text::LineHeight::default(),
            font: renderer.default_font(),
            align_x: advanced_text::Alignment::Default,
            align_y: alignment::Vertical::Top,
            // Advanced shaping, not `Auto`: overflowing text is exactly
            // where non-ASCII shows up (an em dash in an editor's title, a
            // CJK filename) — the panel treats that as the normal case
            // rather than the exotic one.
            shaping: advanced_text::Shaping::Advanced,
            wrapping: Wrapping::None,
        });

        let full = state.paragraph.min_bounds();
        let window = window_width(full.width, self.content.chars().count(), self.max_chars)
            // Never wider than the space the layout actually offered — a
            // narrow screen clips further, it does not push the row over.
            .min(limits.max().width);

        layout::Node::new(limits.resolve(
            Length::Shrink,
            Length::Shrink,
            Size::new(window, full.height),
        ))
    }

    /// Draws the whole text, shifted left by the animation's offset and
    /// clipped to the window.
    ///
    /// `clip_bounds` is the mechanism the renderer already uses for scrolled
    /// and overflowing text, so the tail (and, mid-sweep, the head) is simply
    /// not rasterized outside the window — no fade at the edges, which §5
    /// forbids anyway.
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree
            .state
            .downcast_ref::<MarqueeState<Renderer::Paragraph>>();
        let bounds = layout.bounds();

        // Entirely scrolled or clipped out of view: nothing to draw, and
        // `fill_paragraph` would take an empty clip anyway.
        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };

        // The travel distance, recomputed from the measurement rather than
        // stored: the window and the text are both laid out above, so this
        // is always the *current* overflow, including the frame in which the
        // text changed length.
        let overflow = (state.paragraph.min_bounds().width - bounds.width).max(0.0);
        let offset = offset_at(self.elapsed, overflow, self.dwell, self.speed);

        renderer.fill_paragraph(
            state.paragraph.raw(),
            Point::new(bounds.x - offset, bounds.y),
            self.color,
            clip,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<Marquee<'a>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: advanced_text::Renderer + 'a,
{
    fn from(marquee: Marquee<'a>) -> Self {
        Element::new(marquee)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The §5 values the tests below spell out in whole seconds. Pinned
    /// against the tokens by [`the_tokens_are_the_section_5_values`], so the
    /// timelines stay readable without silently drifting from the theme.
    const DWELL: Duration = Duration::from_secs(2);
    const SPEED: f32 = 24.0;

    /// The panel's frame budget (`MARQUEE_TICK`, a consumer constant — see
    /// the module docs), reproduced here only to sample the sweep at a
    /// realistic rate.
    const TICK: Duration = Duration::from_millis(33);

    /// Seconds as a `Duration`, for readable timeline assertions.
    fn at(seconds: f64) -> Duration {
        Duration::from_secs_f64(seconds)
    }

    #[test]
    fn the_tokens_are_the_section_5_values() {
        let t = Theme::saola();
        assert_eq!(u128::from(t.motion.marquee_dwell), DWELL.as_millis());
        assert_eq!(t.motion.marquee_speed, SPEED);
    }

    #[test]
    fn the_constructor_reads_dwell_and_speed_from_the_theme() {
        let t = Theme::saola();
        let widget = marquee(
            &t,
            "a long overflowing title",
            at(3.0),
            5,
            13.0,
            Color::WHITE,
        );
        assert_eq!(widget.dwell, DWELL);
        assert_eq!(widget.speed, SPEED);
        assert_eq!(widget.elapsed, at(3.0));
        assert_eq!(widget.max_chars, 5);
    }

    // -- the marquee's window ----------------------------------------------

    #[test]
    fn the_window_is_max_chars_of_the_measured_text() {
        // Ten characters measuring 100 px: one character averages 10 px, so a
        // four-character window is 40 px wide and the text has 60 px to
        // travel.
        assert_eq!(window_width(100.0, 10, 4), 40.0);
        assert_eq!(window_width(100.0, 10, 10), 100.0);
        // Never wider than the text itself — a budget larger than the text
        // is a window the whole text already fits in, not empty space the
        // widget would reserve.
        assert_eq!(window_width(100.0, 10, 40), 100.0);
        // An empty string has no advance to average; zero width, no travel.
        assert_eq!(window_width(0.0, 0, 50), 0.0);
    }

    #[test]
    fn the_pixel_window_and_the_character_gate_agree() {
        // The property that lets a consumer gate on characters while the
        // sweep runs on pixels: the window is narrower than the text exactly
        // when the text is longer than the budget, for any measured width.
        // Anything else would let a timer tick with nothing moving.
        for chars in 1..40usize {
            for max_chars in 1..40usize {
                let full = 7.25 * chars as f32; // any average advance
                let window = window_width(full, chars, max_chars);
                assert_eq!(
                    window < full,
                    chars > max_chars,
                    "{chars} chars in a {max_chars}-char window"
                );
            }
        }
    }

    // -- the state machine -------------------------------------------------

    /// 48 px of travel is exactly 2 s of sweep at the §5 rate, which makes
    /// every boundary in the cycle a whole second: dwell 0–2, sweep 2–4,
    /// dwell 4–6, sweep back 6–8.
    const TRAVEL: f32 = 48.0;

    /// [`phase_at`] at the §5 token values.
    fn phase(elapsed: Duration, overflow_px: f32) -> Phase {
        phase_at(elapsed, overflow_px, DWELL, SPEED)
    }

    /// [`offset_at`] at the §5 token values.
    fn offset(elapsed: Duration, overflow_px: f32) -> f32 {
        offset_at(elapsed, overflow_px, DWELL, SPEED)
    }

    #[test]
    fn the_cycle_runs_head_sweep_tail_sweep_and_repeats() {
        // Dwell at the head, parked at the start of the text.
        assert_eq!(phase(at(0.0), TRAVEL), Phase::DwellHead);
        assert_eq!(offset(at(0.0), TRAVEL), 0.0);
        assert_eq!(phase(at(1.999), TRAVEL), Phase::DwellHead);
        assert_eq!(offset(at(1.999), TRAVEL), 0.0);

        // Sweeping left, linearly: half the sweep is half the travel.
        assert_eq!(phase(at(2.0), TRAVEL), Phase::SweepLeft { progress: 0.0 });
        assert_eq!(offset(at(2.0), TRAVEL), 0.0);
        assert_eq!(phase(at(3.0), TRAVEL), Phase::SweepLeft { progress: 0.5 });
        assert_eq!(offset(at(3.0), TRAVEL), 24.0);

        // Dwell at the tail, with the end of the text fully visible.
        assert_eq!(phase(at(4.0), TRAVEL), Phase::DwellTail);
        assert_eq!(offset(at(4.0), TRAVEL), TRAVEL);
        assert_eq!(phase(at(5.999), TRAVEL), Phase::DwellTail);
        assert_eq!(offset(at(5.999), TRAVEL), TRAVEL);

        // Sweeping back the same way, at the same rate.
        assert_eq!(phase(at(6.0), TRAVEL), Phase::SweepRight { progress: 0.0 });
        assert_eq!(offset(at(6.0), TRAVEL), TRAVEL);
        assert_eq!(phase(at(7.0), TRAVEL), Phase::SweepRight { progress: 0.5 });
        assert_eq!(offset(at(7.0), TRAVEL), 24.0);

        // And straight back into the head dwell — the loop closes with no
        // pause of its own beyond the two specced ones.
        assert_eq!(phase(at(8.0), TRAVEL), Phase::DwellHead);
        assert_eq!(offset(at(8.0), TRAVEL), 0.0);
        // A second lap is the first lap, exactly (and a hundredth is too —
        // the modulo happens in f64, so long-lived text doesn't drift).
        assert_eq!(phase(at(11.0), TRAVEL), Phase::SweepLeft { progress: 0.5 });
        assert_eq!(offset(at(803.0), TRAVEL), 24.0);
    }

    #[test]
    fn the_sweep_takes_its_time_from_the_distance_not_a_duration() {
        // 24 px/s is a rate: half the travel is half the sweep, so the whole
        // cycle is shorter for text that only just overflows. 24 px = 1 s
        // each way, so this cycle is 6 s rather than 8.
        assert_eq!(phase(at(2.5), 24.0), Phase::SweepLeft { progress: 0.5 });
        assert_eq!(offset(at(2.5), 24.0), 12.0);
        assert_eq!(phase(at(3.0), 24.0), Phase::DwellTail);
        assert_eq!(phase(at(6.0), 24.0), Phase::DwellHead);

        // Very long text takes proportionally longer to cross, and is
        // still dwelling at its head for exactly 2 s first.
        assert_eq!(phase(at(1.9), 2400.0), Phase::DwellHead);
        assert_eq!(offset(at(52.0), 2400.0), 1200.0);
    }

    #[test]
    fn the_offset_never_leaves_the_travel_and_the_sweeps_are_linear() {
        // Sampled at the consumer's frame rate across two full cycles: the
        // text is never pulled past either end, and each sweep advances by
        // the same distance every frame (linear, per §5 — no easing
        // anywhere).
        let mut previous = 0.0f32;
        let mut elapsed = Duration::ZERO;
        while elapsed < at(16.0) {
            let position = offset(elapsed, TRAVEL);
            assert!(
                (0.0..=TRAVEL).contains(&position),
                "offset left the travel at {elapsed:?}: {position}"
            );
            // Nothing ever jumps more than a frame's worth of travel.
            assert!(
                (position - previous).abs() <= SPEED * TICK.as_secs_f32() + 1e-3,
                "offset jumped at {elapsed:?}: {previous} → {position}"
            );
            previous = position;
            elapsed += TICK;
        }

        // Linearity, stated directly: equal time steps inside one sweep cover
        // equal ground (the breath's cosine ease is the *other* animation).
        let first = offset(at(2.5), TRAVEL) - offset(at(2.0), TRAVEL);
        let second = offset(at(3.0), TRAVEL) - offset(at(2.5), TRAVEL);
        assert!((first - second).abs() < 1e-4, "{first} vs {second}");
    }

    #[test]
    fn nothing_moves_when_there_is_nothing_to_reveal() {
        // Text that fits has no travel, so the machine parks at the head
        // for all time rather than sweeping zero pixels back and forth.
        for seconds in [0.0, 2.0, 4.5, 900.0] {
            assert_eq!(phase(at(seconds), 0.0), Phase::DwellHead);
            assert_eq!(offset(at(seconds), 0.0), 0.0);
            // Negative travel (a window wider than its text) is the same
            // case, and a degenerate measurement must not produce a NaN
            // offset either.
            assert_eq!(offset(at(seconds), -12.0), 0.0);
            assert_eq!(offset(at(seconds), f32::NAN), 0.0);
        }
    }

    #[test]
    fn a_degenerate_speed_parks_rather_than_dividing_by_it() {
        // Not reachable from `Theme::saola()`, but a TOML override could
        // hold a zero or negative rate, and the answer to "sweep at no
        // speed" is the same parked head dwell as "nothing to sweep" — never
        // a division by zero propagating into a draw offset.
        for speed in [0.0, -24.0, f32::NAN] {
            assert_eq!(phase_at(at(5.0), TRAVEL, DWELL, speed), Phase::DwellHead);
            assert_eq!(offset_at(at(5.0), TRAVEL, DWELL, speed), 0.0);
        }
    }
}
