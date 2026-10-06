#!/bin/bash
# Makes the disk image people download: the app and a link to Applications.
#
#   scripts/make-dmg.sh [path/to/Modelwise.app] [out.dmg]
#
# Done by hand (blank image, mount, copy, convert) rather than with
# `hdiutil create -srcfolder`, which fails with "Resource busy" on some
# machines, and rather than Tauri's own script, which needs the Finder.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
bundle="$root/src-tauri/target/release/bundle"
app="${1:-$bundle/macos/Modelwise.app}"
version="$(node -p "require('$root/src-tauri/tauri.conf.json').version")"
arch="$(uname -m)"
[ "$arch" = "arm64" ] && arch=aarch64
out="${2:-$bundle/dmg/Modelwise_${version}_${arch}.dmg}"

[ -d "$app" ] || { echo "No app at $app; run \`npm run tauri build -- --bundles app\` first." >&2; exit 1; }

work="$(mktemp -d "${TMPDIR:-/tmp}/modelwise-dmg.XXXXXX")"
trap 'hdiutil detach "$work/mnt" -quiet 2>/dev/null || true; rm -rf "$work"' EXIT

# Room for the app plus a margin for the filesystem.
size_mb=$(( $(du -sm "$app" | cut -f1) + 20 ))

hdiutil create -size "${size_mb}m" -fs HFS+ -volname Modelwise -ov -quiet "$work/rw.dmg"
mkdir "$work/mnt"
hdiutil attach -nobrowse -quiet -mountpoint "$work/mnt" "$work/rw.dmg"
cp -R "$app" "$work/mnt/"
ln -s /Applications "$work/mnt/Applications"
hdiutil detach "$work/mnt" -quiet

mkdir -p "$(dirname "$out")"
rm -f "$out"
hdiutil convert -quiet -format UDZO -imagekey zlib-level=9 -o "$out" "$work/rw.dmg"
echo "$out"
