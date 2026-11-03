#!/bin/sh
# packaging/package.sh [--no-appimage] <package-folder>
# Builds the Linux package folder (e.g. dist/DiRTRallyLauncher-v0.1.0-linux-x86_64/),
# its .zip beside it, and the native AppImage (sdk/packaging/appimage-wine.sh
# --native) in the same parent folder.
#
# Environment: REA_COMPAT_DIR holds the legacy libraries already (no downloads);
# REA_RELEASE_DIR holds the x86_64 Linux build (DiRTRallyLauncher
# and libdirtrally_exec_read.so; default target/release); REA_CACHE caches
# pinned downloads. Everything the folder holds must be listed in
# packaging/package.allow. Needs ar and tar (binutils) for the .deb files.
set -eu
ROOT=$(cd "$(dirname "$0")/.." && pwd -P)
appimage=1
if [ "${1:-}" = --no-appimage ]; then appimage='' && shift; fi
[ $# -eq 1 ] || { echo "usage: $0 [--no-appimage] <package-folder>" >&2; exit 2; }
pkg=$1
name=$(basename "$pkg")
case $name in '' | .* | *[!A-Za-z0-9._-]*) echo "package: bad package name '$name'" >&2; exit 2 ;; esac
dist=$(dirname "$pkg")
release=${REA_RELEASE_DIR:-$ROOT/target/release}
for f in DiRTRallyLauncher libdirtrally_exec_read.so; do
  [ -f "$release/$f" ] || { echo "package: no $release/$f (cargo build --release --workspace)" >&2; exit 1; }
done
command -v ar >/dev/null || { echo "package: ar (binutils) is needed to unpack the .deb files" >&2; exit 1; }

rm -rf "$pkg"
mkdir -p "$pkg/linux" "$pkg/licenses/compat" "$pkg/game-libs/compat"
cp "$release/DiRTRallyLauncher" "$pkg/"
cp "$release/libdirtrally_exec_read.so" "$pkg/game-libs/"
chmod +x "$pkg/DiRTRallyLauncher"
cp "$ROOT/README.md" "$ROOT/THIRD_PARTY.md" "$ROOT/CHANGELOG.md" \
  "$ROOT/LICENSE-MIT" "$ROOT/LICENSE-APACHE" "$pkg/"
cp "$ROOT/linux/dirtrally-play" "$ROOT/linux/README.md" "$pkg/linux/"
# The helper finds the SDK play library beside itself.
cp "$ROOT/sdk/rea-play.sh" "$pkg/linux/rea-play.sh"
chmod +x "$pkg/linux/dirtrally-play"
cp "$ROOT"/packaging/licenses/*.txt "$pkg/licenses/"

# The legacy libraries: each pinned .deb is checked by SHA-256, then only
# its shared libraries (and its copyright file) are taken.
. "$ROOT/sdk/packaging/pins.sh"
PIN_FILE=$ROOT/packaging/third-party.txt
[ -z "${REA_CACHE:-}" ] || PIN_CACHE=$REA_CACHE
work=$(mktemp -d "${TMPDIR:-/tmp}/dirtrally-package.XXXXXX")
trap 'rm -rf "$work"' EXIT
# REA_COMPAT_DIR: a folder that already holds the libraries and their
# *.copyright files (tests, offline
# builds); the pins are then not fetched.
if [ -n "${REA_COMPAT_DIR:-}" ]; then
  pins=
  cp "$REA_COMPAT_DIR"/*.so.* "$pkg/game-libs/compat/"
  cp "$REA_COMPAT_DIR"/*.copyright "$pkg/licenses/compat/"
else
  pins=$(pin_names | grep '^compat-')
fi
for pin in $pins; do
  deb=$(pin_fetch "$pin" "$work") || exit 1
  unpack=$work/${pin}.d
  mkdir -p "$unpack"
  data=$(ar t "$deb" | grep '^data\.tar') || { echo "package: $pin has no data archive" >&2; exit 1; }
  case $data in
    *.xz) unz="xz -dc" ;;
    *.gz) unz="gzip -dc" ;;
    *.zst) unz="zstd -dc" ;;
    *) unz=cat ;;
  esac
  [ "$unz" = cat ] || command -v "${unz%% *}" >/dev/null || { echo "package: ${unz%% *} is needed to unpack $pin" >&2; exit 1; }
  (cd "$unpack" && ar p "$deb" "$data" | $unz | tar -x -f - --wildcards \
    './lib/x86_64-linux-gnu/*.so.*' './usr/lib/x86_64-linux-gnu/*.so.*' './usr/share/doc/*/copyright' 2>/dev/null || true)
  found=0
  for lib in "$unpack"/lib/x86_64-linux-gnu/*.so.* "$unpack"/usr/lib/x86_64-linux-gnu/*.so.*; do
    [ -e "$lib" ] || continue
    # Keep the soname the game asks for: the file the symlinks point at,
    # under the name of the link (libssl.so.1.0.0, libidn.so.11, ...).
    if [ -L "$lib" ]; then
      # Only a link that stays inside the unpacked package.
      real=$(readlink -f "$lib")
      case $real in "$(readlink -f "$unpack")"/*) ;; *) echo "package: $pin: $(basename "$lib") points outside the package" >&2; exit 1 ;; esac
      cp -L "$lib" "$pkg/game-libs/compat/$(basename "$lib")"
    elif ! ls "$pkg/game-libs/compat/$(basename "$lib")" >/dev/null 2>&1; then
      cp "$lib" "$pkg/game-libs/compat/"
    fi
    found=1
  done
  [ "$found" = 1 ] || { echo "package: $pin held no library" >&2; exit 1; }
  for copyright in "$unpack"/usr/share/doc/*/copyright; do
    [ -f "$copyright" ] && cp "$copyright" "$pkg/licenses/compat/${pin#compat-}.copyright"
  done
done
# Only the sonames the game needs (packaging/compat-libs.txt); the rest of
# each package is dropped, so an old copy never shadows a system library
# nobody asked for.
keep=$ROOT/packaging/compat-libs.txt
for lib in "$pkg"/game-libs/compat/*; do
  grep -qxF "$(basename "$lib")" "$keep" || rm -f "$lib"
done
while read -r so; do
  case $so in '' | '#'*) continue ;; esac
  [ -f "$pkg/game-libs/compat/$so" ] || { echo "package: $so missing from the pinned packages" >&2; exit 1; }
done < "$keep"

(cd "$dist" && rm -f "$name.zip" && zip -qr "$name.zip" "$name")

if [ -n "$appimage" ]; then
  sh "$ROOT/sdk/packaging/appimage-wine.sh" --native --app dirtrally --exe DiRTRallyLauncher \
    --env-prefix DIRTRALLY \
    --icon "$ROOT/packaging/appimage/dirtrally-launcher.png" \
    --desktop "$ROOT/packaging/appimage/dirtrally-launcher.desktop" \
    --payload game-libs --env-path DIRTRALLY_GAME_LIBS=game-libs \
    --doc licenses \
    ${REA_CACHE:+--cache "$REA_CACHE"} \
    --name "${name%-linux-x86_64}" "$pkg" "$dist"
fi
