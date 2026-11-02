//! `launcher.cfg`, the command line and `--help`, from one option table
//! (`rea_launcher::cfg::spec`, docs/STANDARDS.md "Settings window and CLI").
//!
//! [`OPTIONS`] names every setting once: its key, its flag, the values it
//! takes, and the label and description the window and `--help` show. The
//! parser, `--help`, the first-run template and the window rows all come
//! from it, and the window's parity test fails when a row has no control
//! or a control has no row.
//!
//! Every option here was seen working in the real game
//! (docs/RUNTIME_NOTES.md): `resolution` is the size the game opens at
//! (Feral's `ScreenW`/`ScreenH`) and the gamescope size the Linux helper
//! uses; `original` keeps Feral's own pre-game window, where the game
//! shipped its display choices.

use std::path::PathBuf;

use rea_launcher::cfg::{
    self, ArgError, KeyStyle, Kind, OptionSpec, Rejected, Resolution, Value, spec,
};
use rea_launcher::display;

/// The settings file in the launcher's config folder.
pub const FILE_NAME: &str = "launcher.cfg";

/// Flags spell keys with hyphens.
pub const STYLE: KeyStyle = KeyStyle::Underscore;

/// The launcher's own switches: they take no value and never reach
/// `launcher.cfg`.
pub const SWITCHES: &[&str] = &["--settings", "--no-window", "--check", "--linux"];

/// Looks, as `(value, label)` for the option table.
pub const LOOKS: &[(&str, &str)] = &[
    ("original", "Original"),
    ("recommended", "Recommended"),
    ("steam_deck", "Steam Deck"),
];

/// The smallest and largest sizes the game is offered.
pub const MIN_SIZE: (u32, u32) = (640, 480);
pub const MAX_SIZE: (u32, u32) = (7680, 4320);

/// Every option: `launcher.cfg` key, flag and window control.
pub const OPTIONS: &[OptionSpec] = &[
    OptionSpec::new(
        "game",
        Kind::Path,
        "Game folder",
        "The folder with your DiRT Rally for Linux (it holds bin/DirtRally). It is only \
         read, never changed.",
    ),
    OptionSpec::new(
        "look",
        Kind::Choice(LOOKS),
        "Look",
        "Original opens the game's own pre-game window first, as it shipped. Recommended \
         starts the race game straight away at your screen's size. Steam Deck does the \
         same at 1280x800.",
    ),
    OptionSpec::new(
        "resolution",
        Kind::Size {
            min: MIN_SIZE,
            max: MAX_SIZE,
            aliases: cfg::ORIGINAL_OR_FULLSCREEN,
        },
        "Resolution",
        "The size the game opens at: fullscreen for your screen's size, a size such as \
         2560x1440, or original to choose it in the game's own pre-game window.",
    )
    .with_example("fullscreen"),
    OptionSpec::new(
        "runtime",
        Kind::Path,
        "Runtime folder",
        "Where the launcher keeps its private copy of the game. Default: \
         ~/.local/share/dirtrally-launcher/runtime.",
    )
    .launcher_only(),
];

/// Comment lines at the top of a new `launcher.cfg`.
const HEADING: &str = "Settings for DiRTRallyLauncher (DiRT Rally).\n\
    Every key is also a command-line flag: resolution = 1920x1080 is --resolution 1920x1080.\n\
    A key left commented out keeps the game's own value.";

/// A look: a named set of option values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Look {
    /// The game as it shipped: Feral's pre-game window first.
    Original,
    /// Straight into the game at the screen's size.
    Recommended,
    SteamDeck,
}

impl Look {
    pub const ALL: [Look; 3] = [Look::Original, Look::Recommended, Look::SteamDeck];

