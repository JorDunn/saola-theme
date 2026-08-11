//! Text overflow — style guide §5/§7's **default** mode, the character cap.
//!
//! The style guide gives text that does not fit exactly two treatments. This
//! module is the first and the default: [`truncate`] cuts at a character
//! limit and ends the string in a single `…`, with no motion at all. The
//! second is the opt-in ping-pong sweep in [`crate::marquee`], enabled only
//! by explicit configuration (§5, "The window-title marquee — opt-in,
//! exact"). Any surface that has to shorten a label — a panel's focused
//! window title, a file manager's grid label — reaches for this one unless
//! its consumer asked for motion.
//!
//! Ported from saola-panel's private `truncate` in
//! `modules/window_title.rs`, which is where the rule was first written
//! down; it lives here now because it is the design language's answer, not
//! one module's.
//!
//! # The limit is characters, and that is on purpose
//!
//! §7: "The limit is measured in characters, not pixels — approximate under
//! a proportional face, but it is the knob a human can reason about." An `m`
//! and an `l` are not the same width, so a 50-character cap is not a
//! 50-column cap; the guide blesses the approximation rather than making
//! every consumer measure glyphs. Widgets that genuinely need pixels (the
//! marquee, which sweeps at a pixel rate) measure with the renderer instead.
//!
//! Characters here means `char` — Unicode scalar values — not grapheme
//! clusters, the same concession the panel documents: a family-emoji
//! sequence or a combining accent counts as several, and getting that right
//! needs a segmentation crate for a knob that is already an approximation of
//! visual width.

/// The one glyph the style guide allows for elided text: U+2026, a single
/// character, never three periods (which would cost three of the budget and
/// kern like a typo).
const ELLIPSIS: char = '…';

/// Caps `s` at `max_chars` **characters**, ending in a single [`ELLIPSIS`]
/// when anything was cut.
///
/// Two things are deliberate:
///
/// 1. **The ellipsis is inside the budget.** A string cut at `max_chars =
///    50` renders as 49 characters plus `…`, never 51 — the knob names the
///    width the caller gets, not the width before an extra glyph is bolted
///    on.
/// 2. **Characters, never bytes.** `&s[..max_chars]` would panic the first
///    time the text contained a non-ASCII character (an em dash in an
///    editor's title, a CJK filename, an emoji) and the cut landed
///    mid-codepoint: a `str` may only be sliced at a UTF-8 character
///    boundary, and byte `max_chars` is not one in general.
///    [`str::char_indices`] hands back the *byte offset where a real
///    character starts*, so the slice below is always on a boundary and this
///    function cannot panic on any input.
///
/// A string already within the limit comes back unchanged — no ellipsis —
/// and `max_chars == 0` yields an empty string.
///
/// Pure function of its arguments, which is what makes it unit-testable
/// without a renderer or a theme.
///
/// ```
/// use saola_theme::overflow::truncate;
///
/// assert_eq!(truncate("nvim", 50), "nvim");
/// assert_eq!(truncate("a very long window title", 8), "a very …");
/// ```
pub fn truncate(s: &str, max_chars: usize) -> String {
    // Fast path: nothing to cut. `chars().count()` walks the string, but so
    // does rendering it, and labels are short.
    if s.chars().count() <= max_chars {
        return s.to_owned();
    }
    // A budget of one leaves room for the ellipsis alone; zero leaves room
    // for nothing. Both are defensive — a sane consumer config rejects a
    // non-positive limit — but neither may panic or emit a stray ellipsis.
    if max_chars == 0 {
        return String::new();
    }

    // The byte offset where the (max_chars - 1)th character starts — i.e.
    // where the kept prefix ends, leaving exactly one character of budget
    // for the ellipsis. `nth` is safe here because the fast path above
    // proved there are more than `max_chars` characters; `map_or` covers the
    // impossible `None` without an `unwrap`.
    let cut = s
        .char_indices()
        .nth(max_chars - 1)
        .map_or(s.len(), |(offset, _char)| offset);

    let mut capped = String::with_capacity(cut + ELLIPSIS.len_utf8());
    capped.push_str(&s[..cut]);
    capped.push(ELLIPSIS);
    capped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_string_within_the_limit_is_untouched() {
        assert_eq!(truncate("nvim", 50), "nvim");
        // Exactly at the limit is still untouched — the cap is inclusive.
        assert_eq!(truncate("abcde", 5), "abcde");
    }

    #[test]
    fn an_overlong_string_is_cut_with_one_ellipsis() {
        assert_eq!(truncate("abcdef", 5), "abcd…");
        // The ellipsis is one character and lives *inside* the budget, so
        // the result is never longer than the limit.
        assert_eq!(truncate("abcdef", 5).chars().count(), 5);
        assert_eq!(truncate("abcdef", 5).matches('…').count(), 1);
    }

    #[test]
    fn truncation_counts_characters_not_bytes() {
        // Every one of these is multi-byte UTF-8; a byte slice at 5 would
        // land mid-codepoint and panic. Em dashes are ordinary in real
        // titles ("nvim — src/main.rs"), CJK and emoji less so but perfectly
        // legal — and both are ordinary in filenames.
        let em_dashes = "————————";
        assert_eq!(truncate(em_dashes, 5).chars().count(), 5);
        assert_eq!(truncate(em_dashes, 5), "————…");

        let cjk = "設定ウィンドウのタイトル";
        assert_eq!(truncate(cjk, 4), "設定ウ…");

        let emoji = "🦌🦌🦌🦌🦌";
        assert_eq!(truncate(emoji, 3), "🦌🦌…");

        // A cut that lands *between* a multi-byte character and an ASCII one
        // is a boundary too.
        assert_eq!(truncate("é-abc", 3), "é-…");
    }

    #[test]
    fn tiny_limits_degrade_to_the_ellipsis_alone() {
        assert_eq!(truncate("abcdef", 1), "…");
        // A zero budget has no room for the ellipsis either. Not reachable
        // from a validated consumer config, but it must not panic or
        // produce a stray glyph.
        assert_eq!(truncate("abcdef", 0), "");
    }
}
