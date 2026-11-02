//! Where things live on Linux, and the steps before the game starts that
//! touch the player's files: the game's `preferences`.
//!
//! - Settings and log: `$XDG_CONFIG_HOME/dirtrally-launcher/`
//!   (`~/.config/dirtrally-launcher/`), as the Linux helper expects.
//! - The private game copy: `$XDG_DATA_HOME/dirtrally-launcher/runtime`
//!   (`~/.local/share/dirtrally-launcher/runtime`) unless `runtime` is set.
//! - What the package ships for the game: `game-libs/` beside the
//!   launcher (legacy libraries in `game-libs/compat`, the exec-read
//!   library in `game-libs`). `DIRTRALLY_GAME_LIBS` names another folder
//!   (a development build). These are kept off the launcher's own
//!   `LD_LIBRARY_PATH` on purpose: the old libraries would break the
//!   settings window and gamescope.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use rea_launcher::cfg::Resolution;
use rea_launcher::fsx;

use crate::plan::{self, Package};
use crate::prefs;

/// The launcher's folder name under the XDG folders.
pub const APP_DIR: &str = "dirtrally-launcher";

/// The package folder with the game's libraries, beside the launcher.
pub const GAME_LIBS_DIR: &str = "game-libs";

/// Names another game-libs folder.
pub const GAME_LIBS_ENV: &str = "DIRTRALLY_GAME_LIBS";

/// An XDG base folder: the variable when it is an absolute path, else
/// `$HOME/<fallback>`.
fn xdg(var: Option<OsString>, home: &Path, fallback: &str) -> PathBuf {
    var.map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| home.join(fallback))
}

/// The launcher's folders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Places {
    /// `launcher.cfg` and `launcher.log`.
    pub config: PathBuf,
    /// The default runtime folder.
    pub runtime: PathBuf,
}

/// [`Places`] for `home` and the two XDG variables.
pub fn places(home: &Path, config_home: Option<OsString>, data_home: Option<OsString>) -> Places {
    Places {
        config: xdg(config_home, home, ".config").join(APP_DIR),
        runtime: xdg(data_home, home, ".local/share")
            .join(APP_DIR)
            .join("runtime"),
    }
}

/// What the package ships for the game: from [`GAME_LIBS_ENV`], else
/// `game-libs` beside `exe`.
pub fn package(exe: &Path, game_libs: Option<OsString>) -> Package {
    let root = game_libs
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| exe.parent().unwrap_or(Path::new(".")).join(GAME_LIBS_DIR));
    Package {
        compat: root.join("compat"),
        exec_read: root,
    }
}

/// What is missing from `package`, for the player: the launcher was
/// unpacked without its `game-libs` folder.
pub fn check_package(package: &Package) -> Result<(), String> {
    let lib = package.exec_read.join(plan::EXEC_READ_LIB);
    if lib.is_file() && package.compat.is_dir() {
        return Ok(());
    }
    Err(format!(
        "The launcher's game libraries are missing ({} and {}). Unpack the whole \
         download again: keep the {GAME_LIBS_DIR} folder next to the launcher.",
        lib.display(),
        package.compat.display()
    ))
}

/// Marks, in the launcher's config folder, that the launcher hid Feral's
/// pre-game window: going back to the Original look brings it back once.
pub const HIDDEN_MARKER: &str = "feral-window-hidden";

/// How the game starts for `resolution` on `screen`: `original` keeps
/// Feral's pre-game window; `fullscreen` (or unset) is the screen's size.
pub fn start_for(
    resolution: Option<Resolution>,
    screen: (u32, u32),
    restore: bool,
) -> prefs::Start {
    match resolution {
        Some(Resolution::Original) => prefs::Start::FeralWindow { restore },
        Some(Resolution::Size(width, height)) => prefs::Start::Direct { width, height },
        Some(Resolution::Fullscreen) | None => prefs::Start::Direct {
            width: screen.0,
            height: screen.1,
        },
    }
}

/// The screen size from `xrandr --current` output (`Screen 0: … current
/// 1920 x 1080, …`). Inside gamescope this is the game's size.
pub fn screen_from_xrandr(output: &str) -> Option<(u32, u32)> {
    let line = output.lines().find(|line| line.starts_with("Screen "))?;
    let current = line
        .split(',')
        .find_map(|part| part.trim().strip_prefix("current "))?;
    let (w, h) = current.split_once(" x ")?;
    let size = (w.trim().parse().ok()?, h.trim().parse().ok()?);
    (size.0 > 0 && size.1 > 0).then_some(size)
}

