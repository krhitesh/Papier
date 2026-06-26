//! Persistent settings model for Papier.
//!
//! Persisted as JSON to `~/Library/Application Support/Papier/settings.json`.
//! Pure data + serde — no Cocoa dependencies, so the round-trip and clamping
//! logic is unit-testable headlessly.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Intensity is clamped to the spec's 15–30% range.
pub const MIN_INTENSITY: f32 = 0.15;
pub const MAX_INTENSITY: f32 = 0.30;

/// Default texture catalog name (must exist in `paper-grain`'s catalog).
pub const DEFAULT_TEXTURE: &str = "classic-matte";

/// User-facing, persisted configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// Master on/off for the overlay.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Overlay alpha in [0.15, 0.30]. Always re-clamped on load via `normalize`.
    #[serde(default = "default_intensity")]
    pub intensity: f32,
    /// Selected texture catalog name.
    #[serde(default = "default_texture")]
    pub texture: String,
    /// Bundle identifiers for which the overlay is hidden when frontmost.
    #[serde(default)]
    pub exclusion_list: Vec<String>,
    /// Pause the overlay while running on battery.
    #[serde(default)]
    pub pause_on_battery: bool,
    /// Whether the login item (launch-at-login) is registered.
    #[serde(default)]
    pub login_item: bool,
}

fn default_enabled() -> bool {
    true
}
fn default_intensity() -> f32 {
    0.18
}
fn default_texture() -> String {
    DEFAULT_TEXTURE.to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            enabled: default_enabled(),
            intensity: default_intensity(),
            texture: default_texture(),
            exclusion_list: Vec::new(),
            pause_on_battery: false,
            login_item: false,
        }
    }
}

impl Settings {
    /// Re-clamp / sanitize fields that must hold invariants regardless of what
    /// was on disk (a hand-edited file could violate the intensity range).
    pub fn normalize(&mut self) {
        self.intensity = clamp_intensity(self.intensity);
        if self.texture.trim().is_empty() {
            self.texture = DEFAULT_TEXTURE.to_string();
        }
    }

    /// Load from the standard location, falling back to defaults on any error
    /// (missing file, parse failure). Always normalized before return.
    pub fn load() -> Settings {
        let mut s = settings_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|raw| serde_json::from_str::<Settings>(&raw).ok())
            .unwrap_or_default();
        s.normalize();
        s
    }

    /// Persist to the standard location, creating the directory if needed.
    pub fn save(&self) -> std::io::Result<()> {
        let path = settings_path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "no settings path")
        })?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }
}

/// Clamp an intensity into the allowed [15%, 30%] range.
pub fn clamp_intensity(value: f32) -> f32 {
    value.clamp(MIN_INTENSITY, MAX_INTENSITY)
}

/// `~/Library/Application Support/Papier/settings.json`.
pub fn settings_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    let mut p = PathBuf::from(home);
    p.push("Library/Application Support/Papier");
    p.push("settings.json");
    Some(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip_preserves_fields() {
        let original = Settings {
            enabled: false,
            intensity: 0.22,
            texture: "whisper-weave".to_string(),
            exclusion_list: vec!["com.apple.FinalCutPro".into(), "com.figma.Desktop".into()],
            pause_on_battery: true,
            login_item: true,
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn defaults_are_sane() {
        let s = Settings::default();
        assert!(s.enabled);
        assert_eq!(s.texture, DEFAULT_TEXTURE);
        assert!(s.exclusion_list.is_empty());
        assert!(!s.pause_on_battery);
        assert!(!s.login_item);
        assert!(s.intensity >= MIN_INTENSITY && s.intensity <= MAX_INTENSITY);
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        // Only one field present; the rest must default.
        let json = r#"{ "intensity": 0.2 }"#;
        let parsed: Settings = serde_json::from_str(json).unwrap();
        assert!(parsed.enabled);
        assert_eq!(parsed.texture, DEFAULT_TEXTURE);
        assert!((parsed.intensity - 0.2).abs() < 1e-6);
    }

    #[test]
    fn normalize_clamps_intensity_above_range() {
        let mut s = Settings {
            intensity: 0.9,
            ..Default::default()
        };
        s.normalize();
        assert!((s.intensity - MAX_INTENSITY).abs() < 1e-6);
    }

    #[test]
    fn normalize_clamps_intensity_below_range() {
        let mut s = Settings {
            intensity: 0.01,
            ..Default::default()
        };
        s.normalize();
        assert!((s.intensity - MIN_INTENSITY).abs() < 1e-6);
    }

    #[test]
    fn normalize_leaves_in_range_intensity() {
        let mut s = Settings {
            intensity: 0.21,
            ..Default::default()
        };
        s.normalize();
        assert!((s.intensity - 0.21).abs() < 1e-6);
    }

    #[test]
    fn normalize_replaces_empty_texture() {
        let mut s = Settings {
            texture: "   ".into(),
            ..Default::default()
        };
        s.normalize();
        assert_eq!(s.texture, DEFAULT_TEXTURE);
    }

    #[test]
    fn clamp_intensity_endpoints() {
        assert_eq!(clamp_intensity(0.0), MIN_INTENSITY);
        assert_eq!(clamp_intensity(1.0), MAX_INTENSITY);
        assert_eq!(clamp_intensity(MIN_INTENSITY), MIN_INTENSITY);
        assert_eq!(clamp_intensity(MAX_INTENSITY), MAX_INTENSITY);
    }
}
