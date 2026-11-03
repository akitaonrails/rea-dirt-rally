# Changelog

All notable changes to the DiRT Rally launcher. Versions are git tags
(`vX.Y.Z`) matching the `dirtrally-launcher` crate version.

## [Unreleased]

### Added

- If the launcher can't start the game (no game folder chosen, no DiRT
  Rally in it, or a build it doesn't know), it now says so in a small
  window, even when you started it from the menu or the AppImage. The
  window has a "Report the edition" button for the unknown-build form and
  a "Copy details" button for bug reports. It used to close without a
  word and leave the reason in `launcher.log`.
- Only one launcher runs at a time. A second start while the game or the
  settings window is open starts nothing, says why and exits with code 5.

### Changed

- Updated rea-launcher-sdk to v0.4.1.
- `dirtrally-play --play` (and a plain start) while the game is running
  now exits with code 5, not 0, so a script or a shortcut doesn't mistake
  it for a game that ran.
- The copy progress counts what this start copies. On a later start that
  copies only a few changed files, the bar shows those files, not the
  whole game.
- A runtime folder with a `:` in its name (which Linux cannot put in a
  library search path) is refused before the private copy is made, not
  after copying the whole game.

### Fixed

- **A hand-written `look = steam_deck`** (or `--play --look steam_deck`)
  now opens gamescope at 1280x800. Before, the game ran at 1280x800 in a
  gamescope the size of the screen.
- **Original look on a large screen:** gamescope now opens at your
  screen's size, so Feral's pre-game window offers your screen's sizes;
  it used to open at 1920x1080 on any screen.
- `dirtrally-play --play --resolution 1280x720` now sizes gamescope for
  that run too, not only the game.

### Internal

- Golden snapshots (`crates/dirtrally-launcher/tests/snapshots.rs`) of
  `--help`, the `launcher.cfg` template and first run, and for each look
  and a few option combinations the start plan: the `preferences` values,
  the game's environment, Feral's controller list and the Linux helper's
  gamescope command line. `start::plan` is the one place a start is
  decided. They compare through the SDK's `rea_launcher::snapshots!`.
- The settings window reads pads through the SDK's joystick reader, and
  the copy progress comes from the SDK's sync; the launcher's own copies
  of both are gone. `sdk/scripts/launcher-check` passes, with the owner's
  DRM decision recorded in `.rea-check-waivers`.

## [0.1.0] - 2026-11-02

First release: plays DiRT Rally's Linux version (Feral Interactive's 1.0,
build 31946) on a modern Linux desktop, from a zip or an AppImage.

### Added

- **The start-up crash on newer CPUs is fixed.** The 2017 build crashed right
  after starting on AMD Ryzen 5000 and newer (and recent Intel) CPUs; a small
  library of the launcher's (`dirtrally-exec-read`) fixes it without
  touching the game.
- **The libraries modern Linux no longer ships** (OpenSSL 1.0, libidn11,
  librtmp, OpenLDAP 2.4, GConf and their companions) come with the launcher,
  from pinned Steam Runtime packages, and reach only the game.
- **Straight into the game.** Feral's pre-game window and its "Unsupported
  Linux distribution" warning are skipped; the game opens fullscreen at
  your screen's size (4K works). The **Original** look keeps Feral's window.
- **Newer gamepads work** (an 8BitDo, recent Xbox-style pads): the launcher
  adds each standard-layout pad to the game's own controller list, so the
  game offers its ready-made **Controller preset 1**. Pads seen once are
  remembered, so a wireless pad asleep at start is still known.
- **A settings window**: the game folder with **Browse…** and a check of the
  version found (an unknown build is refused with its SHA-256 and a report
  link; the Windows version is recognised and refused with the reason), the
  looks, the resolution, and **Play**, which makes the private copy with its
  progress the first time. Mouse, keyboard and gamepad.
- **Your game folder is only read**: the game plays from a private copy
  (about 40 GB the first time; hard links when it is on the same disk), and
  the game's own `preferences` are backed up once before the launcher first
  changes them.
- **gamescope** gives a fullscreen session on the right monitor; it can come
  from a distrobox (`DIRTRALLY_RUN="distrobox enter gaming --"`).
- Command line: `--check`, `--play`, `--game`, `--look`, `--resolution`;
  `dirtrally-play stop` and `status`.

### Known limitations

- The Windows version of the game is not supported yet (it stops while
  loading under Wine).
- Not yet checked: the Steam Deck and wheels.
- RaceNet shut down in 2025, so online events and leagues no longer work;
  offline career and single stages do.
