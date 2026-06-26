# Papier

A Rust-native macOS **screen-texture engine** that lays a subtle, static paper-grain
matte over your display to attenuate contrast and glare — plus a set of matching
**Ghostty terminal themes**. Both are driven by one shared grain engine.

It's an independent, native re-implementation of the idea behind
[Paperman](https://paperman.cc) (which ships as a Tauri/WebView app). Papier is
not affiliated with or endorsed by Paperman / DoublOne Studios.

---

## What's in here

Two deliverables built on one engine:

1. **Papier** — a macOS menu-bar agent (`crates/papier-app`) that draws a
   click-through, all-Spaces overlay: a full-coverage neutral matte **veil**
   (lifts blacks, lowers whites → lower contrast) carrying fine grain tooth.
2. **Ghostty themes** — six themes (`ghostty-themes/`): `Paper`, `Forest`,
   `Lamplight`, each in a plain (palette-only) and a textured (paper-grain
   background) variant.

The grain for both comes from **`paper-grain`** (`crates/paper-grain`): a pure-Rust,
seamlessly-tileable fractal-noise engine — the same texture in the terminal and the
overlay.

## The science (and honest limits)

Grounded in the anti-glare / e-paper legibility literature (Lin et al., *Displays*
2008 — Paperman's cited source [1]). Findings that shaped the defaults are written
up in [`docs/research/legibility-tuning.md`](docs/research/legibility-tuning.md):

- A matte/anti-glare surface **improves legibility and reduces visual fatigue**.
- **"More matte is not better"** — a 6× haze range was statistically identical, so
  defaults are kept **low and restrained** (intensity at the floor, neutral tint).
- The effect is **contrast attenuation + diffuse matte**, with **no color shift**
  (the grain is neutral by default; warmth is opt-in).

**Honest caveats:**
- A normal macOS window can only **alpha-composite** over the desktop — it cannot
  `multiply`/`screen` against other apps' pixels. Papier achieves the paper feel by
  alpha-blending a contrast-attenuating veil, not by true blending.
- The cited study used a *physical* matte film diffusing *real* glare on
  *reflective* e-paper; Papier is a *digital* veil over an *emissive* display. The
  literature supports the direction, not exact numeric calibration.

## Repository layout

```
crates/
  paper-grain/            # shared engine: tileable grayscale fractal-noise grain (+ tests)
  papier-app/             # the macOS menu-bar app (Papier)
tools/
  gen-ghostty-textures/   # bakes the textured-theme PNGs
  gen-icon/               # renders the app icon (paper sheet + grain + dog-ear)
ghostty-themes/           # 6 theme files + textures/ + install-ghostty.sh + README
scripts/build-app.sh      # assembles target/Papier.app
docs/
  superpowers/specs/      # design spec
  research/               # legibility-tuning.md (paper findings → knobs)
```

## Requirements

- macOS 13+ (Papier); Ghostty 1.2+ (for the `background-image` themes).
- Rust toolchain — install with `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`.

## Build & install — Papier

```bash
# from the repo root
bash scripts/build-app.sh          # → target/Papier.app (self-signed, no notarization)
ditto target/Papier.app ~/Applications/Papier.app
open ~/Applications/Papier.app     # launches as a menu-bar agent (look for "P")
```

Papier runs as an `LSUIElement` agent (no Dock icon). Quit from the **P** menu or
`killall papier`.

### First launch (Gatekeeper)

Release builds are **ad-hoc signed but not notarized** (no Apple Developer
account), so a *downloaded* copy is quarantined and macOS shows
"Apple could not verify 'Papier'". To run it:

```bash
xattr -dr com.apple.quarantine /Applications/Papier.app   # after moving it into /Applications
open /Applications/Papier.app
```

Or, via the GUI: try to open it once, then **System Settings → Privacy &
Security → Open Anyway** (on macOS 15+/26 right-click → Open no longer bypasses
this). Building from source locally avoids the prompt entirely (local builds
aren't quarantined).

### Using it

Click the **P** menu-bar item:

- **Enabled** — toggle the overlay.
- **Intensity** — veil opacity, clamped 15–30% (default 15%; raise it in bright rooms).
- **Warmth** — neutral → warm cream (default neutral / 0).
- **Texture** — Classic Matte · Whisper Weave · Sunbaked Parchment.
- **Snooze** — hide for 15 / 30 / 60 min.
- **Pause on Battery**, **Launch at Login**, **Quit**.

Settings persist to `~/Library/Application Support/Papier/settings.json`.

## Install — Ghostty themes

```bash
bash ghostty-themes/install-ghostty.sh   # copies themes + textures to ~/.config/ghostty
```

Then set in your Ghostty config, e.g.:

```
theme = papier-paper-textured     # or papier-forest / papier-lamplight (± -textured)
```

See [`ghostty-themes/README.md`](ghostty-themes/README.md) for details.

## Development

```bash
cargo build                              # whole workspace
cargo test                               # paper-grain + papier-app suites
cargo run -p gen-ghostty-textures        # re-bake ghostty-themes/textures/*.png
cargo run -p gen-icon                    # re-render crates/papier-app/resources/AppIcon.png
```

Overlay tuning knobs live in `crates/papier-app/src/grain.rs` (`VEIL_TONE`,
`GRAIN_AMP`) and `crates/papier-app/src/settings.rs` (intensity range, warmth,
defaults). Grain feature size is set by `base_frequency` in
`crates/paper-grain/src/lib.rs`.

GUI behaviors that can't be verified headlessly (multi-monitor, exclusion, battery
pause, login item) are enumerated in
[`crates/papier-app/MANUAL-VERIFICATION.md`](crates/papier-app/MANUAL-VERIFICATION.md).

## Status

Core overlay (matte veil + grain, click-through, all-Spaces, intensity/warmth/texture)
is implemented and verified visually. Remaining behaviors are implemented and listed
for manual verification. Self-signed only — no code signing / notarization.
