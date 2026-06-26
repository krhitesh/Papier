# Papier — Manual Verification Checklist

This app coordinates the macOS window server, the menu bar, power sources, and
launch-at-login. None of that is exercisable in a headless CI/agent session, so
every runtime behavior below must be verified by a human on a real login GUI
session. The automated bar that **is** met here: the crate compiles
(`cargo build -p papier-app`), the workspace release-builds (`cargo build
--release`), the bundle assembles (`scripts/build-app.sh`), and the headless unit
tests pass (`cargo test -p papier-app` → settings serde round-trip + intensity
clamp, exclusion allowlist logic).

## How to build & launch

```bash
cd <repo-root>
bash scripts/build-app.sh           # produces target/Papier.app
open target/Papier.app              # launches as a menu-bar agent (no Dock icon)
# or run the raw binary for stderr logging:
./target/release/papier-app
```

## Blend-technique note (read first — sets expectations)

The real Paperman **website** blends grain with CSS `mix-blend-mode:
multiply/screen` *inside the page*. A normal macOS desktop window **cannot** do
that: the window server only **alpha-composites** a transparent window over the
screen — it has no access to other apps' pixels to multiply/screen against them.

Therefore Papier's technique is an **alpha-blended baked grain overlay**: a
transparent, click-through `NSPanel` per screen whose background is a tiled
dark-fleck grain pattern, shown at 15–30% alpha. This produces the matte /
low-contrast "paper" feel in practice, but it is *additive darkening by alpha*,
not a true multiply against the desktop. This limitation is stated in code
comments (`overlay.rs`, `grain.rs`, `main.rs`) and is the honest answer to "is
this the same implementation as the closed-source app?": **the look is matched;
the exact native blend is unverifiable from outside.**

## Internals investigation (what we'd inspect IF Paperman were installed)

The real Paperman app is **closed-source and is NOT installed on this machine**,
so the following could not be run. If a human installs it, these are the exact
inspections to mirror/confirm the native technique:

1. **Linked frameworks (Metal vs CoreAnimation path):**
   ```bash
   otool -L "/Applications/Paperman.app/Contents/MacOS/Paperman"
   ```
   Look for `Metal.framework` / `MetalKit` (GPU shader compositing) vs only
   `QuartzCore`/`CoreGraphics` (CoreAnimation layer compositing, which is what
   Papier uses). If Metal is present, Paperman likely renders grain on the GPU;
   the *window-server compositing limitation above still applies* regardless.

2. **Window level & collection behavior (runtime):**
   With Paperman running, inspect its windows:
   ```bash
   # Quartz window dump — find Paperman's window, read kCGWindowLayer.
   # (Compare to Papier's NSScreenSaverWindowLevel == 1000.)
   /usr/bin/python3 -c 'import Quartz, json; \
     print(json.dumps(Quartz.CGWindowListCopyWindowInfo( \
       Quartz.kCGWindowListOptionAll, Quartz.kCGNullWindowID), default=str))' \
     | grep -i paperman
   ```
   Confirm whether it sits at screensaver level, joins all Spaces, and ignores
   mouse events (click-through).

3. **Embedded technical notes / entitlements:**
   ```bash
   codesign -d --entitlements - "/Applications/Paperman.app"
   strings "/Applications/Paperman.app/Contents/MacOS/Paperman" | grep -i \
     'multiply\|screen\|turbulence\|noise\|CAMetalLayer\|CGSWindow'
   ```

**Outcome (CONFIRMED — Paperman 2.12.0 inspected at /Applications/Paperman.app):**
- **Paperman is a Tauri app** (Rust backend + a transparent `WKWebView`): binary
  strings show `__TAURI_INVOKE__`, `__tauriModule`, "Tauri IPC/Isolation
  Pattern", and Rust/`objc2`/`cocoa::appkit::NSWindow::setBackgroundColor`
  symbols. The webview renders the *same website grain* (CSS `feTurbulence` +
  `mix-blend-mode`).
- `otool -L`: links **only CoreGraphics / CoreVideo / QuartzCore — no Metal, no
  ScreenCaptureKit.** Entitlements: app-sandbox, network.client,
  downloads.read-write — **no screen-capture**. So Paperman does **not** read the
  screen or run a GPU shader.
