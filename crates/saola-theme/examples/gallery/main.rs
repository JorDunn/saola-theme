//! The Saola gallery: a live catalog of every token and style helper.
//!
//! Run with `cargo run -p saola-theme --example gallery`.
//!
//! Stage 6 scope: four pages — Widgets (every style helper, from Stage 5),
//! Colors (every color token as labeled swatches), Typography (the three
//! IBM Plex families + the full size scale), and Spacing (radii and size
//! visualizers) — plus a runtime ink/paper surface toggle in the sidebar
//! that proves the surface axis (the thing that replaces a dark/light
//! theme pair in Saola) actually works at runtime, not just in style code.

// Rust directory examples resolve modules relative to this file exactly
// like a binary crate's `src/main.rs` would, so `mod pages;` here pulls in
// `pages/mod.rs`, which in turn declares the three page submodules.
mod pages;

use iced::widget::{
    button, checkbox, column, container, pick_list, progress_bar, radio, row, rule, scrollable,
    slider, text, text_input, toggler, Space,
};
use iced::{Element, Fill, Size, Task};
use saola_theme::style::container::{DashState, SessionStatus};
use saola_theme::{convert, style, Surface, Theme};

/// The options shown in the Widgets page's pick list demo.
const PICK_LIST_OPTIONS: &[&str] = &["Ink", "Paper", "Terracotta"];

fn main() -> iced::Result {
    iced::application(Gallery::new, Gallery::update, Gallery::view)
        .theme(Gallery::theme)
        .title("Saola Gallery")
        .default_font(convert::ui_font(&Theme::saola()))
        .window_size(Size::new(960.0, 640.0))
        .run()
}

/// The pages of the gallery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Widgets,
    Colors,
    Typography,
    Spacing,
}

#[derive(Debug, Clone)]
enum Message {
    PageSelected(Page),
    /// Buttons need an `on_press` to be enabled; the demo ones do nothing.
    DemoPressed,
    TextInputChanged(String),
    CheckboxToggled(bool),
    TogglerToggled(bool),
    SliderChanged(f32),
    PickListSelected(&'static str),
    /// The Stage 8 kit's own text-input specimens share one value field
    /// (the demo is about the border/ring treatment, not distinct content).
    KitTextInputChanged(String),
    /// `true` = Ascending, `false` = Descending — the radio group's choice.
    RadioSelected(bool),
    /// Index into the segmented control's labels (Files/Folders/All).
    SegmentSelected(usize),
    /// Flips `Gallery::surface` between `Surface::Ink` and `Surface::Paper`.
    /// No payload: there are only two surfaces, so "toggle" always means
    /// "the other one" — nothing the message needs to carry.
    SurfaceToggled,
}

struct Gallery {
    theme: Theme,
    page: Page,
    /// Which surface context the current page renders its content in (or,
    /// for the Colors/Widgets pages that always show both, which one comes
    /// first). This is the runtime axis Saola uses instead of a dark/light
    /// theme. Flipped by `Message::SurfaceToggled` from the sidebar's
    /// toggler.
    surface: Surface,
    text_input_value: String,
    checkbox_checked: bool,
    toggler_toggled: bool,
    slider_value: f32,
    pick_list_selected: Option<&'static str>,
    kit_text_input_value: String,
    radio_selected: bool,
    segment_selected: usize,
}

impl Gallery {
    fn new() -> Self {
        Self {
            theme: Theme::saola(),
            page: Page::Widgets,
            surface: Surface::Ink,
            text_input_value: String::new(),
            checkbox_checked: true,
            toggler_toggled: true,
            slider_value: 40.0,
            pick_list_selected: Some(PICK_LIST_OPTIONS[0]),
            kit_text_input_value: String::new(),
            radio_selected: true,
            segment_selected: 0,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PageSelected(page) => self.page = page,
            Message::DemoPressed => {}
            Message::TextInputChanged(value) => self.text_input_value = value,
            Message::CheckboxToggled(checked) => self.checkbox_checked = checked,
            Message::TogglerToggled(toggled) => self.toggler_toggled = toggled,
            Message::SliderChanged(value) => self.slider_value = value,
            Message::PickListSelected(selected) => self.pick_list_selected = Some(selected),
            Message::KitTextInputChanged(value) => self.kit_text_input_value = value,
            Message::RadioSelected(selected) => self.radio_selected = selected,
            Message::SegmentSelected(index) => self.segment_selected = index,
            Message::SurfaceToggled => {
                self.surface = match self.surface {
                    Surface::Ink => Surface::Paper,
                    Surface::Paper => Surface::Ink,
                };
            }
        }
        Task::none()
    }

