//! Gallery pages beyond Widgets: Colors, Typography, and Spacing.
//!
//! Rust directory examples (`examples/gallery/main.rs`) resolve modules
//! exactly like a normal binary crate's `src/main.rs` would — relative to
//! `main.rs`, not to the crate root. So `mod pages;` in `main.rs` pulls in
//! this file, and this file's `pub mod` lines below pull in the sibling
//! `colors.rs` / `typography.rs` / `spacing.rs` files the usual way.

pub mod colors;
pub mod spacing;
pub mod typography;
