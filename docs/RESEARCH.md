# DiRT Rally (2015) — research

Phase 1 (community fixes) and Phase 2 (static look at the game) for the
DiRT Rally launcher. Observations, inferences and decisions are kept apart.
Research date: 2026-11-02.

## Owner decisions (recorded)

- **Pre-activated copies only.** DiRT Rally (Steam 310560) was withdrawn
  from sale on 3 Oct 2022 and RaceNet shut down on 8 Nov 2025. The owner
  opened an exception to `STANDARDS.md` "DRM" for this game: the launcher
  accepts a game folder that already runs without Steam, identified by
  SHA-256, and runs it unchanged.
- **The launcher never applies a crack** and never adds, replaces or patches
  Steam or DRM files. The README says a pre-activated copy is required and
  that we can't and won't explain how to make one.
- Local testing may use the owner's own copies in `.work/` (ignored by Git),
  including copying a crack folder over a scratch install there. Nothing from
  them is committed.

## Editions on hand

| Copy | What it is | Identity |
|---|---|---|
| Linux, ACTiVATED | Feral native port, BranchName `DirtRally_1_0`, BuildID 31946, `1.0RC14`, dated 2017-03-01. Crack already applied: `lib/x86_64/libsteam_api.so` is a Steam emulator reading `bin/activated.ini`. The game binary shows no crack strings. | `bin/DirtRally` SHA-256 `de81c75ce02318d6ba89d3d24c463ac311a823d9eb16e676e7a5fb66d6c9e39e` |
| Windows, RELOADED v1.1 | UDF ISO: Inno Setup 5.5.7 `setup.exe`, a 27 GB `setup-1.bin` (FreeArc archive unpacked by the repack's own `innoextractor.dll`), and a separate `crack/` folder (`drt.exe`, Steam emulator `steam_api.dll`, `steamclient.dll`, `steam_api.ini`, …). | `crack/drt.exe` SHA-256 `f7c2dd739e2cd478bfa4bb7238bf121071fb813bc9c84efe3599c80a86d52eea`; installed `drt.exe` not yet seen |

Steam's last Windows patch is v1.23 (2016-10-18); the RELOADED copy is v1.1.

## Engine facts (observed)

- EGO 2.5. Windows: D3D11. Linux: Feral IndirectX (D3D→OpenGL) on SDL2 2.0.5,
  with a CEF-based pre-game launcher (`share/FeralUI`).
- Thread layout comes from `system/hardware_settings_restrictions.xml`
  (`<restrict_if name="cores" …>` → `system/workerMap{2,3,4,6,8}Core.xml`),
  same mechanism as DiRT 2007. No map above 8 cores.
- Linux config: `$HOME/.local/share/feral-interactive/DiRT Rally/` —
  `preferences` (Feral XML registry), `VFS/User/AppData/Roaming/My Games/DiRT Rally/hardwaresettings/`,
  `SaveData/` (links to Steam userdata). Windows config:
  `Documents\My Games\DiRT Rally\hardwaresettings\hardware_settings_config.xml`.
- UDP telemetry: `<motion_platform><udp enabled extradata ip port delay/>`
  in `hardware_settings_config.xml`.

## Problems and community fixes

| # | Problem | Community fix (source) | Decision |
|---|---|---|---|
| 1 | Windows build won't start on CPUs with more than ~15 logical threads (`ASSERT: Failed to initialise the workers`). Editing worker maps did not help users. | `WINE_CPU_TOPOLOGY=8:0,…,7` (ProtonDB 310560; Proton issue #5927, "fails at 16; 15 works"). Proton has no built-in cap for 310560. Windows: msconfig processor limit. | **Configure**: cap at 8 under Wine. Native Windows: start suspended + affinity — unverified idea, test before shipping. |
| 2 | Earlier attempt (distrobox-gaming #6): Windows copy quit ~2–3 s after DXVK init under Proton; crack's `steamclient.dll` fought Proton's `lsteamclient`. | — | Use plain Wine (no `lsteamclient`) plus #1. Inference: the early quit matches #1. To verify. |
| 3 | Native port: legacy libraries missing on modern distros (OpenSSL 1.0.0, libidn11, librtmp0, OpenLDAP 2.4, GConf, gnutls26 chain, nettle6, heimdal, dbus-glib, libffi6). | ArchWiki; harvesting from the Steam scout runtime. | **Bundle** a curated set (open-source, licences to list in `THIRD_PARTY.md`), pinned. Verified: with them `ldd` resolves. |
| 4 | Native port: glibc ≥ 2.37 no longer expands AT_PLATFORM, so RPATH misses `lib/x86_64`. | steam-runtime #613; ferallinuxscripts #3. | **Configure** `LD_LIBRARY_PATH` to the runtime's `lib/x86_64`. |
| 5 | Native port: segfault on the main thread right after `SDL2 initialised` on Zen 3+/PKU CPUs. Observed here: faulting read from an anonymous `--xp` (execute-only) mapping. | mprotect shim (bolokanar, ferallinuxscripts #3, no licence); HN 25818126. | **Our own** small LD_PRELOAD shim (exec ⇒ also read for `mmap`/`mprotect`). Verified: the game gets past init and runs. |
| 6 | Native port: Feral pre-game launcher and an "Unsupported Linux distribution" alert. | — | Alert: **configure** `SeenSpecificationAlertUUIDLnxDistributionUnsupported=1` (verified). Pre-game launcher: still shown; `GameOptionsDialogShouldShow=0`, `MainPregameUIShown=0`, `GameOptionsDialogShouldShowBigPicture=0` and `CrashCounter=0` did not hide it. Open. |
| 7 | Native port: the game script preloads its own `libsteam_api.so`. | Feral `DirtRally.sh`. | Mirror the game's own script environment; never change that file. |
| 8 | RaceNet gone (online events, leagues, ~100 achievements). Offline career works per reports. | None. | Document. Maybe skip the login timeout — idea only. |
| 9 | Windows build under Wine: renderer. | DXVK 2.x. | **Bundle** (pinned), as the family does. |
| 10 | Feral port may stall on 32 threads in races (support branch fixed a 16C/32T crash). | `feral_support_branch` (Steam-only). | Not available to us. Native race not yet tested here. |

Not used: Ultra+ mod (built for v1.02), camera mods (unknown licence),
Revive/VR DLLs (unofficial download), SimHub/RacingDSX (links only).

## Sources

- PCGamingWiki: https://www.pcgamingwiki.com/wiki/DiRT_Rally
- ProtonDB: https://www.protondb.com/app/310560
- Proton #5927: https://github.com/ValveSoftware/Proton/issues/5927
- steam-runtime #613: https://github.com/ValveSoftware/steam-runtime/issues/613
- ferallinuxscripts #3: https://github.com/FeralInteractive/ferallinuxscripts/issues/3
- PKU/execute-only: https://news.ycombinator.com/item?id=25818126
- Windows many-core threads: https://steamcommunity.com/app/310560/discussions/0/2650805184969919589/
- Telemetry format: https://docs.simucube.com/TunerSoftware/games/DirtRally.html
- Delisting: https://traxion.gg/dirt-4-and-dirt-rally-removed-from-digital-sale/ , https://www.ea.com/legal/service-updates/a-h
- RaceNet shutdown: https://delistedgames.com/codemasters-old-dirt-and-grid-titles-lose-online-services-on-march-16th-2026/