    fn theme(&self) -> iced::Theme {
        saola_theme::to_iced_theme(&self.theme)
    }

    fn view(&self) -> Element<'_, Message> {
        let content = match self.page {
            Page::Widgets => self.widgets_page(),
            Page::Colors => pages::colors::view(&self.theme, self.surface),
            Page::Typography => pages::typography::view(&self.theme, self.surface),
            Page::Spacing => pages::spacing::view(&self.theme, self.surface),
        };

        // The app's own outermost frame is always ink — per Architecture,
        // "ink: every shell surface" — so this container never reads
        // `self.surface`. What *does* read it is each page's own content
        // above (and the sidebar's nav-button styling below): the surface
        // toggle changes what's drawn *inside* the shell, not the shell.
        container(row![self.sidebar(), content].spacing(24))
            .style(style::container::ink_surface(&self.theme))
            .padding(24)
            .width(Fill)
            .height(Fill)
            .into()
    }

    fn sidebar(&self) -> Element<'_, Message> {
        let t = &self.theme;
        let nav = |label, page: Page| {
            let btn = button(text(label).size(t.typography.size.body));
            if self.page == page {
                // The current page is "selected" — terracotta.
                btn.style(style::button::active(t, Surface::Ink))
                    .on_press(Message::PageSelected(page))
            } else {
                btn.style(style::button::bare(t, Surface::Ink))
                    .on_press(Message::PageSelected(page))
            }
        };

        let surface_caption = match self.surface {
            Surface::Ink => "On ink",
            Surface::Paper => "On paper",
        };

