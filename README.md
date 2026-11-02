<!--
  README for agents: this file follows sdk/docs/playbooks/readme.md (the
  canonical outline, tone and rules). Keep its headings and their order.
  Fill every TODO as the launcher grows, delete each TODO comment once its
  section is done, and delete an optional section only when it doesn't
  apply. The top half (down to Uninstall) is for players: plain words, no
  offsets, hashes, function names or env vars. Never write a release or SDK
  version in prose or links: downloads go through releases/latest and file
  names use <version>.
  When the content is complete, run the player-facing text (this file and
  linux/README.md) through the `humanizer` skill: wording only, facts,
  commands, paths, option names and links unchanged. Then delete this
  comment.
-->

# DiRT Rally Launcher

Play **your own copy** of DiRT Rally on Windows 10/11, the Steam Deck and
Linux (Proton, Wine or the AppImage). Point the launcher at your installed
game and press **Play**: everything the game needs is bundled and set up
for you, with no mods, patchers or settings to fix by hand. Your game
folder is only read, never changed: the launcher plays from its own
private copy.

This project contains no game files: you need your own copy of the game.

> **Work in progress.** This launcher was just started; the sections below
> are filled in as each fix is verified in the real game
> ([docs/RUNTIME_NOTES.md](docs/RUNTIME_NOTES.md)). Reports are welcome at
> <https://github.com/akitaonrails/rea-dirt-rally/issues>.
<!-- TODO: when it plays, say where it was played end to end and what is
not tested yet (native Windows, Steam Deck, Proton without the helper).
No version number. Remove the note once every platform is verified. -->

