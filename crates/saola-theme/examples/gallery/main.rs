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
    button, canvas, checkbox, column, combo_box, container, pick_list, progress_bar, radio, row,
    scrollable, slider, table, text, text_editor, text_input, toggler, tooltip, vertical_slider,
    Space,
};
use iced::{Element, Fill, Point, Rectangle, Size, Task};
use saola_theme::canvas::SelectionChrome;
use saola_theme::indeterminate::indeterminate_rule;
use saola_theme::marquee::marquee;
use saola_theme::style::container::{DashState, ScrimKind, SessionStatus};
use saola_theme::widget::Emphasis;
use saola_theme::{
    avatar, chrome, convert, icon, motion, style, widget, ColorExt, Icon, Surface, Theme,
};

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
    /// The Stage 17 text-editor specimen: every edit arrives as a
    /// `text_editor::Action` the update loop replays onto the shared
    /// `Content` (the widget owns no text of its own).
    EditorAction(text_editor::Action),
    CheckboxToggled(bool),
    TogglerToggled(bool),
    SliderChanged(f32),
    PickListSelected(&'static str),
    /// The Stage 16 combo box specimen — shares `PICK_LIST_OPTIONS` with
    /// the pick list demo above it, since both are "closed-set chooser"
    /// specimens and don't need distinct option lists to make their point.
    ComboBoxSelected(&'static str),
    /// The Stage 8 kit's own text-input specimens share one value field
    /// (the demo is about the border/ring treatment, not distinct content).
    KitTextInputChanged(String),
    /// `true` = Ascending, `false` = Descending — the radio group's choice.
    RadioSelected(bool),
    /// Index into the segmented control's labels (Files/Folders/All).
    SegmentSelected(usize),
    /// Index into the icon view-switcher's glyphs: 0 = list, 1 = grid.
    ViewSelected(usize),
    /// Flips `Gallery::surface` between `Surface::Ink` and `Surface::Paper`.
    /// No payload: there are only two surfaces, so "toggle" always means
    /// "the other one" — nothing the message needs to carry.
    SurfaceToggled,
    /// The notification-centre mock's Do Not Disturb row.
    DndToggled(bool),
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
    /// The icon view-switcher's selection (list vs grid).
    view_selected: usize,
    /// The Stage 16 combo box's search/filter state — owned here (not
    /// rebuilt per `view()`) because `combo_box::State` carries a `RefCell`
    /// the widget mutates as the user types, the same reason every other
    /// stateful specimen above (`text_input_value`, `pick_list_selected`…)
    /// lives on `Gallery` rather than being constructed fresh each frame.
    combo_box_state: combo_box::State<&'static str>,
    combo_box_selected: Option<&'static str>,
    /// The Stage 17 text editor's buffer — owned here for the same reason
    /// as `combo_box_state` above: `text_editor::Content` is the widget's
    /// stateful backing store (text, cursor, undo), so rebuilding it per
    /// `view()` would discard the user's edits every frame. Both surface
    /// specimens borrow this one buffer, exactly as the two text-input
    /// specimens share `text_input_value`.
    editor_content: text_editor::Content,
    /// The notification-centre mock's Do Not Disturb toggle.
    dnd_toggled: bool,
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
            view_selected: 0,
            combo_box_state: combo_box::State::new(PICK_LIST_OPTIONS.to_vec()),
            combo_box_selected: None,
            editor_content: text_editor::Content::with_text(
                "Multi-line notes live here.\nSecond line to prove the wrap.",
            ),
            dnd_toggled: false,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PageSelected(page) => self.page = page,
            Message::DemoPressed => {}
            Message::TextInputChanged(value) => self.text_input_value = value,
            Message::EditorAction(action) => self.editor_content.perform(action),
            Message::CheckboxToggled(checked) => self.checkbox_checked = checked,
            Message::TogglerToggled(toggled) => self.toggler_toggled = toggled,
            Message::SliderChanged(value) => self.slider_value = value,
            Message::PickListSelected(selected) => self.pick_list_selected = Some(selected),
            Message::ComboBoxSelected(selected) => self.combo_box_selected = Some(selected),
            Message::KitTextInputChanged(value) => self.kit_text_input_value = value,
            Message::RadioSelected(selected) => self.radio_selected = selected,
            Message::SegmentSelected(index) => self.segment_selected = index,
            Message::ViewSelected(index) => self.view_selected = index,
            Message::SurfaceToggled => {
                self.surface = match self.surface {
                    Surface::Ink => Surface::Paper,
                    Surface::Paper => Surface::Ink,
                };
            }
            Message::DndToggled(toggled) => self.dnd_toggled = toggled,
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
        // A living specimen of `sizes.window_sidebar` (200 px, "standard
        // width of an app window's navigation sidebar") — this nav column
        // *is* a navigation sidebar, so it's sized from the token instead
        // of a local constant.
        .width(t.sizes.window_sidebar)
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
                text("Containers").size(t.typography.size.section_heading),
                self.labeled_surface_row(primary, self.containers_row(primary)),
                self.labeled_surface_row(secondary, self.containers_row(secondary)),
                // The T1 application-window chrome. Ink-only as a section
                // (see `window_chrome_row`): a window floats on the shell,
                // and both surfaces are already inside the one specimen.
                text("Window chrome").size(t.typography.size.section_heading),
                self.labeled_surface_row(Surface::Ink, self.window_chrome_row()),
                text("Rows, tiles & menus").size(t.typography.size.section_heading),
                self.labeled_surface_row(primary, self.rows_column(primary)),
                self.labeled_surface_row(secondary, self.rows_column(secondary)),
                // The Stage 17 table specimen (the future saola-files
                // detailed list view): an app-window widget like the rows
                // above it, so it gets the paired ink/paper treatment.
                text("Table").size(t.typography.size.section_heading),
                self.labeled_surface_row(primary, self.table_column(primary)),
                self.labeled_surface_row(secondary, self.table_column(secondary)),
                // The file-picker breadcrumb trail (Stage 14, style guide
                // §7): unlike the shell-chrome sections below, a breadcrumb
                // renders in an app window on either surface, so it gets
                // its own paired ink/paper section here rather than joining
                // the ink-only run.
                text("Breadcrumb").size(t.typography.size.section_heading),
                self.labeled_surface_row(primary, self.breadcrumb_row(primary)),
                self.labeled_surface_row(secondary, self.breadcrumb_row(secondary)),
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
                // The modal dialog kit (Stage 12): a dialog only ever floats
                // over its scrim, which is always ink-tinted, so — like
                // Scrims/Marquee/Avatar/Selection chrome above — this
                // specimen exists only on ink.
                text("Dialog").size(t.typography.size.section_heading),
                self.labeled_surface_row(Surface::Ink, self.dialog_column()),
                // The Stage 13 toast internals: like Dialog above, a toast
                // only ever floats on the shell layer, so this section is
                // ink-only too.
                text("Notification toast").size(t.typography.size.section_heading),
                self.labeled_surface_row(Surface::Ink, self.notification_column()),
                // The power/boot menu's bare-icon row (Stage 14, style
                // guide §6): like the toast above, it only ever floats on
                // the shell scrim, so this section is ink-only too.
                text("Bare-icon menu").size(t.typography.size.section_heading),
                self.labeled_surface_row(Surface::Ink, self.bare_icon_menu_row()),
                // The Stage 16 notification-centre mock: like the toast and
                // bare-icon menu above, the centre is shell chrome that
                // never renders on a paper window, so this section is
                // ink-only too.
                text("Notification centre").size(t.typography.size.section_heading),
                self.labeled_surface_row(Surface::Ink, self.notification_centre_column()),
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
    /// in a `container::window` card when `surface` is `Paper` (matching how
    /// the rest of the app only ever shows paper as a window floating on the
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
                    .style(style::container::window(t, Surface::Paper))
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
        let caption = t.on(s).tertiary.into_iced();

        // The Stage 15 indeterminate rule, beside the determinate one: the
        // gallery is a static catalog with no animation clock (same device
        // as `marquee_column`'s sweep and `notification_column`'s toast),
        // so three fixed-elapsed-time frames stand in for the ping-pong —
        // parked at the near edge, mid-sweep, and reversing at the far
        // edge. At this column's 160 px width and `SEGMENT_FRACTION`
        // (30%), the travel is 112 px; at `motion.marquee_speed` (24 px/s,
        // reused — see the module docs on `saola_theme::indeterminate`)
        // that's a 4667 ms one-way sweep, so 0 / 2333 / 4667 ms land
        // exactly on "parked" / "half" / "far edge".
        let indeterminate_frame = |label: &'static str, ms: u64| {
            column![
                indeterminate_rule(t, s, Duration::from_millis(ms)).width(160),
                text(label).size(t.typography.size.label).color(caption),
            ]
            .spacing(t.sizes.gap_tight)
        };

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
            // Horizontal beside vertical: `style::slider::rest` styles both
            // — `iced::widget::vertical_slider` re-exports the horizontal
            // module's `Catalog`/`Status`/`Style` verbatim, so no second
            // helper exists or is needed (see the doc note on
            // `style::slider::rest`).
            row![
                slider(0.0..=100.0, self.slider_value, Message::SliderChanged)
                    .style(style::slider::rest(t, s)),
                vertical_slider(0.0..=100.0, self.slider_value, Message::SliderChanged)
                    .style(style::slider::rest(t, s))
                    .height(96),
            ]
            .spacing(16)
            .align_y(iced::Center),
            row![
                column![
                    text("Determinate")
                        .size(t.typography.size.label)
                        .color(caption),
                    widget::progress_rule(t, s, self.slider_value / 100.0),
                ]
                .spacing(t.sizes.gap_tight)
                .width(Fill),
                column![
                    text("Indeterminate — ping-pong, no dwell")
                        .size(t.typography.size.label)
                        .color(caption),
                    row![
                        indeterminate_frame("0 ms — near edge", 0),
                        indeterminate_frame("2333 ms — mid-sweep", 2333),
                        indeterminate_frame("4667 ms — far edge, reversing", 4667),
                    ]
                    .spacing(12),
                ]
                .spacing(t.sizes.gap_tight),
            ]
            .spacing(24)
            .align_y(iced::Center),
            pick_list(
                PICK_LIST_OPTIONS,
                self.pick_list_selected,
                Message::PickListSelected
            )
            .style(style::pick_list::field(t, s))
            .menu_style(style::pick_list::menu(t, s)),
            // The Stage 16 combo box: `combo_box::Catalog` forwards
            // straight to `text_input::Catalog` + `menu::Catalog` with no
            // fields of its own (see `style::combo_box`'s module docs), so
            // its field and dropdown reuse `style::text_input::rest` and
            // `style::pick_list::menu` verbatim — `style::combo_box::field`/
            // `menu` below are thin, surface-aware names over exactly
            // those two helpers. Only the closed field renders here: the
            // dropdown is an overlay iced opens on focus, the same reason
            // `pick_list` above never shows its own menu open in this
            // static catalog either.
            combo_box::ComboBox::new(
                &self.combo_box_state,
                "Search widgets…",
                self.combo_box_selected.as_ref(),
                Message::ComboBoxSelected,
            )
            .input_style(style::combo_box::field(t, s))
            .menu_style(style::combo_box::menu(t, s)),
            // The Stage 17 multi-line editor: `text_input::rest`'s look at
            // the over-rounded-rectangle radius (`radii.inset`) instead of
            // the pill — see `style::text_editor`'s module docs for the two
            // deliberate deviations. Click in for the 2 px accent focus
            // ring; drag-select for the accent selection.
            text_editor(&self.editor_content)
                .placeholder("Notes…")
                .style(style::text_editor::rest(t, s))
                .padding([10, 14])
                .height(96)
                .on_action(Message::EditorAction),
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

    /// Three container helpers that shipped without a gallery specimen
    /// (CLAUDE.md's "new style helpers get a specimen" rule): `card`, `tile`,
    /// and `tooltip` — in the given surface context.
    fn containers_row(&self, s: Surface) -> Element<'_, Message> {
        let t = &self.theme;

        let card = container(
            column![
                text("Card")
                    .font(convert::display_font(t))
                    .size(t.typography.size.section_heading),
                text("radii.card · popover shadow on ink, fill_subtle inset on paper")
                    .size(t.typography.size.secondary)
                    .color(convert::ColorExt::into_iced(t.on(s).secondary)),
            ]
            .spacing(6),
        )
        .style(style::container::card(t, s))
        .padding(18)
        .width(260);

        let tile = container(
            text("Tile — radii.tile, a fill_subtle recess (not a floating layer)")
                .size(t.typography.size.secondary)
                .color(convert::ColorExt::into_iced(t.on(s).primary)),
        )
        .style(style::container::tile(t, s))
        .padding(14)
        .width(220);

        // `tooltip` is exercised through the real `iced::widget::tooltip` —
        // it styles via `container::Catalog`, same as `card`/`tile` above,
        // so `style::container::tooltip` slots straight into `.style(...)`.
        // The bubble is always ink (readable on either surface, per its
        // doc), so only the trigger button below varies with `s`.
        let tooltip_demo = tooltip(
            button(text("Hover me").size(t.typography.size.body))
                .style(style::button::rest(t, s))
                .padding([10, 18])
                .on_press(Message::DemoPressed),
            container(text("A tooltip — ink, radii.tile, popover shadow"))
                .style(style::container::tooltip(t))
                .padding(10),
            tooltip::Position::Bottom,
        );

        row![card, tile, tooltip_demo]
            .spacing(16)
            .align_y(iced::Center)
            .into()
    }

    /// The application-window chrome (`chrome::window_frame` +
    /// `chrome::window_header` over `style::container::window`) at both
    /// surfaces, side by side: the ivory window the style guide ships as the
    /// default, and the ink one it offers as a user preference.
    ///
    /// Ink-only as a *section* — a window always floats on the shell, so
    /// showing these on a paper card would be a window inside a window. Both
    /// surfaces already appear inside the one specimen, which is the point:
    /// against the ink shell you can see that the ink window's
    /// `on_ink.divider` border still draws an edge, where a `palette.ink`
    /// border would vanish.
    fn window_chrome_row(&self) -> Element<'_, Message> {
        let t = &self.theme;

        // Both frames are fixed-size: `window_frame` fills whatever it is
        // given, so a specimen needs a box to fill. 320x180 is enough to
        // show the 46 px header, the 24 px corners, and a line of body text.
        let frame = |s: Surface, title: &'static str, body: &'static str| {
            let header = chrome::window_header(
                t,
                s,
                title,
                Message::DemoPressed,
                Message::DemoPressed,
                // `None`: a fixed-size specimen has nothing to maximise into,
                // the same call saola-capture makes for its fixed window.
                None,
            );
            let content = container(
                text(body)
                    .size(t.typography.size.secondary)
                    .color(convert::ColorExt::into_iced(t.on(s).secondary)),
            )
            .padding([0, 16]);

            container(chrome::window_frame(t, s, header, content.into()))
                .width(320)
                .height(180)
        };

        row![
            frame(Surface::Paper, "Paper window", "Ink text on paper."),
            frame(Surface::Ink, "Ink window", "Ivory text on ink."),
        ]
        .spacing(16)
        .align_y(iced::Center)
        .into()
    }

    /// The Stage 12 modal dialog kit's full assembly recipe, as one static
    /// specimen: a `ScrimKind::Modal` backdrop (`style::container::scrim`)
    /// filling the swatch region, with a `style::dialog::surface` card
    /// (`sizes.dialog_width`) centered on top of it — title at
    /// `size.dialog_title` in the display face, body text, and a
    /// `widget::footer_strip` holding `button::rest` (Cancel) /
    /// `button::active` (Discard) actions. Ink-only: a dialog only ever
    /// floats over its (always ink-tinted) scrim, so unlike most Widgets
    /// sections there is no "on paper" twin.
    fn dialog_column(&self) -> Element<'_, Message> {
        let t = &self.theme;
        let size = t.typography.size.body;

        let title = text("Discard changes?")
            .font(convert::display_font(t))
            .size(t.typography.size.dialog_title)
            .color(convert::ColorExt::into_iced(t.on_paper.primary));

        let body = text("This file has unsaved edits. Discarding them can't be undone.")
            .size(size)
            .color(convert::ColorExt::into_iced(t.on_paper.secondary));

        let footer = widget::footer_strip(
            t,
            Surface::Paper,
            row![
                button(text("Cancel").size(size))
                    .style(style::button::rest(t, Surface::Paper))
                    .padding(t.paddings.dialog_button)
                    .on_press(Message::DemoPressed),
                Space::new().width(Fill),
                button(text("Discard").size(size))
                    .style(style::button::active(t, Surface::Paper))
                    .padding(t.paddings.dialog_button)
                    .on_press(Message::DemoPressed),
            ]
            .align_y(iced::Center),
        );

        let dialog = container(column![title, body, footer].spacing(16))
            .style(style::dialog::surface(t))
            .padding(t.sizes.popover_padding)
            .width(t.sizes.dialog_width);

        // The scrim behind it, per the recipe: `ScrimKind::Modal` filling
        // the swatch region, the dialog centered both axes. The style
        // guide's 7px compositor blur (`sizes.scrim_blur_modal`) has no
        // iced-side render step — see `style::dialog`'s module docs.
        container(dialog)
            .style(style::container::scrim(t, ScrimKind::Modal))
            .width(Fill)
            .height(320)
            .align_x(iced::Center)
            .align_y(iced::Center)
            .into()
    }

    /// The Stage 13 notification toast internals: `notification_card` +
    /// `notification::icon_tile` + `notification::life_rule`, as a
    /// fixed-elapsed-times filmstrip — the gallery has no animation clock
    /// (same device as [`Self::marquee_column`]'s sweep and
    /// [`Self::session_status_column`]'s breath), so three frames stand in
    /// for the toast living out `motion.toast_in` → `toast_idle` →
    /// `toast_out`: arriving (alpha rising, life still full), mid-idle
    /// (alpha settled, life half-drained), and fading out (alpha falling,
    /// life already at zero — nothing left to count down). The urgent
    /// variant sits beside the filmstrip as a fourth, static frame:
    /// `card_urgent` gets the ring but never the rule (10b: "a terracotta
    /// ring and no life rule").
    fn notification_column(&self) -> Element<'_, Message> {
        let t = &self.theme;
        let caption = convert::ColorExt::into_iced(t.on_ink.tertiary);
        let icon_tint = widget::role(t, Surface::Ink, Emphasis::Rest);

        let toast = |elapsed_ms: u64| {
            let elapsed = Duration::from_millis(elapsed_ms);
            let alpha = motion::toast_alpha(t, elapsed);
            let life = motion::life_fraction(t, elapsed);

            let tile = container(icon(Icon::Image, t.sizes.icon_menu, icon_tint))
                .width(t.sizes.icon_tile)
                .height(t.sizes.icon_tile)
                .align_x(iced::Center)
                .align_y(iced::Center)
                .style(style::notification::icon_tile(t));

            let texts = column![
                text("Screenshot saved")
                    .font(convert::display_font(t))
                    .size(t.typography.size.section_heading)
                    .color(t.on_ink.primary.with_opacity(alpha)),
                text("~/Pictures/capture-0142.png")
                    .size(t.typography.size.secondary)
                    .color(t.on_ink.secondary.with_opacity(alpha)),
            ]
            .spacing(4);

            let body =
                container(row![tile, texts].spacing(12).align_y(iced::Center)).padding([14, 18]);

            let rule = progress_bar(0.0..=1.0, life)
                .length(Fill)
                .girth(t.sizes.life_rule)
                .style(style::notification::life_rule(t));

            container(column![body, rule])
                .style(style::container::notification_card(t, alpha))
                .width(300)
        };

        let filmstrip = row![
            column![
                toast(0),
                text("0 ms — arriving")
                    .size(t.typography.size.label)
                    .color(caption),
            ]
            .spacing(t.sizes.gap_tight),
            column![
                toast(3000),
                text("3000 ms — mid-idle, draining")
                    .size(t.typography.size.label)
                    .color(caption),
            ]
            .spacing(t.sizes.gap_tight),
            column![
                toast(6000),
                text("6000 ms — fading out, expired")
                    .size(t.typography.size.label)
                    .color(caption),
            ]
            .spacing(t.sizes.gap_tight),
        ]
        .spacing(16);

        let urgent = container(
            column![
                text("Battery critical")
                    .font(convert::display_font(t))
                    .size(t.typography.size.section_heading)
                    .color(convert::ColorExt::into_iced(t.on_paper.primary)),
                text("6% remaining — plug in now")
                    .size(t.typography.size.secondary)
                    .color(convert::ColorExt::into_iced(t.on_paper.secondary)),
            ]
            .spacing(6),
        )
        .style(style::container::card_urgent(t, Surface::Ink))
        .padding(18)
        .width(300);

        column![
            filmstrip,
            text(format!(
                "toast_in {} ms · toast_idle {} ms · toast_out {} ms — life_rule drains across toast_idle only",
                t.motion.toast_in, t.motion.toast_idle, t.motion.toast_out
            ))
            .size(t.typography.size.label)
            .color(caption),
            column![
                text("Urgent — card_urgent, no life rule (10b)")
                    .size(t.typography.size.label)
                    .color(caption),
                urgent,
            ]
            .spacing(t.sizes.gap_tight),
        ]
        .spacing(16)
        .into()
    }

    /// The power/boot menu's bare-icon row (Stage 14, style guide §6):
    /// `widget::bare_icon_item` for a handful of items, one forced into
    /// `hovered = true` so the "ivory 55% at rest, full terracotta hovered"
    /// contrast the concepts describe is visible without a live cursor —
    /// the same fixed-snapshot device the toast filmstrip above uses for
    /// motion it can't otherwise show statically.
    fn bare_icon_menu_row(&self) -> Element<'_, Message> {
        let t = &self.theme;
        row![
            widget::bare_icon_item(t, Icon::Lock, "Lock", false, Some(Message::DemoPressed)),
            widget::bare_icon_item(
                t,
                Icon::RotateCcw,
                "Restart",
                true,
                Some(Message::DemoPressed)
            ),
            widget::bare_icon_item(t, Icon::X, "Cancel", false, Some(Message::DemoPressed)),
        ]
        .spacing(t.sizes.island_gap)
        .into()
    }

    /// A mini notification-centre mock (Stage 16, style guide §6): the
    /// centre itself is `sizes.notification_centre_width` (460 px) wide,
    /// grouped by app with collapsible groups. `widget::group_header` only
    /// ever shows one disclosure state at a time, so this specimen stacks
    /// one collapsed group and one expanded group to put both readings on
    /// screen at once, a hairline, then the DND `toggler` row the centre's
    /// header carries — everything constrained to the real
    /// `notification_centre_width` token rather than the page's own `Fill`.
    /// Ink-only, like the toast and bare-icon menu sections above: the
    /// notification centre is shell chrome, never drawn on a paper window.
    fn notification_centre_column(&self) -> Element<'_, Message> {
        let t = &self.theme;
        let s = Surface::Ink;

        let dnd_row = container(
            row![
                text("Do Not Disturb").size(t.typography.size.body),
                Space::new().width(Fill),
                toggler(self.dnd_toggled)
                    .style(style::toggles::toggler(t, s))
                    .on_toggle(Message::DndToggled),
            ]
            .align_y(iced::Center)
            .width(Fill),
        )
        .padding(t.paddings.strip);

        column![
            widget::group_header(t, "Messages", 3, true, Some(Message::DemoPressed)),
            widget::group_header(t, "Mail", 12, false, Some(Message::DemoPressed)),
            widget::hairline(t, s),
            dnd_row,
        ]
        .spacing(t.sizes.gap_tight)
        .width(t.sizes.notification_centre_width)
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

    /// The Stage 17 `iced::widget::table` specimen: a 3-column, 4-row
    /// detailed list (the saola-files columns view's shape). What Saola
    /// *can* own here is the cell content and the geometry — header cells
    /// in the `text::label` role per `section_label` conventions, body
    /// cells in `text::body`/`text::secondary`, the row idiom's
    /// "horizontal hairlines only" via `.separator_x(0.0)` /
    /// `.separator_y(sizes.hairline)`, and `paddings.strip`-shaped cell
    /// padding. What it can't own yet is the separator *color*:
    /// iced_widget 0.14.2's `Table` has no `.style(...)` builder (see
    /// `style::table`'s module docs), so the hairlines below draw in the
    /// app palette's derived `background.strong` instead of
    /// `style::table::rest`'s `divider` role until an iced release ships
    /// the builder.
    fn table_column(&self, s: Surface) -> Element<'_, Message> {
        let t = &self.theme;

        // Rows are cloned per column by `Table::new`, so plain `Copy`
        // tuples of `&'static str` keep the demo free of allocation.
        type Row = (&'static str, &'static str, &'static str);
        const FILES: [Row; 4] = [
            ("Field notes.md", "4.1 KiB", "Today 09:12"),
            ("Concept sketches", "12 items", "Yesterday"),
            ("saola-mock.svg", "18.6 KiB", "Aug 2"),
            ("Recordings", "3 items", "Jul 28"),
        ];

        table(
            [
                table::column(widget::text::label(t, s, "NAME"), move |file: Row| {
                    widget::text::body(t, s, file.0)
                })
                .width(Fill),
                table::column(widget::text::label(t, s, "SIZE"), move |file: Row| {
                    widget::text::secondary(t, s, file.1)
                }),
                table::column(widget::text::label(t, s, "MODIFIED"), move |file: Row| {
                    widget::text::secondary(t, s, file.2)
                }),
            ],
            FILES,
        )
        .width(Fill)
        .padding_x(t.paddings.strip[1])
        .padding_y(t.paddings.strip[0])
        // The row idiom: hairlines between rows, no vertical column rules.
        .separator_x(0.0)
        .separator_y(t.sizes.hairline)
        .into()
    }

    /// `widget::breadcrumb` (Stage 14, style guide §7): a short trail
    /// ending on the current folder, and a longer one so the
    /// `Icon::ChevronRight` separator chain reads across more than one hop.
    /// The current crumb (last segment, `on_press: None`) draws
    /// emphasized — `style::button::breadcrumb`'s terracotta "on" look —
    /// while every other crumb is quiet until hovered.
    fn breadcrumb_row(&self, s: Surface) -> Element<'_, Message> {
        let t = &self.theme;
        let short = widget::breadcrumb(
            t,
            s,
            [("Home", Some(Message::DemoPressed)), ("Documents", None)],
        );
        // Owned labels — the reason crumbs are by-value: a real consumer
        // builds these from a `PathBuf` each frame.
        let long = widget::breadcrumb(
            t,
            s,
            [
                ("Home".to_string(), Some(Message::DemoPressed)),
                ("Projects".to_string(), Some(Message::DemoPressed)),
                ("saola-theme".to_string(), Some(Message::DemoPressed)),
                ("src".to_string(), None),
            ],
        );
        column![short, long].spacing(12).into()
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

        // Its icon-only sibling, the list/grid view switcher —
        // `segmented_row_icons` computes each glyph's tint internally from
        // `is_selected` + surface (unlike `icon_button`, where tint is the
        // caller's job).
        let view_switcher = widget::segmented_row_icons(
            t,
            s,
            &[(0usize, Icon::List), (1, Icon::LayoutGrid)],
            &self.view_selected,
            Message::ViewSelected,
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
            row![segmented, view_switcher].spacing(12),
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
            .style(style::container::window(t, Surface::Paper))
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
