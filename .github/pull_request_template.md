<!--
The PR title is the Conventional Commit, e.g. `feat: add the split divider style`.
It becomes the squash commit and the changelog line; CI checks it.
A `BREAKING CHANGE:` footer goes at the end of this body.

Use `Closes #<n>` for an ordinary task.
Use `Refs #<n>` for a `theme-request`: that issue stays open until the release
tag that ships it exists (docs/DISPATCH.md, "Release").
-->

<!-- `Closes #<n>` for a task, `Refs #<n>` for a theme-request. -->
Closes #

## What and why

## Validation

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo build -p saola-theme --example gallery`
- [ ] `cargo tree -p saola-tokens -e normal` shows only serde/toml/thiserror

## Visual check (run by the orchestrator after merge)

<!-- Gallery specimens to look at, on both surfaces. Worded results only; screenshots stay local. -->

## Notes for the next task

<!-- New public API (paths and signatures), token names, and anything a consumer must do on the tag bump. -->
