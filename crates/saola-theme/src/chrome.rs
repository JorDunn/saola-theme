//! Shared window chrome for ordinary (non-layer-shell) Saola toplevels.
//!
//! Saola runs on niri, which draws **no server-side decorations**, so every
//! ordinary window paints its own chrome: the rounded
//! [`style::container::window`] frame (ivory by default, ink when the user
//! prefers it — style guide §2), the
//! [`saola_tokens::Sizes::window_header`] title bar (title, close pill), and
//! the invisible interactive move/resize regions. This module is the shared
//! home of what its consumers grew independently — saola-files'
//! `src/ui/window.rs` (the first *ordinary* window in the desktop: full
//! frame, drag, double-click maximise, eight resize grips) and
//! saola-capture's `modules/app.rs` `header` (a fixed-size window that needs
//! only the drag-by-header title bar).
//!
//! Per the style guide there is **no minimise button** (niri has no taskbar
//! to minimise into) and no maximise button either — double-clicking the
//! header toggles maximise, matching how niri users expect windows to
//! behave. The close pill is the window's only affordance for going away
//! short of niri's own close keybind, since there is no system close button
//! with decorations off.
//!
//! Division of labor: **this module owns geometry and layout; the consumer
//! owns the runtime calls.** Chrome interactions surface as plain messages
//! (`on_close`, `on_drag`, an `Fn(ResizeEdge) -> M` for the grips) because
//! the corresponding actions need the window's `Id` and a `Task` — runtime
//! concerns a design-system crate has no business holding. saola-files
//! shows the canonical dispatch, one line per interaction:
//! `window::latest().and_then(window::drag)` for the header press,
//! `…and_then(window::toggle_maximize)` for the double-click,
//! `…and_then(window::close)` for the pill, and
//! `window::latest().and_then(move |id| window::drag_resize(id, direction))`
//! for a grip press (with [`ResizeEdge::direction`] supplying the
//! `iced::window::Direction`).
//!
//! The window itself is created with `decorations: false, transparent:
//! true`; the compositor-visible shape is whatever we paint, which is
//! the frame's rounded rectangle. The corners outside that radius stay
//! genuinely transparent only if the *application-level* clear color is
//! transparent too — see [`transparent_clear`] for that oft-rediscovered
//! pairing.

use iced::widget::{button, column, container, mouse_area, row, text, Space, Stack};
use iced::{alignment, mouse, window, Element, Fill, Length};

use crate::convert::{self, ColorExt};
use crate::icon::{self, Icon};
use crate::{style, Surface, Theme};

/// Thickness of the invisible resize strips along each window edge.
///
/// Not a design-system size: this is an interaction hit zone, not a drawn
/// element (the strips paint nothing). 4 px matches typical CSD grab edges.
/// It lives here as a module constant, not a token — it would move to
/// `saola_tokens::Sizes` only if a second consumer ever needs a different
/// value (the same bar `RESIZE_EDGE` cleared when saola-files first
/// authored it).
pub const RESIZE_EDGE: f32 = 4.0;

/// Side length of the invisible corner resize squares. Larger than the edge
/// strips because diagonal grabs are aimed at a point, not a line. A module
/// constant for the same reason as [`RESIZE_EDGE`].
pub const RESIZE_CORNER: f32 = 12.0;

/// Horizontal inset of the header row from the window edge, in logical
/// pixels.
///
/// The two consumers disagreed — saola-files ships `padding([0, 18])`,
/// saola-capture used `sizes.popover_padding` (20). This module picks
/// files' 18: it is the value tuned on the desktop's reference ordinary
/// window, and a window header is not a popover, so borrowing the popover
/// padding token was coincidence, not intent. A module constant, not a
/// token, per the [`RESIZE_EDGE`] rationale.
pub const HEADER_INSET: f32 = 18.0;

// ─── Resize grips ────────────────────────────────────────────────────────

/// The edge or corner of the window a resize grip grabs.
///
/// A chrome-owned enum (rather than `iced::window::Direction` directly) so
/// the module's public API stays message-shaped: the consumer's `update`
/// converts with [`ResizeEdge::direction`] at the moment it builds the
/// `window::drag_resize` task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResizeEdge {
    North,
    South,
    East,
    West,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

