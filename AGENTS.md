# DiRT Rally launcher — instructions for coding agents

This repository makes the player's own copy of **DiRT Rally** run on modern
Windows and Linux (Proton/Wine, AppImage) with a settings window, sensible
defaults and no manual steps. It is built on
[rea-launcher-sdk](https://github.com/akitaonrails/rea-launcher-sdk) v0.4.1.

**Read `sdk/docs/STANDARDS.md` before any work; the playbooks are in
`sdk/docs/playbooks/`** (vendored from the SDK at the tag in `sdk/VERSION`).

## Always

- The game's files are read-only: never committed, copied into the repo or published.
- Everything outside this repository is the user's and read-only (game installs,
  other repos, Wine prefixes, saves, home configs, containers, the SDK checkout)
  unless the user asked for or approved that specific change: back up first,
  never delete by a broad glob, ask when unsure.
- Headless by default; open a visible window only when a check needs a person, and say so first.
- Never print, log or commit credentials.
- Push, tag, release or post only with the user's go-ahead.
- Research community fixes first (`docs/RESEARCH.md`); patch only what nobody solved.
- Never edit `sdk/`; SDK gaps and bugs go to the SDK repository as issues or PRs.
- Clean up after a release or a long session: `sdk/scripts/clean`, then `--yes` (log scratch in `.work/.task-created`).

Running the real game in a test: `sdk/docs/STANDARDS.md`, "Running the real
game in a test" (under "Testing").

## Layout

| Path | What |
|---|---|
| `crates/dirtrally-launcher/src/edition.rs` | Known builds (SHA-256), the unknown-edition refusal |
| `crates/dirtrally-launcher/src/config.rs` | The option table: `launcher.cfg`, flags, `--help`, template, looks |
| `crates/dirtrally-launcher/src/cli.rs` | Modes (`--settings`, `--no-window`, `--check`, `--help`), exit codes, who sees a message (`Invocation::audience`, the SDK's rule) |
| `crates/dirtrally-launcher/src/report.rs` | The native build's error and "already running" messages for the SDK's message window |
| `crates/dirtrally-launcher/src/form.rs`, `ui.rs`, `palette.rs` | The settings window and its headless tests |
| `crates/dirtrally-launcher/src/runtime.rs` | The runtime copy's `Policy` |
| `crates/dirtrally-launcher/src/start.rs` | `start::plan` (one start, before any file is touched), Linux folders, `preferences`, pads |
| `crates/dirtrally-launcher/tests/snapshots.rs`, `tests/snapshots/` | Golden snapshots; `DIRTRALLY_UPDATE_SNAPSHOTS=1` rewrites them, review the diff |
| `crates/dirtrally-launcher/src/main.rs` | Windows wiring: console, window, `launcher.log`, the game's process |
| `linux/dirtrally-play`, `linux/tests/` | The Linux helper (sources `sdk/rea-play.sh`) and its tests |
| `packaging/` | Pins, allowlist, package script, AppImage desktop entry and icon |
| `sdk/` | Vendored by the SDK's `scripts/sdk-sync`; never edit by hand |
| `docs/RESEARCH.md`, `docs/RUNTIME_NOTES.md`, `docs/CONTRIBUTIONS.md` | Findings, verification, drafts to share |

## Checks (all must pass before a commit)

```sh
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --target i686-pc-windows-gnu --workspace --all-targets -- -D warnings
cargo deny check
for t in linux/tests/*.sh packaging/tests/*.sh; do sh "$t"; done
../rea-launcher-sdk/scripts/sdk-sync --check .
cargo build --release --locked --target i686-pc-windows-gnu
sdk/scripts/wine-test --locked --workspace     # Windows-target tests under Wine, headless
sdk/scripts/launcher-check                     # the family's rules: no errors
```

MinGW, lld and Wine may live in the `gaming` distrobox: run the last two as
`distrobox enter gaming -- sh -c 'cd /absolute/path && …'`.

Conventional Commits; every behaviour change gets a CHANGELOG "Unreleased"
entry, and every change that still needs a real-game check a line in
`docs/RUNTIME_NOTES.md`.

## DiRT Rally: what is specific to this game

- **Editions:** two build families. Windows DX11 `drt.exe` (installed copy;
  the pre-activated RELOADED v1.1 executable is SHA-256
  `f7c2dd739e2cd478bfa4bb7238bf121071fb813bc9c84efe3599c80a86d52eea`), run
  under Wine + DXVK on Linux. Feral native Linux port 1.0RC14 (BuildID 31946)
  `bin/DirtRally`, SHA-256
  `de81c75ce02318d6ba89d3d24c463ac311a823d9eb16e676e7a5fb66d6c9e39e`, run
  natively (Linux only).
- **Renderer, input, audio:** Windows: D3D11, XInput, XAudio2/Wwise.
  Linux port: Feral IndirectX (D3D→OpenGL) on bundled SDL2 2.0.5, CEF
  pre-game launcher, ALSA via SDL.
- **Files the game writes:** Windows: `Documents\My Games\DiRT Rally\`
  (`hardwaresettings\hardware_settings_config.xml`, saves). Linux port:
  `$HOME/.local/share/feral-interactive/DiRT Rally/` (`preferences`
  Feral XML registry, `VFS/User/…/hardwaresettings/`, `SaveData/`).
- **Community components used:** DXVK (Windows build under Wine),
  `WINE_CPU_TOPOLOGY` cap for the many-thread crash, legacy runtime
  libraries for the Linux port (see `docs/RESEARCH.md`).
- **Patches:** none to game files. The Linux port gets a small LD_PRELOAD
  library of ours that makes execute-only mappings readable (PKU crash).
- **Graphics:** the game decides its own quality (its menu, its first-boot
  preset); the launcher sets only the size. 4K works natively (verified).
- **Owner decisions:** (2026-11-02) the game is no longer sold, so the
  launcher accepts a **pre-activated** copy, identified by SHA-256, and runs
  it unchanged. It **never applies a crack** and never adds, replaces,
  patches or debugs Steam/DRM files; the README says we can't and won't
  explain how to get such a copy. Local tests may use the owner's copies in
  `.work/` only.
- **Known problems and their status:** `docs/RESEARCH.md` table;
  `docs/RUNTIME_NOTES.md` for what ran.
