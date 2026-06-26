#!/usr/bin/env bash
#
# install-ghostty.sh — install the Papier Ghostty themes + paper-grain textures.
#
# Copies the 6 theme files to ~/.config/ghostty/themes/ and the baked textures
# to ~/.config/ghostty/textures/, rewriting the __TEXTURE_DIR__ placeholder in
# the textured theme files to that absolute path. Idempotent: re-running
# overwrites the installed copies with fresh ones from this repo.

set -euo pipefail

# Resolve the directory this script lives in (the ghostty-themes/ source dir).
SRC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/ghostty"
THEMES_DST="$CONFIG_DIR/themes"
TEXTURES_DST="$CONFIG_DIR/textures"

THEMES=(
  papier-paper
  papier-forest
  papier-lamplight
  papier-paper-textured
  papier-forest-textured
  papier-lamplight-textured
)

mkdir -p "$THEMES_DST" "$TEXTURES_DST"

# 1. Copy textures to a stable absolute location.
cp -f "$SRC_DIR/textures/"*.png "$TEXTURES_DST/"

# 2. Copy theme files, rewriting __TEXTURE_DIR__ to the installed textures dir.
for theme in "${THEMES[@]}"; do
  src="$SRC_DIR/$theme"
  dst="$THEMES_DST/$theme"
  if [ ! -f "$src" ]; then
    echo "error: missing theme file: $src" >&2
    exit 1
  fi
  # sed is a no-op for plain themes (they contain no placeholder).
  sed "s#__TEXTURE_DIR__#$TEXTURES_DST#g" "$src" > "$dst"
done

echo "Installed ${#THEMES[@]} themes to $THEMES_DST"
echo "Installed textures to $TEXTURES_DST"
echo
echo "Enable a theme by adding to $CONFIG_DIR/config (or config.ghostty), e.g.:"
echo "  theme = papier-lamplight-textured"
