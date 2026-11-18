#!/bin/sh
# Checks the bundle that `npm run bundle` builds, without opening it
# (SPEC-UI §4.10, Definition of Done §9), and fails at the first condition
# that does not hold:
# - Portolano.app and its disk image exist;
# - Contents/MacOS holds the app binary `portolano-app` (never `Portolano`,
#   which collides on APFS) and the CLI sidecar `portolano`, the very binary
#   of target/release (same Mach-O UUID: the signature changes the bytes,
#   not the UUID);
# - the sidecar, run from the bundle, answers --version with the version of
#   the app;
# - app and sidecar carry an ad hoc signature with the hardened runtime, and
#   `codesign --verify --deep --strict` accepts the app.
# Then it prints the sizes. Only the sidecar runs, with --version: the app is
# never launched and the Keychain is never touched.
set -eu

root=$(cd "$(dirname "$0")/../.." && pwd)
bundle="$root/target/release/bundle"
app="$bundle/macos/Portolano.app"
macos="$app/Contents/MacOS"
plist="$app/Contents/Info.plist"

fail() {
    echo "check-bundle: $1" >&2
    exit 1
}

# The Mach-O UUID of a binary, the identity the linker gives it.
uuid() {
    dwarfdump --uuid "$1" | awk '{ print $2 }'
}

# The size of a file or a directory, in MiB with one decimal.
size() {
    du -sk "$1" | awk '{ printf "%.1f MiB", $1 / 1024 }'
}

[ -d "$app" ] || fail "manca $app: esegui prima npm run bundle"
version=$(plutil -extract CFBundleShortVersionString raw "$plist") ||
    fail "Info.plist senza CFBundleShortVersionString"

dmg=""
for candidate in "$bundle/dmg/Portolano_${version}_"*.dmg; do
    [ -f "$candidate" ] && dmg=$candidate
done
[ -n "$dmg" ] || fail "manca il .dmg della versione $version in $bundle/dmg"

executable=$(plutil -extract CFBundleExecutable raw "$plist")
[ "$executable" = "portolano-app" ] ||
    fail "l'eseguibile dell'app è '$executable' invece di 'portolano-app'"
for binary in portolano-app portolano; do
    [ -f "$macos/$binary" ] && [ -x "$macos/$binary" ] ||
        fail "manca l'eseguibile Contents/MacOS/$binary"
done

# The sidecar in the bundle is the CLI built in release, not another build.
release=$(uuid "$root/target/release/portolano") ||
    fail "manca target/release/portolano: la CLI compilata in release"
bundled=$(uuid "$macos/portolano") || fail "il sidecar nel bundle non è un binario Mach-O"
[ -n "$release" ] && [ "$bundled" = "$release" ] ||
    fail "il sidecar nel bundle (UUID $bundled) non è la CLI compilata in release (UUID $release)"

answer=$("$macos/portolano" --version) || fail "il sidecar non risponde a --version"
[ "$answer" = "portolano $version" ] ||
    fail "il sidecar risponde '$answer' invece di 'portolano $version'"

for target in "$app" "$macos/portolano"; do
    details=$(codesign -dv --verbose=2 "$target" 2>&1) || fail "$target non è firmato"
    echo "$details" | grep -q '^Signature=adhoc$' || fail "$target non ha una firma ad hoc"
    echo "$details" | grep -Eq '^CodeDirectory .*flags=0x[0-9a-f]+\([^)]*runtime' ||
        fail "$target non ha il runtime rinforzato"
done
codesign --verify --deep --strict "$app" || fail "codesign --verify --deep --strict non passa"

echo "Portolano.app $version: $(size "$app") (portolano-app $(size "$macos/portolano-app"), sidecar portolano $(size "$macos/portolano"))"
echo "$(basename "$dmg"): $(size "$dmg")"
echo "BUNDLE OK"