    /// The `launcher.cfg` value.
    pub fn key(self) -> &'static str {
        match self {
            Look::Original => "original",
            Look::Recommended => "recommended",
            Look::SteamDeck => "steam_deck",
        }
    }

    pub fn from_key(key: &str) -> Option<Look> {
        Look::ALL.into_iter().find(|look| look.key() == key)
    }

    /// The window's card title (the option table's label for the value).
    pub fn label(self) -> &'static str {
        LOOKS
            .iter()
            .find(|(key, _)| *key == self.key())
            .map_or("", |(_, label)| label)
    }

    /// The look recommended on `screen`: Steam Deck on a Deck's screen.
    pub fn recommended(screen: Option<(u32, u32)>) -> Look {
        if display::is_steam_deck(screen) {
            Look::SteamDeck
        } else {
            Look::Recommended
        }
    }

    /// The look's values.
    pub fn picture(self) -> Picture {
        let resolution = match self {
            Look::Original => Resolution::Original,
            Look::Recommended => Resolution::Fullscreen,
            Look::SteamDeck => {
                let (w, h) = display::STEAM_DECK_SCREEN;
                Resolution::Size(w, h)
            }
        };
        Picture {
            resolution: Some(resolution),
        }
    }
}

/// The picture options for one run. `None` keeps the game's own value.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Picture {
    pub resolution: Option<Resolution>,
}

