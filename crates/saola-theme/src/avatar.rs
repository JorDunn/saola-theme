//! The user-avatar composite: a photo, or an initials disc.
//!
//! Ported from saola-lockscreen's `modules/reveal.rs` (its `Avatar` enum,
//! `Avatar::resolve`, `avatar_candidates`, `initials_from`, and the avatar
//! arm of `Reveal::view`), because nothing in it is lockscreen-specific: the
//! greeter shows one of these per listed account, and both surfaces must
//! resolve and draw it identically (style guide §7 — config `avatar` path,
//! else `~/.face`, else an initials disc).
//!
//! # Decoding stays with the consumer
//!
//! This crate never decodes an image — the `image` crate is not a
//! dependency here, and must not become one (the token crate below this one
//! proves its purity with `cargo tree`; this crate keeps its dependency list
//! to iced + tokens). Instead, [`Avatar::resolve`] takes the decoder as a
//! closure and the consumer supplies it.
//!
//! That is not just dependency hygiene — the consumer *has* to own the
//! decode anyway, for a renderer reason saola-lockscreen confirmed live
//! (`wallpaper.rs`, "Root cause", verified against
//! `iced_wgpu-0.14.0/src/image/cache.rs`): `Handle::from_path` /
//! `Handle::from_bytes` are decoded **off-thread** by `iced_wgpu`, which
//! draws nothing for the frame(s) until the background load finishes — and
//! on a static surface like a lock screen or greeter, the "later frame" that
//! would composite the result may be minutes away. `Handle::from_rgba` is
//! the one variant the cache resolves synchronously. So consumers should do
//! what the lockscreen does: decode the bytes themselves (it uses the
//! `image` crate) and hand this module a `Handle::from_rgba` — see
//! saola-lockscreen's `wallpaper::decode`.
//!
//! # Bounded read
//!
//! Every candidate [`Avatar::resolve`] reads is user-controlled — the
//! configured path comes from AccountsService, and `~/.face` is whatever
//! sits at that name in the target account's home directory — and both the
//! greeter and the lockscreen resolve it *before* the user has authenticated.
//! A symlink or bind-mount to `/dev/zero` would read forever and exhaust
//! memory; a FIFO would block the caller forever; stat-then-read has a
//! TOCTOU window and stat alone does not distinguish a device node from a
//! regular file anyway. So `resolve` takes a `max_bytes` cap and the read
//! itself is bounded (see `read_bounded`) — an oversize or non-regular-file
//! candidate is rejected without blocking or growing without limit. Callers
//! with no stronger opinion should pass [`Avatar::MAX_BYTES_DEFAULT`].

use std::fs::File;
use std::io::Read as _;
use std::path::{Path, PathBuf};

use iced::widget::image::Handle;
use iced::widget::{container, image, text};
use iced::{ContentFit, Element};
use saola_tokens::{Surface, Theme};

use crate::convert::{ui_font, ColorExt};
use crate::icon::{icon, Icon};
use crate::style;

/// §7's avatar, resolved once at boot (ported from saola-lockscreen's
/// `reveal::Avatar`).
pub enum Avatar {
    /// A decoded image: the config `avatar` path, else `~/.face`.
    Photo(Handle),
    /// The last resort: a disc with the user's initials.
    Initials(String),
}

/// What [`Avatar::resolve`] came back with: the avatar itself, plus the one
/// failure worth telling the user about.
///
/// The warning is data rather than a side effect (the lockscreen
/// `eprintln!`s at its call site; a design-system crate should not write to
/// anyone's stderr, and pulling in a log facade for one line would be a new
/// dependency with no design-system reason). The consumer decides how to
/// report it.
pub struct Resolution {
    pub avatar: Avatar,
    /// `Some(path)` when the *explicitly configured* avatar path could not
    /// be read or decoded — the user asked for this specific file, so its
    /// failure deserves a warning even if a later candidate (`~/.face`)
    /// succeeded. A missing `~/.face` is the normal case for most systems
    /// and says nothing, so it is never reported here (same reasoning,
    /// verbatim, as saola-lockscreen's `Avatar::resolve`).
    pub configured_failed: Option<PathBuf>,
}

