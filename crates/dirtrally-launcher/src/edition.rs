//! Which builds of DiRT Rally this launcher knows, by SHA-256
//! (`rea_launcher::identify`).
//!
//! Two families exist: Feral's native Linux port (`bin/DirtRally`) and the
//! Windows DX11 build (`drt.exe`). Each build the owner has gets a row in
//! [`EDITIONS`]. Any other build is refused with its digest and the link to
//! the "unknown edition" issue template (docs/STANDARDS.md, "Every
//! edition").
//!
//! DiRT Rally is no longer sold, and the launcher needs a copy that already
//! runs without Steam (owner decision in AGENTS.md). It runs that copy
//! unchanged: it never adds, replaces or patches Steam or DRM files.

use std::path::{Path, PathBuf};

use rea_launcher::identify::{self, Known};
use rea_launcher::paths;

/// The supported game's executable, for messages ("the folder with …").
pub const GAME_EXE: &str = Family::FeralLinux.exe();

/// Where players report a build the launcher does not know.
pub const UNKNOWN_EDITION_ISSUE: &str =
    "https://github.com/akitaonrails/rea-dirt-rally/issues/new?template=unknown-edition.yml";

/// The game's executable in each family, relative to the game folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// Feral Interactive's native Linux port: `bin/DirtRally`.
    FeralLinux,
    /// The Windows DX11 build: `drt.exe`.
    Windows,
}

impl Family {
    pub const ALL: [Family; 2] = [Family::FeralLinux, Family::Windows];

    /// The executable, relative to the game folder (`/`-separated).
    pub const fn exe(self) -> &'static str {
        match self {
            Family::FeralLinux => "bin/DirtRally",
            Family::Windows => "drt.exe",
        }
    }
}

/// What the launcher knows about one build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edition {
    /// For the log and the window.
    pub name: &'static str,
    pub family: Family,
    /// Why this build cannot be played yet, for the player; `None` when it
    /// can.
    pub blocked: Option<&'static str>,
}

/// Every known build. Hashes come from the owner's copies (docs/RESEARCH.md).
pub static EDITIONS: &[Known<Edition>] = &[
    Known::new(
        "de81c75ce02318d6ba89d3d24c463ac311a823d9eb16e676e7a5fb66d6c9e39e",
        Edition {
            name: "Linux 1.0 (Feral, build 31946)",
            family: Family::FeralLinux,
            blocked: None,
        },
    ),
    Known::new(
        "f7c2dd739e2cd478bfa4bb7238bf121071fb813bc9c84efe3599c80a86d52eea",
        Edition {
            name: "Windows v1.1",
            family: Family::Windows,
            blocked: Some(
                "This Windows copy of DiRT Rally does not start under Wine yet: it stops \
                 while loading, before the game itself runs. The launcher does not change \
                 Steam or copy-protection files, so it cannot fix that. The Linux version of \
                 the game is supported.",
            ),
        },
    ),
];

/// The build of these exe bytes, or their digest when it is unknown.
pub fn identify(bytes: &[u8]) -> Result<&'static Edition, [u8; 32]> {
    identify::identify(bytes, EDITIONS)
}

/// The message for a build that is not in [`EDITIONS`]: what happened, the
/// SHA-256 and where to report it.
pub fn unknown_message(exe: &str, digest: &[u8; 32]) -> String {
    format!(
        "This {exe} is not a build this launcher knows, so it was left alone.\n\n\
         SHA-256: {}\n\n\
         Please report it, with that SHA-256 and where your copy of the game came \
         from, so it can be supported:\n{UNKNOWN_EDITION_ISSUE}",
        identify::hex(digest)
    )
}

/// The game's executable in a folder, and its known build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// The game folder: the one holding `bin/DirtRally` or `drt.exe`.
    pub root: PathBuf,
    /// The exe, spelled as it is on disk.
    pub exe: PathBuf,
    pub edition: &'static Edition,
}

/// Where the game's exe is in `dir`: in `dir` itself, or in its only
/// subfolder (a download that unpacks to `DiRT Rally/…`).
fn locate(dir: &Path) -> Option<(PathBuf, Family, PathBuf)> {
    let here = |root: &Path| {
        Family::ALL.into_iter().find_map(|family| {
            paths::find_ignoring_case(root, family.exe())
                .filter(|exe| exe.is_file())
                .map(|exe| (root.to_path_buf(), family, exe))
        })
    };
    if let Some(found) = here(dir) {
        return Some(found);
    }
    let mut subdirs = std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir());
    match (subdirs.next(), subdirs.next()) {
        (Some(only), None) => here(&only.path()),
        _ => None,
    }
}

