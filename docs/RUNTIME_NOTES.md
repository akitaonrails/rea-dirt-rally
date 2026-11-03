# Runtime notes

What ran in the real game, and what has not been checked yet.

## 2026-11-02: first headless runs (Linux native, ACTiVATED 1.0RC14)

Host: Ryzen 9 7950X3D (32 threads, PKU), RTX 5090, Arch `gaming` distrobox,
`gamescope --backend headless`, isolated `HOME` under `.work/`, game read
in place (read-only).

- Old libraries from the Steam scout runtime, scoped to the game process
  only: `ldd` resolves. (Leaking them into gamescope crashes gamescope.)
- Without a shim: SIGSEGV on the main thread at `0x1f3e7ee`,
  `mov (%rax,%rdi,1),%rbx`, `rax` inside an anonymous `--xp` mapping.
  An 8-CPU `get_nprocs`/`sysconf` shim did **not** help.
- With an exec-implies-read shim (`mmap`/`mprotect`) plus the game's own
  `libsteam_api.so` preload: no crash for 130 s; Feral launcher renders.
- `SeenSpecificationAlertUUIDLnxDistributionUnsupported=1` removes the
  unsupported-distro alert. The pre-game launcher is still shown (see
  RESEARCH #6). `CrashCounter` rises on each forced kill.
- Not yet: pressing Play, menus, a stage, audio (runs used
  `SDL_AUDIODRIVER=dummy`), gamepad, clean quit.

## 2026-11-02: Windows RELOADED install (testing)

- `setup.exe /VERYSILENT /SUPPRESSMSGBOXES` under Wine with the null
  driver stops at the repack's own message box, which ignores
  `/SUPPRESSMSGBOXES`. Killed cleanly; nothing installed. Needs another way
  to install for testing.
- Visible install (owner clicked through): works. The repack's installer
  plays loud background music (`bassmusic.dll`); next time start it with
  Wine audio off (`WINEDLLOVERRIDES=winepulse.drv,winealsa.drv=d`).

## 2026-11-02: Windows RELOADED v1.1 under plain Wine 11.21 (testing)

- The installer already places the pre-activated `drt.exe` (same SHA-256
  as the ISO's `crack/drt.exe`). Owner-approved test: the ISO's `crack/*`
  copied over the scratch install in `.work/` only.
- DXVK 3.1.1 (x32 `d3d11`, `dxgi`, `d3d10core`), gamescope headless, audio
  drivers off: `steam_apir.dll failed to initialize, aborting`, status
  `c0000142`, before any game code runs. Same with
  `WINE_CPU_TOPOLOGY=8:0,…,7`. `steam_apir.dll` imports only system DLLs
  (no missing runtime). That DLL belongs to the crack; per the owner
  decision we don't debug it. The Windows edition is blocked here.
- `drt.exe` imports: d3d11, D3DCOMPILER_43, d3dx11_43, XINPUT1_3, DINPUT8,
  bink2w32 (shipped), steam_api, winhttp, gdiplus, …
- gamescope 3.x segfaults on teardown after the session ends (both runs);
  harmless for the test, worth watching in the helper.

## 2026-11-02: native port plays a stage (from a private copy)

- The game writes into its own folder: CEF creates an empty directory
  `bin/libpdf.so` at start. So it must run from a private runtime copy, never
  in place. (Two early test runs in place created that empty directory in
  the owner's folder; it was removed with `rmdir`, nothing else changed.)
- Scratch runtime: `bin/`, `lib/`, `config/` copied; `share/` pointed at
  the source read-only (test only).
- Feral's pre-game launcher: a click on PLAY (XTest, test tool) starts the
  game. `SDL_AUDIODRIVER=dummy` trips an SDL assertion dialog
  (`open_audio_device … device->hidden != NULL`); `SDL_AUDIODRIVER=disk`
  with `SDL_DISKAUDIOFILE=/dev/null` works headless.
- Title screen ("v1.22", PRESS START OR ENTER) at 1920×1080, attract video.
- `-benchmark`: loads and drives a full Wales stage (Hafren) to the finish
  on all 32 threads, no crash — the many-core crash does not affect this
  native build. Default quality preset after first run: `ultralow`.
- Not yet: real audio (PipeWire), gamepad, quality/resolution settings,
  clean quit, saves.

## 2026-11-02: launcher pieces against the real game

- `dirtrally-exec-read` (our Rust library) instead of the C test shim:
  benchmark stage drives normally.
- `LD_PRELOAD=libsteam_api.so:libdirtrally_exec_read.so` by name, found
  through `LD_LIBRARY_PATH` (runtime `lib/x86_64`, compat libs, the
  library's folder): the game starts as before.
- A fresh home seeded with only
  `SeenSpecificationAlertUUIDLnxDistributionUnsupported=1` in `Setup`
  (`prefs::configure(None)`): the game accepts it, fills in the rest (86
  lines) and keeps the value; no unsupported-distro alert.

## 2026-11-02: skipping Feral's pre-game window

- Decompiled the decision (`0x482f10`, REA/Ghidra): the window is skipped
  when the command line holds `no-feral-options`, or when
  `GameOptionsDialogShown` is set, `GameOptionsDialogShouldShow` is 0 and
  nothing forces it (`GameOptionsDialogForceDialog`, a crash count).
- Skipped with `ScreenW`/`ScreenH` still 0: the game crashes in `WinMain`
  (null read at `0x175d7e4`). With `ScreenW=1920`, `ScreenH=1080`,
  `GameOptionsDialogShown=1`, `GameOptionsDialogShouldShow=0`,
  `CrashCounter=0`: no window, the game goes straight to its intro.
- `gfxconfig_default_preset_name=ultra` with no hardware settings file yet:
  the game writes its Ultra preset itself (3072 shadows, textures 3, 4×MSAA).
  Not wired to an option yet.
- Pending: the launcher binary doing this end to end (the next run).

## 2026-11-02: the launcher end to end (headless, isolated HOME)

- First run, source on the NAS, runtime on local disk: the launcher copied
  11,043 files (40 GB, nothing linkable across disks), wrote the
  preferences, started the game from the copy; intro video and attract
  loop played. (Old binary: Feral's window still shown, PLAY clicked.)
- Packaged launcher (`dist/…-linux-x86_64`, game-libs from the pinned scout
  packages, no host libraries): second start brought 0 files; `look =
  recommended` read the screen as 2560×1440 (xrandr inside gamescope),
  skipped Feral's window, and the game loaded straight to its 2560×1440
  loading screen with no input. SIGTERM: the game exits with status 0.
- Not yet: audio through PipeWire, gamepad, a full race from the menus,
  Steam Deck, the settings window (Linux), the AppImage.

## 2026-11-02: 4K

- No resolution cap in the game data: the sizes in
  `feral_hardware_settings_options.xml` are only first-boot defaults of
  each preset. Feral's `FeralUIResolutionWarningThreshold` (3686400 =
  2560×1440) only drives a warning in the pre-game window, which the
  launcher skips.
- gamescope 3840×2160, `look = recommended`: the launcher set 3840×2160, the
  game wrote `resolution width="3840" height="2160"` itself, and the
  `-benchmark` stage rendered real 3D frames at 3840×2160.
- Owner decision: graphics quality stays the game's (its own menu and
  first-boot preset); the launcher sets only the size. No 4K patch needed.

## 2026-11-02: owner's session (desktop, gamescope from the gaming box)

- Played through `DIRTRALLY_RUN="distrobox enter gaming --"`: gamescope on
  the focused 4K monitor, gamepad mapped by hand in the game's menu, normal
  play.
- Without gamescope (host, no gamescope installed) the game went
  fullscreen on the rotated monitor (DP-2 is Xwayland display 0).
- After a restart the controls were back on the keyboard and the gamepad
  needed mapping again. The profile itself loads (the game reads
  `~/.local/share/Steam/ACTiVATED/310560/remote/savegame@profile#…`, where
  the pre-activated copy keeps saves). The 8BitDo Ultimate 2 (2dc8:310b,
  xpad) is unknown to SDL 2.0.5, so the game sees a plain joystick, not a
  gamepad. Fix: SDL_GAMECONTROLLERCONFIG from sysfs (`pads.rs`). Pending:
  the owner's check that the game now uses its gamepad preset on start.
- Owner's test of `SDL_GAMECONTROLLERCONFIG`: worse, the pad vanished from
  the game's Controls (keyboard only). Feral keeps its own controller list
  (`share/inputdevices.json`: vendor/product, `Category`, `Type`
  `Xbox360Gamepad`, `SDLMapping`); a pad SDL calls a game controller but
  Feral does not list is dropped. The variable is gone; the launcher now
  adds connected standard-layout pads to that list in the private copy,
  rebuilt from the game's own file at each start (`pads::input_devices`).
  Headless: the game loads with the edited list (74 entries, the 8BitDo
  added). Pending: the owner's check in the Controls menu and across a
  restart.
- SDL call log (test preload, headless, pad on): with the game's own list
  the 8BitDo is a plain joystick (`IsGameController 0 -> 0`); with our
  entry the game calls `SDL_GameControllerAddMapping` with the device's
  real GUID (`03000000c82d00000b31000014010000`), opens it as a game
  controller (`IsGameController 0 -> 1`).
- Owner, desktop: the pad was not active at start (profile still on
  keyboard); in Options → Controls → Controls preset the list now offers
  "Controller preset 1/2"; preset 1 came mapped correctly and the gamepad
  worked. Pending: the preset kept across a restart.
- Wireless pads sleep: the launcher now remembers every standard pad it has
  seen (`~/.config/dirtrally-launcher/known-pads.json`), so a pad off at
  start is still in the copy's list.
- Owner: after a restart the game started on the gamepad with Controller
  preset 1, correctly mapped. Gamepad work verified.

## 2026-11-02: the AppImage (native)

- `DiRTRallyLauncher-v0.1.0-x86_64.AppImage --check`: ready to play (its
  bundled game-libs found through `DIRTRALLY_GAME_LIBS`).
- Headless gamescope, the AppImage's helper with its own gamescope off:
  `--play` started the game from the private copy to its title screen;
  `stop` ended the session; no game or gamescope processes left.

## 2026-11-02: the settings window (Linux, headless)

- The real window in headless gamescope at 880×640, four states
  screenshotted (`docs/images/launcher-*.jpg`): no folder (Play off), the
  Windows copy refused with its reason (Play off), ready, and the first
  copy into an empty folder ("2.3 of 43.1 GB", bar moving; the partial
  copy deleted after).
- Through the helper: the window opened, Enter pressed Play, the copy was
  already up to date, the window closed and the game started to its title
  screen; `stop` ended the session, nothing left running.
- Owner, desktop (gamescope from the gaming box): the window works as
  expected, including starting the game.

## 2026-11-02: before the first release

- Independent review; fixed: relative/`~` runtime paths, a non-empty
  foreign runtime folder, preferences that are not UTF-8 (now left alone,
  and backed up once before the first change), an empty `HOME`, an empty
  controller list, `DIRTRALLY_RUN` with spaces in `HOME`, `package.sh`
  dot names, link targets and literal soname matching. Windows path forms
  (`C:\\`, `~\\`, `\\\\server\\share`) tested under Wine.
- The game never wrote under `share/` in any session here: after many runs
  (benchmark, title screen, the owner's play) the only file newer than the
  copy's manifest under `share/` is the launcher's own `inputdevices.json`.
  The game writes `bin/libpdf.so/` (copied, not linked) and its settings
  under `$HOME`.
- After the fixes: the packaged launcher started the game to its title
  screen from the existing copy; SIGTERM ended it with status 0.

## Pending checks (code changed after v0.1.0)

- Original look through the helper on a 4K screen: gamescope now opens at
  3840x2160 (was 1920x1080). Check that Feral's pre-game window offers 4K
  and that the game then fills the screen.
- `dirtrally-play --play --resolution 1280x720`: gamescope opens at
  1280x720 for that run.
