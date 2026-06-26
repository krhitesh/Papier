# Paperman Clone (Rust macOS app) + Ghostty Themes — Design Spec

**Date:** 2026-06-26
**Status:** Approved (pending final spec review)
**Author:** brainstormed with Claude

## 1. Overview

Two deliverables built on **one shared texture engine**:

1. **A Rust-native macOS app** that reproduces the [Paperman](https://paperman.cc) "digital matte surface" — a click-through screen overlay that applies a subtle procedural paper grain to reduce visual contrast/glare.
2. **Six Ghostty terminal themes** derived from Paperman's three color palettes, each in a plain (palette-only) and a textured (paper-grain background) variant.

The paper grain for both comes from a single Rust crate (`paper-grain`), so the terminal textures and the app overlay are literally the same engine.

Reference facts about the real product were extracted verbatim from the live site during design.

### Build order (Ghostty ships first)

Per decision, the lower-risk, faster-value Ghostty themes ship before the app:

1. **Phase 0** — `paper-grain` shared engine (needed by both).
2. **Phase 1** — Ghostty: 6 themes + baked texture PNGs + install script. **← first shippable deliverable.**
3. **Phases 2–6** — the macOS app, decomposed.

## 2. Decisions (locked during brainstorming)

| Topic | Decision |
|---|---|
| App scope | **Full clone** — multi-texture, multi-monitor, per-app exclusion, battery pause, snooze, launch-at-login |
| App architecture | **Pure Rust + Cocoa bindings** via `objc2` family. No Swift. |
| Grain rendering | Pre-generated **tileable fractal-noise** texture, **static** (no animation loop, ~0% CPU), composited per-frame-free. Faithful to Paperman's `feTurbulence` recipe (the low-contrast "A" look, not soft-light/vignette). |
| App packaging | **Proper `.app` bundle** (Info.plist, icon, menu-bar agent, launch-at-login). Self-signed / local run; **no notarization**. |
| App name | **Papier** — the product we ship (clones the original "Paperman"). Crate `papier-app`, bundle `Papier.app`, themes branded "Papier". |
| Ghostty themes | All three palettes (Light "Paper", Dark-green "Forest", Dark-warm "Lamplight"), **plain + textured = 6 themes**. |
| Grain look | Faithful Paperman recipe ("A"): fractal noise, desaturated, `multiply`(light)/`screen`(dark) feel, baked per-theme. |
| Success criterion | **Visual + behavioral parity** with the real app (the verifiable bar), **plus** a best-effort investigation to discover/mirror the native blend technique where findable. |
| Code location | Everything under `~/paperman/` (a Cargo workspace), never loose in `$HOME`. |

## 3. Repository layout

```
~/paperman/
  Cargo.toml                    # workspace
  crates/
    paper-grain/                # LIB: noise → tileable grayscale texture (image buffer + PNG)
    papier-app/                 # BIN: the macOS .app (Papier)
  tools/
    gen-ghostty-textures/       # BIN: bakes textured-theme PNGs (uses paper-grain)
  ghostty-themes/
    papier-paper                # 6 theme files
    papier-paper-textured
    papier-forest
    papier-forest-textured
    papier-lamplight
    papier-lamplight-textured
    textures/                   # baked *.png (build output)
  scripts/
    build-app.sh                # assembles Paperman.app
    install-ghostty.sh          # installs themes + textures to ~/.config/ghostty
  docs/superpowers/specs/
```

## 4. Shared engine — `paper-grain` crate

Single source of truth for the texture. Pure Rust, no macOS dependencies (so it's unit-testable and reused by the texture tool).

**Responsibilities:**
- Generate **fractal/value noise** (via the `noise` crate or a small hand-rolled value-noise fBm) matching Paperman's `feTurbulence type='fractalNoise'`.
- **Desaturate** to grayscale (Paperman's `feColorMatrix saturate=0`).
- Produce a **seamlessly tileable** tile (Paperman's `stitchTiles='stitch'`), verified by a wrap-around continuity test.
- Parameterize per **texture**: `base_frequency`, `octaves`, `seed`, tile size. Catalog (from the site): Classic Matte (fine), Whisper Weave, Sunbaked Parchment (coarser). Frequencies anchored to the site's observed values (overlay ≈1.5, previews ≈0.9–1.2).
- Output as an RGBA image buffer (for the app) and as a PNG file (for Ghostty), via the `image` crate.
- For Ghostty baking: emit **dark-fleck** grain (for the light Paper theme → darkens like multiply) and **light-fleck** grain (for dark themes → lightens like screen), at a configurable intensity.

**Public API (sketch):**
```rust
pub struct TextureSpec { pub base_frequency: f64, pub octaves: u32, pub seed: u32, pub size: u32 }
pub enum Polarity { DarkFleck, LightFleck }     // for baking onto light vs dark backgrounds
pub fn generate_tile(spec: &TextureSpec) -> GrayTile;          // tileable grayscale
pub fn bake_png(tile: &GrayTile, polarity: Polarity, intensity: f32, path: &Path) -> io::Result<()>;
```

**Tests:** determinism given seed; tile dimensions; **tileability** (left edge ≈ right edge, top ≈ bottom within tolerance); intensity clamps.

## 5. Ghostty themes (Phase 1 — ships first)

**Palettes (approved).** Each theme sets `background`, `foreground`, `cursor-color`, `cursor-text`, `selection-background`, `selection-foreground`, and `palette = 0..15`. ANSI palettes were derived to fit each mood (warm/earthy/low-contrast); base 8 colors:

- **Paper (light)** — bg `#F8F5EE`, fg `#1C1A14`, cursor `#7A5C1E`, selection `#EDD7BF`. ANSI 0–7: `#2B2820 #9E4636 #3A6B47 #9A7B1F #3F6680 #7E5468 #4A8478 #6B6250` (brights = lighter/warmer variants ending in `#1C1A14`).
- **Forest (dark green)** — bg `#0C150A`, fg `#E8E5D8`, cursor `#7BA876`, selection `#2A3A24`. ANSI 0–7: `#1F2A1B #C97A5E #7BA876 #C9A85E #6E96A8 #A88AA0 #7FB8A8 #C9C6B8`.
- **Lamplight (dark warm)** — bg `#1A1710`, fg `#EDE6D6`, cursor `#C9962E`, selection `#3C3628`. ANSI 0–7: `#3C3628 #C0664A #8A9A5B #C9962E #7E8CA0 #B07A6B #80A89A #C8BBA8`.

(Full 16-color palettes finalized in the implementation plan; bright variants are lightened/saturated derivatives of the base 8.)

**Plain variants:** colors only.

**Textured variants:** same colors **plus**
```
background-image = <abs path>/textures/<theme>.png
background-image-opacity = 0.18   # light ; 0.13 for dark themes
background-image-repeat = true
background-image-position = center
background-image-fit = none       # tile at native size
```
PNGs baked by `gen-ghostty-textures` using the faithful recipe (dark-fleck for Paper, light-fleck for Forest/Lamplight).

**Verification note (no assumption):** Ghostty `background-image` (added in 1.2.0, Sept 2025) is confirmed supported on the installed version. During implementation, **verify whether `background-image` is honored inside a theme file or must live in a `config-file` include**; structure the textured deliverable accordingly (theme file for colors + an includable snippet for the image if required). No assumption baked in.

**Install:** `scripts/install-ghostty.sh` copies the 6 files to `~/.config/ghostty/themes/` and writes absolute texture paths. README documents `theme = Papier Paper Textured` usage. Theme display names: "Papier Paper", "Papier Forest", "Papier Lamplight" (+ " Textured").

**Phase 1 done when:** all 6 load in Ghostty; plain shows correct palette; textured shows the paper grain at the chosen intensity; install script works on a clean `~/.config/ghostty`.

## 6. Rust macOS app (Phases 2–6)

**Stack:** `objc2`, `objc2-app-kit`, `objc2-foundation`, `objc2-quartz-core`, `objc2-service-management`; `image`; `serde`/`serde_json`. IOKit power sources via the modern objc2 core bindings or a minimal FFI.

**Modules:**
- `overlay/` — one borderless **`NSPanel`** per `NSScreen`: `.nonactivatingPanel`, `ignoresMouseEvents = true` (click-through), high window level (`kCGScreenSaverWindowLevel`), `collectionBehavior` = `canJoinAllSpaces | stationary | fullScreenAuxiliary | ignoresCycle`, `isOpaque = false`, clear background, no shadow. `alphaValue` = intensity. Layer-backed view tiles the grain via a pattern `NSColor`. Rebuild on `NSApplication.didChangeScreenParametersNotification`.
- `grain/` — wraps `paper-grain`; texture catalog + current selection.
- `menubar/` — `NSStatusItem` menu: on/off toggle, intensity `NSSlider` (clamped **15–30%**), texture picker, snooze submenu, quit.
- `exclusion/` — observe `NSWorkspace.didActivateApplicationNotification`; hide overlay when frontmost app's bundle-id ∈ allowlist; restore otherwise.
- `power/` — poll IOKit power source; pause overlay on battery when enabled.
- `settings/` — serde model persisted to `~/Library/Application Support/Papier/settings.json` (intensity, texture, exclusion list, pause-on-battery, enabled, login-item).
- `loginitem/` — `SMAppService` register/unregister.
- `app.rs` / `main.rs` — `NSApplication` (as `LSUIElement` agent), delegate, state coordinator wiring modules.

**Packaging:** `scripts/build-app.sh` builds release binary and assembles `Papier.app` (`Contents/MacOS`, `Info.plist` with `CFBundleName=Papier`, `LSUIElement=true`, `Resources/AppIcon.icns`). Self-signed/local; no notarization.

### Known technical risk + investigation (Phase 2 spike)

The website blends grain with **`mix-blend-mode: multiply/screen` inside the page**. A normal macOS desktop window **cannot** multiply/screen against *other apps'* pixels — the window server only alpha-blends a transparent window over the screen. Therefore the app's baseline technique is **alpha-blending a baked grain texture** at low opacity (dark-baked for a matte/darkening feel), which produces the paper effect in practice.

Because the real app is **closed-source**, "same implementation" is unverifiable from outside. Resolution:
- **Primary success bar = visual + behavioral parity** (grain look, 15–30% intensity, no color shift, static/0% CPU, click-through, all monitors, exclusion, battery pause).
- **Best-effort internals investigation (Phase 2):** research the native technique — inspect the shipped binary if installed (linked frameworks, Metal vs CoreAnimation, window level/collection behavior), check any public technical notes — and mirror it where discoverable. **Fall back to alpha-blend** if inconclusive.
- **Phase 2 is a spike** that must confirm the overlay floats above apps, passes clicks through, and *looks right* before later phases build on it.

## 7. Phasing & verification

| Phase | Delivers | Verify |
|---|---|---|
| 0 | `paper-grain` engine | unit tests: determinism, dimensions, **tileability**, intensity clamp |
| 1 | **Ghostty: 6 themes + baked PNGs + install script (SHIP)** | all load; plain palette correct; textured grain visible; install script clean-run |
| 2 | App overlay spike: 1 click-through grain panel on main screen + internals investigation | floats above apps; clicks pass through; paper feel acceptable; technique decision recorded |
| 3 | Multi-monitor panels, texture catalog, intensity, screen-change handling | every display covered; intensity clamps 15–30%; survives monitor hotplug |
| 4 | Menu-bar UI + settings persistence | toggle/slider/picker work; settings survive restart |
| 5 | Per-app exclusion + battery pause | overlay hides for listed apps; pauses on battery |
| 6 | `.app` bundle + launch-at-login + icon | double-click launches as menu-bar agent; starts at login |

**Testing strategy:** `paper-grain`, settings serde round-trip, exclusion-allowlist logic, and intensity clamping get Rust unit tests. Cocoa/overlay behavior is verified **manually per phase** (window server interactions are not unit-testable) — each phase lists its manual check above.

## 8. Out of scope

- Code signing / notarization / distribution outside this machine.
- Windows support (Paperman is cross-platform; this build is macOS-only).
- The website's typographic/gradient polish in the terminal (a terminal carries palette + texture only — explicitly accepted).
- Live-animated grain (real app is static; we match that).

## 9. Dependencies / prerequisites

- Install Rust toolchain via `rustup` (not currently present).
- Xcode 26 + macOS 26.2 present. Ghostty 1.2+ installed (`background-image` confirmed).
- Grain crate to be chosen during Phase 0 (`noise` crate vs hand-rolled value-noise) based on tileability quality.
