# Decision: tabular numerals (concept 4a — "the clock and percentages never reflow")

**Date:** 2026-07-26 (Stage 9)
**Status:** resolved — no code change needed; the UI face already does it.

## The question

The resolved panel design requires tabular figures: `09:41` and `11:11` must occupy
identical width so the bar clock and battery percentage never reflow as digits change.
CLAUDE.md deferred this because it was unknown whether iced 0.14 can enable the
OpenType `tnum` feature.

## What was checked (exact sources)

### 1. Does iced 0.14 expose OpenType font features? — **No.**

- `iced_core-0.14.0/src/font.rs` — `iced::Font` is exactly
  `{ family: Family, weight: Weight, stretch: Stretch, style: Style }`. No features
  field of any kind. (Checked in `~/.cargo/registry/src/index.crates.io-*/`;
  resolved versions per `Cargo.lock`: iced 0.14.0, iced_core 0.14.0,
  iced_graphics 0.14.0, cosmic-text 0.15.0.)
- `iced_graphics-0.14.0/src/text.rs`, `pub fn to_attributes(font: Font)` (line 259) —
  the single point where an `iced::Font` becomes a `cosmic_text::Attrs`. It maps
  only `family`/`weight`/`stretch`/`style` and leaves `Attrs::font_features` at its
  empty default. There is no other path from widget code into the shaper's
  feature list.
- iced **master** (checked 2026-07-26 on GitHub, `core/src/font.rs`): the `Font`
  struct is unchanged — still no features field. No shipped or imminent API.

### 2. Could the layer *below* iced do it? — Yes, the plumbing exists downstream.

- `cosmic-text-0.15.0/src/attrs.rs` — `Attrs` has `pub font_features: FontFeatures`
  (a `Vec<Feature { tag: FeatureTag, value: u32 }>`; `FeatureTag::new(b"tnum")` is
  constructible — there's no named const for it, but the type is open).
- `cosmic-text-0.15.0/src/shape.rs` (lines ~154–199) — `attrs.font_features` is
  converted to `harfrust::Feature`s and passed into `shape_with_plan`, so the
  feature *would* be honored if anything filled it in. iced never does.

Conclusion on the API question: **iced 0.14 cannot set `tnum`** (or any OpenType
feature). The gap is purely iced's `Font` type; cosmic-text is ready.

### 3. Does it matter? — No: IBM Plex's default figures are already tabular.

Checked the installed faces (`/usr/share/fonts/TTF/IBMPlex*.ttf`) by parsing
`cmap`/`hmtx`/`GSUB` directly (no fontTools on the machine; script logic:
cmap format 4 lookup → glyph ids for `0`–`9` → hmtx advances; GSUB FeatureList
tag scan):

| Face | digit advances (upem 1000) | GSUB has `tnum`? | GSUB has `pnum`? |
|---|---|---|---|
| IBM Plex Sans Regular / Text / Medium / SemiBold | all **600** | no | no |
| IBM Plex Serif Regular | all **600** | no | no |
| IBM Plex Mono Regular | all **600** (mono) | no | no |

The GSUB feature lists contain `lnum onum zero frac …` but **no `tnum` and no
`pnum`** — meaning Plex has no proportional figure set at all. The default lining
figures are the *only* figure widths, and they are uniform: tabular by design.
(`onum` swaps in oldstyle shapes, which we never enable; irrelevant here.)

### 4. End-to-end proof through the production shaping path.

A scratch harness against **cosmic-text 0.15.0 itself** (the exact shaper iced 0.14
renders through, including harfrust shaping and default kerning) measured
`Buffer::layout_runs()::line_w` at 16 px for equal-slot strings:

- `IBM Plex Sans` Regular: `09:41` / `11:11` / `23:59` / `00:00` → all **43.072 px**;
  `78%` / `17%` / `41%` / `99%` → all **34.032 px**; `0123456789` vs `1111111111` →
  both **95.99999 px** (= 10 × 0.6 em, confirming the 600-unit advance and that
  **no digit-pair kerning** perturbs anything).
- Same equalities hold at SemiBold and in Plex Mono.

## Decision

**Do nothing to the token model or the iced layer.** `Typography` stays as-is; no
numeric variant, no feature flag, no fallback-to-mono guidance for alignment:

- Numeric UI strings (bar clock, battery/volume percentages, notification counts)
  use the normal UI face via `convert::ui_font` — they are tabular for free.
- IBM Plex Mono remains reserved for its designed roles (keycaps, paths, hex,
  terminal), *not* as a numeric-alignment crutch.
- The Typography page of the gallery carries the proof specimen: two columns of
  equal-slot numeric strings, each string in its own shrink-width keycap chip —
  tabular figures make the chips stack flush; a proportional set would leave the
  columns ragged.

Rationale: the design requirement is satisfied by the font itself; adding token
metadata for a feature the renderer cannot express would be dead configuration.

## Caveats

- The guarantee is a property of **IBM Plex**, not of the stack. If a system is
  missing the Plex faces, cosmic-text falls back to some other sans whose figures
  may be proportional, silently breaking the no-reflow property. Saola should
  ensure Plex is installed (packaging dependency) — the theme crate cannot.
- Strings with *different* slot counts (`7%` vs `78%`) still differ in width — by
  exactly one digit advance. Fixed-width layout for those is a consumer concern
  (e.g. reserve three digit slots for percentages), not a font concern.
- `:` `%` `.` are proportional in Plex Sans, but in fixed-format strings (HH:MM)
  they always occupy the same slot, so totals stay equal — verified above.

## Revisit when

- **iced grows a font-feature API**: watch `iced::Font` for a features field and
  `iced_graphics::text::to_attributes` for forwarding into
  `cosmic_text::Attrs::font_features` (the downstream plumbing already exists as
  of cosmic-text 0.15). No upstream iced issue tracks this as of 2026-07-26.
  If it lands, nothing here *needs* to change — but `zero` (slashed zero) or
  `ss0x` stylistic sets would become expressible if the design ever wants them.
- **The UI family ever changes from IBM Plex**: re-run the two checks (static
  digit advances + GSUB tag scan; dynamic equal-slot widths through cosmic-text)
  against the new face before trusting the no-reflow property.
