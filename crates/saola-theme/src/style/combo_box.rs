//! Combo-box style: [`field`] for the text input half, [`menu`] for the
//! dropdown overlay — `.input_style(...)` and `.menu_style(...)` on the
//! widget, respectively.
//!
//! `iced::widget::combo_box::Catalog` (`combo_box.rs`,
//! `iced_widget-0.14.2/src/combo_box.rs`) is declared as
//! `pub trait Catalog: text_input::Catalog + menu::Catalog` with no
//! combo-box-specific style fields of its own — a `ComboBox` is a
//! `TextInput` plus a `menu::Menu` overlay, wired together by the widget's
//! `update`/`overlay` methods, not by anything the `Catalog` adds. Reading
//! the source confirms both halves really are forwarded verbatim:
//! `ComboBox::input_style` sets the wrapped `TextInput`'s own style
//! (`text_input::StyleFn`, `text_input::Status`/`Style` unchanged) and
//! `ComboBox::menu_style` builds the same `menu::StyleFn` the overlay's
//! `menu::Menu::new` consumes in `overlay()` — the exact type
//! [`crate::style::pick_list::menu`] already targets, including the
//! hovered-row highlight (`menu::Style::selected_background` /
//! `border.radius`) flowing through the identical path pick_list's dropdown
//! uses. So this module has nothing to adapt: it composes
//! [`crate::style::text_input::rest`] and
//! [`crate::style::pick_list::menu`] under combo-box-shaped names, matching
//! the plan's "the natural widget for launcher-style search-with-suggestions
//! and settings pickers" — an editable field with no separate visual
//! identity from an ordinary text input.
//!
//! ```no_run
//! use iced::widget::combo_box::{ComboBox, State};
//! use saola_theme::{style, Surface, Theme};
//!
//! let t = Theme::saola();
//! let state: State<String> = State::new(vec!["Wi-Fi".into()]);
//! let _picker: iced::Element<'_, String> =
//!     ComboBox::new(&state, "Search…", None, |s: String| s)
//!         .input_style(style::combo_box::field(&t, Surface::Ink))
//!         .menu_style(style::combo_box::menu(&t, Surface::Ink))
//!         .into();
//! ```

use iced::overlay::menu;
use iced::widget::text_input;
use saola_tokens::{Surface, Theme};

use crate::style::{pick_list, text_input as text_input_style};

/// The combo box's editable field — exactly
/// [`crate::style::text_input::rest`], handed to `ComboBox::input_style`.
/// A combo box's closed/typing look has no state `text_input::rest` doesn't
/// already cover (`Status::Focused` while the menu is open included), so
/// there is nothing combo-box-specific to draw.
pub fn field(
    t: &Theme,
    s: Surface,
) -> impl Fn(&iced::Theme, text_input::Status) -> text_input::Style + Clone {
    text_input_style::rest(t, s)
}

/// The combo box's dropdown menu — exactly [`crate::style::pick_list::menu`],
/// handed to `ComboBox::menu_style`. Both widgets' overlays are built from
/// the same `iced::overlay::menu::Menu`, so the popover-card look (ivory
/// background, `radii.selection`-rounded accent highlight on the hovered
/// row) is identical whether the list was opened by a pick list or filtered
/// by a combo box's typed query.
pub fn menu(t: &Theme, s: Surface) -> impl Fn(&iced::Theme) -> menu::Style + Clone {
    pick_list::menu(t, s)
}
