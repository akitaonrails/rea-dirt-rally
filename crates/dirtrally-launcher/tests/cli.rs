//! The command line end to end: every flag, `--help`, bad values and their
//! messages, `--check`; on Windows (and under `sdk/scripts/wine-test`) the
//! real DiRTRallyLauncher program too (Linux), isolated in a scratch HOME.

use std::path::{Path, PathBuf};

use dirtrally_launcher::cli::{self, Invocation, Mode};
use dirtrally_launcher::config::{self, OPTIONS, Settings};
use dirtrally_launcher::edition;
use rea_launcher::cfg::{Kind, Resolution};
use rea_launcher::identify;

fn parse(args: &[&str]) -> Result<Invocation, String> {
    cli::parse(args.iter().map(|arg| (*arg).to_owned()))
}

fn mode(args: &[&str]) -> Mode {
    parse(args).unwrap().mode
}

/// A scratch folder with a hostile name (spaces, apostrophe, accents).
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("dirtrally-cli-{name}-{}", std::process::id()))
        .join("Jogos d'Ação");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn modes_follow_the_family_flags() {
    assert_eq!(
        mode(&[]),
        Mode::Window {
            settings_only: false
        }
    );
    assert_eq!(
        mode(&["--settings"]),
        Mode::Window {
            settings_only: true
        }
    );
    assert_eq!(mode(&["--no-window"]), Mode::Play);
    assert_eq!(
        mode(&["--resolution", "fullscreen"]),
        Mode::Play,
        "an option flag plays"
    );
    assert_eq!(mode(&["--check", "--game", "C:\\Games"]), Mode::Check);
    for help in ["--help", "-h", "/?"] {
        assert_eq!(mode(&[help]), Mode::Help);
    }
    let helper = parse(&["--settings", "--linux"]).unwrap();
    assert!(helper.linux && !helper.wants_console());
    assert!(parse(&["--check"]).unwrap().wants_console());
}

/// Who sees a message comes from the SDK's rule (report::Audience), never
/// from `--no-window` or `--linux` alone (STANDARDS "Messages reach the
/// player").
#[test]
fn the_player_sees_the_errors_of_the_helpers_game_run() {
    let watching = |args: &[&str], wine: bool| parse(args).unwrap().audience(wine).interactive;
    // linux/dirtrally-play and the AppImage start the game run like this,
    // inside gamescope, for a player in front of the screen.
    assert!(watching(&["--no-window", "--linux"], false));
    assert!(watching(&["--no-window", "--linux"], true));
    assert!(
        watching(&["--no-window"], true),
        "under Wine nobody reads stdout"
    );
    assert!(watching(&[], false), "a double-clicked launcher");
    assert!(!watching(&["--no-window"], false), "a script");
    // The helper's settings window shows its own messages.
    assert!(!watching(&["--settings", "--linux"], false));
    assert!(!watching(&["--settings", "--linux"], true));
    // The helper reads --check's output and exit code itself.
    assert!(!watching(&["--check", "--linux"], false));
    let check = parse(&["--check"]).unwrap().audience(false);
    assert!(check.command && check.interactive);
    let help = parse(&["--help"]).unwrap().audience(false);
    assert!(help.command);
}

#[test]
fn every_option_has_a_working_flag() {
    for row in OPTIONS {
        let number;
        let value = match row.kind {
            Kind::Toggle => "off",
            Kind::Choice(choices) => choices[0].0,
            Kind::Size { .. } => "1280x720",
            Kind::Range { min, .. } => {
                number = min.to_string();
                &number
            }
            Kind::Path | Kind::Text => "C:\\Games\\Some Game",
        };
        let flag = row.flag_name(config::STYLE);
        let parsed = parse(&[&flag, value]).unwrap_or_else(|e| panic!("{flag} {value}: {e}"));
        assert_ne!(
            parsed.flags,
            Settings::default(),
            "{flag} {value} set nothing"
        );
        assert!(config::usage().contains(&flag), "--help lacks {flag}");
    }
    let flags = parse(&["--look", "original", "--resolution", "800X600"])
        .unwrap()
        .flags;
    assert_eq!(flags.picture().resolution, Some(Resolution::Size(800, 600)));
}

#[test]
fn bad_command_lines_say_what_is_wrong() {
    let message = |args: &[&str]| parse(args).unwrap_err();
    assert!(message(&["--look", "shiny"]).contains("original, recommended or steam_deck"));
    assert!(message(&["--resolution", "10x10"]).contains("WIDTHxHEIGHT"));
    assert!(message(&["--resolution"]).contains("--resolution needs a value"));
    assert!(message(&["--frobnicate"]).contains("unknown argument --frobnicate"));
    assert!(message(&["--frobnicate"]).contains("--help"));
    assert!(message(&["--settings", "--no-window"]).contains("cannot be used together"));
}

#[test]
fn help_lists_every_option_and_switch() {
    let usage = config::usage();
    for switch in config::SWITCHES.iter().chain(&["--help"]) {
        assert!(usage.contains(switch), "{switch}");
    }
    assert!(usage.contains("--resolution"));
    assert!(usage.contains("--look original|recommended|steam_deck"));
}

