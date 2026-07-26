//! Prints an Alacritty `colors.*` config for a Saola theme to stdout.
//!
//! Usage: `saola-export-alacritty [theme.toml]`
//!
//! With no argument, renders the built-in `Theme::saola()`. With one
//! argument, loads that path as a (possibly partial — see
//! `saola_tokens::Theme::from_toml_str`) theme TOML file and renders that
//! instead. No `clap`: a single optional positional argument doesn't earn a
//! new dependency (see the Boundaries section of `CLAUDE.md`).

use std::process::ExitCode;

use saola_tokens::Theme;

fn main() -> ExitCode {
    let theme = match std::env::args().nth(1) {
        None => Theme::saola(),
        Some(path) => match Theme::from_path(&path) {
            Ok(theme) => theme,
            Err(err) => {
                eprintln!("saola-export-alacritty: failed to load theme {path:?}: {err}");
                return ExitCode::FAILURE;
            }
        },
    };

    print!("{}", saola_export_alacritty::render(&theme));
    ExitCode::SUCCESS
}