impl Avatar {
    /// Recommended `max_bytes` for [`Avatar::resolve`]: 16 MiB, generous for
    /// any real avatar file.
    pub const MAX_BYTES_DEFAULT: u64 = 16 * 1024 * 1024;

    /// Resolve the avatar per §7's order: config override, then
    /// `$HOME/.face`, then an initials disc. Called once, at boot — the
    /// lockscreen calls its original from `Lockscreen::boot`; the greeter
    /// calls this once per listed account.
    ///
    /// Never fails: an unreadable or undecodable candidate falls through to
    /// the next one, and the initials disc always works.
    ///
    /// `home` is a parameter rather than an environment read (pass
    /// `std::env::var_os("HOME")`'s result; `None` when it is unset) so the
    /// precedence stays unit-testable — see [`avatar_candidates`].
    ///
    /// `max_bytes` bounds every candidate read (see the module's "Bounded
    /// read" section) — [`Avatar::MAX_BYTES_DEFAULT`] is the recommended
    /// value absent a stronger opinion. A candidate over the cap, or not a
    /// regular file, is treated exactly like an unreadable one: it falls
    /// through to the next candidate, and if it was the configured path, it
    /// is reported in `configured_failed`.
    ///
    /// `decode` turns one candidate's bytes into a [`Handle`], or `None` for
    /// "could not decode". Supply a real decoder (see the module docs on why
    /// `Handle::from_rgba` and consumer-side decoding, per
    /// saola-lockscreen's `wallpaper::decode`) — not
    /// `Handle::from_bytes`-without-looking, which would defeat both the
    /// fallthrough (an undecodable file must fall through to the next
    /// candidate) and the synchronous-cache workaround.
    pub fn resolve(
        configured: Option<&Path>,
        display_name: &str,
        home: Option<&Path>,
        max_bytes: u64,
        mut decode: impl FnMut(&[u8]) -> Option<Handle>,
    ) -> Resolution {
        let mut configured_failed = None;

        for candidate in avatar_candidates(configured, home) {
            let handle = read_bounded(&candidate, max_bytes).and_then(|bytes| decode(&bytes));
            match handle {
                Some(handle) => {
                    return Resolution {
                        avatar: Avatar::Photo(handle),
                        configured_failed,
                    }
                }
                None => {
                    // Only worth surfacing when the user asked for this
                    // specific file (see `Resolution::configured_failed`).
                    if Some(candidate.as_path()) == configured {
                        configured_failed = Some(candidate);
                    }
                }
            }
        }

        Resolution {
            avatar: Avatar::Initials(initials_from(display_name)),
            configured_failed,
        }
    }
}

/// Read `path` only if it is a regular file of at most `max_bytes` bytes.
///
/// Used by [`Avatar::resolve`] because every candidate path is
/// user-controlled and read before login (see the module's "Bounded read"
/// section) — a device node, FIFO, or oversize file must never be read to
/// completion.
fn read_bounded(path: &Path, max_bytes: u64) -> Option<Vec<u8>> {
    // Open first, then stat the open handle (fstat, not a path-based stat):
    // a path-based check followed by a separate read has a TOCTOU window
    // where the path could be swapped out from under it (e.g. for a symlink
    // to a device file). Stat-ing the handle we're about to read from closes
    // that window.
    let file = File::open(path).ok()?;
    let metadata = file.metadata().ok()?;

    // Reject anything that isn't a plain file: this is what actually keeps
    // out FIFOs, sockets, and device nodes (a stat-only check cannot tell
    // `/dev/zero` from a real file by size alone, since some device nodes
    // report a size of zero). Oversize files are skipped here without
    // reading a single byte of them.
    if !metadata.is_file() || metadata.len() > max_bytes {
        return None;
    }

    let len = metadata.len() as usize;
    let mut buf = Vec::with_capacity(len);
    // `take(max_bytes + 1)` bounds the read regardless of what the fstat
    // said — if the file grew after the fstat and before this read, the
    // reader stops one byte past the cap instead of streaming without
    // limit, and the length check below catches the mismatch.
    file.take(max_bytes + 1).read_to_end(&mut buf).ok()?;

    if buf.len() as u64 > max_bytes {
        return None;
    }

    Some(buf)
}

