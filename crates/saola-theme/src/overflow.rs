//! Text overflow — style guide §5/§7's **default** mode, the width cap.
//!
//! The style guide gives text that does not fit exactly two treatments. This
//! module is the first and the default: [`truncate`] cuts at a width budget
//! and ends the string in a single `…`, with no motion at all. The second is
//! the opt-in ping-pong sweep in [`crate::marquee`], enabled only by explicit
//! configuration (§5, "The window-title marquee — opt-in, exact"). Any
//! surface that has to shorten a label — a panel's focused window title, a
//! file manager's grid label — reaches for this one unless its consumer asked
//! for motion.
//!
//! Ported from saola-panel's private `truncate` in
//! `modules/window_title.rs`, which is where the rule was first written
//! down; it lives here now because it is the design language's answer, not
//! one module's.
//!
//! # The limit is width units, and that is on purpose
//!
//! §7: "The limit is measured in characters, not pixels — approximate under
//! a proportional face, but it is the knob a human can reason about." The
//! budget here is that same human knob, counted in **width units**: roughly
//! N narrow characters, or N/2 wide ones. An `m` and an `l` are still not the
//! same width, so a 50-unit cap is not a 50-column cap; the guide blesses
//! that approximation rather than making every consumer measure glyphs.
//! Widgets that genuinely need pixels (the marquee, which sweeps at a pixel
//! rate) measure with the renderer instead.
//!
//! # Why a plain `char` count was not enough
//!
//! One average advance cannot describe two scripts. A full-width CJK glyph
//! is about 1.0 em wide where average Latin is about 0.55 em, so a filename
//! cut to the same *character* count renders about twice as wide in Japanese
//! as in English — measured in saola-files: ~145 px of text in a 96 px grid
//! tile, from a budget tuned on Latin names.
//!
//! So the unit is Unicode UAX #11 East Asian Width, via the `unicode-width`
//! crate: a Wide or Fullwidth character costs **2**, everything else costs
//! **1**, and a character with no advance at all (a control character) costs
//! **0**. The `…` is one unit and is spent from inside the budget. For
//! input that is entirely narrow the arithmetic is exactly a character count,
//! so behaviour is byte-for-byte what it was before width awareness.
//!
//! Two caveats worth knowing before tuning a budget:
//!
//! - **Ambiguous-width characters count as narrow.** UAX #11 leaves a class
//!   (Greek, Cyrillic, `±`, box drawing, `★`) whose width depends on the
//!   surrounding locale's font; `unicode-width`'s default resolves them to 1,
//!   and so do we. That is right for Saola, whose faces are Latin-metric.
//! - **This is still an approximation for proportional Latin.** Two units of
//!   `ll` and two units of `mm` do not paint the same number of pixels.
//!   Width awareness fixes the *script*-scale error (2x), not the
//!   glyph-scale one. A consumer that needs a hard single-line guarantee
//!   pairs this with `text::Wrapping::None`, which is the actual promise that
//!   a label never becomes two lines; `truncate` is what keeps the one line
//!   from being absurdly long.
//!
//! Characters here means `char` — Unicode scalar values — not grapheme
//! clusters, the same concession the panel documents: a family-emoji
//! sequence or a combining accent is measured per scalar, and getting that
//! right needs a segmentation crate for a knob that is already an
//! approximation of visual width.

use unicode_width::UnicodeWidthChar;

/// The one glyph the style guide allows for elided text: U+2026, a single
/// character, never three periods (which would cost three of the budget and
/// kern like a typo). It is East-Asian-Width Narrow, so it costs exactly one
/// unit — the reservation below depends on that.
const ELLIPSIS: char = '…';

/// The width of `ch` in the budget's units: 2 for a UAX #11 Wide or
/// Fullwidth character, 1 for everything with an ordinary advance, 0 for a
/// character that paints nothing.
///
/// `UnicodeWidthChar::width` returns `None` for control characters (which
/// have no defined advance at all, and which a label should not contain in
/// the first place). Charging them 0 is the honest answer — they take no
/// space — and `unwrap_or` keeps the no-panic rule without a branch.
fn width_units(ch: char) -> usize {
    UnicodeWidthChar::width(ch).unwrap_or(0)
}

