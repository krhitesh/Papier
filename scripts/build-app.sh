#!/usr/bin/env bash
# Build Papier as a UNIVERSAL (arm64 + x86_64) release binary and assemble a
# Papier.app bundle.
#
# Output: <repo>/target/Papier.app  (a menu-bar agent; LSUIElement=true).
# Self-signed / local run only — no notarization (per spec §2, §8).
set -euo pipefail

# Make cargo available in non-interactive shells.
# shellcheck disable=SC1091
source "$HOME/.cargo/env"

# Resolve repo root from this script's location (scripts/ -> repo root).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

BIN_NAME="papier-app"          # cargo binary name
EXEC_NAME="papier"             # CFBundleExecutable inside the bundle
APP_DIR="$REPO_ROOT/target/Papier.app"
VERSION="${PAPIER_VERSION:-0.0.1}"

echo "==> Building universal release binary ($BIN_NAME: arm64 + x86_64)"
TARGETS=(aarch64-apple-darwin x86_64-apple-darwin)
for t in "${TARGETS[@]}"; do
  rustup target add "$t" >/dev/null 2>&1 || true
  cargo build --release -p papier-app --target "$t" --manifest-path "$REPO_ROOT/Cargo.toml"
done
ARM_BIN="$REPO_ROOT/target/aarch64-apple-darwin/release/$BIN_NAME"
X86_BIN="$REPO_ROOT/target/x86_64-apple-darwin/release/$BIN_NAME"
for b in "$ARM_BIN" "$X86_BIN"; do
  [[ -x "$b" ]] || { echo "error: release binary not found at $b" >&2; exit 1; }
done

echo "==> Assembling $APP_DIR"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# Executable — lipo the two arches into one universal binary.
lipo -create -output "$APP_DIR/Contents/MacOS/$EXEC_NAME" "$ARM_BIN" "$X86_BIN"
chmod +x "$APP_DIR/Contents/MacOS/$EXEC_NAME"
echo "==> Universal binary: $(lipo -archs "$APP_DIR/Contents/MacOS/$EXEC_NAME")"

# Info.plist — LSUIElement agent, bundle id cc.papier.app.
cat > "$APP_DIR/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>Papier</string>
    <key>CFBundleDisplayName</key>
    <string>Papier</string>
    <key>CFBundleIdentifier</key>
    <string>cc.papier.app</string>
    <key>CFBundleExecutable</key>
    <string>$EXEC_NAME</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>LSMinimumSystemVersion</key>
    <string>13.0</string>
    <key>LSUIElement</key>
    <true/>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
PLIST

# Icon. A real .icns is generated from Resources/AppIcon.png if present (see
# tools/gen-icon). Otherwise the bundle still launches with a text menu-bar glyph.
ICON_SRC="$REPO_ROOT/crates/papier-app/resources/AppIcon.png"
ICON_DST="$APP_DIR/Contents/Resources/AppIcon.icns"
if [[ -f "$ICON_SRC" ]] && command -v sips >/dev/null 2>&1 && command -v iconutil >/dev/null 2>&1; then
  echo "==> Generating AppIcon.icns from $ICON_SRC"
  ICONSET="$(mktemp -d)/AppIcon.iconset"
  mkdir -p "$ICONSET"
  for size in 16 32 128 256 512; do
    sips -z "$size" "$size" "$ICON_SRC" --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
    sips -z "$((size*2))" "$((size*2))" "$ICON_SRC" --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
  done
  iconutil -c icns "$ICONSET" -o "$ICON_DST"
else
  echo "==> NOTE: no AppIcon.png source found — icon is a TODO; bundle ships without AppIcon.icns."
fi

# Best-effort ad-hoc code signature so macOS treats it as a stable identity for
# the login-item registration. Non-fatal if codesign is unavailable.
if command -v codesign >/dev/null 2>&1; then
  echo "==> Ad-hoc signing (self-signed; no notarization)"
  codesign --force --deep --sign - "$APP_DIR" || echo "   (codesign failed; continuing unsigned)"
fi

echo "==> Done: $APP_DIR (v$VERSION)"
