//! Generates Papier's app icon: a warm rounded paper sheet carrying the real
//! `paper-grain` matte texture, with a dog-ear folded top-right corner. Rendered
//! at 1024×1024 with 2×2 supersampling and written to
//! `crates/papier-app/resources/AppIcon.png`, which `scripts/build-app.sh`
//! converts into `AppIcon.icns`.

use std::path::PathBuf;

use paper_grain::{generate_tile, texture_spec, GrayTile};

const N: f64 = 1024.0;
const M: f64 = 90.0; // margin
const L: f64 = M;
const T: f64 = M;
const RT: f64 = N - M; // right
const B: f64 = N - M; // bottom
const R: f64 = 165.0; // corner radius (TL, BL, BR; TR is the fold)
const F: f64 = 300.0; // fold leg length
const C: f64 = (L + (RT - F)) - T; // diagonal: y = x - C, through A=(RT-F,T)

fn lerp3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn dist(x: f64, y: f64, cx: f64, cy: f64) -> f64 {
    ((x - cx).powi(2) + (y - cy).powi(2)).sqrt()
}

/// Inside the rounded-rect silhouette (TR corner is square — the fold fills it).
fn in_sheet(x: f64, y: f64) -> bool {
    if x < L || x > RT || y < T || y > B {
        return false;
    }
    if x < L + R && y < T + R {
        return dist(x, y, L + R, T + R) <= R; // TL
    }
    if x < L + R && y > B - R {
        return dist(x, y, L + R, B - R) <= R; // BL
    }
    if x > RT - R && y > B - R {
        return dist(x, y, RT - R, B - R) <= R; // BR
    }
    true
}

/// Sample the icon at a point → premultiplied-free RGBA in 0..1 (a == coverage).
fn sample(x: f64, y: f64, tile: &GrayTile) -> [f64; 4] {
    if !in_sheet(x, y) {
        return [0.0, 0.0, 0.0, 0.0];
    }

    let top = [0.980, 0.969, 0.949]; // #FAF7F2 warm paper
    let bottom = [0.894, 0.859, 0.804]; // #E4DBCD
    let flap_top = [1.0, 0.996, 0.984];
    let flap_bottom = [0.965, 0.953, 0.929];

    let vt = ((y - T) / (B - T)).clamp(0.0, 1.0);
    // Signed distance to the fold diagonal y = x - C (flap side: y < x - C).
    let dline = (x - C - y) / std::f64::consts::SQRT_2;
    let in_tr_box = x >= RT - F && x <= RT && y >= T && y <= T + F;
    let flap = in_tr_box && y < x - C;

    let mut col = if flap {
        let mut c = lerp3(flap_top, flap_bottom, vt);
        // Soft crease shadow on the flap right at the fold line.
        if dline < 30.0 {
            let k = (dline / 30.0).clamp(0.0, 1.0);
            let s = 0.86 + 0.14 * k;
            c = [c[0] * s, c[1] * s, c[2] * s];
        }
        c
    } else {
        let mut c = lerp3(top, bottom, vt);
        // Matte grain tooth (neutral luminance perturbation).
        let gx = (x as usize) % tile.size as usize;
        let gy = (y as usize) % tile.size as usize;
        let g = tile.pixels[gy * tile.size as usize + gx] as f64 / 255.0 - 0.5;
        let f = 1.0 + g * 2.0 * 0.06;
        c = [
            (c[0] * f).clamp(0.0, 1.0),
            (c[1] * f).clamp(0.0, 1.0),
            (c[2] * f).clamp(0.0, 1.0),
        ];
        // Shadow the sheet under the folded corner (sheet side of the line).
        if in_tr_box && dline < 0.0 && dline > -55.0 {
            let k = (-dline / 55.0).clamp(0.0, 1.0); // 1 at the crease → 0 away
            let shadow = (1.0 - k) * 0.20;
            let s = 1.0 - shadow;
            c = [c[0] * s, c[1] * s, c[2] * s];
        }
        c
    };

    // Subtle darkened rim near the outer silhouette for definition.
    let _ = &mut col;
    [col[0], col[1], col[2], 1.0]
}

fn main() {
    let tile = generate_tile(&texture_spec("classic-matte").expect("catalog has classic-matte"));
    let n = N as u32;
    let mut img = image::RgbaImage::new(n, n);

    // 2×2 supersample for clean rounded edges + diagonal.
    let offsets = [0.25_f64, 0.75];
    for py in 0..n {
        for px in 0..n {
            let mut acc = [0.0f64; 4];
            for oy in offsets {
                for ox in offsets {
                    let s = sample(px as f64 + ox, py as f64 + oy, &tile);
                    // Premultiply so transparent edge samples don't darken color.
                    acc[0] += s[0] * s[3];
                    acc[1] += s[1] * s[3];
                    acc[2] += s[2] * s[3];
                    acc[3] += s[3];
                }
            }
            let a = acc[3] / 4.0;
            let (r, g, b) = if acc[3] > 0.0 {
                (acc[0] / acc[3], acc[1] / acc[3], acc[2] / acc[3])
            } else {
                (0.0, 0.0, 0.0)
            };
            img.put_pixel(
                px,
                py,
                image::Rgba([
                    (r * 255.0).round() as u8,
                    (g * 255.0).round() as u8,
                    (b * 255.0).round() as u8,
                    (a * 255.0).round() as u8,
                ]),
            );
        }
    }

    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/papier-app/resources/AppIcon.png");
    std::fs::create_dir_all(out.parent().unwrap()).expect("create resources dir");
    img.save(&out).expect("write AppIcon.png");
    println!("wrote {} ({}x{})", out.display(), n, n);
}