#[test]
fn check_needs_a_game_folder_with_a_known_edition() {
    let cwd = Path::new("/");
    let none = cli::check(&Settings::default(), cwd).unwrap_err();
    assert!(none.contains("--game"), "{none}");

    let dir = scratch("check");
    let game = parse(&["--check", "--game", &dir.display().to_string()])
        .unwrap()
        .flags;
    let missing = cli::check(&game, cwd).unwrap_err();
    assert!(missing.contains("There is no"), "{missing}");

    std::fs::write(dir.join("drt.exe"), b"some other build").unwrap();
    let unknown = cli::check(&game, cwd).unwrap_err();
    let digest = identify::hex(&identify::sha256(b"some other build"));
    assert!(unknown.contains(&digest), "{unknown}");
    assert!(
        unknown.contains(edition::UNKNOWN_EDITION_ISSUE),
        "{unknown}"
    );
}

/// The real program, as a player's terminal or the Linux helper runs it,
/// with HOME and the XDG folders in a scratch folder.
#[cfg(unix)]
mod program {
    use super::*;
    use std::process::{Command, Output};

    fn run(name: &str, args: &[&str]) -> (Output, PathBuf) {
        let home = scratch(name).join("home");
        let output = Command::new(env!("CARGO_BIN_EXE_DiRTRallyLauncher"))
            .args(args)
            .env("HOME", &home)
            .env("XDG_CONFIG_HOME", home.join(".config"))
            .env("XDG_DATA_HOME", home.join(".local/share"))
            .env_remove("DIRTRALLY_GAME_LIBS")
            // Headless: the window can never open on the desktop.
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .output()
            .unwrap();
        (output, home.join(".config/dirtrally-launcher/launcher.log"))
    }

    fn text(bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).into_owned()
    }

    #[test]
    fn help_goes_to_the_terminal() {
        let (output, _) = run("help", &["--help"]);
        assert_eq!(output.status.code(), Some(cli::OK));
        assert!(text(&output.stdout).contains("--look original|recommended|steam_deck"));
    }

    #[test]
    fn a_bad_flag_exits_2_with_its_message() {
        let (output, log) = run("bad", &["--look", "shiny"]);
        assert_eq!(output.status.code(), Some(cli::USAGE_ERROR));
        assert!(text(&output.stderr).contains("original, recommended or steam_deck"));
        assert!(std::fs::read_to_string(log).unwrap().contains("--look"));
    }

    #[test]
    fn check_refuses_an_unknown_edition_with_its_digest() {
        let dir = scratch("unknown-game");
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        std::fs::write(dir.join("bin/DirtRally"), b"some other build").unwrap();
        let game = dir.display().to_string();
        let (output, log) = run("unknown", &["--check", "--game", &game]);
        assert_eq!(output.status.code(), Some(cli::REFUSED));
        let digest = identify::hex(&identify::sha256(b"some other build"));
        assert!(text(&output.stderr).contains(&digest));
        let log = std::fs::read_to_string(log).unwrap();
        assert!(
            log.contains(&digest) && log.contains(edition::UNKNOWN_EDITION_ISSUE),
            "{log}"
        );
    }

    /// The helper's game run with an unknown build: refused with exit 1,
    /// the text and the unknown-edition form printed for the log (a window
    /// too on a desktop; these runs have none).
    #[test]
    fn the_helpers_game_run_reports_an_unknown_build_with_the_form() {
        let dir = scratch("unknown-play-game");
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        std::fs::write(dir.join("bin/DirtRally"), b"some other build").unwrap();
        let game = dir.display().to_string();
        let (output, log) = run("unknown-play", &["--no-window", "--linux", "--game", &game]);
        assert_eq!(output.status.code(), Some(cli::REFUSED));
        let stderr = text(&output.stderr);
        assert!(
            stderr.starts_with("DiRT Rally: This bin/DirtRally"),
            "{stderr}"
        );
        assert!(
            stderr.contains(&format!(
                "Report the edition: {}",
                edition::UNKNOWN_EDITION_ISSUE
            )),
            "{stderr}"
        );
        let digest = identify::hex(&identify::sha256(b"some other build"));
        assert!(std::fs::read_to_string(log).unwrap().contains(&digest));
    }

    #[test]
    fn no_game_folder_yet_is_reported_with_the_form() {
        let (output, _) = run("no-game", &["--no-window", "--linux"]);
        assert_eq!(output.status.code(), Some(cli::REFUSED));
        let stderr = text(&output.stderr);
        assert!(stderr.contains("No game folder yet"), "{stderr}");
        assert!(stderr.contains(edition::UNKNOWN_EDITION_ISSUE), "{stderr}");
    }

    #[test]
    fn settings_only_says_no_window_is_possible_yet() {
        let (output, _) = run("settings", &["--settings", "--linux"]);
        assert_eq!(output.status.code(), Some(cli::WINDOW_UNAVAILABLE));
    }
}