impl ResizeEdge {
    /// The four edge strips, in the order they are stacked.
    pub const EDGES: [ResizeEdge; 4] = [
        ResizeEdge::West,
        ResizeEdge::East,
        ResizeEdge::North,
        ResizeEdge::South,
    ];

    /// The four corner squares, in the order they are stacked — always
    /// *after* [`Self::EDGES`], so corners win where the two overlap.
    pub const CORNERS: [ResizeEdge; 4] = [
        ResizeEdge::NorthWest,
        ResizeEdge::NorthEast,
        ResizeEdge::SouthWest,
        ResizeEdge::SouthEast,
    ];

    /// Every grip, edges first, corners after (the stacking order).
    pub const ALL: [ResizeEdge; 8] = [
        ResizeEdge::West,
        ResizeEdge::East,
        ResizeEdge::North,
        ResizeEdge::South,
        ResizeEdge::NorthWest,
        ResizeEdge::NorthEast,
        ResizeEdge::SouthWest,
        ResizeEdge::SouthEast,
    ];

    /// Whether this grip is one of the corner squares.
    pub fn is_corner(self) -> bool {
        matches!(
            self,
            ResizeEdge::NorthWest
                | ResizeEdge::NorthEast
                | ResizeEdge::SouthWest
                | ResizeEdge::SouthEast
        )
    }

    /// The `iced::window::Direction` this grip resizes toward — what the
    /// consumer passes to `window::drag_resize` when the grip's message
    /// arrives.
    pub fn direction(self) -> window::Direction {
        match self {
            ResizeEdge::North => window::Direction::North,
            ResizeEdge::South => window::Direction::South,
            ResizeEdge::East => window::Direction::East,
            ResizeEdge::West => window::Direction::West,
            ResizeEdge::NorthWest => window::Direction::NorthWest,
            ResizeEdge::NorthEast => window::Direction::NorthEast,
            ResizeEdge::SouthWest => window::Direction::SouthWest,
            ResizeEdge::SouthEast => window::Direction::SouthEast,
        }
    }
}

/// The pure geometry of one grip: its hit-zone size, which window edge or
/// corner it pins to, and the resize cursor it shows. Factored out of the
/// widget-building code so the layout facts are unit-testable without a
/// renderer.
struct GripSpec {
    width: Length,
    height: Length,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
    cursor: mouse::Interaction,
}

/// Maps a [`ResizeEdge`] to its [`GripSpec`]: edges are full-length strips
/// [`RESIZE_EDGE`] thick along their side; corners are [`RESIZE_CORNER`]
/// squares pinned into their corner. Cursors follow the resize axis
/// (`ResizingDiagonallyDown` is the NW–SE axis, `ResizingDiagonallyUp` is
/// SW–NE).
fn grip_spec(edge: ResizeEdge) -> GripSpec {
    use alignment::{Horizontal, Vertical};
    use mouse::Interaction::{
        ResizingDiagonallyDown, ResizingDiagonallyUp, ResizingHorizontally, ResizingVertically,
    };

    let strip = Length::Fixed(RESIZE_EDGE);
    let square = Length::Fixed(RESIZE_CORNER);

    match edge {
        ResizeEdge::West => GripSpec {
            width: strip,
            height: Fill,
            align_x: Horizontal::Left,
            align_y: Vertical::Center,
            cursor: ResizingHorizontally,
        },
        ResizeEdge::East => GripSpec {
            width: strip,
            height: Fill,
            align_x: Horizontal::Right,
            align_y: Vertical::Center,
            cursor: ResizingHorizontally,
        },
        ResizeEdge::North => GripSpec {
            width: Fill,
            height: strip,
            align_x: Horizontal::Center,
            align_y: Vertical::Top,
            cursor: ResizingVertically,
        },
        ResizeEdge::South => GripSpec {
            width: Fill,
            height: strip,
            align_x: Horizontal::Center,
            align_y: Vertical::Bottom,
            cursor: ResizingVertically,
        },
        ResizeEdge::NorthWest => GripSpec {
            width: square,
            height: square,
            align_x: Horizontal::Left,
            align_y: Vertical::Top,
            cursor: ResizingDiagonallyDown,
        },
        ResizeEdge::NorthEast => GripSpec {
            width: square,
            height: square,
            align_x: Horizontal::Right,
            align_y: Vertical::Top,
            cursor: ResizingDiagonallyUp,
        },
        ResizeEdge::SouthWest => GripSpec {
            width: square,
            height: square,
            align_x: Horizontal::Left,
            align_y: Vertical::Bottom,
            cursor: ResizingDiagonallyUp,
        },
        ResizeEdge::SouthEast => GripSpec {
            width: square,
            height: square,
            align_x: Horizontal::Right,
            align_y: Vertical::Bottom,
            cursor: ResizingDiagonallyDown,
        },
    }
}