/// Sets the launcher's values for `start` in the game's own `preferences`
/// under `home` ([`prefs::configure`]), creating the file when the game has
/// none, and keeps [`HIDDEN_MARKER`] in `config` up to date. Returns the file.
pub fn write_prefs(home: &Path, config: &Path, start: prefs::Start) -> Result<PathBuf, String> {
    let path = plan::prefs_dir(home).join("preferences");
    let existing = match std::fs::read(&path) {
        Ok(bytes) => Some(String::from_utf8_lossy(&bytes).into_owned()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(format!("Cannot read {}: {error}", path.display())),
    };
    let text = prefs::configure(existing.as_deref(), start)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if existing.as_deref() != Some(text.as_str()) {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|error| format!("Cannot create {}: {error}", dir.display()))?;
        }
        fsx::write_atomic(&path, &text)
            .map_err(|error| format!("Cannot save {}: {error}", path.display()))?;
    }
    let marker = config.join(HIDDEN_MARKER);
    let result = match start {
        prefs::Start::Direct { .. } => {
            std::fs::create_dir_all(config).and_then(|()| std::fs::write(&marker, b""))
        }
        prefs::Start::FeralWindow { .. } => match std::fs::remove_file(&marker) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => other,
        },
    };
    result.map_err(|error| format!("Cannot update {}: {error}", marker.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn folders_follow_xdg_and_fall_back_to_home() {
        let home = Path::new("/home/José");
        let p = places(home, None, None);
        assert_eq!(p.config, Path::new("/home/José/.config/dirtrally-launcher"));
        assert_eq!(
            p.runtime,
            Path::new("/home/José/.local/share/dirtrally-launcher/runtime")
        );
        let p = places(home, Some("/cfg".into()), Some("/data".into()));
        assert_eq!(p.config, Path::new("/cfg/dirtrally-launcher"));
        assert_eq!(p.runtime, Path::new("/data/dirtrally-launcher/runtime"));
        // A relative XDG value is ignored, as the spec says.
        let p = places(home, Some("relative".into()), Some("".into()));
        assert_eq!(p.config, Path::new("/home/José/.config/dirtrally-launcher"));
    }

    #[test]
    fn the_game_libraries_sit_beside_the_launcher_unless_named() {
        let exe = Path::new("/opt/DiRT Rally Launcher/DiRTRallyLauncher");
        let package = package(exe, None);
        assert_eq!(
            package.exec_read,
            Path::new("/opt/DiRT Rally Launcher/game-libs")
        );
        assert_eq!(
            package.compat,
            Path::new("/opt/DiRT Rally Launcher/game-libs/compat")
        );
        let package = super::package(exe, Some("/dev/libs".into()));
        assert_eq!(package.compat, Path::new("/dev/libs/compat"));
        let package = super::package(exe, Some("".into()));
        assert_eq!(
            package.exec_read,
            Path::new("/opt/DiRT Rally Launcher/game-libs")
        );
    }

    #[test]
    fn a_launcher_without_its_libraries_says_what_to_do() {
        let root = std::env::temp_dir().join(format!("dirtrally-start-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let package = package(&root.join("DiRTRallyLauncher"), None);
        let error = check_package(&package).unwrap_err();
        assert!(error.contains("game-libs"), "{error}");
        fs::create_dir_all(&package.compat).unwrap();
        fs::write(package.exec_read.join(plan::EXEC_READ_LIB), b"lib").unwrap();
        assert!(check_package(&package).is_ok());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_resolution_decides_how_the_game_starts() {
        use prefs::Start;
        let screen = (3840, 2160);
        assert_eq!(
            start_for(Some(Resolution::Original), screen, true),
            Start::FeralWindow { restore: true }
        );
        assert_eq!(
            start_for(Some(Resolution::Size(1280, 800)), screen, false),
            Start::Direct {
                width: 1280,
                height: 800
            }
        );
        for unset in [Some(Resolution::Fullscreen), None] {
            assert_eq!(
                start_for(unset, screen, false),
                Start::Direct {
                    width: 3840,
                    height: 2160
                }
            );
        }
    }

    #[test]
    fn the_screen_size_is_read_from_xrandr() {
        let out = "Screen 0: minimum 16 x 16, current 2560 x 1440, maximum 32767 x 32767\n\
                   XWAYLAND0 connected 2560x1440+0+0\n";
        assert_eq!(screen_from_xrandr(out), Some((2560, 1440)));
        assert_eq!(screen_from_xrandr("Can't open display"), None);
        assert_eq!(screen_from_xrandr("Screen 0: current 0 x 0,"), None);
    }

    #[test]
    fn the_player_starts_fresh_then_keeps_their_game_settings() {
        let root =
            std::env::temp_dir().join(format!("dirtrally-prefs-José-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let (home, config) = (root.join("home"), root.join("config"));
        let direct = prefs::Start::Direct {
            width: 1920,
            height: 1080,
        };
        // No preferences yet: seeded, and the hidden window is remembered.
        let path = write_prefs(&home, &config, direct).unwrap();
        let seeded = fs::read_to_string(&path).unwrap();
        assert!(seeded.contains("LnxDistributionUnsupported") && seeded.contains("\"ScreenW\""));
        assert!(config.join(HIDDEN_MARKER).is_file());
        // The game adds its own values; the launcher keeps them all.
        let played = seeded.replace(
            "                    </key>\n                </key>",
            "                        <value name=\"FullScreen\" type=\"integer\">0</value>\n                    </key>\n                </key>",
        );
        fs::write(&path, &played).unwrap();
        write_prefs(&home, &config, direct).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), played);
        // Back to Original: the window returns once and the marker goes.
        let original = start_for(
            Some(Resolution::Original),
            (1920, 1080),
            config.join(HIDDEN_MARKER).is_file(),
        );
        write_prefs(&home, &config, original).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            text.contains("\"GameOptionsDialogShouldShow\" type=\"integer\">1<"),
            "{text}"
        );
        assert!(!config.join(HIDDEN_MARKER).exists());
        fs::remove_dir_all(&root).unwrap();
    }
}
