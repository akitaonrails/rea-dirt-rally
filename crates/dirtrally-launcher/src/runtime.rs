//! The private runtime copy of the game folder (`rea_launcher::runtime`).
//!
//! The player's folder is only read. DiRT Rally's Linux port writes into its
//! own folder (its embedded browser creates `bin/libpdf.so/` at start,
//! docs/RUNTIME_NOTES.md), so the game always runs from a copy the
//! launcher owns.
//!
//! [`GamePolicy`]: the 40 GB of game data under `share/` is only read, so it
//! is hard-linked when the runtime is on the same disk (a near-instant
//! setup) and copied otherwise. `bin/`, `lib/` and `config/` are small and
//! always copied. The game keeps its settings and saves under
//! `$HOME/.local/share/feral-interactive/DiRT Rally/`, never in its folder,
//! so nothing here is game-written. The launcher places nothing in the
//! runtime: its libraries live in its own package.

use std::io;
use std::path::Path;

use rea_launcher::paths;
use rea_launcher::runtime::{self, Policy, Stamp, SyncReport};

/// The runtime's record of what it brought over.
pub const MANIFEST: &str = ".dirtrally-manifest";

/// What DiRT Rally's Linux port does with its files.
pub struct GamePolicy;

/// The read-only game data: `share/…` in any letter case.
fn is_data(rel: &str) -> bool {
    rel.split('/')
        .next()
        .is_some_and(|top| top.eq_ignore_ascii_case("share"))
        && rel.contains('/')
}

impl Policy for GamePolicy {
    fn link(&self, rel: &str) -> bool {
        is_data(rel)
    }

    fn game_written(&self, _rel: &str, _source: Stamp) -> bool {
        false
    }

    /// Feral's controller list: the launcher writes it at every start from
    /// the game's own copy and the connected pads (`start::write_input_devices`).
    fn owned(&self, rel: &str, _source: Stamp) -> bool {
        rel.eq_ignore_ascii_case(INPUT_DEVICES)
    }
}

/// Feral's controller list, relative to the game folder.
pub const INPUT_DEVICES: &str = "share/inputdevices.json";

/// Brings the game folder into `runtime`, which may never overlap it.
pub fn prepare(game: &Path, runtime: &Path) -> Result<SyncReport, String> {
    paths::check_separate(game, runtime)?;
    sync(game, runtime).map_err(|error| format!("Cannot prepare {}: {error}", runtime.display()))
}

fn sync(game: &Path, runtime: &Path) -> io::Result<SyncReport> {
    runtime::sync(game, runtime, &GamePolicy, MANIFEST)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    fn scratch(name: &str) -> std::path::PathBuf {
        let root =
            std::env::temp_dir().join(format!("dirtrally-runtime-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        root
    }

    /// A tiny folder in the Linux port's shape.
    fn feral_layout(game: &Path) {
        fs::create_dir_all(game.join("bin")).unwrap();
        fs::create_dir_all(game.join("lib/x86_64")).unwrap();
        fs::create_dir_all(game.join("share/data/system")).unwrap();
        fs::write(game.join("bin/DirtRally"), b"exe").unwrap();
        fs::set_permissions(
            game.join("bin/DirtRally"),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        fs::write(game.join("lib/x86_64/libSDL2-2.0.5.so"), b"lib").unwrap();
        fs::write(
            game.join("share/data/system/hardware_settings_config.xml"),
            b"xml",
        )
        .unwrap();
    }

    #[test]
    fn only_the_game_data_is_linked() {
        for rel in ["share/data/game.bin", "Share/FeralUI/x.html", "share/a"] {
            assert!(GamePolicy.link(rel), "{rel}");
        }
        for rel in [
            "bin/DirtRally",
            "lib/x86_64/libcef.so",
            "config/x.sh",
            "share",
            "DirtRally.sh",
        ] {
            assert!(!GamePolicy.link(rel), "{rel}");
        }
    }

    #[test]
    fn the_player_starts_from_a_copy_and_their_folder_never_changes() {
        let root = scratch("story");
        let (game, rt) = (root.join("Jogos d'Ação/DiRT Rally"), root.join("runtime"));
        feral_layout(&game);

        let report = prepare(&game, &rt).unwrap();
        assert_eq!(report.brought, 3);
        // Same disk: the data is a hard link, the program a copy.
        let data = "share/data/system/hardware_settings_config.xml";
        assert_eq!(
            fs::metadata(rt.join(data)).unwrap().ino(),
            fs::metadata(game.join(data)).unwrap().ino()
        );
        assert_ne!(
            fs::metadata(rt.join("bin/DirtRally")).unwrap().ino(),
            fs::metadata(game.join("bin/DirtRally")).unwrap().ino()
        );
        // The program stays executable.
        let mode = fs::metadata(rt.join("bin/DirtRally"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o111, 0o111);

        // The game makes its folder in the runtime; the next start keeps it
        // and brings nothing new.
        fs::create_dir_all(rt.join("bin/libpdf.so")).unwrap();
        assert_eq!(prepare(&game, &rt).unwrap().brought, 0);
        assert!(rt.join("bin/libpdf.so").is_dir());
        assert!(!game.join("bin/libpdf.so").exists());

        // A runtime inside the game folder is refused.
        assert!(prepare(&game, &game.join("runtime")).is_err());
        fs::remove_dir_all(&root).unwrap();
    }
}

#[cfg(test)]
mod owned_tests {
    use super::*;

    #[test]
    fn only_feral_s_controller_list_is_the_launcher_s() {
        let stamp = Stamp { size: 1, mtime: 1 };
        assert!(GamePolicy.owned("share/inputdevices.json", stamp));
        assert!(GamePolicy.owned("Share/InputDevices.json", stamp));
        assert!(!GamePolicy.owned("share/data/game.bin", stamp));
        assert!(!GamePolicy.owned("bin/DirtRally", stamp));
    }
}
