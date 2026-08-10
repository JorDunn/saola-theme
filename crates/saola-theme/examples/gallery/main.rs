//! The Saola gallery: a live catalog of every token and style helper.
//!
//! Run with `cargo run -p saola-theme --example gallery`.
//!
//! Four pages — Widgets (every style helper and bundled composite, from
//! the Stage 5 controls through the upstreamed consumer kit: icons, text
//! roles, scrims, marquee, avatar, selection chrome), Colors (every color
//! token as labeled swatches), Typography (the three IBM Plex families +
//! the full size scale), and Spacing (radii and size visualizers) — plus a
//! runtime ink/paper surface toggle in the sidebar that proves the surface
//! axis (the thing that replaces a dark/light theme pair in Saola)
//! actually works at runtime, not just in style code.

// Rust directory examples resolve modules relative to this file exactly
// like a binary crate's `src/main.rs` would, so `mod pages;` here pulls in
// `pages/mod.rs`, which in turn declares the three page submodules.
mod pages;

use std::time::Duration;

use iced::widget::{
    button, canvas, checkbox, column, container, pick_list, progress_bar, radio, row, scrollable,
    slider, text, text_input, toggler, Space,
};
use iced::{Element, Fill, Point, Rectangle, Size, Task};
use saola_theme::canvas::SelectionChrome;
use saola_theme::marquee::marquee;
use saola_theme::style::container::{DashState, ScrimKind, SessionStatus};
use saola_theme::widget::Emphasis;
use saola_theme::{avatar, convert, icon, style, widget, Icon, Surface, Theme};

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
            widget::hairline(t, Surface::Ink),
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
                text("Rows, tiles & menus").size(t.typography.size.section_heading),
                self.labeled_surface_row(primary, self.rows_column(primary)),
                self.labeled_surface_row(secondary, self.rows_column(secondary)),
                text("Composites").size(t.typography.size.section_heading),
                self.labeled_surface_row(primary, self.composites_column(primary)),
                self.labeled_surface_row(secondary, self.composites_column(secondary)),
                text("Text roles").size(t.typography.size.section_heading),
                self.labeled_surface_row(primary, self.text_roles_column(primary)),
                self.labeled_surface_row(secondary, self.text_roles_column(secondary)),
                text("Status marks").size(t.typography.size.section_heading),
                self.labeled_surface_row(primary, self.status_marks_row(primary)),
                self.labeled_surface_row(secondary, self.status_marks_row(secondary)),
                text("Icons").size(t.typography.size.section_heading),
                self.labeled_surface_row(primary, self.icons_column(primary)),
                self.labeled_surface_row(secondary, self.icons_column(secondary)),
                // The remaining sections are shell chrome — scrims dim the
                // wallpaper, the marquee and avatar are lock/panel pieces,
                // and the selection chrome dims a capture surface — so like
                // the Panel section they exist only on ink.
                text("Scrims").size(t.typography.size.section_heading),
                self.labeled_surface_row(Surface::Ink, self.scrims_column()),
                text("Marquee").size(t.typography.size.section_heading),
                self.labeled_surface_row(Surface::Ink, self.marquee_column()),
                text("Avatar").size(t.typography.size.section_heading),
                self.labeled_surface_row(Surface::Ink, self.avatar_row()),
                text("Selection chrome").size(t.typography.size.section_heading),
                self.labeled_surface_row(Surface::Ink, self.selection_chrome_pane()),
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
        // the compact inner pills at their shared `panel_pill_clock` height
        // (32 inside the 48 bar) and the bar/cluster gap tokens.
        let inner_pill = |label: &'static str| {
            container(
                text(label)
                    .font(convert::ui_font(t))
                    .size(t.typography.size.bar),
            )
            .style(style::container::translucent_panel(t))
            .height(t.sizes.panel_pill_clock)
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
                inner_pill("Nala Sinephro — Space 1.8"),
                inner_pill("Fri 24 Jul · 09:41"),
            ]
            .spacing(t.sizes.bar_element_gap)
            .align_y(iced::Center),
        )
        .style(style::container::bar_pill(t))
        .height(t.sizes.panel_bar)
        .padding([0, 16])
        .align_y(iced::Center);

        // The islands trigger shape: `widget::hover_pill` owns the
        // paint-order workaround (the hover-carrying bare button lives
        // *inside* the ink pill, so its fill lands above the ink) — hover
        // this pill to see the whole thing tint.
        let island = widget::hover_pill(
            t,
            t.sizes.panel_pill,
            text("Islands trigger")
                .font(convert::ui_font(t))
                .size(t.typography.size.bar),
            Message::DemoPressed,
        );

        column![
            minimap,
            self.session_status_column(),
            ledger,
            island,
            popover
        ]
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

        let helpers = row![
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
        .spacing(12);

        // `emphasis`: `rest` or `active` behind one closure type, picked by
        // a `bool` — the helper consumers use for a button that flips
        // between the two states without duplicating the builder chain.
        let emphasis_row = row![
            button(text("Emphasis off").size(size))
                .style(style::button::emphasis(t, s, false))
                .padding(pad)
                .on_press(Message::DemoPressed),
            button(text("Emphasis on").size(size))
                .style(style::button::emphasis(t, s, true))
                .padding(pad)
                .on_press(Message::DemoPressed),
        ]
        .spacing(12);

        column![helpers, emphasis_row].spacing(12).into()
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
            widget::hairline(t, s),
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

        // The quiet prompt pair: `rest`/`rejected`'s exact states over a
        // translucent `fill_subtle` recess instead of the solid control
        // fill — the lock screen's password field, and any lone field on a
        // scrim. `prompt_rejected` keeps its tinted ring in every
        // interactive state, exactly like `rejected` above.
        let prompt_states = row![
            text_input("Prompt", &self.kit_text_input_value)
                .style(style::text_input::prompt(t, s))
                .padding([10, 14])
                .on_input(Message::KitTextInputChanged),
            text_input("Prompt rejected", &self.kit_text_input_value)
                .style(style::text_input::prompt_rejected(t, s))
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
        let segmented = container(row(segments).spacing(t.sizes.segment_inset))
            .style(style::segmented::track(t, s))
            .padding(t.sizes.segment_inset);

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

        // The §6 notification card: opaque ink with every color alpha-scaled
        // by its `alpha` parameter, which exists for toast fades
        // (`motion::toast_alpha` drives it) — a static catalog shows it at
        // 1.0. Its content colors are named explicitly (`on_ink`, not
        // `on(s)`): the card is ink on both surfaces.
        let toast_card = container(
            column![
                text("Screenshot saved")
                    .font(convert::display_font(t))
                    .size(t.typography.size.section_heading),
                text("~/Pictures/capture-0142.png")
                    .size(t.typography.size.secondary)
                    .color(convert::ColorExt::into_iced(t.on_ink.secondary)),
            ]
            .spacing(6),
        )
        .style(style::container::notification_card(t, 1.0))
        .padding(18)
        .width(300);

        // Keycap and badge chips, plus the two static shapes: `chip` (a
        // resting control's look with no hover — the ledger clock pill) and
        // `disc` (`tile`'s recipe closed into a circle by `radii.pill`).
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
        let clock_chip = container(
            text("Fri 24 Jul · 09:41")
                .font(convert::ui_font(t))
                .size(t.typography.size.bar),
        )
        .style(style::container::chip(t, s))
        .padding([4, 12]);
        let disc = container(
            text("JD")
                .font(convert::ui_font(t))
                .size(t.typography.size.keycap),
        )
        .center_x(t.sizes.hit_target_bar)
        .center_y(t.sizes.hit_target_bar)
        .style(style::container::disc(t, s));
        let chips = row![
            keycap("↵"),
            keycap("⇥"),
            text("  "),
            badge("3"),
            badge("12"),
            text("  "),
            clock_chip,
            disc,
        ]
        .spacing(8)
        .align_y(iced::Center);

        column![
            text_input_states,
            prompt_states,
            radios,
            segmented,
            urgent_card,
            toast_card,
            chips,
        ]
        .spacing(16)
        .width(Fill)
        .into()
    }

    /// The file-manager kit: `button::list_row` rest/selected/focused side
    /// by side, the same rows composed inside a `container::inset` panel
    /// (the sidebar/toolbar shape — `tile`'s recipe at `radii.inset`), the
    /// grid-view `selection_tile` twins, the `menu_row` treatment (styled
    /// and bundled), and the `widget::hairline`/`vertical_hairline`
    /// constructors — in the given surface context.
    fn rows_column(&self, s: Surface) -> Element<'_, Message> {
        let t = &self.theme;
        let size = t.typography.size.body;

        let list_row = |label: &'static str, selected: bool, focused: bool| {
            button(text(label).size(size))
                .style(style::button::list_row(t, s, selected, focused))
                .padding([8, 14])
                .on_press(Message::DemoPressed)
        };

        // Rest, selected, and focused side by side, at their natural width.
        // `focused` is the keyboard cursor: iced buttons have no
        // `Status::Focused`, so the consumer tracks it and the row draws the
        // 2 px accent ring in place of its transparent border.
        let rows = row![
            list_row("Rest row", false, false),
            list_row("Selected row", true, false),
            list_row("Focused row", false, true),
        ]
        .spacing(12);

        // The inset panel, filled the way a sidebar fills it: full-width
        // list rows on the recessed `fill_subtle` ground.
        let inset_panel = container(
            column![
                list_row("Home", false, false).width(Fill),
                list_row("Documents", true, false).width(Fill),
                list_row("Trash", false, false).width(Fill),
            ]
            .spacing(t.sizes.gap_tight),
        )
        .style(style::container::inset(t, s))
        .padding(10)
        .width(240);

        // `selection_tile`: `list_row`'s exact recipe at `radii.tile` — the
        // grid-view analogue of a list row, at the `grid_tile` geometry.
        let tile = |label: &'static str, selected: bool, focused: bool| {
            button(
                container(text(label).size(t.typography.size.secondary))
                    .align_x(iced::Center)
                    .align_y(iced::Center)
                    .width(Fill)
                    .height(Fill),
            )
            .style(style::button::selection_tile(t, s, selected, focused))
            .width(t.sizes.grid_tile)
            .height(t.sizes.grid_tile)
            .on_press(Message::DemoPressed)
        };
        let tiles = row![
            tile("Rest", false, false),
            tile("Selected", true, false),
            tile("Focused", false, true),
        ]
        .spacing(t.sizes.grid_tile_gap);

        // `menu_row`, both ways: the raw style helper (quiet at rest, full
        // terracotta on hover — hover is the selection preview) and the
        // bundled `widget::menu_row`, whose leading glyph takes its tint
        // from `widget::role` because an `Svg`'s tint can't ride the button
        // status. `menu_width` is the menu-scale width token.
        let styled_menu_row = |label: &'static str, enabled: bool| {
            let row = button(text(label).size(size))
                .style(style::button::menu_row(t, s, enabled))
                .width(Fill)
                .padding(t.paddings.strip);
            if enabled {
                row.on_press(Message::DemoPressed)
            } else {
                row
            }
        };
        let menu = container(
            column![
                styled_menu_row("Styled row — hover me", true),
                styled_menu_row("Styled row — disabled", false),
                widget::menu_row(
                    t,
                    s,
                    Some(Icon::FolderOpen),
                    "Open",
                    widget::role(t, s, Emphasis::Rest),
                    Some(Message::DemoPressed),
                ),
                widget::menu_row(
                    t,
                    s,
                    Some(Icon::Pencil),
                    "Rename",
                    widget::role(t, s, Emphasis::Rest),
                    Some(Message::DemoPressed),
                ),
                widget::menu_row(
                    t,
                    s,
                    Some(Icon::Trash2),
                    "Unavailable",
                    widget::role(t, s, Emphasis::Disabled),
                    None,
                ),
            ]
            .spacing(2),
        )
        .width(t.sizes.menu_width);

        // The vertical hairline needs a bounded height to fill (a shrink
        // row would hand it no height at all), so the demo row is fixed.
        let vertical_demo = row![
            text("Name").size(size),
            widget::vertical_hairline(t, s),
            text("Size").size(size),
        ]
        .spacing(12)
        .height(24)
        .align_y(iced::Center);

        column![
            rows,
            inset_panel,
            tiles,
            menu,
            widget::hairline(t, s),
            vertical_demo
        ]
        .spacing(16)
        .width(Fill)
        .into()
    }

    /// The bundled composite constructors from `widget`: the pill/icon
    /// buttons (tokens for height, padding, and centering pre-applied), the
    /// generic `segmented_row` (sharing the Kit section's selection state —
    /// same control, one constructor call), and the small envelopes
    /// (`section_label`, `quiet_row`, `separator`, `empty_state`,
    /// `footer_strip`) — in the given surface context.
    fn composites_column(&self, s: Surface) -> Element<'_, Message> {
        let t = &self.theme;

        // `pill_button` = `button::emphasis` + `hit_target_bar` height +
        // `paddings.pill_button` + the centering sandwich; `icon_button` is
        // its icon-bearing sibling, whose tint is the caller's job (an
        // `Svg`'s color can't follow the button's status), so the disabled
        // one passes the disabled role by hand.
        let buttons = row![
            widget::pill_button(t, s, "Save", Some(Message::DemoPressed), true),
            widget::pill_button(t, s, "Cancel", Some(Message::DemoPressed), false),
            widget::pill_button(t, s, "Disabled", None, false),
            widget::icon_button(
                t,
                s,
                Icon::Check,
                Some("Apply"),
                widget::role(t, s, Emphasis::Rest),
                Some(Message::DemoPressed),
            ),
            widget::icon_button(
                t,
                s,
                Icon::X,
                None,
                widget::role(t, s, Emphasis::Rest),
                Some(Message::DemoPressed),
            ),
            widget::icon_button(
                t,
                s,
                Icon::RefreshCw,
                None,
                widget::role(t, s, Emphasis::Disabled),
                None,
            ),
        ]
        .spacing(12)
        .align_y(iced::Center);

        // The generic `segmented_row`, driving the same state as the Kit
        // section's hand-assembled control — flipping one flips the other,
        // which is the point: same tokens, one constructor call, and the
        // assembled track totals exactly `hit_target_bar`.
        let segmented = widget::segmented_row(
            t,
            s,
            &[(0usize, "Files"), (1, "Folders"), (2, "All")],
            &self.segment_selected,
            Message::SegmentSelected,
        );

        // `empty_state` centers in all the space it's given, so the specimen
        // hands it a bounded band.
        let empty = container(widget::empty_state(t, s, "This folder is empty"))
            .width(Fill)
            .height(90);

        // `footer_strip`: the fixed-height card band a window docks its
        // transient chrome into (a progress readout, an undo toast).
        let footer = widget::footer_strip(
            t,
            s,
            row![
                widget::text::secondary(t, s, "Moving 3 items…"),
                Space::new().width(Fill),
                widget::pill_button(t, s, "Undo", Some(Message::DemoPressed), false),
            ]
            .spacing(12)
            .align_y(iced::Center),
        );

        column![
            buttons,
            segmented,
            widget::section_label(t, s, "PLACES"),
            widget::quiet_row(t, s, "Media — no player"),
            widget::separator(t, s),
            empty,
            footer,
        ]
        .spacing(16)
        .width(Fill)
        .into()
    }

    /// The five text roles from `widget::text` — pre-sized, pre-fonted,
    /// pre-colored constructors, one per named role — in the given surface
    /// context.
    fn text_roles_column(&self, s: Surface) -> Element<'_, Message> {
        let t = &self.theme;
        column![
            widget::text::body(t, s, "Body — primary emphasis, regular UI face"),
            widget::text::secondary(t, s, "Secondary — the detail line under a title"),
            widget::text::label(t, s, "LABEL — MONO-MEDIUM, TERTIARY"),
            widget::text::hint(t, s, "Hint — quaternary, quiet enough to ignore"),
            widget::text::error(
                t,
                s,
                "Error — accent text; severity is carried by the wording"
            ),
        ]
        .spacing(8)
        .into()
    }

    /// The `widget::Emphasis` ladder as labeled dots: the four-way status
    /// mark role (`Live`/`Rest`/`Quiet`/`Disabled`) that picks the tint an
    /// icon constructor takes. The dot itself is `widget::dot`, handed an
    /// inline style painting `widget::role`'s color — the ladder is about
    /// the *colors*, so the specimen paints them directly.
    fn status_marks_row(&self, s: Surface) -> Element<'_, Message> {
        let t = &self.theme;
        let caption = convert::ColorExt::into_iced(t.on(s).tertiary);
        let radius = t.radii.pill;

        let labeled = |emphasis: Emphasis, label: &'static str| {
            let color = widget::role(t, s, emphasis);
            column![
                widget::dot(t.sizes.dash_height, move |_: &iced::Theme| {
                    iced::widget::container::Style {
                        background: Some(iced::Background::Color(color)),
                        border: style::border_none(radius),
                        ..Default::default()
                    }
                }),
                text(label).size(t.typography.size.label).color(caption),
            ]
            .spacing(6)
            .align_x(iced::Center)
        };

        row![
            labeled(Emphasis::Live, "live"),
            labeled(Emphasis::Rest, "rest"),
            labeled(Emphasis::Quiet, "quiet"),
            labeled(Emphasis::Disabled, "disabled"),
        ]
        .spacing(22)
        .into()
    }

    /// The shared icon module: a sampling of `Icon` variants at the three
    /// in-context icon sizes, then the leveled glyph ladders — the pure
    /// state → glyph functions a bar readout and its popover both call, so
    /// they can never drift onto different mappings.
    fn icons_column<'a>(&'a self, s: Surface) -> Element<'a, Message> {
        let t = &self.theme;
        let tint = widget::role(t, s, Emphasis::Rest);
        let caption = convert::ColorExt::into_iced(t.on(s).tertiary);
        let label_size = t.typography.size.label;

        // A caption column wide enough for the longest label, so the glyph
        // rows line up into columns.
        let captioned = |label: String, glyphs: Vec<Element<'a, Message>>| {
            row![
                container(text(label).size(label_size).color(caption)).width(150),
                row(glyphs).spacing(t.sizes.pill_gap).align_y(iced::Center),
            ]
            .spacing(12)
            .align_y(iced::Center)
        };

        const SAMPLE: [Icon; 8] = [
            Icon::House,
            Icon::Folder,
            Icon::FileText,
            Icon::Music,
            Icon::Wifi,
            Icon::BatteryFull,
            Icon::Play,
            Icon::Check,
        ];
        let strip = |name: &str, size: f32| {
            let glyphs = SAMPLE
                .iter()
                .map(|kind| icon(*kind, size, tint).into())
                .collect();
            captioned(format!("{name} · {size:.0}px"), glyphs)
        };
        let ladder = |name: &'static str, kinds: Vec<Icon>| {
            let glyphs = kinds
                .into_iter()
                .map(|kind| icon(kind, t.sizes.icon_menu, tint).into())
                .collect();
            captioned(name.to_owned(), glyphs)
        };

        column![
            strip("icon_bar", t.sizes.icon_bar),
            strip("icon_row", t.sizes.icon_row),
            strip("icon_menu", t.sizes.icon_menu),
            // Each ladder climbs its levels left to right, ending on the
            // state that overrides the level (charging, mute, offline-first
            // for Wi-Fi).
            ladder(
                "battery 5 → 90, charging",
                vec![
                    icon::battery_icon(5.0, false),
                    icon::battery_icon(20.0, false),
                    icon::battery_icon(50.0, false),
                    icon::battery_icon(90.0, false),
                    icon::battery_icon(50.0, true),
                ],
            ),
            ladder(
                "wi-fi off, 10 → 90",
                vec![
                    icon::wifi_icon(false, None),
                    icon::wifi_icon(true, Some(10)),
                    icon::wifi_icon(true, Some(30)),
                    icon::wifi_icon(true, Some(60)),
                    icon::wifi_icon(true, Some(90)),
                ],
            ),
            ladder(
                "volume 0 → 80, muted",
                vec![
                    icon::volume_icon(0, false),
                    icon::volume_icon(30, false),
                    icon::volume_icon(80, false),
                    icon::volume_icon(50, true),
                ],
            ),
            ladder(
                "brightness 10 → 90",
                vec![
                    icon::brightness_icon(10),
                    icon::brightness_icon(50),
                    icon::brightness_icon(90),
                ],
            ),
        ]
        .spacing(10)
        .into()
    }

    /// Every `ScrimKind` as a small labeled tile, drawn on a paper card —
    /// paper stands in for the wallpaper each translucent ink scrim dims
    /// (on the ink shell they'd all vanish into the background).
    /// `lock_rest` paints its real three-stop gradient, the reason the
    /// specimen exists as a *styled container* rather than the Colors
    /// page's flat token swatches.
    fn scrims_column(&self) -> Element<'_, Message> {
        let t = &self.theme;
        let label_color = convert::ColorExt::into_iced(t.on_paper.secondary);
        let label_size = t.typography.size.label;

        let kinds: [(&'static str, ScrimKind); 10] = [
            ("boot", ScrimKind::Boot),
            ("shutdown", ScrimKind::Shutdown),
            ("lock_awake", ScrimKind::LockAwake),
            ("lock_rest ▒", ScrimKind::LockRest),
            ("launcher", ScrimKind::Launcher),
            ("overview", ScrimKind::Overview),
            ("capture", ScrimKind::Capture),
            ("modal", ScrimKind::Modal),
            ("translucent_panel", ScrimKind::TranslucentPanel),
            ("canvas", ScrimKind::Canvas),
        ];
        let tiles: Vec<Element<'_, Message>> = kinds
            .into_iter()
            .map(|(name, kind)| {
                column![
                    widget::swatch(108.0, 48.0, style::container::scrim(t, kind)),
                    text(name).size(label_size).color(label_color),
                ]
                .spacing(t.sizes.gap_tight)
                .width(108)
                .into()
            })
            .collect();

        // `row!` doesn't wrap, so the ten tiles are chunked by hand into
        // rows of five (the Colors page's `swatch_grid` idiom; `Element`
        // isn't `Clone`, so the Vec is consumed with `into_iter`).
        let mut rows: Vec<Element<'_, Message>> = Vec::new();
        let mut current: Vec<Element<'_, Message>> = Vec::new();
        for tile in tiles {
            current.push(tile);
            if current.len() == 5 {
                rows.push(row(std::mem::take(&mut current)).spacing(12).into());
            }
        }
        if !current.is_empty() {
            rows.push(row(current).spacing(12).into());
        }

        container(column(rows).spacing(12))
            .style(style::container::paper_window(t))
            .padding(20)
            .into()
    }

    /// The §5 ping-pong marquee at three fixed elapsed times — the gallery
    /// is a static catalog with no animation clock, so like the semaphore's
    /// breath this is a filmstrip of one run (dwell 2000 ms, then a
    /// 24 px/s sweep: 0 ms parks at the head, the later frames sit
    /// mid-sweep).
    fn marquee_column(&self) -> Element<'_, Message> {
        let t = &self.theme;
        let caption = convert::ColorExt::into_iced(t.on_ink.tertiary);
        const TITLE: &str = "Nala Sinephro — Space 1.8 · 01 Space 1 (Continuum Mix, 2021 Remaster)";

        let frame = |label: &'static str, ms: u64| {
            column![
                marquee(
                    t,
                    TITLE,
                    Duration::from_millis(ms),
                    24,
                    t.typography.size.bar,
                    convert::ColorExt::into_iced(t.on_ink.secondary),
                ),
                text(label).size(t.typography.size.label).color(caption),
            ]
            .spacing(t.sizes.gap_tight)
        };

        column![
            frame("0 ms — dwelling at the head", 0),
            frame("3000 ms — 1 s into the sweep", 3000),
            frame("6000 ms — 4 s into the sweep", 6000),
            text(format!(
                "marquee_dwell {} ms · marquee_speed {} px/s — translation only, no fade",
                t.motion.marquee_dwell, t.motion.marquee_speed
            ))
            .size(t.typography.size.label)
            .color(caption),
        ]
        .spacing(10)
        .into()
    }

    /// The §7 avatar composite, initials arm — the gallery bundles no photo
    /// asset, and the initials disc is the part the design system owns end
    /// to end (`initials_from` + `container::disc` + the
    /// `avatar_initials`/`avatar_lock` tokens).
    fn avatar_row(&self) -> Element<'_, Message> {
        let t = &self.theme;
        let caption = convert::ColorExt::into_iced(t.on_ink.tertiary);
        let initials = avatar::Avatar::Initials(avatar::initials_from("Jordan Dunn"));

        row![
            avatar::view(t, &initials, t.sizes.avatar_lock),
            column![
                text(format!("avatar_lock · {:.0}px", t.sizes.avatar_lock))
                    .size(t.typography.size.label)
                    .color(caption),
                text("initials_from(\"Jordan Dunn\") → \"JD\"")
                    .size(t.typography.size.label)
                    .color(caption),
            ]
            .spacing(t.sizes.gap_tight),
        ]
        .spacing(16)
        .align_y(iced::Center)
        .into()
    }

    /// The canvas-side token bridge: a small fixed scene painted by
    /// `SelectionChrome` (capture scrim bands around a hole, the dashed
    /// accent edge, the eight handles). Paper again stands in for the
    /// content being captured, for the same reason as the scrim tiles.
    fn selection_chrome_pane(&self) -> Element<'_, Message> {
        let t = &self.theme;
        canvas(SelectionDemo {
            chrome: SelectionChrome::new(t, ScrimKind::Capture),
            paper: convert::ColorExt::into_iced(t.palette.paper),
        })
        .width(320)
        .height(180)
        .into()
    }
}

/// The Selection chrome section's `canvas::Program`: a static scene — no
/// state, no interaction — that exercises the three `SelectionChrome`
/// drawing operations on a fixed hole.
struct SelectionDemo {
    chrome: SelectionChrome,
    /// The stand-in "content" fill behind the scrim (`palette.paper`).
    paper: iced::Color,
}

impl<Message> canvas::Program<Message> for SelectionDemo {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        // Frame coordinates are local, so the drawable area is the bounds'
        // size at the origin — not `bounds` itself, which is in window
        // space.
        let outer = Rectangle::with_size(bounds.size());
        let hole = Rectangle::new(
            Point::new(outer.width * 0.3, outer.height * 0.28),
            Size::new(outer.width * 0.4, outer.height * 0.44),
        );

        frame.fill_rectangle(Point::ORIGIN, outer.size(), self.paper);
        self.chrome.fill_scrim_around(&mut frame, outer, hole);
        self.chrome.stroke_dashed_edge(&mut frame, hole);
        self.chrome.fill_handles(&mut frame, hole);

        vec![frame.into_geometry()]
    }
}