- **Conclusion:** Paperman's `mix-blend-mode` blends grain *within its own
  transparent page*; the composited page is then **alpha-composited over the
  desktop** by the window server — the SAME fundamental compositing as Papier.
  Papier is the native-Rust (objc2 + `NSPanel`) equivalent of what Paperman does
  in a Tauri webview. The "can't true-multiply against other apps' pixels" note
  holds for both.
- **Papier's matte technique (updated):** a full-coverage **neutral
  contrast-attenuation veil** (lifts blacks, lowers whites toward a light paper
  tone) carrying fine grain tooth — not sparse dark flecks. Tunable via
  `VEIL_TONE` / `GRAIN_AMP` in `grain.rs`; intensity via the window `alphaValue`.

## Behavioral checklist (human verification on a GUI session)

### Phase 2 — overlay spike (single + multi screen)
- [ ] On launch a faint paper grain appears across the **entire** main screen.
- [ ] Grain is **static** — no flicker/animation; CPU stays ~0% at idle
      (check Activity Monitor for the `papier` process).
- [ ] **Click-through:** clicking anywhere lands on the app *underneath* the
      overlay (e.g. you can click a Finder icon through the grain). The overlay
      never takes focus / never becomes the key window.
- [ ] Overlay **floats above** ordinary app windows (it is at screensaver level).
- [ ] Overlay is present on **all Spaces** and survives switching Spaces.

### Phase 3 — multi-monitor, intensity, texture, hotplug
- [ ] With 2+ displays, **every** display shows the grain.
- [ ] **Hotplug:** unplug/replug a monitor (or change resolution); the overlay
      rebuilds and re-covers all displays (driven by
      `applicationDidChangeScreenParameters:`).
- [ ] Menu-bar **intensity slider** is clamped to 15%–30%; dragging it changes
      overlay opacity live; you cannot push it outside that range.
- [ ] **Texture picker** switches between Classic Matte / Whisper Weave /
      Sunbaked Parchment; the checkmark moves; the grain pattern visibly changes.

### Phase 4 — menu bar + settings persistence
- [ ] Menu-bar item ("P") opens a menu with: Enabled toggle, Intensity slider,
      texture list, Snooze submenu, Pause on Battery, Launch at Login, Quit.
- [ ] **Enabled** toggle hides/shows the overlay and updates its checkmark.
- [ ] **Quit** terminates the agent and removes the menu-bar item.
- [ ] Settings **persist across restart**: change intensity + texture, quit,
      relaunch — the new values are restored. Confirm the file:
      `~/Library/Application Support/Papier/settings.json`.
- [ ] Hand-edit `intensity` in that JSON to `0.9`, relaunch → it is **re-clamped**
      to 0.30 (normalize on load).

### Phase 5 — per-app exclusion + battery pause
- [ ] Add a bundle id to `exclusion_list` in settings.json (e.g.
      `"com.apple.FinalCutPro"`), relaunch. When that app becomes frontmost the
      overlay **hides**; when another app is frontmost it **restores**.
      (Driven by `NSWorkspaceDidActivateApplicationNotification`.)
- [ ] Enable **Pause on Battery**. Unplug AC power → within ~30s the overlay
      pauses (hides). Replug → it restores. (Polls `IOPSCopyPowerSourcesInfo`
      every 30s.)
- [ ] **Snooze** for 15 min hides the overlay; it auto-restores after the timer
      (shorten the constant for a quick test if desired).

### Phase 6 — bundle + launch-at-login + icon
- [ ] `open target/Papier.app` launches it as a **menu-bar agent** (no Dock icon,
      no app menu) — confirms `LSUIElement=true`.
- [ ] **Launch at Login** toggle registers the app via `SMAppService`. Verify in
      System Settings → General → Login Items that "Papier" appears; toggling it
      off unregisters it. (Login-item registration generally requires the app to
      run from a stable, signed location — `/Applications` — so move the bundle
      there before testing this.)
- [ ] Icon: a real `AppIcon.icns` is a **TODO**. Drop a square PNG at
      `crates/papier-app/resources/AppIcon.png` and re-run `build-app.sh` to
      generate the iconset; until then the bundle ships without an app icon and
      the menu-bar item uses the text glyph "P".

## Known limitations / TODOs surfaced in code
- `// TODO(manual-verify)` in `loginitem.rs`: SMAppService errors are swallowed;
  surface them in the UI once the menu has a status affordance.
- App icon is a placeholder/TODO (see Phase 6 above).
- The alpha-blend technique cannot reproduce a true `multiply` against the
  desktop (window-server limitation) — accepted per spec §6.