/// The ordered avatar candidates. Pure function of its inputs (`$HOME` is a
/// parameter, not an environment read) so the §7 precedence is unit-testable
/// without touching the filesystem — the same discipline the lockscreen's
/// `config_dir_from` uses, and its `avatar_candidates` before it.
fn avatar_candidates(configured: Option<&Path>, home: Option<&Path>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = configured {
        candidates.push(path.to_path_buf());
    }
    if let Some(home) = home {
        candidates.push(home.join(".face"));
    }
    candidates
}

/// Up to two initials from a display name, uppercased.
///
/// `"Jordan Dunn"` → `"JD"`; `"jordan"` → `"J"`; an empty or symbol-only
/// name → `"?"`, because §7 wants a disc with *something* in it and a blank
/// disc reads as a rendering bug. Pure, and unit-tested below. (Ported
/// verbatim from saola-lockscreen's `reveal::initials_from`.)
pub fn initials_from(display_name: &str) -> String {
    let initials: String = display_name
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .filter(|c| c.is_alphanumeric())
        .take(2)
        .collect();

    if initials.is_empty() {
        "?".to_string()
    } else {
        initials.to_uppercase()
    }
}

/// One avatar, `size` × `size` logical pixels — the avatar arm of the
/// lockscreen's `Reveal::view`, as a reusable composite. The default call
/// site passes `t.sizes.avatar_lock` (88 px, the lock/greeter avatar circle
/// diameter token).
///
/// A photo is cover-fit: a non-square photo is cropped to the square slot
/// rather than distorted. **Known iced 0.14 constraint** (documented at the
/// lockscreen's call site and flagged in its Stage handoff): iced 0.14 has
/// no way to clip a raster image to a rounded rect — a container's border
/// radius does not clip its children — so a photo avatar renders *square*
/// while the initials fallback is a true disc
/// ([`style::container::disc`], ink surface).
///
/// The initials are set in the UI face at `typography.size.avatar_initials`
/// (the token minted from the lockscreen's choice of `launcher_input` for
/// this slot), in `on_ink.primary` — the disc's own text role, named
/// explicitly because iced's `text` inherits the application default, not
/// its container's color.
///
/// Teaching note (ported from the lockscreen, found live in its nested-niri
/// smoke test): `center_x`/`center_y` take the *length to centre within*,
/// and set it. Passing `Fill` — the obvious-looking spelling — silently
/// overrides any earlier `.width`/`.height` and grows the disc to the whole
/// surface; centring within the avatar's own `size` is both the fix and why
/// there is no separate `.width`/`.height` call on the container.
pub fn view<'a, Message: 'a>(t: &Theme, avatar: &Avatar, size: f32) -> Element<'a, Message> {
    match avatar {
        Avatar::Photo(handle) => image(handle.clone())
            .width(size)
            .height(size)
            .content_fit(ContentFit::Cover)
            .into(),
        Avatar::Initials(initials) => container(
            text(initials.clone())
                .font(ui_font(t))
                .size(t.typography.size.avatar_initials)
                .color(t.on_ink.primary.into_iced()),
        )
        .center_x(size)
        .center_y(size)
        .style(style::container::disc(t, Surface::Ink))
        .into(),
    }
}

