//! DiRTRallyLauncher.exe: a Windows GUI program (it also runs under Proton and
//! Wine). Everything testable lives in the library; this file wires it to
//! Windows: the console for `--help`/`--check`/`--no-window`, the settings
//! window, `launcher.log`, the runtime copy and the game's process.

#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
fn main() {
    std::process::exit(app::run());
}

#[cfg(unix)]
fn main() {
    std::process::exit(linux::run());
}

/// The Linux launcher: DiRT Rally's supported edition is Feral's native
/// port, so this is the program players run (through linux/dirtrally-play
/// or the AppImage). Each step is a library call; this only wires them.
#[cfg(unix)]
mod linux {
    use std::path::PathBuf;
    use std::process::Command;

    use dirtrally_launcher::cli::{self, Invocation, Mode};
    use dirtrally_launcher::{config, pads, plan, runtime, start, ui};
    use rea_launcher::log::Log;
    use rea_launcher::{cfg, fsx};

    fn home() -> PathBuf {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default()
    }

    fn places() -> start::Places {
        start::places(
            &home(),
            std::env::var_os("XDG_CONFIG_HOME"),
            std::env::var_os("XDG_DATA_HOME"),
        )
    }

    pub fn run() -> i32 {
        if let Err(error) = start::checked_home(std::env::var_os("HOME")) {
            eprintln!("{error}");
            return cli::REFUSED;
        }
        let places = places();
        let log = Log::new(places.config.join("launcher.log"));
        log.line(concat!("DiRTRallyLauncher ", env!("CARGO_PKG_VERSION")));
        let invocation = match cli::parse(std::env::args().skip(1)) {
            Ok(invocation) => invocation,
            Err(error) => {
                log.line(&format!("error: {error}"));
                eprintln!("{error}");
                return cli::USAGE_ERROR;
            }
        };
        match run_mode(&log, &places, invocation) {
            Ok(code) => code,
            Err(error) => {
                log.line(&format!("error: {error}"));
                eprintln!("{error}");
                cli::REFUSED
            }
        }
    }

