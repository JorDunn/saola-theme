//! Table style: [`rest`] — the separator hairlines of
//! `iced::widget::table`, for detailed list views (the future saola-files
//! columns view).
//!
//! `iced::widget::table` is a real built-in in iced 0.14 (`table.rs`,
//! `iced_widget-0.14.2/src/table.rs`), and its style surface is much
//! smaller than a "grid of styled cells" mental model suggests.
//! Verified against the source:
//!
//! - `table::Catalog` (line 684) has no `Status`:
//!   `type Class<'a>` / `fn default()` / `fn style(&self, class) -> Style`
//!   — a container-shaped catalog, like `container`/`rule`/`progress_bar`.
//! - `table::Style` holds exactly **two fields**, both `Background`:
//!   `separator_x` and `separator_y`. Despite the doc comments on the
//!   struct calling both "line separator between cells", the draw code is
//!   unambiguous: `separator_x` paints the full-height **vertical** lines
//!   *between columns* (quads of `width: separator_x` thickness), and
//!   `separator_y` paints the full-width **horizontal** lines *between
//!   rows*. Line *thicknesses* are widget builders (`.separator_x(px)` /
//!   `.separator_y(px)`, both defaulting to 1.0; `0.0` suppresses that
//!   axis entirely), not style fields.
//! - Cell content, the header row, hover, and selection are **not**
//!   styleable here — every cell (headers included) is the caller's own
//!   `Element`, so those live in the consumer's cell builders (header
//!   cells in the [`crate::widget::text::label`] role per `section_label`
//!   conventions; row hover/selection would be `button::list_row`-styled
//!   elements *inside* cells, if a consumer wants them). The stage plan's
//!   "fill_subtle hover and accent selection if the Style exposes them"
//!   condition is therefore not met — the Style exposes neither.
//!
//! So a Saola table style is exactly "what color are the hairlines": the
//! surface's `divider` role on both axes, the same role
//! [`crate::style::rule::rest`] reads.
//!
//! ## The iced 0.14.2 consumption gap (verified, load-bearing)
//!
//! `Table` in `iced_widget` 0.14.2 exposes **no `.style(...)` or
//! `.class(...)` builder** — its `class` field is hardwired to
//! `<Theme as Catalog>::default()` in `Table::new` and nothing can replace
//! it. (The `impl From<Style> for StyleFn` next to the catalog is clear
//! evidence a builder was intended upstream; it just isn't in this
//! release.) With `Theme = iced::Theme` the drawn separators therefore
//! always come from `table::default`, i.e.
//! `extended_palette().background.strong.color` — for a Saola app theme
//! ([`crate::convert::to_iced_theme`], background = ink) that is ink
//! OKLCH-lightened by 0.15: inside the identity's neighborhood on ink, but
//! not the `divider` token, and visibly too heavy on a paper window.
//!
//! Until an iced release ships the builder, consumers control the
//! *geometry* (`.separator_x(0.0)` for the row idiom's "no vertical
//! rules", `.separator_y(sizes.hairline)`) and inherit the derived color.
//! [`rest`] is the canonical recipe, ready to hand to `.style(...)` the
//! moment it exists — its closure shape (`impl Fn(&iced::Theme) -> Style`)
//! matches the catalog's `StyleFn` exactly.
//!
//! ```
//! use saola_theme::{style, Surface, Theme};
//!
//! let t = Theme::saola();
//! let table_style = style::table::rest(&t, Surface::Paper);
//! // The closure ignores the iced theme (every Saola helper does — the
//! // tokens were copied in above), so any variant demonstrates it.
//! let _hairlines: iced::widget::table::Style = table_style(&iced::Theme::Light);
//! ```

use iced::widget::table::Style;
use iced::Background;
use saola_tokens::{Surface, Theme};

use crate::convert::ColorExt;

/// A table's separator hairlines at rest: the surface's `divider` role on
/// both axes — vertical column rules (`separator_x`) and horizontal row
/// rules (`separator_y`) alike. Thickness (including turning an axis off
/// with `0.0`) belongs to the widget's `.separator_x(..)`/`.separator_y(..)`
/// builders, not the style; the Saola row idiom is
/// `.separator_x(0.0).separator_y(t.sizes.hairline)`.
///
/// See the module docs for why no widget can consume this closure on
/// iced_widget 0.14.2 yet.
pub fn rest(t: &Theme, s: Surface) -> impl Fn(&iced::Theme) -> Style + Clone {
    let divider = t.on(s).divider.into_iced();

    move |_| Style {
        separator_x: Background::Color(divider),
        separator_y: Background::Color(divider),
    }
}
