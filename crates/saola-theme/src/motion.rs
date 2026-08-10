//! Motion math over the `motion.*` tokens — pure `std` arithmetic, no iced.
//!
//! The token crate stores durations as plain milliseconds ([`saola_tokens::Motion`])
//! and deliberately carries no math; iced carries no subtree opacity or
//! animation curves. That left every consumer re-deriving the same three
//! pieces of arithmetic from the tokens — a clamped progress fraction
//! (saola-capture's toast, saola-files' undo toast *and* its `core::fs::ops`,
//! three independent derivations), the three-phase toast envelope (same
//! three), and the breathing curve (saola-panel's Claude semaphore). This
//! module is those derivations, promoted, so surfaces can't drift apart.
//!
//! Everything here is a pure function of `(tokens, elapsed)` — feed it a
//! `Duration` measured from the animation's start (e.g.
//! `Instant::now() - shown_at`) on every tick and paint the result;
//! there is no state to keep.

use std::time::Duration;

use saola_tokens::Theme;

/// `elapsed / duration_ms`, clamped to `0.0..=1.0` — the progress fraction
/// every animation here is built from. A zero `duration_ms` (a pathological
/// theme override, not reachable from the built-in theme) returns `1.0` —
/// "already fully elapsed" — rather than dividing by zero.
pub fn fraction(elapsed: Duration, duration_ms: u32) -> f32 {
    if duration_ms == 0 {
        return 1.0;
    }
    (elapsed.as_secs_f32() / Duration::from_millis(u64::from(duration_ms)).as_secs_f32())
        .clamp(0.0, 1.0)
}

/// A toast's opacity at `elapsed`: the style guide §5 three-phase envelope —
/// fade in over `motion.toast_in`, hold at `1.0` for `motion.toast_idle`,
/// fade out over `motion.toast_out`, then stay at `0.0` (the consumer
/// removes the toast at `motion.toast_total`; a straggling frame after that
/// renders as fully transparent, not a flash).
///
/// Pair the result with [`crate::style::container::notification_card`] for
/// the card chrome and [`crate::convert::ColorExt::with_opacity`] for the
/// content painted inside it.
pub fn toast_alpha(t: &Theme, elapsed: Duration) -> f32 {
    let in_dur = Duration::from_millis(t.motion.toast_in.into());
    let idle_dur = Duration::from_millis(t.motion.toast_idle.into());

    if elapsed < in_dur {
        fraction(elapsed, t.motion.toast_in)
    } else if elapsed < in_dur + idle_dur {
        1.0
    } else {
        let fade_elapsed = elapsed.saturating_sub(in_dur + idle_dur);
        1.0 - fraction(fade_elapsed, t.motion.toast_out)
    }
}

/// A toast's life-rule fraction at `elapsed`: the style guide §6 lifetime
/// countdown, shaped from the same phase boundaries as [`toast_alpha`] but
/// inverted for "remaining life" instead of "opacity" — `1.0` (full) while
/// the toast is still arriving (`motion.toast_in`; nothing to count down
/// yet), draining linearly to `0.0` across `motion.toast_idle` (the
/// countdown proper — the fraction of visible time left), then staying at
/// `0.0` through `motion.toast_out` (the toast is already expired and
/// fading away, so the rule has nothing left to show — consistent with
/// [`crate::style::container::card_urgent`]'s "a terracotta ring and no
/// life rule": a card that never counts down never needs this function
/// either).
///
/// Pair the result with [`crate::style::notification::life_rule`] as an
/// iced `progress_bar`'s `value` (range `0.0..=1.0`): the bar drains from
/// full to empty as the toast ages, the same clamped-progress idiom
/// [`toast_alpha`] uses for opacity.
pub fn life_fraction(t: &Theme, elapsed: Duration) -> f32 {
    let in_dur = Duration::from_millis(t.motion.toast_in.into());
    let idle_dur = Duration::from_millis(t.motion.toast_idle.into());

    if elapsed < in_dur {
        1.0
    } else if elapsed < in_dur + idle_dur {
        let idle_elapsed = elapsed.saturating_sub(in_dur);
        1.0 - fraction(idle_elapsed, t.motion.toast_idle)
    } else {
        0.0
    }
}

/// A breathing dot's opacity at `elapsed`: a cosine sweep between
/// `motion.breathe_min_opacity` and `1.0`, one full breath (dim → bright →
/// dim) per `motion.breathe` ms, looping. Feed the result to
/// [`crate::style::container::status_dot`] as its `breath` argument for the
/// two running session states (the other three pass `1.0`).
///
/// The exact math is ported from saola-panel's semaphore (`phase_of` +
/// `breath_at`), so any second surface that breathes can never desync from
/// the panel:
///
/// - The phase is computed by an **integer-millisecond modulo** before any
///   float division — a dot that has been breathing for an hour stays as
///   precise as one that started a second ago, instead of losing bits to an
///   ever-growing `f32`.
/// - The curve is a cosine, not a triangle: `(1 - cos(2πp)) / 2` has zero
///   slope at both turning points, so the fade slows to a stop and reverses
///   smoothly — a constant-rate ramp reverses direction instantly at each
///   end, which the eye reads as a twitch, not a breath.
/// - Phase `0` is the **dim** end (`breathe_min_opacity`), never zero — a
///   breathing dot must never vanish — and a zero `motion.breathe` degrades
///   to a steady dim dot rather than a `NaN`.
pub fn breath(t: &Theme, elapsed: Duration) -> f32 {
    let min = t.motion.breathe_min_opacity;
    let phase = phase_of(elapsed, t.motion.breathe);
    let eased = (1.0 - (phase * std::f32::consts::TAU).cos()) / 2.0;
    min + (1.0 - min) * eased
}