/// Caps `s` at `max_units` **width units** (see the module docs: a narrow
/// character costs 1, a wide/fullwidth one 2), ending in a single
/// [`ELLIPSIS`] when anything was cut.
///
/// Three things are deliberate:
///
/// 1. **The ellipsis is inside the budget.** A string cut at `max_units =
///    50` renders as 49 units of text plus `…`, never 51 — the knob names
///    the width the caller gets, not the width before an extra glyph is
///    bolted on.
/// 2. **The result never overflows the budget, even by half a glyph.** When
///    the next character is wide and only one unit of room is left, that
///    character is dropped rather than squeezed in: the cut can come in
///    *under* the budget by a unit, never over it.
/// 3. **Characters, never bytes.** `&s[..max_units]` would panic the first
///    time the text contained a non-ASCII character (an em dash in an
///    editor's title, a CJK filename, an emoji) and the cut landed
///    mid-codepoint: a `str` may only be sliced at a UTF-8 character
///    boundary, and byte `max_units` is not one in general.
///    [`str::char_indices`] hands back the *byte offset where a real
///    character starts*, so the slice below is always on a boundary and this
///    function cannot panic on any input.
///
/// A string whose total width is already within the limit comes back
/// unchanged — no ellipsis, including when it lands exactly on the limit —
/// and `max_units == 0` yields an empty string.
///
/// Pure function of its arguments, which is what makes it unit-testable
/// without a renderer or a theme.
///
/// ```
/// use saola_theme::overflow::truncate;
///
/// assert_eq!(truncate("nvim", 50), "nvim");
/// assert_eq!(truncate("a very long window title", 8), "a very …");
/// // Wide glyphs cost two units each, so the same budget keeps half as
/// // many of them — and paints about the same number of pixels.
/// assert_eq!(truncate("設定ウィンドウ", 8), "設定ウ…");
/// ```
pub fn truncate(s: &str, max_units: usize) -> String {
    // Fast path: nothing to cut. Summing per-`char` (rather than asking
    // `unicode_width` for the whole string's width) is deliberate — it has to
    // be the *same* accounting the loop below uses, or a string could pass
    // this check under one rule and be cut under another.
    let total: usize = s.chars().map(width_units).sum();
    if total <= max_units {
        return s.to_owned();
    }
    // A budget of one leaves room for the ellipsis alone; zero leaves room
    // for nothing. Both are defensive — a sane consumer config rejects a
    // non-positive limit — but neither may panic or emit a stray ellipsis.
    if max_units == 0 {
        return String::new();
    }

    // One unit is reserved for the ellipsis, so the kept prefix gets the
    // rest. `max_units` is at least 1 here, so this cannot underflow.
    let prefix_budget = max_units - 1;

    // Walk to the byte offset where the kept prefix ends: the first
    // character that would not fit in `prefix_budget` stops it. A wide
    // character straddling the last unit fails this test and is left out
    // whole, which is why the result can come in a unit short.
    //
    // `s.len()` is the loop's unreachable fallback (the fast path proved the
    // total exceeds `max_units`, so some character must fail to fit) and is a
    // valid slice end regardless — no `unwrap`, no indexing.
    let mut used = 0usize;
    let mut cut = s.len();
    for (offset, ch) in s.char_indices() {
        // `saturating_add` over `+` so a pathological input can't panic on
        // debug-build overflow; the no-panic rule has no exceptions.
        if used.saturating_add(width_units(ch)) > prefix_budget {
            cut = offset;
            break;
        }
        used = used.saturating_add(width_units(ch));
    }

    let mut capped = String::with_capacity(cut + ELLIPSIS.len_utf8());
    capped.push_str(&s[..cut]);
    capped.push(ELLIPSIS);
    capped
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Total rendered width of a result, in the same units the budget uses —
    /// the invariant every truncation has to satisfy.
    fn units(s: &str) -> usize {
        s.chars().map(width_units).sum()
    }

    #[test]
    fn a_string_within_the_limit_is_untouched() {
        assert_eq!(truncate("nvim", 50), "nvim");
        // Exactly at the limit is still untouched — the cap is inclusive.
        assert_eq!(truncate("abcde", 5), "abcde");
        // And inclusive for wide characters too: two wide glyphs are four
        // units, which fits a four-unit budget exactly.
        assert_eq!(truncate("設定", 4), "設定");
        assert_eq!(truncate("", 0), "");
    }

    #[test]
    fn an_overlong_string_is_cut_with_one_ellipsis() {
        assert_eq!(truncate("abcdef", 5), "abcd…");
        // The ellipsis is one unit and lives *inside* the budget, so the
        // result is never wider than the limit.
        assert_eq!(units(&truncate("abcdef", 5)), 5);
        assert_eq!(truncate("abcdef", 5).matches('…').count(), 1);
    }

    #[test]
    fn narrow_input_behaves_exactly_as_the_character_cap_did() {
        // Every one of these was asserted verbatim before width awareness
        // (saola-theme 0.9.0): for text with no wide characters, a unit is a
        // character and the two rules coincide.
        assert_eq!(truncate("nvim", 50), "nvim");
        assert_eq!(truncate("abcde", 5), "abcde");
        assert_eq!(truncate("abcdef", 5), "abcd…");
        assert_eq!(truncate("a very long window title", 8), "a very …");
        // Em dashes are ordinary in real titles ("nvim — src/main.rs") and
        // are multi-byte but *narrow*, so they cost one unit each, exactly
        // as they cost one character each before.
        let em_dashes = "————————";
        assert_eq!(truncate(em_dashes, 5), "————…");
        assert_eq!(units(&truncate(em_dashes, 5)), 5);
        // A cut that lands *between* a multi-byte character and an ASCII one
        // is a UTF-8 boundary too — the reason this cannot slice by byte.
        assert_eq!(truncate("é-abc", 3), "é-…");
    }

    #[test]
    fn wide_characters_cost_two_units_each() {
        // The bug this fixes: 12 CJK characters are 24 units, so an 8-unit
        // budget keeps 3 of them plus the ellipsis (7 units) — about half
        // the character count, and about the pixel width the budget names.
        let cjk = "設定ウィンドウのタイトル";
        assert_eq!(truncate(cjk, 8), "設定ウ…");
        assert_eq!(units(&truncate(cjk, 8)), 7);
        // The old character cap at the same number would have kept 7 of
        // them — roughly twice as wide as asked for.
        assert!(units("設定ウィンドウ") > 8);

        // Halving the budget halves the glyphs, as a width knob should.
        assert_eq!(truncate(cjk, 4), "設…");
        assert_eq!(units(&truncate(cjk, 4)), 3);
    }

    #[test]
    fn mixed_scripts_are_charged_per_character() {
        // "src/" is 4 narrow units, then two wide ones. Budget 9 leaves 8
        // for the prefix: 4 + 2 + 2 fills it exactly, and " .txt" is cut.
        let mixed = "src/設定 notes.txt";
        assert_eq!(truncate(mixed, 9), "src/設定…");
        assert_eq!(units(&truncate(mixed, 9)), 9);

        // Wide first, narrow after: 2 + 1 + 1 + 1 = 5 units of prefix in a
        // 6-unit budget.
        assert_eq!(truncate("設 abc def", 6), "設 ab…");
        assert_eq!(units(&truncate("設 abc def", 6)), 6);
    }

    #[test]
    fn a_wide_character_straddling_the_reserve_is_dropped_whole() {
        // Budget 4 reserves 1 for the ellipsis, leaving 3 for the prefix.
        // "ab" spends 2; the next glyph is wide and wants 2 more, which
        // would make the rendered result 5 units in a 4-unit budget. It is
        // left out entirely rather than allowed to overhang, so the result
        // comes in a unit *under* the cap.
        assert_eq!(truncate("ab設定x", 4), "ab…");
        assert_eq!(units(&truncate("ab設定x", 4)), 3);

        // Same shape at the smallest budget that can hold anything: 2 units
        // reserve 1, and a wide leading glyph cannot fit in the remaining 1.
        assert_eq!(truncate("設定", 2), "…");
        assert_eq!(units(&truncate("設定", 2)), 1);
        // One narrow unit of room, and a narrow character to spend it on.
        assert_eq!(truncate("ab設", 2), "a…");
    }

    #[test]
    fn emoji_are_wide() {
        // Checked against unicode-width 0.2, not assumed: emoji-presentation
        // characters are UAX #11 Wide, so U+1F98C DEER costs 2 units, the
        // same as a CJK ideograph.
        assert_eq!(width_units('🦌'), 2);
        assert_eq!(width_units('☕'), 2);
        // Which means a 3-unit budget holds exactly one of them plus the
        // ellipsis — the character cap would have kept two, at 5 units.
        let emoji = "🦌🦌🦌🦌🦌";
        assert_eq!(truncate(emoji, 3), "🦌…");
        assert_eq!(units(&truncate(emoji, 3)), 3);
        // A budget of 4 cannot buy a second deer (2 + 2 + 1 = 5 > 4), so it
        // renders the same as 3 — the straddle rule again.
        assert_eq!(truncate(emoji, 4), "🦌…");
        assert_eq!(truncate(emoji, 5), "🦌🦌…");
    }

    #[test]
    fn ambiguous_width_characters_count_as_narrow() {
        // UAX #11 Ambiguous — `unicode-width`'s default resolves these to 1,
        // which is right for Saola's Latin-metric faces. Pinned as a test so
        // a future crate bump that changed the default would be caught here
        // rather than in a consumer's layout.
        assert_eq!(width_units('±'), 1);
        assert_eq!(width_units('★'), 1);
        assert_eq!(truncate("±±±±±±", 5), "±±±±…");
    }

    #[test]
    fn zero_width_characters_are_free() {
        // Control characters have no defined advance; charging them 0 keeps
        // the budget honest about painted width. A string of them therefore
        // "fits" any budget and comes back untouched.
        assert_eq!(width_units('\u{7}'), 0);
        assert_eq!(truncate("\u{7}\u{7}", 1), "\u{7}\u{7}");
    }

    #[test]
    fn tiny_limits_degrade_to_the_ellipsis_alone() {
        assert_eq!(truncate("abcdef", 1), "…");
        assert_eq!(truncate("設定", 1), "…");
        // A zero budget has no room for the ellipsis either. Not reachable
        // from a validated consumer config, but it must not panic or
        // produce a stray glyph.
        assert_eq!(truncate("abcdef", 0), "");
        assert_eq!(truncate("設定", 0), "");
    }

    #[test]
    fn no_result_ever_exceeds_its_budget() {
        // The invariant, swept: whatever the script mix, the rendered width
        // of the result is at most the budget — and a result that was cut
        // always ends in exactly one ellipsis.
        let samples = [
            "a very long window title",
            "設定ウィンドウのタイトル",
            "src/設定 notes.txt",
            "🦌 deer 🦌 rusa 🦌",
            "————————",
            "Ａ Ｂ Ｃ fullwidth",
            "",
        ];
        for s in samples {
            for max_units in 0..24usize {
                let out = truncate(s, max_units);
                assert!(
                    units(&out) <= max_units,
                    "{out:?} exceeds {max_units} units"
                );
                // A cut result ends in exactly one ellipsis — unless the
                // budget was 0, the one case with no room for even that.
                if out != s && max_units > 0 {
                    assert_eq!(out.matches('…').count(), 1, "{out:?}");
                }
            }
        }
    }
}
