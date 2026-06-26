//! paper-grain — the shared texture engine for the Papier overlay app and the
//! Ghostty textured themes.
//!
//! Reproduces Paperman's grain recipe:
//!   `feTurbulence type='fractalNoise'` + desaturate (`feColorMatrix saturate=0`)
//!   + seamless tiling (`stitchTiles='stitch'`).
//!
//! The noise is a hand-rolled **periodic value-noise fBm**. Periodicity is the
//! correctness lever: the integer lattice wraps modulo a per-octave period, so
//! sampling the unit square `[0,1)^2` produces a tile whose opposite edges meet
//! continuously — i.e. it is seamlessly tileable, matching SVG's `stitchTiles`.
//!
//! Pure Rust, no macOS / objc dependencies, so it is unit-testable and reused by
//! the Ghostty texture-baking tool.

use std::path::Path;

/// Parameters for one procedural grain texture (a faithful `feTurbulence` analog).
pub struct TextureSpec {
    /// Lowest-octave frequency in **cycles across the tile** (rounded to an
    /// integer internally so the tile stays seamlessly periodic). On a 200px
    /// tile the dominant feature size is `size / base_frequency` px, so fine
    /// paper grain (~3–5px features, matching Paperman's `feTurbulence` shown at
    /// ~900px) needs `base_frequency` in the tens (~32–48), NOT ~1.
    pub base_frequency: f64,
    /// Vertical:horizontal frequency ratio. `1.0` = isotropic grain; values > 1
    /// pack more cycles on the vertical axis, producing a **directional weave**
    /// (linen/fabric). Anisotropic micro-structure scatters specular highlights
    /// across orientations — broader-angle glare diffusion.
    pub aspect: f64,
    /// Number of fBm octaves (Paperman's `numOctaves`, typically 3–5).
    pub octaves: u32,
    /// Amplitude falloff per octave (fBm "persistence"). Lower (~0.4) = smoother,
    /// base-octave-dominated, less haze; higher (~0.65) = rougher tooth with more
    /// high-frequency energy = stronger diffuse scattering (more matte).
    pub persistence: f64,
    /// PRNG seed for the lattice values (deterministic).
    pub seed: u32,
    /// Tile edge length in pixels; output is `size * size` grayscale samples.
    pub size: u32,
}

/// Which way the baked grain pushes the background luminance.
pub enum Polarity {
    /// Dark flecks for light backgrounds — darkens like `mix-blend-mode: multiply`.
    DarkFleck,
    /// Light flecks for dark backgrounds — lightens like `mix-blend-mode: screen`.
    LightFleck,
}

/// A seamlessly tileable grayscale tile. `pixels.len() == size * size`.
pub struct GrayTile {
    pub size: u32,
    pub pixels: Vec<u8>,
}

/// Generate a seamlessly tileable grayscale noise tile from `spec`.
///
/// The result is deterministic for a fixed `(seed, base_frequency, octaves, size)`
/// and its opposite edges are continuous (left≈right, top≈bottom).
pub fn generate_tile(spec: &TextureSpec) -> GrayTile {
    let size = spec.size.max(1);
    let n = (size as usize) * (size as usize);
    let mut accum = vec![0.0f64; n];

    // fBm: sum octaves with doubling frequency and halving amplitude.
    // Each octave's lattice period == its cycle count, keeping every octave
    // periodic over the tile, hence the whole sum is periodic (seamless).
    let base_cycles = spec.base_frequency.round().max(1.0) as u32;
    let aspect = if spec.aspect > 0.0 { spec.aspect } else { 1.0 };
    let persistence = if spec.persistence > 0.0 { spec.persistence } else { 0.5 };
    let octaves = spec.octaves.max(1);

    let mut amplitude = 1.0f64;
    let mut total_amplitude = 0.0f64;
    let mut period_x = base_cycles;
    let mut period_y = ((base_cycles as f64) * aspect).round().max(1.0) as u32;

    for octave in 0..octaves {
        let octave_seed = spec.seed.wrapping_add(octave.wrapping_mul(0x9E37_79B9));
        for y in 0..size {
            for x in 0..size {
                let u = x as f64 / size as f64;
                let v = y as f64 / size as f64;
                let value = periodic_value_noise(u, v, period_x, period_y, octave_seed);
                accum[(y as usize) * (size as usize) + (x as usize)] += value * amplitude;
            }
        }
        total_amplitude += amplitude;
        amplitude *= persistence;
        period_x = period_x.saturating_mul(2).max(1);
        period_y = period_y.saturating_mul(2).max(1);
    }

    let mut pixels = vec![0u8; n];
    for (dst, &val) in pixels.iter_mut().zip(accum.iter()) {
        // Normalize fBm sum from [0, total_amplitude] to [0,1], then to 0..=255.
        let normalized = (val / total_amplitude).clamp(0.0, 1.0);
        *dst = (normalized * 255.0).round() as u8;
    }

    GrayTile { size, pixels }
}