/// The placeholder avatar: `Icon::UserRound` centred in the same ink disc
/// [`view`] draws for initials, `size` × `size` logical pixels. For a slot
/// that has no account behind it — the greeter's "Not listed?" tile — so
/// it lines up with the user tiles beside it. Not an [`Avatar`] variant
/// because [`Avatar::resolve`] can never produce one: a real account always
/// has at least initials.
///
/// The glyph is `sizes.avatar_glyph` in `on_ink.primary`, the disc's own
/// text role, matching the initials arm of [`view`].
pub fn placeholder<'a, Message: 'a>(t: &Theme, size: f32) -> Element<'a, Message> {
    container(icon(
        Icon::UserRound,
        t.sizes.avatar_glyph,
        t.on_ink.primary.into_iced(),
    ))
    .center_x(size)
    .center_y(size)
    .style(style::container::disc(t, Surface::Ink))
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- candidate order (ported from saola-lockscreen reveal.rs) --------

    #[test]
    fn avatar_candidates_are_config_then_dot_face() {
        let configured = PathBuf::from("/opt/avatars/jordan.png");
        let candidates = avatar_candidates(Some(&configured), Some(Path::new("/home/jordan")));
        assert_eq!(
            candidates,
            vec![configured, PathBuf::from("/home/jordan/.face")]
        );
    }

    #[test]
    fn avatar_candidates_without_config_are_just_dot_face() {
        let candidates = avatar_candidates(None, Some(Path::new("/home/jordan")));
        assert_eq!(candidates, vec![PathBuf::from("/home/jordan/.face")]);
    }

    /// No config path and no `$HOME`: nothing to try, so the initials disc
    /// is the answer. Must not panic and must not invent a path.
    #[test]
    fn avatar_candidates_can_be_empty() {
        assert!(avatar_candidates(None, None).is_empty());
    }

    // ---- initials (ported from saola-lockscreen reveal.rs) ---------------

    #[test]
    fn initials_take_the_first_two_words() {
        assert_eq!(initials_from("Jordan Dunn"), "JD");
        assert_eq!(initials_from("Jordan Michael Dunn"), "JM");
    }

    #[test]
    fn initials_from_a_single_word() {
        assert_eq!(initials_from("jordan"), "J");
    }

    /// A name with nothing alphanumeric in it still produces a legible disc
    /// rather than an empty one.
    #[test]
    fn initials_fall_back_to_a_question_mark() {
        assert_eq!(initials_from(""), "?");
        assert_eq!(initials_from("   "), "?");
        assert_eq!(initials_from("-- --"), "?");
    }

    // ---- resolution ------------------------------------------------------

    /// A decoder that accepts anything readable — isolates the precedence
    /// and warning logic from any real image format.
    fn accept_all(_bytes: &[u8]) -> Option<Handle> {
        Some(Handle::from_rgba(1, 1, vec![0u8; 4]))
    }

    #[test]
    fn no_candidates_resolves_to_initials_with_no_warning() {
        let resolution = Avatar::resolve(
            None,
            "Jordan Dunn",
            None,
            Avatar::MAX_BYTES_DEFAULT,
            accept_all,
        );
        assert!(matches!(resolution.avatar, Avatar::Initials(ref i) if i == "JD"));
        assert_eq!(resolution.configured_failed, None);
    }

    /// An unreadable *configured* path is the one failure worth a warning —
    /// the user named that file — and it still falls through rather than
    /// failing.
    #[test]
    fn a_failed_configured_path_is_reported_and_falls_through() {
        let configured = PathBuf::from("/nonexistent/avatar.png");
        let resolution = Avatar::resolve(
            Some(&configured),
            "Jordan Dunn",
            None,
            Avatar::MAX_BYTES_DEFAULT,
            accept_all,
        );
        assert!(matches!(resolution.avatar, Avatar::Initials(_)));
        assert_eq!(resolution.configured_failed, Some(configured));
    }

    /// A missing `~/.face` is the normal case for most systems and says
    /// nothing — no warning.
    #[test]
    fn a_missing_dot_face_is_silent() {
        let resolution = Avatar::resolve(
            None,
            "Jordan Dunn",
            Some(Path::new("/nonexistent-home")),
            Avatar::MAX_BYTES_DEFAULT,
            accept_all,
        );
        assert!(matches!(resolution.avatar, Avatar::Initials(_)));
        assert_eq!(resolution.configured_failed, None);
    }

    /// A readable, decodable configured file wins: the decoder sees its
    /// bytes and the result is a photo. (Any readable file will do — the
    /// fake decoder accepts everything, so this pins the read-then-decode
    /// plumbing, not a format.)
    #[test]
    fn a_readable_configured_file_becomes_a_photo() {
        let readable = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
        let resolution = Avatar::resolve(
            Some(&readable),
            "Jordan Dunn",
            None,
            Avatar::MAX_BYTES_DEFAULT,
            accept_all,
        );
        assert!(matches!(resolution.avatar, Avatar::Photo(_)));
        assert_eq!(resolution.configured_failed, None);
    }

    /// An *undecodable* configured file falls through exactly like an
    /// unreadable one — decode failure and read failure are the same case.
    #[test]
    fn an_undecodable_configured_file_falls_through_with_a_warning() {
        let readable = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
        let resolution = Avatar::resolve(
            Some(&readable),
            "Jordan Dunn",
            None,
            Avatar::MAX_BYTES_DEFAULT,
            |_bytes| None,
        );
        assert!(matches!(resolution.avatar, Avatar::Initials(_)));
        assert_eq!(resolution.configured_failed, Some(readable));
    }

    // ---- bounded read ------------------------------------------------------

    /// A configured file over the cap must fall through *without* the
    /// decoder ever seeing its bytes — the whole point of the bound is that
    /// an oversize (or unbounded) candidate is never read to completion.
    #[test]
    fn an_oversize_configured_file_falls_through_with_a_warning() {
        let cap = 64u64;
        let path = std::env::temp_dir().join(format!(
            "saola-theme-avatar-{}-an_oversize_configured_file_falls_through_with_a_warning",
            std::process::id()
        ));
        std::fs::write(&path, vec![0u8; (cap + 1) as usize]).expect("write temp file");

        let calls = std::cell::Cell::new(0u32);
        let resolution = Avatar::resolve(Some(&path), "Jordan Dunn", None, cap, |_bytes| {
            calls.set(calls.get() + 1);
            Some(Handle::from_rgba(1, 1, vec![0u8; 4]))
        });

        std::fs::remove_file(&path).ok();

        assert_eq!(
            calls.get(),
            0,
            "decoder must not be called for an oversize file"
        );
        assert!(matches!(resolution.avatar, Avatar::Initials(_)));
        assert_eq!(resolution.configured_failed, Some(path));
    }

    /// A file at exactly the cap is still read in full and decoded.
    #[test]
    fn a_file_at_exactly_the_cap_is_read() {
        let cap = 64u64;
        let path = std::env::temp_dir().join(format!(
            "saola-theme-avatar-{}-a_file_at_exactly_the_cap_is_read",
            std::process::id()
        ));
        std::fs::write(&path, vec![0u8; cap as usize]).expect("write temp file");

        let seen_len = std::cell::Cell::new(None::<usize>);
        let resolution = Avatar::resolve(Some(&path), "Jordan Dunn", None, cap, |bytes| {
            seen_len.set(Some(bytes.len()));
            Some(Handle::from_rgba(1, 1, vec![0u8; 4]))
        });

        std::fs::remove_file(&path).ok();

        assert_eq!(seen_len.get(), Some(cap as usize));
        assert!(matches!(resolution.avatar, Avatar::Photo(_)));
    }

    // ---- placeholder -------------------------------------------------------

    /// `placeholder` just needs to build without panicking — an `Element`
    /// can't be inspected much further than that.
    #[test]
    fn placeholder_builds() {
        let _element = placeholder::<()>(&Theme::saola(), 88.0);
    }

    #[cfg(unix)]
    #[test]
    fn read_bounded_rejects_non_regular_files() {
        assert!(read_bounded(Path::new("/dev/zero"), 64).is_none());
        // A char device, not a regular file — rejected the same way even
        // though it reads as empty rather than infinite.
        assert!(read_bounded(Path::new("/dev/null"), 64).is_none());
    }

    #[test]
    fn read_bounded_rejects_a_directory() {
        assert!(read_bounded(Path::new(env!("CARGO_MANIFEST_DIR")), 64).is_none());
    }
}