/// Where in the breath cycle `elapsed` lands, as a fraction in `0.0..1.0`.
/// See [`breath`] for why the modulo happens in integer milliseconds.
fn phase_of(elapsed: Duration, cycle_ms: u32) -> f32 {
    if cycle_ms == 0 {
        return 0.0;
    }
    let cycle_ms = u128::from(cycle_ms);
    (elapsed.as_millis() % cycle_ms) as f32 / cycle_ms as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme() -> Theme {
        Theme::saola()
    }

    #[test]
    fn fraction_is_clamped_progress() {
        assert_eq!(fraction(Duration::ZERO, 1000), 0.0);
        assert_eq!(fraction(Duration::from_millis(500), 1000), 0.5);
        assert_eq!(fraction(Duration::from_millis(1000), 1000), 1.0);
        // Past the end clamps rather than overshooting.
        assert_eq!(fraction(Duration::from_millis(2500), 1000), 1.0);
        // Zero duration reads as "already fully elapsed", not NaN.
        assert_eq!(fraction(Duration::ZERO, 0), 1.0);
        assert_eq!(fraction(Duration::from_millis(5), 0), 1.0);
    }

    #[test]
    fn toast_alpha_three_phases() {
        let t = theme();
        let in_dur = Duration::from_millis(t.motion.toast_in.into());
        let idle_dur = Duration::from_millis(t.motion.toast_idle.into());
        let out_dur = Duration::from_millis(t.motion.toast_out.into());

        // Fade-in: 0 at the start, 1 at its end, rising in between.
        assert_eq!(toast_alpha(&t, Duration::ZERO), 0.0);
        assert_eq!(toast_alpha(&t, in_dur), 1.0);
        let mid_in = toast_alpha(&t, in_dur / 2);
        assert!(mid_in > 0.0 && mid_in < 1.0);

        // Idle holds at 1.
        assert_eq!(toast_alpha(&t, in_dur + idle_dur / 2), 1.0);

        // Fade-out: down from 1 to 0 at toast_total, and stays 0 after.
        let mid_out = toast_alpha(&t, in_dur + idle_dur + out_dur / 2);
        assert!(mid_out > 0.0 && mid_out < 1.0);
        assert_eq!(toast_alpha(&t, in_dur + idle_dur + out_dur), 0.0);
        assert_eq!(
            toast_alpha(&t, in_dur + idle_dur + out_dur + Duration::from_secs(1)),
            0.0
        );
    }

    #[test]
    fn life_fraction_three_phases() {
        let t = theme();
        let in_dur = Duration::from_millis(t.motion.toast_in.into());
        let idle_dur = Duration::from_millis(t.motion.toast_idle.into());
        let out_dur = Duration::from_millis(t.motion.toast_out.into());

        // Full through the whole entrance — nothing to count down yet.
        assert_eq!(life_fraction(&t, Duration::ZERO), 1.0);
        assert_eq!(life_fraction(&t, in_dur), 1.0);

        // Draining linearly across idle: 1 at the start, 0 at the end,
        // falling in between.
        assert_eq!(life_fraction(&t, in_dur + idle_dur), 0.0);
        let mid_idle = life_fraction(&t, in_dur + idle_dur / 2);
        assert!(mid_idle > 0.0 && mid_idle < 1.0);

        // Empty through the fade-out and beyond — already expired.
        assert_eq!(life_fraction(&t, in_dur + idle_dur + out_dur / 2), 0.0);
        assert_eq!(
            life_fraction(&t, in_dur + idle_dur + out_dur + Duration::from_secs(1)),
            0.0
        );
    }

    #[test]
    fn breath_phase_zero_is_the_dim_end() {
        let t = theme();
        // At phase 0 the cosine eases to 0, so the dot sits at
        // `breathe_min_opacity` — the panel's `breath_at(0.0, min) == min`.
        let start = breath(&t, Duration::ZERO);
        assert!((start - t.motion.breathe_min_opacity).abs() < 1e-6);
    }

    #[test]
    fn breath_peaks_at_half_cycle() {
        let t = theme();
        let half = Duration::from_millis(u64::from(t.motion.breathe) / 2);
        assert!((breath(&t, half) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn breath_segments_are_monotonic() {
        let t = theme();
        let cycle = u64::from(t.motion.breathe);
        // Rising through the first half-cycle…
        let mut last = breath(&t, Duration::ZERO);
        for ms in (0..=cycle / 2).step_by(100) {
            let now = breath(&t, Duration::from_millis(ms));
            assert!(now >= last, "not rising at {ms}ms: {now} < {last}");
            last = now;
        }
        // …falling through the second.
        for ms in (cycle / 2..=cycle).step_by(100) {
            let now = breath(&t, Duration::from_millis(ms));
            assert!(now <= last, "not falling at {ms}ms: {now} > {last}");
            last = now;
        }
    }

    #[test]
    fn breath_period_wraps() {
        let t = theme();
        let cycle = u64::from(t.motion.breathe);
        for ms in [0u64, 300, 1200, 1800] {
            let one = breath(&t, Duration::from_millis(ms));
            let next = breath(&t, Duration::from_millis(ms + cycle));
            let hour_later = breath(&t, Duration::from_millis(ms + cycle * 1500));
            assert!((one - next).abs() < 1e-6);
            assert!((one - hour_later).abs() < 1e-6);
        }
    }

    #[test]
    fn breath_zero_cycle_degrades_to_steady_dim() {
        let mut t = theme();
        t.motion.breathe = 0;
        let value = breath(&t, Duration::from_millis(1234));
        assert!((value - t.motion.breathe_min_opacity).abs() < 1e-6);
    }
}