/// One invisible press-catching grip, pinned to its window edge or corner.
fn grip<'a, M: Clone + 'a>(edge: ResizeEdge, message: M) -> Element<'a, M> {
    let spec = grip_spec(edge);
    container(
        mouse_area(Space::new().width(spec.width).height(spec.height))
            .on_press(message)
            .interaction(spec.cursor),
    )
    .width(Fill)
    .height(Fill)
    .align_x(spec.align_x)
    .align_y(spec.align_y)
    .into()
}

/// Stacks the eight invisible resize grips over `base` (normally the
/// [`window_frame`]).
///
/// The grips paint nothing; they exist to catch presses and show the right
/// resize cursor. Corners come after edges in the stack so they win where
/// the two overlap. A fixed-size window (saola-capture's shape,
/// `.resizable(false)`) simply never calls this.
///
/// `on_resize` maps the grabbed [`ResizeEdge`] into the consumer's message;
/// the consumer's `update` then owns the runtime call — see the module doc
/// comment for the canonical `window::drag_resize` dispatch.
pub fn with_resize_grips<'a, M: Clone + 'a>(
    base: Element<'a, M>,
    on_resize: impl Fn(ResizeEdge) -> M,
) -> Element<'a, M> {
    let mut layers = Stack::with_children([base]).width(Fill).height(Fill);
    for edge in ResizeEdge::ALL {
        layers = layers.push(grip(edge, on_resize(edge)));
    }
    layers.into()
}

// ─── Header and frame ────────────────────────────────────────────────────

/// The `sizes.window_header` (46 px) title bar: a drag surface with the
/// window title on the left and the close pill on the right.
///
/// `s` is the surface of the window this bar sits on, and must match the one
/// given to [`window_frame`] — the title, the close glyph, and the pill's
/// hover fill all resolve through `Theme::on(s)`, so passing the wrong one
/// paints ink chrome on an ink window.
///
/// - Pressing anywhere on the bar's background sends `on_drag` (the
///   consumer starts an interactive move — `window::drag`).
/// - Double-clicking sends `on_maximize` when given (`Some` for a resizable
///   window like saola-files — the consumer calls `window::toggle_maximize`
///   — `None` for a fixed-size one like saola-capture, which has nothing to
///   maximise into).
/// - The close pill sends `on_close`.
///
/// **`mouse_area` wrapping a `button`, and why the Close click doesn't
/// *also* start a drag** (saola-capture's press-capture note, kept here
/// verbatim so nobody re-derives it): iced correctly skips the wrapping
/// `mouse_area`'s `on_press` when a child button already captured a press.
/// The close button has `on_press` set, so it captures its own clicks;
/// every other pixel of the header bar falls through to the `mouse_area`
/// and starts the drag. (Corollary from the same gotcha list: a button
/// with *no* `on_press` renders disabled and stops capturing — always wire
/// `on_close`.)
///
/// Where the consumers disagreed, this takes saola-files' shipped choices:
/// title in the surface's `secondary` role (capture used `primary`;
/// secondary reads as chrome, not content), the close pill as an `Icon::X` at
/// `sizes.icon_row` in a [`style::button::bare`] pill (capture's text
/// "Close" label predates the shared icon set), and the [`HEADER_INSET`]
/// horizontal inset (see that constant).
pub fn window_header<'a, M: Clone + 'a>(
    t: &Theme,
    s: Surface,
    title: &'a str,
    on_close: M,
    on_drag: M,
    on_maximize: Option<M>,
) -> Element<'a, M> {
    let close = button(icon::icon(
        Icon::X,
        t.sizes.icon_row,
        t.on(s).primary.into_iced(),
    ))
    .style(style::button::bare(t, s))
    // 6 px above/below the 16 px glyph makes the 28 px pill saola-files
    // shipped; 12 px keeps the pill's hit zone comfortably wider than the
    // glyph.
    .padding([6.0, 12.0])
    .on_press(on_close);

    let header_row = row![
        text(title)
            .font(convert::ui_font(t))
            .size(t.typography.size.body)
            .color(t.on(s).secondary.into_iced()),
        Space::new().width(Fill),
        close,
    ]
    .align_y(iced::Center)
    .width(Fill);

    let bar = mouse_area(
        container(header_row)
            .height(t.sizes.window_header)
            .padding([0.0, HEADER_INSET])
            .align_y(iced::Center),
    )
    .on_press(on_drag);

    match on_maximize {
        Some(message) => bar.on_double_click(message).into(),
        None => bar.into(),
    }
}

