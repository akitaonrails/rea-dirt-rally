# Changelog

All notable changes to the DiRT Rally launcher. Versions are git tags
(`vX.Y.Z`) matching the `dirtrally-launcher` crate version.

## [Unreleased]

### Changed

- A runtime folder with a `:` in its name (which Linux cannot put in a
  library search path) is refused before the private copy is made, not
  after copying the whole game.

### Internal

- Golden snapshots (`crates/dirtrally-launcher/tests/snapshots.rs`) of
  `--help`, the `launcher.cfg` template and first run, and for each look
  and a few option combinations the start plan: the `preferences` values,
  the game's environment, Feral's controller list and the Linux helper's
  gamescope command line. `start::plan` is the one place a start is
  decided.

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
