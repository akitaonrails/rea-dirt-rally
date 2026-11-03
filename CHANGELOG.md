# Changelog

All notable changes to the DiRT Rally launcher. Versions are git tags
(`vX.Y.Z`) matching the `dirtrally-launcher` crate version.

## [Unreleased]

### Added

- Scaffold from rea-launcher-sdk v0.3.3 (`scripts/new-launcher`): edition
  table and unknown-edition refusal, option table (`look`, `resolution`,
  `vsync` as examples), settings window, `--check`, `--no-window`, the
  Linux helper, packaging and CI.
- Editions: Feral's Linux port 1.0 (build 31946) is recognised and
  playable; the Windows v1.1 `drt.exe` is recognised and refused with the
  reason (it stops while loading under Wine). The game is found in the
  chosen folder or its only subfolder.
- `dirtrally-exec-read`: a small preload library that makes executable
  memory readable, fixing the Linux port's start-up crash on CPUs with
  memory protection keys (AMD Zen 3 and later). Verified on a stage run.
- Runtime policy for the Linux port: the game data (`share/`) is hard-linked
  when on the same disk, the program folders are copied; the game always
  runs from that private copy.
- Start plan (`plan.rs`): the environment Feral's own script sets, plus the
  legacy libraries and the exec-read fix, with library names that survive
  spaces in paths. A folder with `:` is refused with a reason.
- The game's `preferences` file: owned values spliced into the game's own
  text; seeded when missing. Hides the "Unsupported Linux distribution"
  alert.
- Options: `look` (Original keeps Feral's pre-game window; Recommended and
  Steam Deck start the game directly) and `resolution` (`fullscreen`, a size,
  or `original`). The example `vsync` option is gone (not verified in this
  game).
- Linux launcher (`DiRTRallyLauncher`): `--check`, `--no-window` and the
  Linux helper in native mode (`linux/dirtrally-play`); settings and log in
  `~/.config/dirtrally-launcher`, the private copy in
  `~/.local/share/dirtrally-launcher/runtime`.
- Linux package: the launcher, its exec-read library and the legacy
  libraries from pinned Steam scout packages (`game-libs/`), kept off the
  launcher's own library path.
- `DIRTRALLY_RUN`: run gamescope and the game through a wrapper such as
  `distrobox enter gaming --`, keeping the player's HOME and XDG folders.
  Without gamescope, a multi-monitor desktop can put the game on the wrong
  screen (SDL 2.0.5 uses display 0).
- Gamepads: every connected pad with the standard Linux layout (xpad and
  others) that Feral's own controller list lacks is added to that list in
  the private copy, as an Xbox 360-style gamepad.
  The launcher remembers pads it has seen (`known-pads.json`), so a
  wireless pad asleep at start is still listed.
- Settings window on Linux: the game folder with **Browse…** (zenity or
  kdialog) and its edition check, the looks, the resolution, and **Play**,
  which makes the first private copy with its progress before the game
  starts. Mouse, keyboard and gamepad (`/dev/input/js*`). Photo: Chris Cox,
  CC BY 2.0.
- `dirtrally-play --check` checks the game folder without starting anything.
- CI builds the native Linux zip and AppImage (Ubuntu 22.04) and runs the
  Windows-target tests under Wine.
