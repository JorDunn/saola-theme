# saola-theme — agent instructions

Design-system crate for Saola, a Linux desktop environment built in Rust (iced + zbus).
Every Saola component (panel, greeter, notifications, launcher) consumes this crate.

## Commands

```bash
cargo build --workspace
cargo test --workspace                                  # hex parsing, serde round-trips, contrast invariants
cargo clippy --workspace --all-targets -- -D warnings   # CI gate — keep it green
cargo fmt --check                                       # CI gate
cargo run --example gallery                             # visual preview (needs a Wayland/X11 session)
cargo tree -p saola-tokens -e normal                    # must show ONLY serde/toml/thiserror
```

All four CI jobs (fmt, clippy, test, gallery build) must pass before any change is done.

## Architecture

Two crates in a virtual workspace, plus a directory example:

- `crates/saola-tokens` — **pure data**: colors, typography, radii, sizes, shadows, motion,
  terminal palette, with TOML I/O (serde + toml + thiserror). **Never add a GUI dependency
  here** — the crate boundary exists so future exporters (GRUB, Plymouth, Alacritty) can
  depend on tokens without iced. `cargo tree` is the proof.
- `crates/saola-theme` — the iced integration layer consumers import. `convert.rs` bridges
  token types to iced types; `style/*.rs` holds the style helpers; `examples/gallery/` is
  the preview app (an example, not a third crate).

Target: **stable iced 0.14** (`iced_layershell` 0.19.x tracks it). No zbus in this repo.

## Design language (binding — not suggestions)

- **Three colors, never a fourth:** ink `#0C0A00` (every shell surface), paper/ivory
  `#FFFFF0` (a control at rest; light window backgrounds), terracotta `#C67139`
  (on/selected/focused/live). `accent_light` is accent text on ink only; `accent_dark` is
  accent text on paper only. There is no danger/success/warning color — destructive
  confirmation is a consumer pattern, not a palette entry. One scoped exception (style
  guide §1, "Session status semaphore," 2026-07-31): five `status_*` tokens for the
  Claude Code session-status semaphore. Semaphore dots only — never a control, never a
  fill, never a fourth "real" color.
- **The one rule:** ivory fill = a control at rest (takes ink text); terracotta fill =
  on/selected/live (takes ivory text).
- **Surface contexts replace dark/light variants.** The axis is `Surface::Ink` vs
  `Surface::Paper`; text/divider/fill roles are the identity colors alpha-stepped, one
  `OnSurface` set per context (`Theme::on(surface)`). Hover/press move through
  `fill_subtle → fill → fill_strong`, never through new colors.
- **Shape:** everything is a pill (`radii.pill` = 999) or an over-rounded rectangle.
  Checkbox radius is `radii.checkbox` (7.0), not pill. Keyboard focus is a 2 px terracotta
  ring (`style::focus_border`), never a platform default.
- **Source of truth is `design/saola-tokens.json`.** If token values change there,
  update `Theme::saola()` by hand (CSS `rgba(r,g,b,a)` → `Color` with
  `alpha = round(a × 255)`; TOML wire format stays hex `#RRGGBBAA`), then regenerate
  `themes/saola.toml` from `Theme::saola().to_toml_string()`. `design/saola-theme.rs` is a
  frozen prototype (shape reference only, not compiled); `design/saola-alacritty.toml`
  shows the terminal-export target format.

## Conventions

- **Style helper pattern** (copy it exactly for new widgets): helpers take
  `(&Theme, Surface)` and return an `impl Fn(&iced::Theme, Status) -> Style` closure.
  Copy every needed token value into locals *before* the `move` closure — reading `t.*`
  inside the closure body is an E0700 lifetime-capture error. Container-shaped widgets
  (no `Status`) return `impl Fn(&iced::Theme) -> Style`.
- **serde defaults:** every token struct's `Default` is hand-written to the real Saola
  values and all fields are `#[serde(default)]`, so a partial TOML file parses to
  saola-plus-overrides. `Color::default()` is transparent black — never rely on it.
- **Orphan rule:** no `From` impls between token and iced types (both foreign) — use the
  `ColorExt`/`ShadowExt` extension traits in `convert.rs`.
- **`Box::leak` is confined to `convert.rs`** (`leak_font_name`, because
  `iced::Font::with_name` needs `&'static str`). Call once per theme load; never leak
  anywhere else.
- **Opaque-fill state layering:** a flat background can't stack paints, so hover/press on
  opaque fills is pre-composited in token space with `Color::over` (e.g. ivory pill hovers
  through `on_paper.fill_subtle` over paper). Translucent fills just pick a deeper fill
  step and let iced blend.
- Prefer explicit code over generic/dynamic abstraction (e.g. duplicate a `.style(...)`
  call per match arm rather than `Box<dyn Fn>`).
- New style helpers get a specimen in the gallery's Widgets page, shown on both surfaces.

## iced 0.14 gotchas (verified against sources — don't re-derive, don't guess)

- Per-widget `Status` enums vary wildly: `button` has no `Focused`; `slider` has no
  `Disabled`; `checkbox`/`toggler` have no `Pressed` and carry `is_checked`/`is_toggled`
  payloads; `pick_list::Status::Opened` doubles as focus; `scrollable` uses per-axis flags.
  **Read the widget's source in `~/.cargo/registry/src/*/iced_widget-0.14.*/src/` before
  writing a new style module.**
- `pick_list` needs two style closures on two Catalogs: `.style(...)` for the field,
  `.menu_style(...)` for the dropdown; the menu row-highlight radius comes from
  `menu::Style::border.radius`.
- `iced::widget::checkbox` is both a module and a function — alias the import
  (`use iced::widget::checkbox as checkbox_widget;`) before defining your own `checkbox`.
- `horizontal_space`/`vertical_space` don't exist in 0.14; use `Space`.
- `Element` is not `Clone`; consume `Vec<Element>` with `.into_iter()`, never `.chunks()`.
- A button with no `.on_press` renders as `Status::Disabled`.

## Releases

- **Commit messages must follow Conventional Commits** (`feat:`, `fix:`, `feat!:` for
  breaking, etc.) — release-plz derives each crate's semver bump and changelog from them.
  Pre-1.0, a breaking change bumps the minor version (Cargo semantics), and so does a
  plain `feat:` (`features_always_increment_minor` in `release-plz.toml`).
- Never bump versions in Cargo.toml or edit CHANGELOG.md files by hand — the release-plz
  release PR does both, per crate, only for crates that changed. Config lives in
  `release-plz.toml`; the workflow is `.github/workflows/release-plz.yml`.
- release-plz runs in `git_only` mode (versions come from git tags, not crates.io) and
  runs `cargo package --workspace` at the last tag — so every workspace path dependency
  must carry a `version` alongside its `path` (a new exporter crate depending on
  `saola-tokens` by path alone would poison the next tag and break every release-pr run
  after it; see commit 957ee68).
- Commits that shouldn't appear in a changelog use `chore:`/`ci:`/`docs:`/`test:` types.

## Boundaries

- Don't add dependencies without a reason tied to the design system; renderer choice
  belongs to consumers (the lib depends on iced with `default-features = false`,
  `features = ["wayland", "thread-pool"]`; only the gallery dev-dependency gets defaults).
- Live D-Bus theme switching lives in consumers, not here. Alternate themes are future
  TOML files; v0.1 ships only `Theme::saola()` (`Default == saola()`).
- Out of scope for now: the JSON's `mark` logo paths, `"$ref"` token references, easing
  strings, tabular-numeral metadata.
