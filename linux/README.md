# DiRT Rally on Linux

DiRT Rally's supported version is Feral Interactive's Linux port, so the
launcher runs natively: no Wine or Proton. `dirtrally-play` starts the
game in its own session, inside gamescope when it is installed, and ends
the whole session when you stop it.

## The AppImage

```sh
chmod +x DiRTRallyLauncher-*-x86_64.AppImage
./DiRTRallyLauncher-*-x86_64.AppImage               # the settings window, then the game
./DiRTRallyLauncher-*-x86_64.AppImage --play        # saved settings, no window
./DiRTRallyLauncher-*-x86_64.AppImage stop          # from another terminal
```

The AppImage holds the launcher, its game libraries and `dirtrally-play`;
it takes the same commands and options as the zip's helper.

## The zip

```sh
linux/dirtrally-play             # the settings window, then the game
linux/dirtrally-play --play      # saved settings, no window
linux/dirtrally-play --check     # checks the game folder, starts nothing
linux/dirtrally-play status      # running or idle
linux/dirtrally-play stop
```

Keep the zip's folders together: the helper finds `DiRTRallyLauncher`
one folder up, and the launcher finds its `game-libs` folder beside it.

### Steam Deck

Not checked yet.

## gamescope in a container

If gamescope lives in a distrobox (such as the `gaming` box from
[distrobox-gaming](https://github.com/akitaonrails/distrobox-gaming)), run
the game through it. Your settings, the private copy and the game's saves
stay in your own home folder:

```sh
DIRTRALLY_RUN="distrobox enter gaming --" linux/dirtrally-play
```

Without gamescope the game opens on the desktop's first display, which can
be the wrong monitor (or a rotated one) on a multi-monitor desktop.

## Environment

| Variable | What it does |
|---|---|
| `DIRTRALLY_RUN` | A wrapper for gamescope and the game, e.g. `distrobox enter gaming --` (paths with spaces are not supported here) |
| `DIRTRALLY_LAUNCHER` | Another `DiRTRallyLauncher` program |
| `DIRTRALLY_GAME_LIBS` | Another `game-libs` folder (development builds) |
| `DIRTRALLY_SCREEN` | The screen size gamescope and `fullscreen` use, as `WIDTHxHEIGHT` |
| `DIRTRALLY_GPU` | GPU variables to use instead of the detected ones; empty for none |

## Where things are

| What | Where |
|---|---|
| Settings, logs, remembered pads | `~/.config/dirtrally-launcher/` (`launcher.cfg`, `launcher.log`, `play.log`, `known-pads.json`) |
| Private copy of the game | `~/.local/share/dirtrally-launcher/runtime` |
| The game's own settings | `~/.local/share/feral-interactive/DiRT Rally/` |
| Your saves | where your copy of the game keeps them; the copies tested here use `~/.local/share/Steam/ACTiVATED/310560/remote/` |

## Troubleshooting

- **The game opens on the wrong or a rotated monitor**: use gamescope (see
  **gamescope in a container**), or set `resolution` to your main
  monitor's size.
- **"already running"**: a game session is still open; run
  `dirtrally-play stop`.
- **The pad is not offered in the game**: turn it on, then choose
  **Controller preset 1** in **Options → Controls → Controls preset**.
- **"The launcher's game libraries are missing"**: unpack the whole zip
  again; `game-libs` must stay beside `DiRTRallyLauncher`.
