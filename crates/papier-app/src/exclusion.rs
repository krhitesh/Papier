//! Per-app exclusion: hide the overlay while a front-most app whose bundle id is
//! in the allowlist is active; restore it otherwise.
//!
//! The notification wiring (observing
//! `NSWorkspaceDidActivateApplicationNotification`) lives in `app.rs` because it
//! needs the delegate/coordinator. This module holds the *pure* decision logic
//! and a helper to read the current frontmost bundle id, both testable in
//! isolation (the predicate has unit tests).

use objc2_app_kit::NSWorkspace;

/// Should the overlay be suppressed given the frontmost app's bundle id and the
/// user's exclusion allowlist? `None` => no identifiable frontmost app, never
/// suppress.
pub fn should_suppress(frontmost_bundle_id: Option<&str>, allowlist: &[String]) -> bool {
    match frontmost_bundle_id {
        Some(id) => allowlist.iter().any(|a| a == id),
        None => false,
    }
}

/// Read the bundle identifier of the current frontmost application, if any.
pub fn frontmost_bundle_id() -> Option<String> {
    let workspace = NSWorkspace::sharedWorkspace();
    let app = workspace.frontmostApplication()?;
    let bid = app.bundleIdentifier()?;
    Some(bid.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list() -> Vec<String> {
        vec![
            "com.apple.FinalCutPro".to_string(),
            "com.figma.Desktop".to_string(),
        ]
    }

    #[test]
    fn suppresses_when_frontmost_in_allowlist() {
        assert!(should_suppress(Some("com.figma.Desktop"), &list()));
    }

    #[test]
    fn does_not_suppress_when_not_in_allowlist() {
        assert!(!should_suppress(Some("com.apple.Safari"), &list()));
    }

    #[test]
    fn does_not_suppress_when_no_frontmost() {
        assert!(!should_suppress(None, &list()));
    }

    #[test]
    fn empty_allowlist_never_suppresses() {
        assert!(!should_suppress(Some("com.figma.Desktop"), &[]));
    }
}
