//! Papier — a Rust-native macOS menu-bar agent that overlays a subtle, static
//! paper-grain texture across every screen to reduce glare/contrast (a clone of
//! the Paperman "digital matte surface").
//!
//! Architecture (spec §6): `app.rs` coordinates `overlay`, `menubar`,
//! `exclusion`, `power`, `settings`, `loginitem`, and `grain` (which wraps the
//! shared `paper-grain` engine).
//!
//! Blend-technique honesty: a normal macOS window can only be alpha-composited
//! over the screen by the window server; it cannot `multiply`/`screen` against
//! other apps' pixels (an in-page CSS capability the website uses). So Papier
//! alpha-blends a baked dark-fleck grain at low opacity. See
//! `MANUAL-VERIFICATION.md`.

mod app;
mod exclusion;
mod grain;
mod loginitem;
mod menubar;
mod overlay;
mod power;
mod settings;

fn main() {
    app::run();
}
