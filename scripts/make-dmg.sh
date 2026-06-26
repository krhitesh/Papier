#!/usr/bin/env bash
# Build the universal Papier.app and package it into a distributable DMG with a
# drag-to-Applications layout.
#
# Usage: scripts/make-dmg.sh [version]   (default version: 0.0.1)
# Output: <repo>/target/Papier-<version>.dmg
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION="${1:-0.0.1}"

# Build the universal .app (passes version through to Info.plist).
PAPIER_VERSION="$VERSION" bash "$SCRIPT_DIR/build-app.sh"

APP="$REPO_ROOT/target/Papier.app"
DMG="$REPO_ROOT/target/Papier-$VERSION.dmg"

echo "==> Staging DMG contents"
STAGE="$(mktemp -d)"
cp -R "$APP" "$STAGE/Papier.app"
ln -s /Applications "$STAGE/Applications"   # drag-to-install target

echo "==> Creating $DMG"
rm -f "$DMG"
hdiutil create \
  -volname "Papier $VERSION" \
  -srcfolder "$STAGE" \
  -fs HFS+ \
  -format UDZO \
  -ov \
  "$DMG" >/dev/null

rm -rf "$STAGE"
echo "==> Done: $DMG"
echo "    archs: $(lipo -archs "$APP/Contents/MacOS/papier")"
