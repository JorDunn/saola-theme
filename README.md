# saola-theme

The design system for **Saola**, a Linux desktop environment written in Rust.

This is the one place the Saola look is defined. The panel, greeter, launcher, and
notifications all pull their colors, typography, shapes, and widget styles from here —
so the whole desktop stays a single coherent thing instead of a collection of apps that
happen to ship together.

<!-- screenshot: gallery, Widgets page, ink surface -->

## The look

Saola is built from three colors — and only three:

- **Ink** `#0C0A00` — every shell surface
- **Paper** `#FFFFF0` — a control at rest
- **Terracotta** `#C67139` — on, selected, focused, live

One rule ties them together: an ivory fill is a control at rest, a terracotta fill is
something switched on. There is no dark mode and no light mode — instead, every element
knows which *surface* it sits on (ink or paper) and derives its text, dividers, and
hover states from the identity colors by alpha. Everything is a pill or an over-rounded
rectangle, and keyboard focus is always a terracotta ring.

The canonical definition of all of this lives in `design/saola-tokens.json`.

## See it

```bash
cargo run --example gallery
```

The gallery is a live specimen book: every color token, the IBM Plex type scale, the
radius and size system, and every styled widget. A surface toggle in the sidebar
re-renders each page in the other context — the same theme, on ink or on paper, at
runtime.

## What's inside

- **`saola-tokens`** — the design tokens as pure data (serde + TOML, zero GUI
  dependencies), so the same identity can later be exported to GRUB, Plymouth, or
  terminal configs.
- **`saola-theme`** — style helpers for [iced](https://iced.rs) 0.14. This is what
  Saola apps import.

## Use it

```toml
[dependencies]
saola-theme = { git = "https://github.com/JorDunn/saola-theme", branch = "main" }
```

```rust
use saola_theme::{style, Surface, Theme};
use iced::widget::button;

let theme = Theme::saola();
let wifi = button("Wi-Fi").style(style::button::rest(&theme, Surface::Ink));
```

Every style helper takes the theme and the surface it's rendering on; the API docs on
each module cover the rest.

## Status

v0.1 — the first piece of Saola, built ahead of the desktop itself. Expect the API to
move while the panel and greeter take shape.

## License

[TODO — add your license here]
