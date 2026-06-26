//! Grain texture catalog + rendering glue around the shared `paper-grain` engine.
//!
//! Responsibilities:
//! - Expose the texture catalog and the currently-selected texture.
//! - Turn a `paper-grain` `GrayTile` into an RGBA byte buffer of paper-grain
//!   flecks at a given intensity (the same recipe `bake_png` uses, but in-memory
//!   so the overlay never touches disk).
//! - Build an `NSImage` from that buffer so the overlay view can tile it as a
//!   pattern `NSColor`.
//!
//! Blend-technique note (see also MANUAL-VERIFICATION.md): a normal macOS window
//! can only be *alpha-composited* over the screen by the window server — it
//! cannot `multiply`/`screen` against other apps' pixels the way the Paperman
//! website's `mix-blend-mode` does in-page. So we bake **dark flecks** (a matte /
//! darkening feel) into a transparent texture and alpha-blend it at low opacity.

use objc2::rc::Retained;
use objc2::AnyThread;
use objc2_app_kit::{NSBitmapImageRep, NSImage};
use objc2_foundation::NSSize;
use paper_grain::{generate_tile, GrayTile, Polarity, TextureSpec};

/// One entry in the texture catalog.
pub struct TextureEntry {
    /// Catalog key understood by `paper_grain::texture_spec`.
    pub key: &'static str,
    /// Human-readable name for the menu.
    pub display: &'static str,
}

/// The catalog mirrors `paper-grain`'s known names (from the live Paperman site).
pub const CATALOG: &[TextureEntry] = &[
    TextureEntry {
        key: "classic-matte",
        display: "Classic Matte",
    },
    TextureEntry {
        key: "whisper-weave",
        display: "Whisper Weave",
    },
    TextureEntry {
        key: "sunbaked-parchment",
        display: "Sunbaked Parchment",
    },
];

/// Look up the display name for a catalog key (falls back to the key itself).
/// Part of the catalog's public surface (e.g. for tooltips); not yet wired into
/// the menu, which iterates `CATALOG` directly.
#[allow(dead_code)]
pub fn display_name(key: &str) -> &str {
    CATALOG
        .iter()
        .find(|e| e.key == key)
        .map(|e| e.display)
        .unwrap_or(key)
}

/// Resolve a catalog key to a spec, falling back to `classic-matte`.
fn spec_for(key: &str) -> TextureSpec {
    paper_grain::texture_spec(key)
        .or_else(|| paper_grain::texture_spec("classic-matte"))
        .expect("classic-matte is always a valid catalog entry")
}

/// Generate the grayscale tile for a catalog key.
pub fn tile_for(key: &str) -> GrayTile {
    generate_tile(&spec_for(key))
}

/// Bake a `GrayTile` into an in-memory premultiplied-RGBA fleck buffer.
///
/// This mirrors `paper_grain::bake_png`'s recipe (deviation from mid-gray drives
/// alpha) but keeps the bytes in memory. We always use `DarkFleck` for the
/// overlay (matte darkening feel against an arbitrary desktop).
fn bake_rgba(tile: &GrayTile, intensity: f32) -> Vec<u8> {
    let intensity = intensity.clamp(0.0, 1.0);
    // DarkFleck => black flecks; deviation below mid-gray sets alpha.
    let _ = Polarity::DarkFleck; // documents the chosen polarity
    let size = tile.size as usize;
    let mut rgba = vec![0u8; size * size * 4];
    for (i, &g) in tile.pixels.iter().enumerate() {
        let g = g as f32 / 255.0;
        let strength = (0.5 - g).max(0.0) * 2.0; // 0..1 for g in [0,0.5]
        let alpha = (strength * intensity * 255.0).round().clamp(0.0, 255.0) as u8;
        let o = i * 4;
        // Black fleck, premultiplied alpha => RGB stay 0.
        rgba[o] = 0;
        rgba[o + 1] = 0;
        rgba[o + 2] = 0;
        rgba[o + 3] = alpha;
    }
    rgba
}

/// Build an `NSImage` tile from a catalog key + intensity, suitable for use as a
/// pattern color (`NSColor::colorWithPatternImage`). The image is the native
/// tile size (e.g. 200x200) and is meant to be repeated.
pub fn make_pattern_image(key: &str, intensity: f32) -> Retained<NSImage> {
    let tile = tile_for(key);
    let rgba = bake_rgba(&tile, intensity);
    let size = tile.size;

    unsafe {
        // Allocate an NSBitmapImageRep we own and copy our RGBA bytes into its
        // backing store. 4 samples/px, 8 bits each, premultiplied alpha.
        let rep: Retained<NSBitmapImageRep> = NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bitmapFormat_bytesPerRow_bitsPerPixel(
            NSBitmapImageRep::alloc(),
            std::ptr::null_mut(), // let AppKit allocate the planes
            size as isize,
            size as isize,
            8,
            4,
            true,
            false,
            objc2_app_kit::NSDeviceRGBColorSpace,
            // bitmapFormat == 0 => 32-bit RGBA, alpha last, premultiplied.
            objc2_app_kit::NSBitmapFormat::empty(),
            (size as isize) * 4,
            32,
        )
        .expect("NSBitmapImageRep allocation");

        // Copy bytes into the rep's pixel buffer.
        let data_ptr = rep.bitmapData();
        if !data_ptr.is_null() {
            std::ptr::copy_nonoverlapping(rgba.as_ptr(), data_ptr, rgba.len());
        }

        let image = NSImage::initWithSize(
            NSImage::alloc(),
            NSSize::new(size as f64, size as f64),
        );
        image.addRepresentation(&rep);
        image
    }
}
