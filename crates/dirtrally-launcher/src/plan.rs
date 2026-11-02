//! How the Linux port is started: the program, its folder and its
//! environment, as one value ([`Launch`]) that the tests can compare.
//!
//! This mirrors what Feral's own `DirtRally.sh` sets up, plus what a
//! modern system needs (docs/RESEARCH.md #3–#7):
//!
//! - `LD_LIBRARY_PATH`: the runtime's `lib/x86_64` (glibc 2.37 and later no
//!   longer find it on their own), then the launcher's legacy libraries
//!   (OpenSSL 1.0, libidn11, … that modern systems dropped), then the
//!   launcher's own folder, then whatever the caller had.
//! - `LD_PRELOAD`: the game's own `libsteam_api.so`, as the game's script
//!   does, and the launcher's exec-read library (the PKU crash fix). Both
//!   by name only, found through `LD_LIBRARY_PATH`, because `LD_PRELOAD`
//!   splits on spaces and a path may hold one.
//! - `SteamAppId`/`GameAppId`, the OpenGL shader cache folder and the
//!   certificate file, as the game's script sets them; `LC_ALL` removed.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The exec-read library's file name (crate `dirtrally-exec-read`).
pub const EXEC_READ_LIB: &str = "libdirtrally_exec_read.so";

/// The game's own Steam API library, preloaded as its script does.
pub const STEAM_API_LIB: &str = "libsteam_api.so";

/// DiRT Rally's Steam app id, which the game expects in its environment.
pub const STEAM_APP_ID: &str = "310560";

/// The game's settings folder name under `~/.local/share/feral-interactive`.
pub const PREFS_DIR_NAME: &str = "DiRT Rally";

/// Where the launcher finds what it ships.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    /// The legacy libraries (`lib/compat` in the package).
    pub compat: PathBuf,
    /// The folder holding [`EXEC_READ_LIB`].
    pub exec_read: PathBuf,
}

/// One start of the game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launch {
    pub program: PathBuf,
    pub cwd: PathBuf,
    pub args: Vec<String>,
    /// Variables to set, in order.
    pub set: Vec<(String, OsString)>,
    /// Variables to remove.
    pub unset: Vec<String>,
}

/// What the host gives the plan: the caller's variables that matter, and
/// whether a certificate file exists.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Host {
    pub home: PathBuf,
    pub ld_library_path: Option<OsString>,
    pub ld_preload: Option<OsString>,
    pub cert_file: Option<PathBuf>,
}

/// The well-known certificate files, as the game's script tries them.
pub const CERT_FILES: &[&str] = &[
    "/etc/ssl/certs/ca-certificates.crt",
    "/etc/pki/tls/certs/ca-bundle.crt",
    "/var/lib/ca-certificates/ca-bundle.pem",
];

/// The game's own settings folder for `home`.
pub fn prefs_dir(home: &Path) -> PathBuf {
    home.join(".local/share/feral-interactive")
        .join(PREFS_DIR_NAME)
}

/// A path list entry may not hold the list separator.
fn list_entry(path: &Path) -> Result<OsString, String> {
    let text = path.as_os_str();
    if text.as_encoded_bytes().contains(&b':') {
        return Err(format!(
            "The folder {} has a ':' in its name, which Linux cannot use in a library \
             search path. Move it, or choose a runtime folder without ':'.",
            path.display()
        ));
    }
    Ok(text.to_owned())
}

fn join(entries: Vec<OsString>, tail: Option<&OsString>) -> OsString {
    let mut out = OsString::new();
    for (i, entry) in entries
        .iter()
        .chain(tail.filter(|t| !t.is_empty()))
        .enumerate()
    {
        if i > 0 {
            out.push(":");
        }
        out.push(entry);
    }
    out
}

/// The start of the game from `runtime` (the private copy).
pub fn launch(runtime: &Path, package: &Package, host: &Host) -> Result<Launch, String> {
    let bin = runtime.join("bin");
    let libs = vec![
        list_entry(&runtime.join("lib/x86_64"))?,
        list_entry(&package.compat)?,
        list_entry(&package.exec_read)?,
    ];
    let mut preload = OsString::from(format!("{STEAM_API_LIB}:{EXEC_READ_LIB}"));
    if let Some(theirs) = host.ld_preload.as_ref().filter(|p| !p.is_empty()) {
        preload.push(":");
        preload.push(theirs);
    }
    let mut set = vec![
        (
            "LD_LIBRARY_PATH".to_owned(),
            join(libs, host.ld_library_path.as_ref()),
        ),
        ("LD_PRELOAD".to_owned(), preload),
        ("SteamAppId".to_owned(), STEAM_APP_ID.into()),
        ("GameAppId".to_owned(), STEAM_APP_ID.into()),
        (
            "__GL_SHADER_DISK_CACHE_PATH".to_owned(),
            prefs_dir(&host.home).join("driver-gl-shader-cache").into(),
        ),
    ];
    if let Some(cert) = &host.cert_file {
        set.push(("SSL_CERT_FILE".to_owned(), cert.clone().into()));
    }
    Ok(Launch {
        program: bin.join("DirtRally"),
        cwd: bin,
        args: Vec::new(),
        set,
        unset: vec!["LC_ALL".to_owned()],
    })
}

