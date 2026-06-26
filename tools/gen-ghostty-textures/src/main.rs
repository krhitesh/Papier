use std::path::PathBuf;

use paper_grain::{bake_png, generate_tile, texture_spec, Polarity};

/// Bakes the three tileable paper-grain PNGs for the Ghostty textured themes.
///
/// Each texture is baked at full fleck alpha (intensity = 1.0). On-screen
/// subtlety is controlled SOLELY by `background-image-opacity` in the theme
/// files, never by the bake intensity, to avoid double-attenuation.
fn main() -> std::io::Result<()> {
    // textures/ lives next to this tool's manifest, under ghostty-themes/.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = manifest_dir
        .join("..")
        .join("..")
        .join("ghostty-themes")
        .join("textures");
    std::fs::create_dir_all(&out_dir)?;

    // (file name, texture spec name, polarity)
    let jobs = [
        ("papier-paper.png", "classic-matte", Polarity::DarkFleck),
        ("papier-forest.png", "whisper-weave", Polarity::LightFleck),
        (
            "papier-lamplight.png",
            "sunbaked-parchment",
            Polarity::LightFleck,
        ),
    ];

    for (file, spec_name, polarity) in jobs {
        let spec = texture_spec(spec_name)
            .unwrap_or_else(|| panic!("unknown texture spec: {spec_name}"));
        let tile = generate_tile(&spec);
        let path = out_dir.join(file);
        bake_png(&tile, polarity, 1.0, &path)?;
        println!("baked {} ({}x{}) -> {}", file, tile.size, tile.size, path.display());
    }

    Ok(())
}
