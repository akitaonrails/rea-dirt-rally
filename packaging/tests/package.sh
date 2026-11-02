#!/bin/sh
# packaging/package.sh builds a Linux package folder that passes
# packaging/package.allow (no AppImage, no network: stand-in program and
# libraries through REA_RELEASE_DIR and REA_COMPAT_DIR):
# sh packaging/tests/package.sh
set -u
ROOT=$(cd "$(dirname "$0")/../.." && pwd -P)
T=$(mktemp -d "${TMPDIR:-/tmp}/dirtrally-package-test.XXXXXX")
trap 'rm -rf "$T"' EXIT
fails=0
ok() { echo "ok   $1"; }
fail() { echo "FAIL $1"; fails=$((fails + 1)); }

mkdir -p "$T/release" "$T/compat"
printf '#!/bin/sh\n' > "$T/release/DiRTRallyLauncher"
printf 'ELF stand-in\n' > "$T/release/libdirtrally_exec_read.so"
while read -r so; do
  case $so in '' | '#'*) continue ;; esac
  printf 'ELF stand-in\n' > "$T/compat/$so"
done < "$ROOT/packaging/compat-libs.txt"
printf "stand-in licence\n" > "$T/compat/libssl1.0.0.copyright"
# One the game does not need: it must not ship.
printf 'ELF stand-in\n' > "$T/compat/libsqlite3.so.0"

pkg=$T/dist/DiRTRallyLauncher-v0.0.0-linux-x86_64
if REA_RELEASE_DIR=$T/release REA_COMPAT_DIR=$T/compat sh "$ROOT/packaging/package.sh" --no-appimage \
  "$pkg" > "$T/out" 2>&1; then
  ok "package.sh builds the folder and zip"
else
  fail "package.sh: $(cat "$T/out")"
fi
for p in "$pkg" "$pkg.zip"; do
  if sh "$ROOT/sdk/packaging/smoke-package.sh" "$p" "$ROOT/packaging/package.allow" > "$T/out" 2>&1; then
    ok "allowlist passes: $(basename "$p")"
  else
    fail "allowlist: $(basename "$p"): $(cat "$T/out")"
  fi
done
if [ -e "$pkg/game-libs/compat/libsqlite3.so.0" ]; then
  fail "a library the game does not need was shipped"
else
  ok "only the needed libraries ship"
fi
if [ -x "$pkg/DiRTRallyLauncher" ] && [ -x "$pkg/linux/dirtrally-play" ]; then
  ok "the launcher and the helper are executable"
else
  fail "the launcher or the helper is not executable"
fi
rm "$T/compat/libssl.so.1.0.0"
if REA_RELEASE_DIR=$T/release REA_COMPAT_DIR=$T/compat sh "$ROOT/packaging/package.sh" --no-appimage \
  "$T/dist/y" > "$T/out" 2>&1; then
  fail "a missing library must fail"
else
  ok "a missing library fails: $(grep -o 'libssl.so.1.0.0 missing' "$T/out")"
fi
if REA_RELEASE_DIR=$T/nowhere sh "$ROOT/packaging/package.sh" --no-appimage "$T/dist/x" > /dev/null 2>&1; then
  fail "a missing program must fail"
else
  ok "a missing program fails"
fi

if [ "$fails" = 0 ]; then echo "all passed"; exit 0; fi
echo "$fails failed"; exit 1