        column![
            text("Saola")
                .font(convert::display_font(t))
                .size(t.typography.size.panel_heading),
            nav("Widgets", Page::Widgets),
            nav("Colors", Page::Colors),
            nav("Typography", Page::Typography),
            nav("Spacing", Page::Spacing),
            rule::horizontal(1).style(style::rule::rest(t, Surface::Ink)),
            // The surface toggle: a labeled toggler that just flips
            // `self.surface`. It lives on the sidebar, which is always
            // drawn on the ink shell, so it's always styled `Surface::Ink`
            // regardless of which surface it's currently *pointing at*.
            text("Surface")
                .size(t.typography.size.label)
                .color(convert::ColorExt::into_iced(t.on_ink.tertiary)),
            row![
                text("Ink").size(t.typography.size.label),
                toggler(self.surface == Surface::Paper)
                    .style(style::toggles::toggler(t, Surface::Ink))
                    .on_toggle(|_is_paper| Message::SurfaceToggled),
                text("Paper").size(t.typography.size.label),
            ]
            .spacing(8)
            .align_y(iced::Center),
            text(surface_caption)
                .size(t.typography.size.label)
                .color(convert::ColorExt::into_iced(t.on_ink.secondary)),
        ]
        .spacing(10)
        .width(180)
        .into()
    }

    /// `(self.surface, the other one)` — used to order the paired ink/paper
    /// sections on the Widgets page so the surface currently selected in
    /// the sidebar toggle is always the one shown first. Flipping the
    /// toggle visibly reorders this page, the same way it reorders the
    /// Colors page's role-step sections.
    fn ordered_surfaces(&self) -> (Surface, Surface) {
        match self.surface {
            Surface::Ink => (Surface::Ink, Surface::Paper),
            Surface::Paper => (Surface::Paper, Surface::Ink),
        }
    }

    fn widgets_page(&self) -> Element<'_, Message> {
        let t = &self.theme;
        let (primary, secondary) = self.ordered_surfaces();

        scrollable(
            column![
                text("Buttons").size(t.typography.size.section_heading),
                self.labeled_surface_row(primary, self.button_row(primary)),
                self.labeled_surface_row(secondary, self.button_row(secondary)),
                text("Panel").size(t.typography.size.section_heading),
                self.labeled_surface_row(Surface::Ink, self.panel_column()),
                text("Controls").size(t.typography.size.section_heading),
                self.labeled_surface_row(primary, self.controls_column(primary)),
                self.labeled_surface_row(secondary, self.controls_column(secondary)),
                text("Kit").size(t.typography.size.section_heading),
                self.labeled_surface_row(primary, self.kit_column(primary)),
                self.labeled_surface_row(secondary, self.kit_column(secondary)),
            ]
            .spacing(16)
            .width(Fill),
        )
        .style(style::scrollable::rest(t, Surface::Ink))
        .into()
    }

    /// The panel treatments: the columns minimap (stubs at each end, rests,
    /// one focused dash) inside an island pill, and the popover surface.
    /// Both are shell chrome, so this section only exists on ink.
    fn panel_column(&self) -> Element<'_, Message> {
        let t = &self.theme;
        let dash = |state: DashState, width: f32| {
            container(Space::new())
                .style(style::container::dash(t, state))
                .width(width)
                .height(t.sizes.dash_height)
        };
        let minimap = container(
            row![
                dash(DashState::Stub, t.sizes.dash_width_stub),
                dash(DashState::Rest, t.sizes.dash_width_rest),
                dash(DashState::Focused, t.sizes.dash_width_focused),
                dash(DashState::Rest, t.sizes.dash_width_rest),
                dash(DashState::Rest, t.sizes.dash_width_rest),
                dash(DashState::Stub, t.sizes.dash_width_stub),
            ]
            .spacing(t.sizes.dash_gap)
            .align_y(iced::Center),
        )
        .style(style::container::translucent_panel(t))
        .height(t.sizes.panel_pill)
        .padding([0, 16])
        .align_y(iced::Center);

        let popover = container(
            column![
                text("Quick settings")
                    .font(convert::display_font(t))
                    .size(t.typography.size.section_heading),
                text("Opaque ink · popover radius · popover shadow")
                    .size(t.typography.size.secondary)
                    .color(convert::ColorExt::into_iced(t.on_ink.secondary)),
            ]
            .spacing(8),
        )
        .style(style::container::popover(t))
        .padding(22)
        .width(320);

        // The floating ledger bar: bar_pill chrome at panel_bar height, with
        // the compact inner pills at their own heights (media 30, clock 32
        // inside the 48 bar) and the bar/cluster gap tokens.
        let inner_pill = |label: &'static str, height: f32| {
            container(
                text(label)
                    .font(convert::ui_font(t))
                    .size(t.typography.size.bar),
            )
            .style(style::container::translucent_panel(t))
            .height(height)
            .padding([0, 14])
            .align_y(iced::Center)
        };
        let ledger = container(
            row![
                text("alacritty")
                    .font(convert::ui_font(t))
                    .size(t.typography.size.bar),
                text("~/dev/saola — cargo run")
                    .font(convert::ui_font_regular(t))
                    .size(t.typography.size.bar)
                    .color(convert::ColorExt::into_iced(t.on_ink.tertiary)),
                Space::new().width(Fill),
                inner_pill("Nala Sinephro — Space 1.8", t.sizes.panel_pill_media),
                inner_pill("Fri 24 Jul · 09:41", t.sizes.panel_pill_clock),
            ]
            .spacing(t.sizes.bar_element_gap)
            .align_y(iced::Center),
        )
        .style(style::container::bar_pill(t))
        .height(t.sizes.panel_bar)
        .padding([0, 16])
        .align_y(iced::Center);

        column![minimap, self.session_status_column(), ledger, popover]
            .spacing(16)
            .into()
    }

    /// The session-status semaphore: the five Claude Code session states as
    /// dots, plus the breathing range the two "still running" states sweep.
    ///
    /// This is the design system's one documented exception to "three
    /// colors, never a fourth" (see `style::container::status_dot`), so the
    /// specimen states it in words as well as showing it. Ink-only, like
    /// the rest of the panel section: these dots only ever sit on the bar.
    ///
    /// The gallery is a static catalog — it doesn't run an animation clock
    /// — so the breath is shown as a filmstrip: the same dot at three
    /// points of its cycle, which is also the clearest way to check the dim
    /// end is still legible on ink.
    fn session_status_column(&self) -> Element<'_, Message> {
        let t = &self.theme;
        let caption_color = convert::ColorExt::into_iced(t.on_ink.tertiary);

        let dot = |status: SessionStatus, breath: f32| {
            container(Space::new())
                .style(style::container::status_dot(t, status, breath))
                .width(t.sizes.dash_height)
                .height(t.sizes.dash_height)
        };
        // A dot over its label. `move` isn't needed: `dot` and the colors
        // are borrowed for as long as this function's returned Element
        // lives, which is the lifetime of the `&self` borrow anyway.
        let labeled = |status: SessionStatus, breath: f32, label: &'static str| {
            column![
                dot(status, breath),
                text(label)
                    .size(t.typography.size.label)
                    .color(caption_color),
            ]
            .spacing(6)
            .align_x(iced::Center)
        };

        let states = row![
            labeled(SessionStatus::Working, 1.0, "working"),
            labeled(SessionStatus::Subagents, 1.0, "subagents"),
            labeled(SessionStatus::Attention, 1.0, "attention"),
            labeled(SessionStatus::Done, 1.0, "done"),
            labeled(SessionStatus::Idle, 1.0, "idle"),
        ]
        .spacing(22)
        .align_y(iced::Center);

        // Dim end, midpoint, full — the three-frame filmstrip of one breath.
        let floor = t.motion.breathe_min_opacity;
        let mid = floor + (1.0 - floor) / 2.0;
        let filmstrip = |status: SessionStatus| {
            row![dot(status, floor), dot(status, mid), dot(status, 1.0)]
                .spacing(8)
                .align_y(iced::Center)
        };

        column![
            text("Session status — the documented exception to the three-color rule")
                .size(t.typography.size.secondary)
                .color(caption_color),
            states,
            text(format!(
                "Breathing (working · subagents): {} ms per cycle, opacity {floor:.2} → 1.00",
                t.motion.breathe
            ))
            .size(t.typography.size.label)
            .color(caption_color),
            row![
                filmstrip(SessionStatus::Working),
                filmstrip(SessionStatus::Subagents),
            ]
            .spacing(24)
            .align_y(iced::Center),
        ]
        .spacing(10)
        .into()
    }

    /// A "On ink" / "On paper" caption above `content`, wrapping `content`
    /// in a `paper_window` card when `surface` is `Paper` (matching how the
    /// rest of the app only ever shows paper as a window floating on the
    /// ink shell, never as the shell itself).
    fn labeled_surface_row<'a>(
        &'a self,
        surface: Surface,
        content: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let t = &self.theme;
        let caption = match surface {
            Surface::Ink => "On ink",
            Surface::Paper => "On paper",
        };
        let label = text(caption)
            .size(t.typography.size.secondary)
            .color(convert::ColorExt::into_iced(t.on_ink.secondary));
        match surface {
            Surface::Ink => column![label, content].spacing(10).into(),
            Surface::Paper => column![
                label,
                container(content)
                    .style(style::container::paper_window(t))
                    .padding(24)
                    .width(Fill),
            ]
            .spacing(10)
            .into(),
        }
    }

    /// One row of every button helper in the given surface context.
    fn button_row(&self, s: Surface) -> Element<'_, Message> {
        let t = &self.theme;
        let size = t.typography.size.body;
        let pad = [10, 18];

        row![
            button(text("Rest").size(size))
                .style(style::button::rest(t, s))
                .padding(pad)
                .on_press(Message::DemoPressed),
            button(text("Active").size(size))
                .style(style::button::active(t, s))
                .padding(pad)
                .on_press(Message::DemoPressed),
            button(text("Muted").size(size))
                .style(style::button::muted(t, s))
                .padding(pad)
                .on_press(Message::DemoPressed),
            button(text("Bare").size(size))
                .style(style::button::bare(t, s))
                .padding(pad)
                .on_press(Message::DemoPressed),
            // No `on_press` ⇒ iced reports `Status::Disabled`.
            button(text("Disabled").size(size))
                .style(style::button::rest(t, s))
                .padding(pad),
        ]
        .spacing(12)
        .into()
    }

    /// One column of every Stage 5 style helper (text input, checkbox,
    /// toggler, slider, progress bar, pick list, rule, scrollable) in the
    /// given surface context.
    fn controls_column(&self, s: Surface) -> Element<'_, Message> {
        let t = &self.theme;

        column![
            text_input("Type something…", &self.text_input_value)
                .style(style::text_input::rest(t, s))
                .padding([10, 14])
                .on_input(Message::TextInputChanged),
            row![
                checkbox(self.checkbox_checked)
                    .label("Checkbox")
                    .style(style::toggles::checkbox(t, s))
                    .on_toggle(Message::CheckboxToggled),
                toggler(self.toggler_toggled)
                    .label("Toggler")
                    .style(style::toggles::toggler(t, s))
                    .on_toggle(Message::TogglerToggled),
            ]
            .spacing(24),
            slider(0.0..=100.0, self.slider_value, Message::SliderChanged)
                .style(style::slider::rest(t, s)),
            progress_bar(0.0..=100.0, self.slider_value).style(style::progress::bar(t, s)),
            pick_list(
                PICK_LIST_OPTIONS,
                self.pick_list_selected,
                Message::PickListSelected
            )
            .style(style::pick_list::field(t, s))
            .menu_style(style::pick_list::menu(t, s)),
            rule::horizontal(1).style(style::rule::rest(t, s)),
            container(
                scrollable(
                    column![
                        text("Scrollable content"),
                        text("Row 2"),
                        text("Row 3"),
                        text("Row 4"),
                        text("Row 5"),
                    ]
                    .spacing(6)
                )
                .style(style::scrollable::rest(t, s))
                .height(80)
            )
            .width(Fill),
        ]
        .spacing(16)
        .width(Fill)
        .into()
    }

    /// The Stage 8 kit: text-input `rest`/focused/`rejected` side by side,
    /// a radio group, a segmented control, the urgent card, and the
    /// keycap/badge chips — in the given surface context.
    fn kit_column(&self, s: Surface) -> Element<'_, Message> {
        let t = &self.theme;

        // Rest / focused / rejected: three fields sharing one style pattern
        // apart from the border. Click into the middle one to see the
        // ordinary 2px accent focus ring `rest` draws only on
        // `Status::Focused`; the third field's tinted ring is always on,
        // which is the whole point of `rejected` — legible as a distinct
        // third state even before it's touched.
        let text_input_states = row![
            text_input("Rest", &self.kit_text_input_value)
                .style(style::text_input::rest(t, s))
                .padding([10, 14])
                .on_input(Message::KitTextInputChanged),
            text_input("Click to focus", &self.kit_text_input_value)
                .style(style::text_input::rest(t, s))
                .padding([10, 14])
                .on_input(Message::KitTextInputChanged),
            text_input("Rejected", &self.kit_text_input_value)
                .style(style::text_input::rejected(t, s))
                .padding([10, 14])
                .on_input(Message::KitTextInputChanged),
        ]
        .spacing(12);

        // Radio group: 9d's Ascending/Descending rows.
        let radios = row![
            radio(
                "Ascending",
                true,
                Some(self.radio_selected),
                Message::RadioSelected
            )
            .style(style::radio::radio(t, s)),
            radio(
                "Descending",
                false,
                Some(self.radio_selected),
                Message::RadioSelected
            )
            .style(style::radio::radio(t, s)),
        ]
        .spacing(24);

        // Segmented control: 9d's Files | Folders | All, built as a row of
        // buttons inside a `track` container, not a custom widget.
        const SEGMENTS: [&str; 3] = ["Files", "Folders", "All"];
        let segments: Vec<Element<'_, Message>> = SEGMENTS
            .iter()
            .enumerate()
            .map(|(index, label)| {
                button(text(*label).size(t.typography.size.body))
                    .style(style::segmented::segment(
                        t,
                        s,
                        index == self.segment_selected,
                    ))
                    .padding([8, 16])
                    .on_press(Message::SegmentSelected(index))
                    .into()
            })
            .collect();
        let segmented = container(row(segments).spacing(4))
            .style(style::segmented::track(t, s))
            .padding(4);

        // The urgent notification card (10b): `card` plus a 2px accent
        // ring, no other change — "no life rule".
        let urgent_card = container(
            column![
                text("Battery critical")
                    .font(convert::display_font(t))
                    .size(t.typography.size.section_heading),
                text("6% remaining — plug in now")
                    .size(t.typography.size.secondary)
                    .color(convert::ColorExt::into_iced(t.on_paper.secondary)),
            ]
            .spacing(6),
        )
        .style(style::container::card_urgent(t, s))
        .padding(18)
        .width(300);

        // Keycap and badge chips.
        let keycap = |label: &'static str| {
            container(
                text(label)
                    .font(convert::mono_font_medium(t))
                    .size(t.typography.size.keycap),
            )
            .style(style::container::keycap(t, s))
            .padding([2, 8])
        };
        let badge = |count: &'static str| {
            container(text(count).size(t.typography.size.meta))
                .style(style::container::badge(t))
                .padding([2, 8])
        };
        let chips = row![
            keycap("↵"),
            keycap("⇥"),
            text("  "),
            badge("3"),
            badge("12"),
        ]
        .spacing(8)
        .align_y(iced::Center);

        column![text_input_states, radios, segmented, urgent_card, chips,]
            .spacing(16)
            .width(Fill)
            .into()
    }
}