/// The window frame: the [`style::container::window`] surface (24 px radius,
/// 2 px border, window shadow) wrapping `header` above `content`, filling
/// the window.
///
/// `s` picks the window's ground — `Surface::Paper` for the ivory window the
/// style guide ships as the default, `Surface::Ink` for the dark one it
/// offers as a user preference. Pass the same `s` to [`window_header`] so
/// the title bar's text and close pill match the frame under them.
///
/// `header` is normally [`window_header`]; it is a parameter rather than
/// built in place so a consumer can extend the bar (extra pills next to
/// close, a search field) without forking the frame. Stack
/// [`with_resize_grips`] over the result for a resizable window.
pub fn window_frame<'a, M: 'a>(
    t: &Theme,
    s: Surface,
    header: Element<'a, M>,
    content: Element<'a, M>,
) -> Element<'a, M> {
    container(column![header, content].width(Fill).height(Fill))
        .style(style::container::window(t, s))
        .width(Fill)
        .height(Fill)
        .into()
}

// ─── Application clear color ─────────────────────────────────────────────

/// The application-level style every decorationless Saola window needs:
/// the default clear color, with `background_color` forced transparent.
/// Pass it from the `.style(...)` hook of `iced::application`:
///
/// ```ignore
/// fn style(&self, theme: &iced::Theme) -> iced::theme::Style {
///     saola_theme::chrome::transparent_clear(theme)
/// }
/// ```
///
/// Why (the discovery every consumer made once, then copied around —
/// saola-panel first, then saola-files and saola-capture verbatim):
/// `decorations: false, transparent: true` on the window is **not enough**.
/// Without a transparent clear color, iced clears the whole surface to
/// [`crate::to_iced_theme`]'s `background` (`palette.ink`) before drawing
/// anything, so the corners outside the frame's 24 px radius render as
/// square ink wedges — and any translucent surface composites against
/// opaque ink instead of true Wayland transparency. saola-capture caught
/// the layer-shell version of this live, by `grim`-sampling a pixel a full
/// second after a capture flash should have faded to nothing and finding a
/// permanent ink rectangle covering the output. Both flags together —
/// `transparent(true)` on the window, this clear color on the application —
/// are required; neither alone suffices.
pub fn transparent_clear(theme: &iced::Theme) -> iced::theme::Style {
    iced::theme::Style {
        background_color: iced::Color::TRANSPARENT,
        ..iced::theme::default(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_lists_every_grip_once_edges_first() {
        assert_eq!(ResizeEdge::ALL.len(), 8);
        let unique: HashSet<ResizeEdge> = ResizeEdge::ALL.into_iter().collect();
        assert_eq!(unique.len(), 8, "ALL repeats a grip");
        // Stacking order: the four edges, then the four corners, so corner
        // squares sit on top where the regions overlap.
        assert_eq!(ResizeEdge::ALL[..4], ResizeEdge::EDGES);
        assert_eq!(ResizeEdge::ALL[4..], ResizeEdge::CORNERS);
        assert!(ResizeEdge::EDGES.iter().all(|e| !e.is_corner()));
        assert!(ResizeEdge::CORNERS.iter().all(|e| e.is_corner()));
    }

    #[test]
    fn direction_mapping_is_one_to_one() {
        // `window::Direction` derives no `PartialEq`, so compare through
        // `Debug` — which also pins each variant to its same-named twin.
        let directions: HashSet<String> = ResizeEdge::ALL
            .into_iter()
            .map(|e| format!("{:?}", e.direction()))
            .collect();
        assert_eq!(directions.len(), 8, "two grips resize the same way");
        for edge in ResizeEdge::ALL {
            assert_eq!(format!("{edge:?}"), format!("{:?}", edge.direction()));
        }
    }

    #[test]
    fn edge_grips_are_full_length_strips() {
        for edge in ResizeEdge::EDGES {
            let spec = grip_spec(edge);
            let dims = [spec.width, spec.height];
            assert!(
                dims.contains(&Length::Fixed(RESIZE_EDGE)),
                "{edge:?} is not RESIZE_EDGE thick"
            );
            assert!(
                dims.contains(&Length::Fill),
                "{edge:?} does not run the full side"
            );
        }
    }

    #[test]
    fn corner_grips_are_squares() {
        for edge in ResizeEdge::CORNERS {
            let spec = grip_spec(edge);
            assert_eq!(spec.width, Length::Fixed(RESIZE_CORNER), "{edge:?}");
            assert_eq!(spec.height, Length::Fixed(RESIZE_CORNER), "{edge:?}");
        }
    }

    #[test]
    fn grips_pin_to_their_own_edge() {
        use alignment::{Horizontal, Vertical};
        for edge in ResizeEdge::ALL {
            let spec = grip_spec(edge);
            let expected = match edge {
                ResizeEdge::West => (Horizontal::Left, Vertical::Center),
                ResizeEdge::East => (Horizontal::Right, Vertical::Center),
                ResizeEdge::North => (Horizontal::Center, Vertical::Top),
                ResizeEdge::South => (Horizontal::Center, Vertical::Bottom),
                ResizeEdge::NorthWest => (Horizontal::Left, Vertical::Top),
                ResizeEdge::NorthEast => (Horizontal::Right, Vertical::Top),
                ResizeEdge::SouthWest => (Horizontal::Left, Vertical::Bottom),
                ResizeEdge::SouthEast => (Horizontal::Right, Vertical::Bottom),
            };
            assert_eq!((spec.align_x, spec.align_y), expected, "{edge:?}");
        }
    }

    #[test]
    fn cursors_follow_the_resize_axis() {
        use mouse::Interaction::{
            ResizingDiagonallyDown, ResizingDiagonallyUp, ResizingHorizontally, ResizingVertically,
        };
        for edge in ResizeEdge::ALL {
            let expected = match edge {
                ResizeEdge::West | ResizeEdge::East => ResizingHorizontally,
                ResizeEdge::North | ResizeEdge::South => ResizingVertically,
                // NW–SE is the "down" diagonal, SW–NE the "up" one.
                ResizeEdge::NorthWest | ResizeEdge::SouthEast => ResizingDiagonallyDown,
                ResizeEdge::NorthEast | ResizeEdge::SouthWest => ResizingDiagonallyUp,
            };
            assert_eq!(grip_spec(edge).cursor, expected, "{edge:?}");
        }
    }

    #[test]
    fn transparent_clear_only_touches_the_background() {
        let theme = crate::to_iced_theme(&Theme::saola());
        let style = transparent_clear(&theme);
        assert_eq!(style.background_color, iced::Color::TRANSPARENT);
        // Everything else stays whatever the theme's default is.
        assert_eq!(style.text_color, iced::theme::default(&theme).text_color);
    }
}
