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
    /// Lowest-octave frequency in cycles across the tile. Matches Paperman's
    /// `baseFrequency` (overlay ~1.5, previews ~0.9–1.2). Rounded to an integer
    /// number of cycles internally so the tile stays seamlessly periodic.
    pub base_frequency: f64,
    /// Number of fBm octaves (Paperman's `numOctaves`, typically 3).
    pub octaves: u32,
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
    let octaves = spec.octaves.max(1);

    let mut amplitude = 1.0f64;
    let mut total_amplitude = 0.0f64;
    let mut period = base_cycles;

    for octave in 0..octaves {
        let octave_seed = spec.seed.wrapping_add(octave.wrapping_mul(0x9E37_79B9));
        for y in 0..size {
            for x in 0..size {
                let u = x as f64 / size as f64;
                let v = y as f64 / size as f64;
                let value = periodic_value_noise(u, v, period, octave_seed);
                accum[(y as usize) * (size as usize) + (x as usize)] += value * amplitude;
            }
        }
        total_amplitude += amplitude;
        amplitude *= 0.5;
        period = period.saturating_mul(2).max(1);
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

/// Look up a named texture spec from the catalog (from the live Paperman site).
///
/// Known names: `"classic-matte"`, `"whisper-weave"`, `"sunbaked-parchment"`.
/// All use `octaves = 3` and `size = 200`.
pub fn texture_spec(name: &str) -> Option<TextureSpec> {
    let (base_frequency, seed) = match name {
        "classic-matte" => (1.5, 1),
        "whisper-weave" => (1.2, 2),
        "sunbaked-parchment" => (0.9, 3),
        _ => return None,
    };
    Some(TextureSpec {
        base_frequency,
        octaves: 3,
        seed,
        size: 200,
    })
}

// --- internal: periodic value noise -----------------------------------------

/// Bilinearly interpolated value noise on a lattice that wraps with `period`
/// cells across the unit square, with a smootherstep fade. Periodic in both
/// axes -> the unit-square sample is seamlessly tileable.
fn periodic_value_noise(u: f64, v: f64, period: u32, seed: u32) -> f64 {
    let period = period.max(1);
    let fx = u * period as f64;
    let fy = v * period as f64;

    let x0 = fx.floor() as i64;
    let y0 = fy.floor() as i64;
    let tx = fx - x0 as f64;
    let ty = fy - y0 as f64;

    let p = period as i64;
    let wrap = |c: i64| -> u32 { c.rem_euclid(p) as u32 };
    let x0w = wrap(x0);
    let y0w = wrap(y0);
    let x1w = wrap(x0 + 1);
    let y1w = wrap(y0 + 1);

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
            base_frequency: 1.5,
            octaves: 3,
            seed: 42,
            size: 200,
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
    fn tileable_edges_wrap() {
        let t = generate_tile(&spec());
        let s = t.size as usize;

        // Left column vs right column.
        let mut col_diff = 0u64;
        for y in 0..s {
            let left = t.pixels[y * s] as i64;
            let right = t.pixels[y * s + (s - 1)] as i64;
            col_diff += (left - right).unsigned_abs();
        }
        let mean_col = col_diff as f64 / s as f64;

        // Top row vs bottom row.
        let mut row_diff = 0u64;
        for x in 0..s {
            let top = t.pixels[x] as i64;
            let bottom = t.pixels[(s - 1) * s + x] as i64;
            row_diff += (top - bottom).unsigned_abs();
        }
        let mean_row = row_diff as f64 / s as f64;

        // Opposite edges are one pixel step apart on a wrapping lattice, so the
        // mean absolute difference must be small (well under a 12/255 tolerance).
        assert!(
            mean_col < 12.0,
            "left/right mean abs diff too high: {mean_col}"
        );
        assert!(
            mean_row < 12.0,
            "top/bottom mean abs diff too high: {mean_row}"
        );
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
        assert!(texture_spec("classic-matte").is_some());
        assert!(texture_spec("whisper-weave").is_some());
        assert!(texture_spec("sunbaked-parchment").is_some());
        assert!(texture_spec("nope").is_none());
        assert_eq!(texture_spec("classic-matte").unwrap().octaves, 3);
        assert_eq!(texture_spec("sunbaked-parchment").unwrap().size, 200);
    }
}
