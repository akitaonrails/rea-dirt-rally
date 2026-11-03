//! What the native Linux launcher tells the player when it stops before
//! (or instead of) the game: the SDK's message window
//! (`rea_launcher_ui::report::fatal`), with the unknown-edition form as its
//! link when the game folder is the problem.
//!
//! Who sees it comes from the SDK's audience rule ([`mode`]): a window for
//! a person on a desktop (a double-clicked AppImage, the helper's game run
//! inside gamescope), the terminal or only `launcher.log` for scripts.

use rea_launcher_ui::report::{Link, Message, Mode};
use rea_launcher_win32::report::{self as audience, Audience};

use crate::edition::UNKNOWN_EDITION_ISSUE;

/// The window's title and the printed prefix.
pub const TITLE: &str = "DiRT Rally";

/// The link button's text.
pub const REPORT_EDITION: &str = "Report the edition";

/// A second start while a launcher (or the game it started) runs.
pub const ALREADY_RUNNING: &str = "The game or its launcher is already running, so nothing \
     was started. Close the game or the launcher's window, then start it again.";

/// Why a run stopped, in the player's words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stop {
    pub text: String,
    /// The game folder is the problem (none chosen, no game in it, an
    /// unknown or blocked build): the window links the unknown-edition form.
    pub game: bool,
}

impl Stop {
    /// The game folder can't be played. The form's address, which the
    /// edition text ends with, moves to the link button.
    pub fn game(text: impl Into<String>) -> Stop {
        let text = text.into();
        let text = text
            .strip_suffix(&format!(":\n{UNKNOWN_EDITION_ISSUE}"))
            .map(|kept| format!("{kept}."))
            .unwrap_or(text);
        Stop { text, game: true }
    }

    /// Anything else (a file that can't be read or written, a failed copy).
    pub fn other(text: impl Into<String>) -> Stop {
        Stop {
            text: text.into(),
            game: false,
        }
    }

    /// The SDK message: `details` (version, log) goes to Copy details.
    pub fn message<'a>(&'a self, details: &'a str) -> Message<'a> {
        Message {
            title: TITLE,
            text: &self.text,
            link: self.game.then_some(Link {
                label: REPORT_EDITION,
                url: UNKNOWN_EDITION_ISSUE,
            }),
            details: Some(details),
        }
    }
}

impl From<String> for Stop {
    fn from(text: String) -> Stop {
        Stop::other(text)
    }
}

/// The native report mode for a run's [`Audience`] (from
/// `Invocation::audience(false)`).
pub fn mode(audience: Audience) -> Mode {
    match audience.mode() {
        audience::Mode::Command { interactive } => Mode::Command { interactive },
        audience::Mode::Play { interactive } => Mode::Play { interactive },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli;
    use crate::palette::{METRICS, PALETTE};
    use rea_launcher_ui::egui::Role;
    use rea_launcher_ui::report::{
        MessageState, Native, NativeAudience, message_ui, native_audience,
    };
    use rea_launcher_ui::testing::{self, Queryable};

    fn mode_of(args: &[&str]) -> Mode {
        let invocation = cli::parse(args.iter().map(|arg| (*arg).to_owned())).unwrap();
        mode(invocation.audience(false))
    }

    const DESKTOP_NO_TERMINAL: Native = Native {
        stdin_terminal: false,
        stderr_terminal: false,
        display: true,
    };

    #[test]
    fn an_unknown_build_links_the_form_once() {
        let text = crate::edition::unknown_message("bin/DirtRally", &[7; 32]);
        let stop = Stop::game(text);
        assert!(!stop.text.contains(UNKNOWN_EDITION_ISSUE), "{}", stop.text);
        assert!(
            stop.text.ends_with("so it can be supported."),
            "{}",
            stop.text
        );
        let message = stop.message("DiRTRallyLauncher 0.1.0");
        assert_eq!(message.link.unwrap().url, UNKNOWN_EDITION_ISSUE);
        assert!(message.printed().contains(UNKNOWN_EDITION_ISSUE));
        assert!(message.details_text().contains("DiRTRallyLauncher 0.1.0"));
        assert!(Stop::other("disk full").message("").link.is_none());
    }

    #[test]
    fn the_window_shows_the_text_the_link_and_ok() {
        let stop = Stop::game("There is no DiRT Rally in /x: no bin/DirtRally and no drt.exe.");
        let mut ui = testing::harness(
            &PALETTE,
            &METRICS,
            (stop, MessageState::default()),
            |ui, (stop, state)| {
                message_ui(ui, &stop.message("details"), state);
            },
        );
        ui.run();
        assert!(
            ui.query_by_label_contains("There is no DiRT Rally")
                .is_some()
        );
        assert!(
            ui.query_by_role_and_label(Role::Link, REPORT_EDITION)
                .is_some()
        );
        assert!(ui.query_by_role_and_label(Role::Button, "OK").is_some());
    }

    /// The helper's game run and a double-clicked launcher get the window;
    /// scripts and the helper's own window/check runs don't.
    #[test]
    fn who_gets_the_window() {
        let window = |args: &[&str]| {
            native_audience(mode_of(args), DESKTOP_NO_TERMINAL) == NativeAudience::Window
        };
        assert!(window(&["--no-window", "--linux"]));
        assert!(window(&[]));
        assert!(!window(&["--no-window"]), "a script");
        assert!(!window(&["--settings", "--linux"]));
        assert!(!window(&["--check", "--linux"]));
        let headless = Native {
            display: false,
            ..DESKTOP_NO_TERMINAL
        };
        assert_eq!(
            native_audience(mode_of(&["--no-window", "--linux"]), headless),
            NativeAudience::LogOnly
        );
    }
}