    fn run_mode(log: &Log, places: &start::Places, invocation: Invocation) -> Result<i32, String> {
        if invocation.mode == Mode::Help {
            println!("{}", config::usage());
            return Ok(cli::OK);
        }
        let path = places.config.join(config::FILE_NAME);
        if let Mode::Window { settings_only } = invocation.mode {
            let saved = cfg::read_file(&path).ok().flatten();
            match ui::show(&path, saved.as_deref(), invocation.linux, &places.runtime) {
                Ok(ui::Outcome::Play) if settings_only => return Ok(cli::OK),
                Ok(ui::Outcome::Play) => {}
                Ok(ui::Outcome::Closed) if settings_only => return Ok(cli::WINDOW_CLOSED),
                Ok(ui::Outcome::Closed) => return Ok(cli::OK),
                Err(error) => {
                    log.line(&format!("warning: settings window unavailable: {error}"));
                    if settings_only {
                        return Ok(cli::WINDOW_UNAVAILABLE);
                    }
                }
            }
        }
        let existing = cfg::read_file(&path)
            .map_err(|error| format!("Cannot read {}: {error}", path.display()))?;
        let first_run = existing.is_none();
        let text = existing.unwrap_or_else(|| config::first_run_text(None));
        let (saved, warnings) = config::parse_file(&text);
        for warning in &warnings {
            log.line(&format!("warning: {warning}"));
        }
        let settings = saved.clone().merged(invocation.flags);
        let cwd = std::env::current_dir().unwrap_or_default();
        let (game, found) = cli::check(&settings, &cwd)?;
        log.line(&format!(
            "game: {} ({}, {})",
            found.root.display(),
            found.edition.name,
            found.exe.display()
        ));
        let exe = std::env::current_exe().map_err(|error| error.to_string())?;
        let package = start::package(&exe, std::env::var_os(start::GAME_LIBS_ENV));
        start::check_package(&package)?;
        if invocation.mode == Mode::Check {
            println!("{}", cli::check_report(&found));
            return Ok(cli::OK);
        }

        let runtime_dir = settings.runtime.as_deref().map_or_else(
            || places.runtime.clone(),
            |path| start::resolve_runtime(path, &home(), &cwd),
        );
        let report = runtime::prepare(&found.root, &runtime_dir)?;
        for warning in &report.warnings {
            log.line(&format!("warning: {warning}"));
        }
        log.line(&format!(
            "runtime: {} ({} brought, {} linked)",
            runtime_dir.display(),
            report.brought,
            report.linked
        ));
        let screen = screen_size();
        let restore = places.config.join(start::HIDDEN_MARKER).is_file();
        let how = start::start_for(settings.picture().resolution, screen, restore);
        log.line(&format!(
            "picture: {}; screen {}x{}; start {how:?}",
            settings.picture().describe(),
            screen.0,
            screen.1
        ));
        let prefs = start::write_prefs(&home(), &places.config, how)?;
        log.line(&format!("preferences: {}", prefs.display()));
        let connected = pads::pads(std::path::Path::new("/sys/class/input"));
        for pad in &connected {
            log.line(&format!(
                "pad: {} ({:04x}:{:04x}), standard layout: {}",
                pad.name,
                pad.vendor,
                pad.product,
                pad.layout().is_some()
            ));
        }
        let known = start::known_pads(&places.config, &connected)?;
        let added = start::write_input_devices(&found.root, &runtime_dir, &known)?;
        log.line(&format!("controller list: added {added:?}"));

        if first_run || saved.game.as_ref() != Some(&game) {
            let text = config::set(&text, "game", &game.display().to_string());
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)
                    .map_err(|error| format!("Cannot create {}: {error}", dir.display()))?;
            }
            fsx::write_atomic(&path, text)
                .map_err(|error| format!("Cannot save {}: {error}", path.display()))?;
        }

        let launch = plan::launch(&runtime_dir, &package, &plan::host_from_env())?;
        log.line(&format!(
            "start: {} in {}",
            launch.program.display(),
            launch.cwd.display()
        ));
        for (key, value) in &launch.set {
            log.line(&format!("env: {key}={}", value.to_string_lossy()));
        }
        let status = spawn(&launch)?;
        log.line(&format!("game exited with {status}"));
        Ok(status.code().unwrap_or(cli::REFUSED))
    }

    fn spawn(launch: &plan::Launch) -> Result<std::process::ExitStatus, String> {
        let mut command = Command::new(&launch.program);
        command.current_dir(&launch.cwd).args(&launch.args);
        for key in &launch.unset {
            command.env_remove(key);
        }
        for (key, value) in &launch.set {
            command.env(key, value);
        }
        command
            .status()
            .map_err(|error| format!("Cannot start {}: {error}", launch.program.display()))
    }

    /// The size `fullscreen` means: `DIRTRALLY_SCREEN` (WIDTHxHEIGHT, as the
    /// Linux helper takes it), else the X screen (inside gamescope, the
    /// game's size), else 1920x1080.
    fn screen_size() -> (u32, u32) {
        let from_env = std::env::var("DIRTRALLY_SCREEN")
            .ok()
            .and_then(|text| cfg::parse_size(&text, config::MIN_SIZE, config::MAX_SIZE));
        from_env
            .or_else(|| {
                let out = Command::new("xrandr").arg("--current").output().ok()?;
                start::screen_from_xrandr(&String::from_utf8_lossy(&out.stdout))
            })
            .unwrap_or((1920, 1080))
    }
}

#[cfg(windows)]
mod app {
    use std::path::PathBuf;

    use dirtrally_launcher::cli::{self, Invocation, Mode};
    use dirtrally_launcher::config;
    use dirtrally_launcher::{runtime, ui};
    use rea_launcher::log::Log;
    use rea_launcher::{cfg, fsx};
    use rea_launcher_win32::console::{self, Output};
    use rea_launcher_win32::dialog::{self, Icon};
    use rea_launcher_win32::process::Launch;
    use rea_launcher_win32::{display, gpu, wine};

    const TITLE: &str = "DiRT Rally";

    /// `%LOCALAPPDATA%\DiRTRallyLauncher`: launcher.cfg, launcher.log and the
    /// runtime copy.
    fn home() -> PathBuf {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("DiRTRallyLauncher")
    }