/// Bake a `GrayTile` into an RGBA PNG of paper-grain flecks at `intensity`.
///
/// `intensity` is clamped to `[0.0, 1.0]`. The fleck color is black
/// (`DarkFleck`) or white (`LightFleck`); the grain's deviation from mid-gray
/// drives the per-pixel alpha, scaled by `intensity`. Composited over a
/// background this darkens (multiply feel) or lightens (screen feel) it.
pub fn bake_png(
    tile: &GrayTile,
    polarity: Polarity,
    intensity: f32,
    path: &Path,
) -> std::io::Result<()> {
    let intensity = intensity.clamp(0.0, 1.0);
    let size = tile.size;
    let (fleck_rgb, dark): ([u8; 3], bool) = match polarity {
        Polarity::DarkFleck => ([0, 0, 0], true),
        Polarity::LightFleck => ([255, 255, 255], false),
    };

    let mut rgba = vec![0u8; (size as usize) * (size as usize) * 4];
    for (i, &g) in tile.pixels.iter().enumerate() {
        // Deviation from mid-gray (128) decides how strong a fleck is.
        // DarkFleck uses the darker-than-mid portion; LightFleck the lighter.
        let g = g as f32 / 255.0;
        let strength = if dark {
            (0.5 - g).max(0.0) * 2.0 // 0..1 for g in [0,0.5]
        } else {
            (g - 0.5).max(0.0) * 2.0 // 0..1 for g in [0.5,1]
        };
        let alpha = (strength * intensity * 255.0).round().clamp(0.0, 255.0) as u8;
        let o = i * 4;
        rgba[o] = fleck_rgb[0];
        rgba[o + 1] = fleck_rgb[1];
        rgba[o + 2] = fleck_rgb[2];
        rgba[o + 3] = alpha;
    }

    let buf: image::RgbaImage = image::ImageBuffer::from_raw(size, size, rgba)
        .expect("buffer dimensions match size*size*4");
    buf.save(path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

/// All catalog texture keys, in display order. Shared by the app's texture
/// picker and the tests so they can't drift apart.
pub fn texture_names() -> &'static [&'static str] {
    &[
        "classic-matte",
        "whisper-weave",
        "sunbaked-parchment",
        "saddle-linen",
        "painters-press",
        "vellum-mist",
    ]
}

/// Look up a named texture spec from the catalog.
///
/// Each texture's parameters are chosen for a documented vision-science reason
/// (see `docs/research/textures.md`). Common ground rules, from the anti-glare /
/// legibility literature (Lin et al., *Displays* 2008) and the contrast-
/// sensitivity function: keep the grain FINE (high spatial frequency, small
/// features) so it diffuses specular glare without overlapping the mid spatial
/// frequencies that carry text legibility, and keep amplitude restrained
/// ("more matte is not better" — benefit was flat across a 6× haze range).
/// `base_frequency` is in cycles across the 200px tile (feature ≈ size/base px).
pub fn texture_spec(name: &str) -> Option<TextureSpec> {
    //                          base   aspect  oct  persist  seed
    let (base_frequency, aspect, octaves, persistence, seed) = match name {
        // Balanced fine isotropic micro-roughness. Default.
        "classic-matte" => (48.0, 1.0, 3, 0.50, 1),
        // Faint directional weave at the lowest amplitude — the legibility-first
        // pick for text-heavy / bright-screen reading (lowest effective haze).
        "whisper-weave" => (44.0, 2.0, 3, 0.42, 2),
        // Heavier multi-octave diffuse fiber for long / low-light sessions.
        "sunbaked-parchment" => (34.0, 1.2, 4, 0.62, 3),
        // NEW: pronounced linen cross-weave — multi-orientation glare scatter,
        // best under bright, multi-source light (where the study saw most benefit).
        "saddle-linen" => (32.0, 3.5, 3, 0.50, 4),
        // NEW: cold-press watercolor tooth — roughest (most octaves/persistence),
        // strongest diffuse scattering for the brightest environments.
        "painters-press" => (36.0, 1.0, 5, 0.68, 5),
        // NEW: sheer near-translucent vellum — ultra-fine, lowest amplitude;
        // the legibility extreme (highest spatial frequency, least text overlap).
        "vellum-mist" => (64.0, 1.0, 3, 0.38, 6),
        _ => return None,
    };
    Some(TextureSpec {
        base_frequency,
        aspect,
        octaves,
        persistence,
        seed,
        size: 200,
    })
}

// --- internal: periodic value noise -----------------------------------------

/// Bilinearly interpolated value noise on a lattice that wraps with `period_x`
/// cells horizontally and `period_y` vertically, with a smootherstep fade.
/// Periodic in both axes -> the unit-square sample is seamlessly tileable.
/// Separate per-axis periods give anisotropic (woven) grain.
fn periodic_value_noise(u: f64, v: f64, period_x: u32, period_y: u32, seed: u32) -> f64 {
    let period_x = period_x.max(1);
    let period_y = period_y.max(1);
    let fx = u * period_x as f64;
    let fy = v * period_y as f64;

    let x0 = fx.floor() as i64;
    let y0 = fy.floor() as i64;
    let tx = fx - x0 as f64;
    let ty = fy - y0 as f64;

    let px = period_x as i64;
    let py = period_y as i64;
    let wrap_x = |c: i64| -> u32 { c.rem_euclid(px) as u32 };
    let wrap_y = |c: i64| -> u32 { c.rem_euclid(py) as u32 };
    let x0w = wrap_x(x0);
    let y0w = wrap_y(y0);
    let x1w = wrap_x(x0 + 1);
    let y1w = wrap_y(y0 + 1);

    let c00 = lattice_value(x0w, y0w, seed);
    let c10 = lattice_value(x1w, y0w, seed);
    let c01 = lattice_value(x0w, y1w, seed);
    let c11 = lattice_value(x1w, y1w, seed);

    let sx = smootherstep(tx);
    let sy = smootherstep(ty);

    let top = c00 + sx * (c10 - c00);
    let bottom = c01 + sx * (c11 - c01);
    top + sy * (bottom - top)
}

/// Deterministic hash of integer lattice coords -> value in [0,1).
fn lattice_value(x: u32, y: u32, seed: u32) -> f64 {
    // splitmix64-style avalanche over the packed coords + seed.
    let mut h = (x as u64) << 32 | (y as u64);
    h ^= (seed as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h = h.wrapping_add(0x9E37_79B9_7F4A_7C15);
    h = (h ^ (h >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= h >> 31;
    (h >> 11) as f64 / (1u64 << 53) as f64
}

fn smootherstep(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> TextureSpec {
        TextureSpec {
            base_frequency: 48.0,
            aspect: 1.0,
            octaves: 3,
            persistence: 0.5,
            seed: 42,
            size: 200,
        }
    }

    // Metrics used by the fineness regression tests.
    fn adjacent_h_mean_abs_diff(t: &GrayTile) -> f64 {
        let s = t.size as usize;
        let mut acc = 0u64;
        let mut count = 0u64;
        for y in 0..s {
            for x in 0..s - 1 {
                let a = t.pixels[y * s + x] as i64;
                let b = t.pixels[y * s + x + 1] as i64;
                acc += (a - b).unsigned_abs();
                count += 1;
            }
        }
        acc as f64 / count as f64
    }

    // Std-dev of averages over `blocks`×`blocks` macro-cells. Large blobs => high.
    fn block_avg_std(t: &GrayTile, blocks: usize) -> f64 {
        let s = t.size as usize;
        let bs = s / blocks;
        let mut means = Vec::new();
        for by in 0..blocks {
            for bx in 0..blocks {
                let mut sum = 0u64;
                for y in by * bs..(by + 1) * bs {
                    for x in bx * bs..(bx + 1) * bs {
                        sum += t.pixels[y * s + x] as u64;
                    }
                }
                means.push(sum as f64 / (bs * bs) as f64);
            }
        }
        let mean = means.iter().sum::<f64>() / means.len() as f64;
        let var = means.iter().map(|m| (m - mean).powi(2)).sum::<f64>() / means.len() as f64;
        var.sqrt()
    }

    // REGRESSION (grain too coarse): the shipped catalog textures must be FINE
    // paper grain, not large blobs. Calibrated from measured output —
    // coarse grain (base_frequency ~1–8) gives adj_h < 4 and block8_std > 20;
    // fine grain (the fixed 32–48) gives adj_h > 12 and block8_std < 11.
    // Thresholds sit in the gap so a regression back to ~1.5 fails loudly.
    #[test]
    fn catalog_grain_is_fine_not_coarse() {
        // EVERY catalog texture — improved or new, isotropic or woven — must stay
        // fine (no large blobs), so adding "coarse" character never regresses
        // legibility back toward the ~25px-blob bug.
        for &name in texture_names() {
            let t = generate_tile(&texture_spec(name).unwrap());
            let adj = adjacent_h_mean_abs_diff(&t);
            let blob = block_avg_std(&t, 8);
            assert!(
                adj > 8.0,
                "{name}: grain too smooth/coarse (adj_h={adj:.2}, want > 8.0)"
            );
            assert!(
                blob < 16.0,
                "{name}: large-scale blobs present (block8_std={blob:.2}, want < 16.0)"
            );
        }
    }

    #[test]
    fn deterministic_for_fixed_seed() {
        let a = generate_tile(&spec());
        let b = generate_tile(&spec());
        assert_eq!(a.pixels, b.pixels);
    }

    #[test]
    fn output_dimensions_match_size_squared() {
        let t = generate_tile(&spec());
        assert_eq!(t.size, 200);
        assert_eq!(t.pixels.len(), 200 * 200);
    }

    #[test]
    fn seam_is_continuous() {
        // Seamless tiling means the wrap-around seam (col s-1 -> col 0, which is
        // the next tile's first column) is statistically NO WORSE than a normal
        // interior adjacent-pixel step. (The old test asserted opposite edges
        // were *similar in absolute terms*, which only holds for low-frequency
        // grain and breaks for correct fine grain — that was the bug to fix.)
        let t = generate_tile(&spec());
        let s = t.size as usize;

        let interior_h = adjacent_h_mean_abs_diff(&t);

        let mut seam_col = 0u64;
        for y in 0..s {
            let last = t.pixels[y * s + (s - 1)] as i64;
            let first = t.pixels[y * s] as i64;
            seam_col += (last - first).unsigned_abs();
        }
        let seam_col = seam_col as f64 / s as f64;

        let mut seam_row = 0u64;
        for x in 0..s {
            let last = t.pixels[(s - 1) * s + x] as i64;
            let first = t.pixels[x] as i64;
            seam_row += (last - first).unsigned_abs();
        }
        let seam_row = seam_row as f64 / s as f64;

        assert!(
            seam_col <= interior_h * 1.5,
            "vertical seam discontinuous: seam={seam_col:.2} vs interior={interior_h:.2}"
        );
        assert!(
            seam_row <= interior_h * 1.5,
            "horizontal seam discontinuous: seam={seam_row:.2} vs interior={interior_h:.2}"
        );
    }

    #[test]
    fn baked_grain_has_no_hue() {
        // Research-backed invariant: "no color shift" — the grain is pure
        // luminance (Paperman's feColorMatrix saturate=0). Every baked pixel
        // must be neutral (R == G == B); only alpha varies.
        let t = generate_tile(&spec());
        let dir = std::env::temp_dir();
        for (pol, tag) in [(Polarity::DarkFleck, "dark"), (Polarity::LightFleck, "light")] {
            let path = dir.join(format!("paper_grain_hue_{tag}.png"));
            bake_png(&t, pol, 0.5, &path).unwrap();
            let img = image::open(&path).unwrap().to_rgba8();
            for p in img.pixels() {
                assert!(
                    p[0] == p[1] && p[1] == p[2],
                    "{tag}: baked grain has hue (rgb {},{},{})",
                    p[0],
                    p[1],
                    p[2]
                );
            }
            let _ = std::fs::remove_file(&path);
        }
    }

    #[test]
    fn intensity_bounds_attenuation() {
        // Research-backed invariant: contrast attenuation must stay subtle — the
        // overlay can never push opacity past the configured intensity. At the
        // theme default 0.18, no fleck alpha may exceed ceil(0.18*255).
        let t = generate_tile(&spec());
        let dir = std::env::temp_dir();
        let path = dir.join("paper_grain_intensity.png");
        let intensity = 0.18f32;
        bake_png(&t, Polarity::DarkFleck, intensity, &path).unwrap();
        let img = image::open(&path).unwrap().to_rgba8();
        let max_alpha = img.pixels().map(|p| p[3]).max().unwrap();
        let ceiling = (intensity * 255.0).ceil() as u8; // 47
        assert!(
            max_alpha <= ceiling,
            "alpha {max_alpha} exceeds intensity ceiling {ceiling}"
        );
        assert!(max_alpha > 0, "expected some grain");
    }

    #[test]
    fn bake_png_clamps_and_writes_decodable_file() {
        let t = generate_tile(&spec());
        let dir = std::env::temp_dir();
        let path = dir.join("paper_grain_test_tile.png");

        // intensity > 1.0 must be clamped, not panic or overflow.
        bake_png(&t, Polarity::DarkFleck, 5.0, &path).unwrap();

        let decoded = image::open(&path).unwrap().to_rgba8();
        assert_eq!(decoded.width(), t.size);
        assert_eq!(decoded.height(), t.size);

        // Clamped to 1.0: alpha must never exceed 255 (trivially true for u8,
        // but assert some fleck alpha is present so we know it baked something).
        let max_alpha = decoded.pixels().map(|p| p[3]).max().unwrap();
        assert!(max_alpha > 0, "expected some grain flecks");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn texture_catalog_lookup() {
        // Every advertised name resolves; unknown names don't.
        for &name in texture_names() {
            assert!(texture_spec(name).is_some(), "missing spec for {name}");
        }
        assert_eq!(texture_names().len(), 6);
        assert!(texture_spec("nope").is_none());
        assert_eq!(texture_spec("classic-matte").unwrap().octaves, 3);
        assert_eq!(texture_spec("sunbaked-parchment").unwrap().size, 200);
        // New textures carry their distinguishing character.
        assert!(texture_spec("saddle-linen").unwrap().aspect > 2.0); // woven
        assert!(texture_spec("painters-press").unwrap().persistence > 0.6); // rough
        assert!(texture_spec("vellum-mist").unwrap().base_frequency >= 60.0); // ultra-fine
    }
}
