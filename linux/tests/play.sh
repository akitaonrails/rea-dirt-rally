#!/bin/sh
# linux/dirtrally-play in native mode, with a fake launcher that records its
# arguments and gamescope switched off, in the repository layout (library
# in ../sdk) and the package layout (library beside the script):
# sh linux/tests/play.sh
set -u
HERE=$(cd "$(dirname "$0")" && pwd -P)
ROOT=$(cd "$HERE/../.." && pwd -P)
T=$(mktemp -d "${TMPDIR:-/tmp}/dirtrally-play-test.XXXXXX")
trap 'rm -rf "$T"' EXIT
export T HOME=$T XDG_CONFIG_HOME=$T/.config XDG_DATA_HOME=$T/.local/share
unset GAMESCOPE_WAYLAND_DISPLAY DRI_PRIME __NV_PRIME_RENDER_OFFLOAD \
  DIRTRALLY_LAUNCHER DIRTRALLY_WAIT DIRTRALLY_GPU DIRTRALLY_SCREEN 2>/dev/null
fails=0
ok() { echo "ok   $1"; }
fail() { echo "FAIL $1"; fails=$((fails + 1)); }
check() { if [ "$2" = "$3" ]; then ok "$1"; else fail "$1: got '$2', want '$3'"; fi; }
contains() { case "$2" in *"$3"*) ok "$1" ;; *) fail "$1: '$2' lacks '$3'" ;; esac; }

# The fake launcher: the window exits with $T/rc; a game run is recorded.
cat > "$T/fake" <<'FAKE'
#!/bin/sh
case "$*" in
  *--settings*) echo "$*" >> "$T/settings"; exit "$(cat "$T/rc" 2>/dev/null || echo 0)" ;;
esac
echo "$*" >> "$T/ran"
FAKE
chmod +x "$T/fake"

mkdir -p "$T/repo/linux" "$T/repo/sdk" "$T/pkg/linux"
cp "$ROOT/linux/dirtrally-play" "$T/repo/linux/"
cp "$ROOT/sdk/rea-play.sh" "$T/repo/sdk/"
cp "$ROOT/linux/dirtrally-play" "$ROOT/sdk/rea-play.sh" "$T/pkg/linux/"
cp "$T/fake" "$T/repo/DiRTRallyLauncher"
cp "$T/fake" "$T/pkg/DiRTRallyLauncher"
# No gamescope on this "host": the game runs directly.
export REA_TEST=1 DIRTRALLY_GPU=

for layout in repo pkg; do
  play=$T/$layout/linux/dirtrally-play
  check "[$layout] status" "$(sh "$play" status)" idle
  rm -f "$T/ran" "$T/settings"
  echo 0 > "$T/rc"
  REA_GAMESCOPE=off sh "$play" > "$T/out" 2>&1; rc=$?
  check "[$layout] window, then the game: exit 0" "$rc" 0
  contains "[$layout] the window opens first" "$(cat "$T/settings" 2>/dev/null)" "--settings --linux"
  contains "[$layout] then the game, without the window" "$(cat "$T/ran" 2>/dev/null)" "--no-window"
  rm -f "$T/ran"
  echo 3 > "$T/rc"
  REA_GAMESCOPE=off sh "$play" > "$T/out" 2>&1
  check "[$layout] window closed without Play: nothing starts" "$(cat "$T/ran" 2>/dev/null)" ""
  rm -f "$T/ran"
  echo 4 > "$T/rc"
  REA_GAMESCOPE=off sh "$play" > "$T/out" 2>&1
  contains "[$layout] no window possible: the game starts anyway" "$(cat "$T/ran" 2>/dev/null)" "--no-window"
  rm -f "$T/ran"
  REA_GAMESCOPE=off sh "$play" --play > "$T/out" 2>&1
  contains "[$layout] --play skips the window" "$(cat "$T/ran" 2>/dev/null)" "--no-window"
  check "[$layout] stop" "$(sh "$play" stop)" stopped
done

echo "# tests source the script: REA_FUNCTIONS_ONLY=1 + REA_PLAY_HERE"
got=$(REA_FUNCTIONS_ONLY=1 REA_PLAY_HERE=$T/repo/linux sh -c '
  . "$1"
  echo "mode=$REA_MODE app=$REA_APP exe=$REA_EXE state=$REA_STATE"' sh "$T/repo/linux/dirtrally-play")
check "configuration" "$got" "mode=native app=dirtrally-launcher exe=$T/repo/DiRTRallyLauncher state=$T/.config/dirtrally-launcher"

[ "$fails" -eq 0 ] || { echo "$fails failed"; exit 1; }
echo "all passed"