    /// `launcher.cfg` beside the exe (a portable install), else in [`home`].
    fn settings_path() -> PathBuf {
        let portable = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.join(config::FILE_NAME)));
        match portable {
            Some(path) if path.is_file() => path,
            _ => home().join(config::FILE_NAME),
        }
    }

    /// Where messages go: the starting terminal, a dialog when a person
    /// started it without one, or only `launcher.log`.
    struct Out {
        log: Log,
        attached: bool,
        interactive: bool,
    }

    impl Out {
        fn say(&self, text: &str, icon: Icon) {
            match console::output(self.attached, console::stdout_usable(), self.interactive) {
                Output::Terminal if icon == Icon::Info => println!("{text}"),
                Output::Terminal => eprintln!("{text}"),
                Output::Dialog => dialog::message(TITLE, text, icon),
                Output::Nowhere => {}
            }
        }

        fn warn(&self, message: &str) {
            self.log.line(&format!("warning: {message}"));
        }
    }

    pub fn run() -> i32 {
        let log = Log::new(home().join("launcher.log"));
        log.line(concat!("DiRTRallyLauncher ", env!("CARGO_PKG_VERSION")));
        let parsed = cli::parse(std::env::args().skip(1));
        let (attached, interactive) = match &parsed {
            Ok(invocation) => (
                invocation.wants_console() && console::attach_parent(),
                invocation.interactive(),
            ),
            Err(_) => (console::attach_parent(), true),
        };
        let out = Out {
            log,
            attached,
            interactive,
        };
        let invocation = match parsed {
            Ok(invocation) => invocation,
            Err(error) => {
                out.log.line(&format!("error: {error}"));
                out.say(&error, Icon::Error);
                return cli::USAGE_ERROR;
            }
        };
        match run_mode(&out, invocation) {
            Ok(code) => code,
            Err(error) => {
                out.log.line(&format!("error: {error}"));
                out.say(&error, Icon::Error);
                cli::REFUSED
            }
        }
    }

    fn run_mode(out: &Out, invocation: Invocation) -> Result<i32, String> {
        if invocation.mode == Mode::Help {
            out.say(&config::usage(), Icon::Info);
            return Ok(cli::OK);
        }
        let path = settings_path();
        if let Mode::Window { settings_only } = invocation.mode {
            if wine::is_wine() {
                match gpu::prefer_glx() {
                    Ok(true) => out
                        .log
                        .line("settings window: OpenGL through GLX (UseEGL=N)"),
                    Ok(false) => {}
                    Err(error) => out.warn(&error.to_string()),
                }
            }
            let saved = cfg::read_file(&path).ok().flatten();
            match ui::show(
                &path,
                saved.as_deref(),
                invocation.linux,
                &home().join("runtime"),
            ) {
                Ok(ui::Outcome::Play) if settings_only => return Ok(cli::OK),
                Ok(ui::Outcome::Play) => {}
                Ok(ui::Outcome::Closed) if settings_only => return Ok(cli::WINDOW_CLOSED),
                Ok(ui::Outcome::Closed) => return Ok(cli::OK),
                Err(error) => {
                    out.warn(&format!("settings window unavailable: {error}"));
                    if settings_only {
                        return Ok(cli::WINDOW_UNAVAILABLE);
                    }
                }
            }
        }

        let screen = display::screen_size();
        let existing = cfg::read_file(&path)
            .map_err(|error| format!("Cannot read {}: {error}", path.display()))?;
        let first_run = existing.is_none();
        let text = existing.unwrap_or_else(|| config::first_run_text(screen));
        let (saved, warnings) = config::parse_file(&text);
        for warning in &warnings {
            out.warn(warning);
        }
        let settings = saved.clone().merged(invocation.flags);
        let cwd = std::env::current_dir().unwrap_or_default();
        let (game, found) = cli::check(&settings, &cwd)?;
        out.log.line(&format!(
            "game: {} ({}, {})",
            game.display(),
            found.edition.name,
            found.exe.display()
        ));
        if invocation.mode == Mode::Check {
            out.say(&cli::check_report(&found), Icon::Info);
            return Ok(cli::OK);
        }

        let profile = std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .unwrap_or_default();
        let runtime_dir = settings.runtime.as_deref().map_or_else(
            || home().join("runtime"),
            |path| dirtrally_launcher::start::resolve_runtime(path, &profile, &cwd),
        );
        let report = runtime::prepare(&game, &runtime_dir)?;
        for warning in &report.warnings {
            out.warn(warning);
        }
        out.log.line(&format!(
            "runtime: {} ({} brought, {} linked)",
            runtime_dir.display(),
            report.brought,
            report.linked
        ));
        // Phase 4 of the playbook: components, the game's own configuration
        // (generated from the picture below), patches. Log every decision.
        out.log
            .line(&format!("picture: {}", settings.picture().describe()));

        if first_run || saved.game.as_ref() != Some(&game) {
            let text = config::set(&text, "game", &game.display().to_string());
            fsx::write_atomic(&path, text)
                .map_err(|error| format!("Cannot save {}: {error}", path.display()))?;
        }
        let exe_name = found
            .exe
            .file_name()
            .ok_or("the game's exe has no file name")?;
        let exe = runtime_dir.join(exe_name);
        if !wine::is_wine() {
            match gpu::prefer_fast_gpu(&exe) {
                Ok(true) => out.log.line("Windows: high-performance GPU preferred"),
                Ok(false) => {}
                Err(error) => out.warn(&error.to_string()),
            }
        }
        let code = Launch::new(&exe).cwd(&runtime_dir).run()?;
        out.log.line(&format!("game exited with {code}"));
        Ok(code as i32)
    }
}
