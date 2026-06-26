//! Overlay windows: one borderless click-through `NSPanel` per `NSScreen`.
//!
//! Each panel:
//! - `.nonactivatingPanel` style (never steals focus / key),
//! - `ignoresMouseEvents = true` (clicks pass through to apps below),
//! - level = `NSScreenSaverWindowLevel` (floats above ordinary windows),
//! - collectionBehavior = canJoinAllSpaces | stationary | fullScreenAuxiliary |
//!   ignoresCycle (present on every Space, doesn't move, survives fullscreen),
//! - `isOpaque = false`, clear-ish background, no shadow,
//! - `alphaValue = intensity` (the 15–30% paper-grain strength),
//! - background = a tiled pattern `NSColor` built from the baked grain texture.
//!
//! IMPORTANT (blend-technique honesty): the window server only *alpha-composites*
//! this transparent panel over the screen — it cannot `multiply`/`screen`
//! against the pixels of apps underneath (that's an in-page CSS-only capability
//! the Paperman website uses). So the visual effect here is an alpha-blended
//! dark-fleck grain, which yields the matte/low-contrast feel in practice. See
//! MANUAL-VERIFICATION.md.

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSBackingStoreType, NSColor, NSPanel, NSScreen, NSScreenSaverWindowLevel, NSView,
    NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_foundation::NSRect;

use crate::grain;

/// Build one configured overlay panel covering `frame` (a screen's frame),
/// tiled with the grain pattern for `texture_key` at `intensity`.
fn make_panel(
    mtm: MainThreadMarker,
    frame: NSRect,
    texture_key: &str,
    intensity: f32,
) -> Retained<NSPanel> {
    unsafe {
        let style = NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel;
        let panel = NSPanel::initWithContentRect_styleMask_backing_defer(
            mtm.alloc::<NSPanel>(),
            frame,
            style,
            NSBackingStoreType::Buffered,
            false,
        );

        // Click-through + non-intrusive panel behavior.
        panel.setIgnoresMouseEvents(true);
        panel.setFloatingPanel(true);
        panel.setBecomesKeyOnlyIfNeeded(true);
        // Don't free the panel object when it's ordered out — we reuse / retain it.
        panel.setReleasedWhenClosed(false);

        // Float above normal windows.
        panel.setLevel(NSScreenSaverWindowLevel);

        // Present everywhere, stay put, survive fullscreen, ignore cmd-` cycling.
        let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::IgnoresCycle;
        panel.setCollectionBehavior(behavior);

        // Transparent, shadowless, alpha == intensity.
        panel.setOpaque(false);
        panel.setHasShadow(false);
        panel.setAlphaValue(intensity as f64);

        // Background is the tiled grain pattern. A layer-backed content view tiles
        // the same color so the grain fills the whole screen frame.
        let pattern_image = grain::make_pattern_image(texture_key, 1.0);
        let pattern_color = NSColor::colorWithPatternImage(&pattern_image);
        panel.setBackgroundColor(Some(&pattern_color));

        // Layer-backed content view so CoreAnimation composites the tile cheaply
        // and statically (no per-frame work => ~0% CPU once drawn).
        let content = NSView::initWithFrame(mtm.alloc::<NSView>(), frame);
        content.setWantsLayer(true);
        panel.setContentView(Some(&content));

        panel
    }
}

/// One panel per attached screen, ordered front. The caller owns the returned
/// panels for their lifetime (drop => windows released).
pub fn build_for_all_screens(
    mtm: MainThreadMarker,
    texture_key: &str,
    intensity: f32,
) -> Vec<Retained<NSPanel>> {
    let screens = NSScreen::screens(mtm);
    let mut panels = Vec::with_capacity(screens.len());
    for screen in screens.iter() {
        let frame = screen.frame();
        let panel = make_panel(mtm, frame, texture_key, intensity);
        panel.orderFrontRegardless();
        panels.push(panel);
    }
    panels
}

/// Show / hide a set of panels without rebuilding them.
pub fn set_visible(panels: &[Retained<NSPanel>], visible: bool) {
    for panel in panels {
        if visible {
            panel.orderFrontRegardless();
        } else {
            // orderOut: removes from screen but keeps the object alive.
            panel.orderOut(None);
        }
    }
}

/// Update the alpha (intensity) of existing panels in place.
pub fn set_intensity(panels: &[Retained<NSPanel>], intensity: f32) {
    for panel in panels {
        panel.setAlphaValue(intensity as f64);
    }
}
