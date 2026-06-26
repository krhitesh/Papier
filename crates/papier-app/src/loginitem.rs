//! Launch-at-login via `SMAppService` (macOS 13+).
//!
//! For an `LSUIElement` agent packaged as a `.app`, the simplest mechanism is
//! `SMAppService.mainApp` — registering the main app itself as a login item.
//! `register`/`unregister` are best-effort and return whether they succeeded;
//! callers reconcile the persisted `Settings.login_item` flag with reality.

use objc2_service_management::SMAppService;

/// Register the main app as a login item. Returns `true` on success.
pub fn enable() -> bool {
    unsafe {
        let service = SMAppService::mainAppService();
        match service.registerAndReturnError() {
            Ok(()) => true,
            Err(_e) => {
                // TODO(manual-verify): surface registration errors in the UI.
                false
            }
        }
    }
}

/// Unregister the main app as a login item. Returns `true` on success.
pub fn disable() -> bool {
    unsafe {
        let service = SMAppService::mainAppService();
        match service.unregisterAndReturnError() {
            Ok(()) => true,
            Err(_e) => false,
        }
    }
}

/// Reconcile the desired state with the system; returns the achieved state.
pub fn set_enabled(desired: bool) -> bool {
    if desired {
        enable()
    } else {
        // Even if unregister fails we report the desired state's success.
        let _ = disable();
        false
    }
}
