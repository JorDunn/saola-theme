# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.13.0](https://github.com/JorDunn/saola-theme/compare/saola-theme-v0.12.0...saola-theme-v0.13.0) - 2026-08-14

### Changed

- *(saola-theme)* [**breaking**] breadcrumb width budget — end-anchored scrollable trail
- *(saola-theme)* [**breaking**] window-vs-shell chrome context — rest recipes take Chrome

## [0.12.0](https://github.com/JorDunn/saola-theme/compare/saola-theme-v0.11.0...saola-theme-v0.12.0) - 2026-08-11

### Changed

- *(saola-theme)* [**breaking**] surface-generic window chrome — window(t, s) replaces paper_window

## [0.11.0](https://github.com/JorDunn/saola-theme/compare/saola-theme-v0.10.0...saola-theme-v0.11.0) - 2026-08-11

### Added

- *(saola-theme)* unit_budget — canonical px→width-unit calibration for §7 labels

## [0.10.0](https://github.com/JorDunn/saola-theme/compare/saola-theme-v0.9.0...saola-theme-v0.10.0) - 2026-08-11

### Changed

- *(saola-theme)* [**breaking**] width-aware truncate (UAX #11 East Asian Width)

## [0.9.0](https://github.com/JorDunn/saola-theme/compare/saola-theme-v0.8.0...saola-theme-v0.9.0) - 2026-08-11

### Added

- add overflow::truncate, the default text-overflow mode

## [0.8.0](https://github.com/JorDunn/saola-theme/compare/saola-theme-v0.7.0...saola-theme-v0.8.0) - 2026-08-10

### Added

- add segmented_row_icons, an icon-only segmented control

### Changed

- [**breaking**] accept owned String labels in breadcrumb via IntoFragment

## [0.7.0](https://github.com/JorDunn/saola-theme/compare/saola-theme-v0.6.0...saola-theme-v0.7.0) - 2026-08-10

### Added

- dialog kit, toasts, breadcrumbs, progress, combo_box, editor, table

### Fixed

- make text_input placeholder and icon legible on ink surfaces

## [0.6.0](https://github.com/JorDunn/saola-theme/compare/saola-theme-v0.5.0...saola-theme-v0.6.0) - 2026-08-10

### Added

- gallery specimens for the upstreamed helpers
- add the marquee widget — style guide §5's ping-pong sweep
- add the avatar composite — resolution, initials, and view
- add the motion module — fraction, toast_alpha, breath
- Clone style closures, ring helpers, and consumer-ported styles
- add opacity-scaling and scrim-gradient bridges to convert
- port saola-files feedback — inset/list_row/hairline helpers, hairline and window_sidebar tokens

### Changed

- unify ScrimKind and read selection-chrome values from tokens
- [**breaking**] remove dead panel_pill_media and media_title_max_width tokens

## [0.5.0](https://github.com/JorDunn/saola-theme/compare/saola-theme-v0.4.0...saola-theme-v0.5.0) - 2026-08-02

### Added

- add session-status semaphore tokens and status_dot helper
- align column-strip dash with the islands concept

## [0.4.0](https://github.com/JorDunn/saola-theme/compare/saola-theme-v0.3.0...saola-theme-v0.4.0) - 2026-07-26

### Added

- *(theme)* add radio, segmented, rejected input, and card/keycap/badge kit helpers

## [0.3.0](https://github.com/JorDunn/saola-theme/compare/saola-theme-v0.2.0...saola-theme-v0.3.0) - 2026-07-26

### Added

- add ledger bar tokens, weighted font helpers, and bar_pill container

## [0.2.0](https://github.com/JorDunn/saola-theme/compare/saola-theme-v0.1.0...saola-theme-v0.2.0) - 2026-07-26

### Added

- minimap dash + popover containers, muted button, panel geometry tokens

## [0.1.0](https://github.com/JorDunn/saola-theme/releases/tag/saola-theme-v0.1.0) - 2026-07-26

### Added

- Saola design system v0.1 — tokens, iced styles, gallery
