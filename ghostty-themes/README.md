# Papier — Ghostty themes

Six Ghostty terminal themes derived from Paperman's three colour palettes, each
in a **plain** (palette only) and a **textured** (procedural paper-grain
background) variant.

| Theme file                  | Mood        | Background | Texture |
|------------------------------|-------------|------------|---------|
| `papier-paper`               | Light       | `#F8F5EE`  | —       |
| `papier-paper-textured`      | Light       | `#F8F5EE`  | dark-fleck grain @ 0.18 |
| `papier-forest`              | Dark green  | `#0C150A`  | —       |
| `papier-forest-textured`     | Dark green  | `#0C150A`  | light-fleck grain @ 0.13 |
| `papier-lamplight`           | Dark warm   | `#1A1710`  | —       |
| `papier-lamplight-textured`  | Dark warm   | `#1A1710`  | light-fleck grain @ 0.13 |

The grain comes from the shared `paper-grain` Rust crate, baked to tileable
200×200 RGBA PNGs by `tools/gen-ghostty-textures`.

## Install

```sh
./install-ghostty.sh
```

This copies the 6 theme files to `~/.config/ghostty/themes/`, copies the baked
textures to `~/.config/ghostty/textures/`, and rewrites the `__TEXTURE_DIR__`
placeholder in the textured themes to that absolute path. It is idempotent —
re-run it any time to refresh the installed copies (e.g. after re-baking
textures). Honours `$XDG_CONFIG_HOME` if set.

## Enable

Add to your Ghostty config (`~/.config/ghostty/config`, or on macOS
`~/Library/Application Support/com.mitchellh.ghostty/config.ghostty`):

```
theme = papier-lamplight-textured
```

The theme name is the file's name. All six show up under
`ghostty +list-themes` as `(user)` themes once installed.

## Tuning the grain intensity

The on-screen subtlety of the paper grain is controlled **solely** by
`background-image-opacity` in the textured theme files — the PNGs are baked at
full fleck alpha so there is no double attenuation. Ships at:

- `0.18` for the light `papier-paper-textured`
- `0.13` for the dark `papier-forest-textured` / `papier-lamplight-textured`

To make the texture more/less visible, edit `background-image-opacity` in the
installed theme file (`~/.config/ghostty/themes/papier-*-textured`) and reload
Ghostty's config. No re-bake needed.

## Verification finding — does `background-image` work inside a theme file?

**Yes.** A theme file in `~/.config/ghostty/themes/` may set `background-image`
and the related `background-image-*` keys directly; Ghostty merges them into the
resolved config exactly like the colour keys. No `config-file` include or a
separate snippet is required, so the textured themes are self-contained.

Verified on **Ghostty 1.3.1** (`background-image` added in 1.2.0) by pointing
the user config at a textured theme and inspecting the resolved config:

```
$ # config.ghostty: theme = papier-lamplight-textured   (theme installed in ~/.config/ghostty/themes/)
$ ghostty +show-config | grep -iE 'background|foreground'
background = #1a1710
foreground = #ede6d6
background-image = /Users/.../.config/ghostty/textures/papier-lamplight.png
background-image-opacity = 0.13
```

`ghostty +validate-config` returns OK for all six themes, and
`ghostty +list-themes` lists all six as `(user)` themes.