/// [`Host`] from this process's environment.
pub fn host_from_env() -> Host {
    Host {
        home: std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default(),
        ld_library_path: std::env::var_os("LD_LIBRARY_PATH"),
        ld_preload: std::env::var_os("LD_PRELOAD"),
        cert_file: CERT_FILES
            .iter()
            .map(PathBuf::from)
            .find(|path| path.is_file()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package() -> Package {
        Package {
            compat: PathBuf::from("/opt/Dirt Rally Launcher/lib/compat"),
            exec_read: PathBuf::from("/opt/Dirt Rally Launcher/lib"),
        }
    }

    fn value<'a>(launch: &'a Launch, key: &str) -> Option<&'a OsString> {
        launch.set.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    #[test]
    fn the_game_starts_from_the_runtime_with_its_libraries_and_the_fix() {
        let host = Host {
            home: PathBuf::from("/home/José"),
            cert_file: Some(PathBuf::from("/etc/ssl/certs/ca-certificates.crt")),
            ..Host::default()
        };
        let rt = Path::new("/home/José/Jogos d'Ação/runtime");
        let launch = launch(rt, &package(), &host).unwrap();
        assert_eq!(launch.program, rt.join("bin/DirtRally"));
        assert_eq!(launch.cwd, rt.join("bin"));
        assert_eq!(
            value(&launch, "LD_LIBRARY_PATH").unwrap(),
            "/home/José/Jogos d'Ação/runtime/lib/x86_64:/opt/Dirt Rally Launcher/lib/compat:/opt/Dirt Rally Launcher/lib"
        );
        // By name only: a space in a path would split LD_PRELOAD.
        assert_eq!(
            value(&launch, "LD_PRELOAD").unwrap(),
            "libsteam_api.so:libdirtrally_exec_read.so"
        );
        assert_eq!(value(&launch, "SteamAppId").unwrap(), "310560");
        assert_eq!(value(&launch, "GameAppId").unwrap(), "310560");
        assert_eq!(
            value(&launch, "__GL_SHADER_DISK_CACHE_PATH").unwrap(),
            "/home/José/.local/share/feral-interactive/DiRT Rally/driver-gl-shader-cache"
        );
        assert_eq!(
            value(&launch, "SSL_CERT_FILE").unwrap(),
            "/etc/ssl/certs/ca-certificates.crt"
        );
        assert_eq!(launch.unset, ["LC_ALL"]);
        assert!(launch.args.is_empty());
    }

    #[test]
    fn the_callers_paths_follow_ours_and_empty_ones_are_dropped() {
        let host = Host {
            ld_library_path: Some("/usr/lib/extra".into()),
            ld_preload: Some("libgamemodeauto.so.0".into()),
            ..Host::default()
        };
        let launch = launch(Path::new("/rt"), &package(), &host).unwrap();
        let libs = value(&launch, "LD_LIBRARY_PATH").unwrap().to_str().unwrap();
        assert!(libs.starts_with("/rt/lib/x86_64:") && libs.ends_with(":/usr/lib/extra"));
        assert_eq!(
            value(&launch, "LD_PRELOAD").unwrap(),
            "libsteam_api.so:libdirtrally_exec_read.so:libgamemodeauto.so.0"
        );
        assert!(value(&launch, "SSL_CERT_FILE").is_none());

        let empty = Host {
            ld_library_path: Some("".into()),
            ld_preload: Some("".into()),
            ..Host::default()
        };
        let launch = super::launch(Path::new("/rt"), &package(), &empty).unwrap();
        assert!(
            !value(&launch, "LD_LIBRARY_PATH")
                .unwrap()
                .to_str()
                .unwrap()
                .ends_with(':')
        );
        assert_eq!(
            value(&launch, "LD_PRELOAD").unwrap(),
            "libsteam_api.so:libdirtrally_exec_read.so"
        );
    }

    #[test]
    fn a_folder_with_a_colon_is_refused_with_a_reason() {
        let error = launch(
            Path::new("/games/DiRT: Rally"),
            &package(),
            &Host::default(),
        )
        .unwrap_err();
        assert!(
            error.contains("':'") && error.contains("DiRT: Rally"),
            "{error}"
        );
    }
}
