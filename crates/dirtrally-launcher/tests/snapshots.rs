//! Golden snapshots of everything the launcher produces (SDK STANDARDS,
//! "Golden snapshots"): `--help`, the first-run `launcher.cfg`, and for each
//! look and a few option combinations the start plan — the values written
//! into the game's own `preferences` (a new file and a game-written one),
//! the game's environment (`LD_PRELOAD`, `LD_LIBRARY_PATH` for game-libs,
//! …), Feral's controller list with the pads the launcher adds, and the
//! Linux helper's gamescope command line. No game bytes: the preferences
//! and controller list fed in are synthetic, written for these tests.
//!
//! A diff is reviewed like code. To accept an intended change:
//! `DIRTRALLY_UPDATE_SNAPSHOTS=1 cargo test --test snapshots`, then read
//! the diff.

use std::path::PathBuf;

use dirtrally_launcher::config;

fn check(name: &str, actual: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(name);
    if std::env::var_os("DIRTRALLY_UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "{}: {e} (DIRTRALLY_UPDATE_SNAPSHOTS=1 writes it)",
            path.display()
        )
    });
    // Compared without regard to line endings (a Windows checkout).
    assert_eq!(
        expected.replace("\r\n", "\n"),
        actual,
        "{name} changed: review the difference, then update it on purpose"
    );
}

#[test]
fn help_and_first_run_settings() {
    // The version changes every release; the snapshot keeps a placeholder.
    check(
        "help.txt",
        &format!("{}\n", config::usage()).replace(env!("CARGO_PKG_VERSION"), "<version>"),
    );
    check("launcher.cfg", &config::template());
    check(
        "launcher-first-run.cfg",
        &config::first_run_text(Some((2560, 1440))),
    );
    check(
        "launcher-first-run-steam-deck.cfg",
        &config::first_run_text(Some((1280, 800))),
    );
}

/// The start plan and what it writes. Linux only: the supported edition is
/// Feral's native port, and these are Linux paths and variables.
#[cfg(unix)]
mod linux {
    use super::*;

    use std::path::Path;

    use dirtrally_launcher::config::{Look, Settings};
    use dirtrally_launcher::form::Form;
    use dirtrally_launcher::plan::{Host, Package};
    use dirtrally_launcher::{pads, prefs, start};

    /// The settings a look card saves, as the window writes them.
    fn look(look: Look) -> String {
        let template = config::template();
        let mut form = Form::from_text(&template);
        form.choose_look(look);
        form.save(&template)
    }

    const SCREEN: (u32, u32) = (3840, 2160);
    const DECK: (u32, u32) = (1280, 800);
    const RUNTIME: &str = "/home/José/.local/share/dirtrally-launcher/runtime";

    /// A `preferences` file shaped like the game's (synthetic, trimmed):
    /// the launcher must change only its own values in it.
    const GAME_PREFS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<registry>
    <key name="HKEY_CURRENT_USER">
        <key name="Software">
            <key name="Feral Interactive">
                <key name="DiRT Rally">
                    <key name="Setup">
                        <value name="CrashCounter" type="integer">2</value>
                        <value name="FullScreen" type="integer">1</value>
                        <value name="GameOptionsDialogShouldShow" type="integer">1</value>
                        <value name="ScreenH" type="integer">0</value>
                        <value name="ScreenW" type="integer">0</value>
                    </key>
                </key>
            </key>
        </key>
    </key>
</registry>
"#;

    fn package() -> Package {
        Package {
            compat: PathBuf::from("/opt/DiRT Rally Launcher/game-libs/compat"),
            exec_read: PathBuf::from("/opt/DiRT Rally Launcher/game-libs"),
        }
    }

    fn host() -> Host {
        Host {
            home: PathBuf::from("/home/José"),
            cert_file: Some(PathBuf::from("/etc/ssl/certs/ca-certificates.crt")),
            ..Host::default()
        }
    }

    fn parse(text: &str) -> Settings {
        let (settings, warnings) = config::parse_file(text);
        assert!(warnings.is_empty(), "{warnings:?}");
        settings
    }