impl Picture {
    /// The values as `(key, launcher.cfg text)`, for the window and the log.
    pub fn values(&self) -> [(&'static str, Option<String>); 1] {
        [("resolution", self.resolution.map(|r| r.to_string()))]
    }

    /// One line for `launcher.log`.
    pub fn describe(&self) -> String {
        self.values()
            .iter()
            .map(|(key, value)| format!("{key} {}", value.as_deref().unwrap_or("(game's own)")))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// The settings from `launcher.cfg` and the command line. `None` is unset.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Settings {
    pub game: Option<PathBuf>,
    pub look: Option<Look>,
    pub resolution: Option<Resolution>,
    pub runtime: Option<PathBuf>,
}

impl Settings {
    /// One checked value from the file or a flag.
    pub fn apply(&mut self, key: &str, value: Value) -> Result<(), Rejected> {
        let path = |text: &str| (!text.is_empty()).then(|| PathBuf::from(text));
        match (key, value) {
            ("game", Value::Text(text)) => self.game = path(text),
            ("runtime", Value::Text(text)) => self.runtime = path(text),
            ("look", Value::Choice(choice)) => {
                let look = Look::from_key(choice).ok_or(Rejected::Unknown)?;
                // A look sets all of its values; options after it win.
                self.look = Some(look);
                self.resolution = None;
            }
            ("resolution", Value::Size(resolution)) => self.resolution = Some(resolution),
            _ => return Err(Rejected::Unknown),
        }
        Ok(())
    }

    /// These settings with the command line's on top: a flag wins over the
    /// file, and `--look` replaces the file's picture options.
    pub fn merged(mut self, flags: Settings) -> Settings {
        if flags.look.is_some() {
            self.look = flags.look;
            self.resolution = None;
        }
        self.game = flags.game.or(self.game);
        self.runtime = flags.runtime.or(self.runtime);
        self.resolution = flags.resolution.or(self.resolution);
        self
    }

    /// The picture for this run: the look's values, then the options set
    /// on their own.
    pub fn picture(&self) -> Picture {
        let base = self.look.map(Look::picture).unwrap_or_default();
        Picture {
            resolution: self.resolution.or(base.resolution),
        }
    }
}

/// Reads `launcher.cfg` text; the warnings name each line it could not use.
pub fn parse_file(text: &str) -> (Settings, Vec<String>) {
    let mut settings = Settings::default();
    let warnings = spec::parse_file(text, FILE_NAME, OPTIONS, |key, value| {
        settings.apply(key, value)
    });
    (settings, warnings)
}

/// Reads the command line: the settings its flags set, and the switches
/// (`--no-window`, …) in the order given.
pub fn parse_args(
    args: impl IntoIterator<Item = String>,
) -> Result<(Settings, Vec<String>), ArgError> {
    let mut settings = Settings::default();
    let switches = spec::parse_args(args, STYLE, OPTIONS, SWITCHES, |key, value| {
        settings.apply(key, value)
    })?;
    Ok((settings, switches))
}

/// A new `launcher.cfg`: every option commented out, with its description.
pub fn template() -> String {
    spec::template(HEADING, OPTIONS)
}

/// The first run's `launcher.cfg`: the template with the recommended look's
/// values saved explicitly, which is how the default becomes the
/// recommended experience (STANDARDS "The game's own settings stay the
/// game's").
pub fn first_run_text(screen: Option<(u32, u32)>) -> String {
    let look = Look::recommended(screen);
    let mut text = set(&template(), "look", look.key());
    for (key, value) in look.picture().values() {
        if let Some(value) = value {
            text = set(&text, key, &value);
        }
    }
    text
}

/// [`spec::set_key`] for the row of `key`.
pub fn set(text: &str, key: &str, value: &str) -> String {
    match spec::find(OPTIONS, key) {
        Some(row) => spec::set_key(text, row, value),
        None => cfg::set_key(text, key, value),
    }
}

/// `--help`.
pub fn usage() -> String {
    format!(
        "DiRTRallyLauncher {version}: plays your own copy of DiRT Rally for Linux on a modern system.\n\
         \n\
         Usage: DiRTRallyLauncher [options]\n\
         \n\
         Without options the settings window opens; Play saves them and starts the game.\n\
         \n\
         Options (each is also a {FILE_NAME} key):\n\
         {options}\n\
         Launcher switches:\n  \
           --settings   only the settings window (exit 0 after Play, 3 when closed)\n  \
           --no-window  play with the saved settings and these options, no window\n  \
           --check      check the game folder and its edition, then exit (see launcher.log)\n  \
           --linux      started by the Linux helper (linux/dirtrally-play)\n  \
           --help       this text\n",
        version = env!("CARGO_PKG_VERSION"),
        options = spec::usage(OPTIONS, STYLE),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_template_parses_to_nothing_and_lists_every_option() {
        let text = template();
        assert_eq!(parse_file(&text), (Settings::default(), Vec::new()));
        for row in OPTIONS {
            assert!(text.contains(&format!("#{} =", row.key)), "{}", row.key);
        }
    }

    #[test]
    fn the_first_run_saves_the_recommended_look_explicitly() {
        let (settings, warnings) = parse_file(&first_run_text(Some((2560, 1440))));
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(settings.look, Some(Look::Recommended));
        assert_eq!(settings.resolution, Some(Resolution::Fullscreen));
        let deck = parse_file(&first_run_text(Some((800, 1280)))).0;
        assert_eq!(deck.look, Some(Look::SteamDeck));
        assert_eq!(deck.resolution, Some(Resolution::Size(1280, 800)));
    }

    #[test]
    fn a_look_sets_its_values_and_later_options_win() {
        let (settings, _) = parse_file("resolution = 800x600\nlook = original\n");
        assert_eq!(settings.picture().resolution, Some(Resolution::Original));
        let (settings, _) = parse_file("look = original\nresolution = 1280x720\n");
        assert_eq!(
            settings.picture().resolution,
            Some(Resolution::Size(1280, 720))
        );
        let unset = Settings::default().picture();
        assert_eq!(unset, Picture::default(), "unset keeps the game's values");
    }

    #[test]
    fn flags_win_over_the_file_and_a_look_flag_replaces_its_values() {
        let (file, _) = parse_file("game = /games/Old\nresolution = 800x600\n");
        let (flags, _) = parse_args(["--look".into(), "steam_deck".into()]).unwrap();
        let merged = file.clone().merged(flags);
        assert_eq!(merged.game, file.game);
        assert_eq!(
            merged.picture().resolution,
            Some(Resolution::Size(1280, 800))
        );
        let (flags, _) = parse_args(["--resolution".into(), "fullscreen".into()]).unwrap();
        assert_eq!(file.merged(flags).resolution, Some(Resolution::Fullscreen));
    }

    #[test]
    fn bad_lines_are_reported_and_kept() {
        let (settings, warnings) = parse_file("resolution = huge\nfps = 60\nlook = original\n");
        assert_eq!(settings.look, Some(Look::Original));
        assert_eq!(warnings.len(), 2, "{warnings:?}");
        assert!(warnings[1].contains("unknown setting"), "{warnings:?}");
    }
}