/// Looks for the game in `dir` and identifies it. The error is a message
/// for the player: no such folder, no game, or an unknown build
/// ([`unknown_message`]). A known but blocked build is found; the caller
/// decides what to say ([`Edition::blocked`]).
pub fn find(dir: &Path) -> Result<Found, String> {
    if !dir.is_dir() {
        return Err(format!("There is no folder {}.", dir.display()));
    }
    let (root, family, exe) = locate(dir).ok_or_else(|| {
        format!(
            "There is no DiRT Rally in {}: no {} and no {}.",
            dir.display(),
            Family::FeralLinux.exe(),
            Family::Windows.exe()
        )
    })?;
    let digest = identify::sha256_file(&exe)
        .map_err(|error| format!("Cannot read {}: {error}", exe.display()))?;
    match identify::identify_digest(digest, EDITIONS) {
        Ok(edition) if edition.family == family => Ok(Found { root, exe, edition }),
        // A known digest under the other family's name is still unknown here.
        _ => Err(unknown_message(family.exe(), &digest)),
    }
}

/// [`find`], then refuse a blocked build with its reason.
pub fn find_playable(dir: &Path) -> Result<Found, String> {
    let found = find(dir)?;
    match found.edition.blocked {
        Some(reason) => Err(format!(
            "{} ({}).\n\n{reason}",
            found.edition.name,
            found.exe.display()
        )),
        None => Ok(found),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("dirtrally-edition-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn every_known_build_has_its_own_digest() {
        for (i, known) in EDITIONS.iter().enumerate() {
            assert!(!known.value.name.is_empty());
            assert!(
                EDITIONS[i + 1..].iter().all(|k| k.sha256 != known.sha256),
                "{} is listed twice",
                known.value.name
            );
            assert_eq!(
                identify::identify_digest(known.sha256, EDITIONS),
                Ok(&known.value)
            );
        }
    }

    #[test]
    fn the_linux_build_is_playable_and_the_windows_build_says_why_not() {
        let playable: Vec<_> = EDITIONS
            .iter()
            .filter(|k| k.value.blocked.is_none())
            .map(|k| k.value.family)
            .collect();
        assert_eq!(playable, [Family::FeralLinux]);
        let windows = EDITIONS
            .iter()
            .find(|k| k.value.family == Family::Windows)
            .unwrap();
        assert!(windows.value.blocked.unwrap().contains("Linux version"));
    }

    #[test]
    fn an_unknown_build_is_refused_with_its_digest_and_the_issue_link() {
        let digest = identify::sha256(b"not the game");
        assert_eq!(identify(b"not the game"), Err(digest));
        let message = unknown_message("drt.exe", &digest);
        assert!(message.contains(&identify::hex(&digest)), "{message}");
        assert!(message.contains(UNKNOWN_EDITION_ISSUE), "{message}");
    }

    #[test]
    fn the_game_is_found_in_the_folder_or_its_only_subfolder() {
        let dir = temp("layout");
        let root = dir.join("DiRT Rally d'Ação");
        fs::create_dir_all(root.join("bin")).unwrap();
        fs::write(root.join("bin/DirtRally"), b"not the game").unwrap();
        // The folder itself, then its parent with one subfolder.
        for start in [&root, &dir] {
            let error = find(start).unwrap_err();
            assert!(error.contains("bin/DirtRally"), "{error}");
            assert!(error.contains(&identify::hex(&identify::sha256(b"not the game"))));
        }
        // Two subfolders: ambiguous, so nothing is guessed.
        fs::create_dir_all(dir.join("other")).unwrap();
        assert!(find(&dir).unwrap_err().contains("There is no DiRT Rally"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_missing_folder_or_game_is_explained() {
        let dir = temp("empty");
        assert!(
            find(&dir.join("nope"))
                .unwrap_err()
                .contains("There is no folder")
        );
        let error = find(&dir).unwrap_err();
        assert!(
            error.contains("drt.exe") && error.contains("bin/DirtRally"),
            "{error}"
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    /// The owner's real copy (env-gated): `DIRTRALLY_GAME_DIR=… cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn the_owners_copy_is_identified() {
        let Some(dir) = std::env::var_os("DIRTRALLY_GAME_DIR") else {
            return;
        };
        let found = find(Path::new(&dir)).unwrap();
        assert_eq!(found.edition.family, Family::FeralLinux);
        assert!(find_playable(Path::new(&dir)).is_ok());
    }
}
