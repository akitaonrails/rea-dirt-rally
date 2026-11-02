//! The DiRT Rally launcher: everything that can be tested on a Linux host lives
//! in this library; `main.rs` only wires it to Windows.
//!
//! - [`edition`]: which builds of drt.exe are known (SHA-256), and the
//!   refusal for any other build.
//! - [`config`]: the one option table behind `launcher.cfg`, the flags,
//!   `--help`, the template and the window rows.
//! - [`cli`]: what a command line asks for, and the `--check` report.
//! - [`form`]: the settings window's state as `launcher.cfg` text.
//! - [`runtime`]: the private runtime copy of the game folder.
//! - `palette`, `ui`: the settings window (Windows, and the host's tests).

pub mod cli;
pub mod config;
pub mod edition;
pub mod form;
pub mod plan;
pub mod prefs;
pub mod runtime;

#[cfg(any(windows, test))]
pub mod palette;
#[cfg(any(windows, test))]
pub mod ui;