<!-- Showcase video: the owner records it and hosts it outside the
repository, with its poster image. Uncomment and fill:
[![TODO: what the video shows](https://TODO/showcase-poster.jpg)](https://TODO/showcase.mp4)

*TODO: what is shown, on which platform. Click to watch (m:ss, with sound).*
-->

## Problems this launcher solves

<!-- TODO: one row per problem, verified in the real game, from "won't
start" to "nice to have" (5–12 rows). Credit the community project in the
row that uses it. Not yet fixed → Known limitations, never here. -->

| Problem | What the launcher does |
| --- | --- |
| Patching the game folder risks the copy you own. | Your folder is only read; the launcher works on its own private copy. |

## Screenshots

<!-- TODO: the window's tabs, JPEG under ~1 MB each in docs/images/
(launcher-<tab>.jpg, about 1600 px wide), current, with alt text; then an
optional gameplay or before/after image. List them in THIRD_PARTY.md.
| Play | Picture | Controls |
| --- | --- | --- |
| ![The Play tab: TODO](docs/images/launcher-play.jpg) | ![The Picture tab: TODO](docs/images/launcher-picture.jpg) | ![The Controls tab: TODO](docs/images/launcher-controls.jpg) |
-->

## What you need

Your installed PC copy of **DiRT Rally**: the folder with `drt.exe`.

<!-- TODO: the editions and version supported, in words; what to do with
an older copy (an official patch and where to get it). -->

The launcher knows each supported `drt.exe` by its fingerprint
(SHA-256). Another version is refused with its code and a link to the
[unknown version](https://github.com/akitaonrails/rea-dirt-rally/issues/new?template=unknown-edition.yml)
form, so it can be added. Report the code, never the game's files.

## Download and install

Download from the
[latest release](https://github.com/akitaonrails/rea-dirt-rally/releases/latest):

- **Windows, Steam Deck and Proton:** `DiRTRallyLauncher-<version>.zip`
- **Linux:** `DiRTRallyLauncher-<version>-x86_64.AppImage`

### Windows

1. Unzip `DiRTRallyLauncher-<version>.zip` anywhere outside the game folder.
   Keep its folders next to `DiRTRallyLauncher.exe`.
2. Run `DiRTRallyLauncher.exe` and follow [First run](#first-run).

### Steam Deck

1. In Desktop Mode, unzip the release, then in Steam choose *Games → Add a
   Non-Steam Game to My Library…* and pick `DiRTRallyLauncher.exe`.
2. In its *Properties → Compatibility*, force a Proton version.
3. Back in Game Mode, start it: the window works with the controls and
   recommends the **Steam Deck** look.

<!-- TODO: say whether Game Mode has been verified. -->

### Linux: AppImage

```sh
chmod +x DiRTRallyLauncher-*-x86_64.AppImage
./DiRTRallyLauncher-*-x86_64.AppImage
```

It brings its own Wine, so nothing else needs to be installed. It keeps
its Wine prefix in `~/.local/share/dirtrally-launcher/prefix` and uses
[gamescope](https://github.com/ValveSoftware/gamescope) when it is
installed.

### Linux: Steam (Proton)

Unzip the release, add `DiRTRallyLauncher.exe` as a non-Steam game and force a
Proton version in its compatibility settings, as on the Steam Deck.

### Linux: your own Wine

Run `linux/dirtrally-play` from the unzipped release:

```sh
linux/dirtrally-play            # the settings window, then the game
linux/dirtrally-play --play     # the game with the saved settings, no window
linux/dirtrally-play stop       # end the game
linux/dirtrally-play status     # running or idle
```

It needs Wine; gamescope is optional. Options, a distrobox and more:
[linux/README.md](linux/README.md).

## First run

1. In the window's **Play** tab, press **Browse…** and choose the folder
   with `drt.exe`. Under Wine your Linux files are on drive `Z:`, for
   example `Z:\home\you\Games\DiRT Rally`.
2. Pick a look: **Recommended** suits most screens.
3. Press **Play** (or Enter, or Start on a gamepad).

The first start copies the game into the launcher's private folder; later
starts copy only what changed. Your choices are remembered.

<!-- TODO: the size of the first copy and how long it takes; any one-time
button (a download): what it fetches, how big, from where, and that
nothing downloads unless pressed. -->

## The settings window

<!-- TODO: one short paragraph per tab, in player words. -->

| Look | What it sets | When to pick it |
| --- | --- | --- |
| **Original** | The game as it shipped | To play it as in its day |
| **Recommended** | TODO | Most screens (picked on the first run) |
| **Steam Deck** | TODO (1280×800) | The Steam Deck (picked on a Deck) |
| **Custom** | Any change in the other tabs | |

The window works with a gamepad (d-pad moves, A changes the focused
setting, LB/RB switch tabs, Start plays) and with the keyboard alone (Tab
moves, arrows pick, Ctrl+Tab switches tabs, Enter plays). A control that
can't be used right now is greyed out and says why.

## Gamepad

<!-- TODO (optional: delete when the launcher does nothing for pads): what
the launcher does for pads, verified tips, and the layout:
| On the pad | In menus | In a race |
| --- | --- | --- |
-->

## Options

Every option has the same name in the window, in `launcher.cfg` (with
underscores) and on the command line (with hyphens); `DiRTRallyLauncher.exe
--help` lists them. The first run writes a commented `launcher.cfg`
([Where things live](#where-things-live)).

<!-- TODO: one row per option in the option table (config.rs OPTIONS), in
window order; nothing missing, nothing extra. "In the window" is
"Tab: Label" exactly as the window shows it. -->

| `launcher.cfg` / flag | In the window | Values | Unset | What it changes |
| --- | --- | --- | --- | --- |
| `game` / `--game` | Play: Game folder | a folder | asked on the first run | The folder with `drt.exe`; only read. Remembered. |
| `look` / `--look` | Play: Look | `original`, `recommended`, `steam_deck` | the recommended look | Sets the picture options at once; options set after it win. |
| `resolution` / `--resolution` | Picture: Resolution | `WIDTHxHEIGHT`, `original` | the game's | The game's screen size. |
| `vsync` / `--vsync` | Picture: VSync | `on`, `off` | the game's | Waits for the screen's refresh: no tearing. |
| `runtime` / `--runtime` | — (file and flag only) | a folder | `runtime` beside `launcher.log` | Where the launcher keeps its private copy of the game. |

Unset means the game keeps its own value; a value you set and later unset
goes back to the game's, unless you changed it in the game since.

Any option on the command line skips the window and plays; options
override `launcher.cfg` for that run only, and only `--game` is
remembered. The launcher's own switches:

- `--no-window`: play with the saved settings, no window.
- `--check`: check the game folder and the bundled files without playing;
  the report goes to `launcher.log` (exit 0 ready, 1 not).
- `--settings`: only the settings window (exit 0 Play pressed, 3 closed
  without Play, 4 no window possible).
- `--linux`: started by `dirtrally-play` or the AppImage.
- `--help`: every option and its values.

A bad flag or value exits with 2 and a message naming what it takes.

## Where things live

| What | Windows | Linux (AppImage, `dirtrally-play`) |
| --- | --- | --- |
| Settings (`launcher.cfg`) and log (`launcher.log`) | `%LOCALAPPDATA%\DiRTRallyLauncher\` | in the Wine prefix, `drive_c/users/<you>/AppData/Local/DiRTRallyLauncher/` |
| The launcher's copy of the game | `%LOCALAPPDATA%\DiRTRallyLauncher\runtime` (the `runtime` option moves it) | the same folder inside the prefix |
| Your saves | TODO | TODO |
| Wine prefix | — | `~/.local/share/dirtrally-launcher/prefix` |
| `dirtrally-play`'s logs and lock | — | `~/.config/dirtrally-launcher` |

A `launcher.cfg` next to `DiRTRallyLauncher.exe` is used instead (a portable
install). Under Proton everything is inside Steam's prefix for the
non-Steam game (`steamapps/compatdata/<id>/pfx`).

<!-- TODO: where the game writes its saves (a saves folder the launcher
mirrors, or only the runtime copy), and any registry key the launcher
writes. -->

## Troubleshooting

- **"This drt.exe is not a version the launcher knows"**: your copy is
  a version not supported yet. Open the
  [unknown version](https://github.com/akitaonrails/rea-dirt-rally/issues/new?template=unknown-edition.yml)
  form with the code the message shows and where your copy comes from.
<!-- TODO: the 3–6 most likely real problems found in testing, each as a
bold symptom and what to do. -->
- **Anything else:** run `DiRTRallyLauncher.exe --check`, then open a
  [bug report](https://github.com/akitaonrails/rea-dirt-rally/issues/new?template=bug-report.yml)
  with `launcher.log` attached (remove personal paths). Never attach game
  files.

## Known limitations

<!-- TODO: what the launcher doesn't fix, what you lose when a fix can't be
automated, and the platforms not verified yet. -->

- Not yet verified on native Windows, the Steam Deck, or Proton without
  `dirtrally-play`; see [docs/RUNTIME_NOTES.md](docs/RUNTIME_NOTES.md).

## Uninstall

1. **Back up your saves first** (see [Where things live](#where-things-live)).
   <!-- TODO: name the folder(s), and say if a step below deletes them. -->
2. Delete the folder you unzipped the launcher into (or the AppImage).
3. Windows: delete `%LOCALAPPDATA%\DiRTRallyLauncher` (settings, log and the
   private copy of the game).
4. Linux: copy your saves out of the prefix, then delete
   `~/.local/share/dirtrally-launcher` and `~/.config/dirtrally-launcher`. With
   Proton, removing the shortcut in Steam offers to delete its prefix,
   saves included.

Your game folder was never changed, so there is nothing to undo there.

## For developers

Rust (edition 2024), MinGW-w64 with lld for the Windows target, Wine for
the Windows-target tests and `cargo-deny`. The
[`gaming` distrobox](https://github.com/akitaonrails/distrobox-gaming) has
them all.

```sh
cargo fmt --all --check
cargo test --workspace --locked                 # host tests, the settings window headless
cargo clippy --target i686-pc-windows-gnu --workspace --all-targets -- -D warnings
cargo deny check
for t in linux/tests/*.sh packaging/tests/*.sh; do sh "$t"; done
cargo build --release --locked --target i686-pc-windows-gnu
sdk/scripts/wine-test --locked --workspace      # Windows-target tests under Wine, headless
sh ../rea-launcher-sdk/scripts/sdk-sync --check .   # sdk/ matches the pinned SDK
```

- New SDK release: `sdk/scripts/sdk-bump vX.Y.Z` moves the Cargo tags,
  `sdk/` and the workflow pins together and prints the migration notes.
- Free the disk space builds take: `sdk/scripts/clean` (a dry run), then
  `sdk/scripts/clean --yes`.
- Real-game tests read your installed copy and never change it:
  `DIRTRALLY_GAME_DIR=<game folder> cargo test -- --ignored`.
- Third-party components are fetched at package time from pinned URLs
  with SHA-256 (`packaging/`); GitHub Actions (the SDK's reusable
  workflows) runs the checks, builds the zip and the AppImage, and
  publishes a release for each `v*` tag.

Rules for contributors and coding agents: [AGENTS.md](AGENTS.md).
Community fixes and decisions: [docs/RESEARCH.md](docs/RESEARCH.md); what
was verified in the real game: [docs/RUNTIME_NOTES.md](docs/RUNTIME_NOTES.md).

<!-- Optional: ## How it works — 4–6 plain steps of what happens on Play. -->

## Built on rea-launcher-sdk

The private copy of the game, `launcher.cfg` and the option table, the
settings window's widgets, the Linux helper's library (`rea-play.sh`),
packaging and CI come from
[rea-launcher-sdk](https://github.com/akitaonrails/rea-launcher-sdk), shared by every rea launcher. It is pinned
by tag in `Cargo.toml` and vendored in `sdk/` (the tag is in
`sdk/VERSION`). This repository keeps what is specific to DiRT Rally.

## Contributing

- **About DiRT Rally** (its versions, fixes, settings, `dirtrally-play`'s game
  settings, this README): issues and pull requests here.
- **Shared code** (the private copy, the settings window's widgets, the
  option table, `rea-play.sh`, packaging, CI workflows, anything under
  `sdk/`): issues and pull requests at
  [rea-launcher-sdk](https://github.com/akitaonrails/rea-launcher-sdk/issues).
- **How to tell:** code under `sdk/` or from a `rea_launcher*` crate is
  the SDK's; if the same problem would happen with another rea launcher,
  it's the SDK's. Not sure? Open it here and it will be moved.
- Never patch `sdk/` or copy SDK code here: fix the SDK and bump the tag.
- Never attach game files (executables, saves, data) to an issue.

## Credits

<!-- TODO: one line per community project the launcher bundles,
configures, downloads or reimplements (author, link, what it does here),
the window's imagery (author, licence), Wine and Kron4ek's builds, the
AppImage tools, and where the fixes were found (PCGamingWiki). Every
THIRD_PARTY.md entry has a line here. -->

- The Barlow fonts by Jeremy Tribby (SIL Open Font License).
- [rea-launcher-sdk](https://github.com/akitaonrails/rea-launcher-sdk).

Full list with licences: [THIRD_PARTY.md](THIRD_PARTY.md).

## Legal

DiRT Rally belongs to its publisher. This project is not affiliated with or
endorsed by them, contains no game files, and only runs a copy you own.
<!-- TODO: name the copyright holder ("DiRT Rally is © …"). -->
Our code is MIT OR Apache-2.0 ([LICENSE-MIT](LICENSE-MIT),
[LICENSE-APACHE](LICENSE-APACHE)); bundled components keep their own
licences ([THIRD_PARTY.md](THIRD_PARTY.md)).
