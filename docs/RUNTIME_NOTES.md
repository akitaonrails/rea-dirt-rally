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