    /// One start: `cfg` (launcher.cfg text) with `flags` on top, on `screen`.
    struct Case<'a> {
        title: &'a str,
        cfg: &'a str,
        flags: &'a [&'a str],
        screen: (u32, u32),
        /// The launcher hid Feral's window on an earlier start.
        hidden_before: bool,
        host: Host,
    }

    fn describe(case: &Case) -> String {
        let (flags, _) = config::parse_args(case.flags.iter().map(|f| (*f).to_owned())).unwrap();
        let settings = parse(case.cfg).merged(flags);
        let plan = start::plan(
            &settings,
            case.screen,
            case.hidden_before,
            Path::new(RUNTIME),
            &package(),
            &case.host,
        )
        .unwrap();
        let mut out = format!("# {}\n\n", case.title);
        out.push_str(&format!(
            "flags: {}\nscreen: {}x{}\nhidden before: {}\npicture: {}\nstart: {:?}\n",
            if case.flags.is_empty() {
                "(none)".to_owned()
            } else {
                case.flags.join(" ")
            },
            case.screen.0,
            case.screen.1,
            case.hidden_before,
            settings.picture().describe(),
            plan.start,
        ));
        out.push_str("\n## preferences values set (Setup key)\n");
        for (name, value) in prefs::start_values(plan.start) {
            out.push_str(&format!("{name} = {value:?}\n"));
        }
        out.push_str("\n## preferences, none yet (seeded)\n");
        out.push_str(&prefs::configure(None, plan.start).unwrap());
        out.push_str("\n## preferences, the game's own file\n");
        out.push_str(&prefs::configure(Some(GAME_PREFS), plan.start).unwrap());
        let launch = &plan.launch;
        out.push_str("\n## the game's process\n");
        out.push_str(&format!(
            "program: {}\ncwd: {}\nargs: {:?}\n",
            launch.program.display(),
            launch.cwd.display(),
            launch.args
        ));
        for (key, value) in &launch.set {
            out.push_str(&format!("set {key}={}\n", value.to_string_lossy()));
        }
        for key in &launch.unset {
            out.push_str(&format!("unset {key}\n"));
        }
        out
    }

    #[test]
    fn looks() {
        let cases = [
            (
                "look-original.txt",
                Case {
                    title: "Original: Feral's pre-game window, as shipped",
                    cfg: &look(Look::Original),
                    flags: &[],
                    screen: SCREEN,
                    hidden_before: false,
                    host: host(),
                },
            ),
            (
                "look-recommended.txt",
                Case {
                    title: "Recommended (the first run) on a 3840x2160 screen",
                    cfg: &config::first_run_text(Some(SCREEN)),
                    flags: &[],
                    screen: SCREEN,
                    hidden_before: false,
                    host: host(),
                },
            ),
            (
                "look-steam-deck.txt",
                Case {
                    title: "Steam Deck (the first run on a Deck)",
                    cfg: &config::first_run_text(Some((800, 1280))),
                    flags: &[],
                    screen: DECK,
                    hidden_before: false,
                    host: host(),
                },
            ),
        ];
        for (name, case) in &cases {
            check(name, &describe(case));
        }
    }

    #[test]
    fn option_combinations() {
        let caller = Host {
            ld_library_path: Some("/usr/lib/extra".into()),
            ld_preload: Some("libgamemodeauto.so.0".into()),
            cert_file: None,
            ..host()
        };
        let cases = [
            (
                "combo-back-to-original.txt",
                Case {
                    title: "Back to Original after the launcher had hidden Feral's window",
                    cfg: &look(Look::Original),
                    flags: &[],
                    screen: SCREEN,
                    hidden_before: true,
                    host: host(),
                },
            ),
            (
                "combo-size-flag-and-caller-env.txt",
                Case {
                    title: "Recommended, --resolution 1920x1080 for one run; the caller's \
                            LD_LIBRARY_PATH and LD_PRELOAD (gamemode), no certificate file",
                    cfg: &config::first_run_text(Some(SCREEN)),
                    flags: &["--resolution", "1920x1080"],
                    screen: SCREEN,
                    hidden_before: true,
                    host: caller,
                },
            ),
            (
                "combo-hand-written.txt",
                Case {
                    title: "Hand-written launcher.cfg: Original, then a size (the size wins)",
                    cfg: "game = /games/DiRT Rally\nlook = original\nresolution = 2560x1440\n",
                    flags: &[],
                    screen: SCREEN,
                    hidden_before: false,
                    host: host(),
                },
            ),
            (
                "combo-nothing-set.txt",
                Case {
                    title: "Nothing set (an emptied launcher.cfg): the screen's size",
                    cfg: &config::template(),
                    flags: &[],
                    screen: (2560, 1440),
                    hidden_before: false,
                    host: host(),
                },
            ),
        ];
        for (name, case) in &cases {
            check(name, &describe(case));
        }
    }

    /// A synthetic Feral controller list: one pad Feral knows, one listed
    /// for every product of a vendor.
    const GAME_DEVICES: &str = r#"[
    {
        "Name": "X360 Controller",
        "VendorID": "0x045e",
        "ProductIDs": ["0x028e"],
        "Category": "Gamepad",
        "Type": "Xbox360Gamepad"
    },
    {
        "Name": "Any Logitech wheel",
        "VendorID": "0x046d",
        "ProductIDs": ["*"],
        "Category": "Wheel"
    }
]
"#;

    fn pad(name: &str, vendor: u16, product: u16, keys: &str, abs: &str) -> pads::Pad {
        pads::Pad {
            name: name.into(),
            bus: 3,
            vendor,
            product,
            version: 0x0114,
            keys: keys.into(),
            abs: abs.into(),
        }
    }

    #[test]
    fn controller_list() {
        // xpad's standard layout (BTN_SOUTH…BTN_THUMBR; X/Y/Z/RX/RY/RZ, hat 0).
        let (keys, abs) = ("7cdb000000000000 0 0 0 0", "3003f");
        let connected = [
            pad("8BitDo Ultimate 2, Wireless", 0x2dc8, 0x310b, keys, abs),
            pad("Xbox 360 pad (already listed)", 0x045e, 0x028e, keys, abs),
            pad(
                "Logitech pad (vendor listed with *)",
                0x046d,
                0xc24f,
                keys,
                abs,
            ),
            pad(
                "Flight stick (not a gamepad)",
                0x044f,
                0xb10a,
                "3 0 0 0 0",
                "3",
            ),
        ];
        let mut out = String::from("# Pads connected at start\n\n");
        for pad in &connected {
            out.push_str(&format!(
                "{} ({:04x}:{:04x}): {}\n",
                pad.name,
                pad.vendor,
                pad.product,
                pad.layout().as_deref().unwrap_or("no standard layout")
            ));
        }
        let known = pads::remember(&[], &connected);
        out.push_str("\n## known-pads.json\n");
        out.push_str(&serde_json::to_string_pretty(&known).unwrap());
        out.push_str("\n\n## share/inputdevices.json in the private copy\n");
        out.push_str(&pads::input_devices(GAME_DEVICES, &known).unwrap());
        out.push_str("\n## the same list when no pad is new\n");
        let unchanged = pads::input_devices(GAME_DEVICES, &known[..0]).unwrap();
        assert_eq!(unchanged, GAME_DEVICES, "the game's own bytes, unchanged");
        out.push_str("(the game's own file, byte for byte)\n");
        check("controller-list.txt", &out);
    }

    /// The Linux helper's gamescope command line for one `launcher.cfg`
    /// and helper arguments: linux/dirtrally-play loaded as its tests load
    /// it, with the launch replaced by a recorder (`REA_LAUNCH_CMD`), on a
    /// 3840x2160 screen, from an empty environment.
    fn helper(cfg: &str, args: &[&str]) -> String {
        let root = std::env::temp_dir()
            .join(format!("dirtrally-snap-helper-{}", std::process::id()))
            .join("home");
        let _ = std::fs::remove_dir_all(&root);
        let config = root.join(".config/dirtrally-launcher");
        std::fs::create_dir_all(&config).unwrap();
        std::fs::write(config.join("launcher.cfg"), cfg).unwrap();
        let recorder = root.join("record");
        std::fs::write(
            &recorder,
            "#!/bin/sh\necho '--- launch'\nfor a in \"$@\"; do printf '%s\\n' \"$a\"; done\n",
        )
        .unwrap();
        let exe = root.join("DiRTRallyLauncher");
        std::fs::write(&exe, "#!/bin/sh\n").unwrap();
        for file in [&recorder, &exe] {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../linux/dirtrally-play");
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(
                r#"script=$1; shift; . "$script" || exit 9
                REA_TEST=1 REA_GAMESCOPE_CHECK=true REA_LAUNCH_CMD=$REC
                rea_start "$@""#,
            )
            .arg("sh")
            .arg(&script)
            .args(args)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", &root)
            .env("REC", &recorder)
            .env("REA_FUNCTIONS_ONLY", "1")
            .env("REA_PLAY_HERE", script.parent().unwrap())
            .env("DIRTRALLY_LAUNCHER", &exe)
            .env("DIRTRALLY_GPU", "")
            .env("DIRTRALLY_SCREEN", "3840x2160")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "{stdout}{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let _ = std::fs::remove_dir_all(root.parent().unwrap());
        let stdout = stdout.replace(&root.display().to_string(), "<home>");
        // The helper's own lines, then the command as one shell line.
        let (said, command) = stdout.split_once("--- launch\n").expect("no launch");
        let quoted: Vec<String> = command
            .lines()
            .map(|arg| {
                if arg.is_empty() || arg.contains([' ', '\'', '"']) {
                    format!("'{}'", arg.replace('\'', r"'\''"))
                } else {
                    arg.to_owned()
                }
            })
            .collect();
        format!("{said}{}\n", quoted.join(" "))
    }

    #[test]
    fn helper_command_lines() {
        let mut out = String::from(
            "# linux/dirtrally-play --play on a 3840x2160 screen, gamescope installed\n",
        );
        let first = config::first_run_text(Some(SCREEN));
        let cases: [(&str, String, &[&str]); 7] = [
            ("Recommended (first run)", first.clone(), &[]),
            ("Original", look(Look::Original), &[]),
            ("Steam Deck", look(Look::SteamDeck), &[]),
            (
                "Hand-written size 2560x1440",
                "resolution = 2560x1440\n".into(),
                &[],
            ),
            (
                "Hand-written look = steam_deck, no resolution",
                "look = steam_deck\n".into(),
                &[],
            ),
            (
                "Recommended, --look steam_deck for one run",
                first.clone(),
                &["--look", "steam_deck"],
            ),
            (
                "Recommended, --resolution 1280x720 for one run",
                first,
                &["--resolution", "1280x720"],
            ),
        ];
        for (title, cfg, args) in &cases {
            out.push_str(&format!("\n## {title}\n"));
            out.push_str(&helper(cfg, args));
        }
        check("helper-gamescope.txt", &out);
    }
}
