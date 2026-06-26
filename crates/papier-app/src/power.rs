//! Power-source polling via IOKit's IOPowerSources C API.
//!
//! IOKit's power-source functions are plain C functions returning
//! CoreFoundation objects, so we declare a minimal FFI surface and reuse the
//! `objc2-core-foundation` CF types for the returned blob + array. We only need
//! a single boolean: "is the machine currently running on battery?".
//!
//! This module does no caching/timer of its own — the app's coordinator polls
//! `on_battery()` on whatever cadence it wants (e.g. a repeating NSTimer).

use objc2_core_foundation::{CFArray, CFDictionary, CFRetained, CFString, CFType};
use std::ffi::c_void;

// IOKit IOPowerSources C API. Linked from the IOKit framework.
#[link(name = "IOKit", kind = "framework")]
extern "C" {
    /// Returns a CFTypeRef blob describing the current power sources.
    fn IOPSCopyPowerSourcesInfo() -> *const c_void;
    /// Returns a CFArrayRef of power-source descriptors (entries into the blob).
    fn IOPSCopyPowerSourcesList(blob: *const c_void) -> *const c_void;
    /// Returns a CFDictionaryRef of details for one source entry.
    fn IOPSGetPowerSourceDescription(
        blob: *const c_void,
        ps: *const c_void,
    ) -> *const c_void;
}

/// Returns `true` if at least one power source reports running on "Battery
/// Power". Returns `false` when on AC / no battery / on any error (fail-open:
/// we'd rather keep the overlay than wrongly pause it).
pub fn on_battery() -> bool {
    unsafe {
        let blob = IOPSCopyPowerSourcesInfo();
        if blob.is_null() {
            return false;
        }
        // Take ownership of the copied blob (Copy => +1 retain).
        let blob_owned: CFRetained<CFType> =
            CFRetained::from_raw(std::ptr::NonNull::new_unchecked(blob as *mut CFType));

        let list = IOPSCopyPowerSourcesList(blob);
        if list.is_null() {
            return false;
        }
        let list_owned: CFRetained<CFArray> =
            CFRetained::from_raw(std::ptr::NonNull::new_unchecked(list as *mut CFArray));

        let count = list_owned.count();
        for i in 0..count {
            let ps = list_owned.value_at_index(i);
            if ps.is_null() {
                continue;
            }
            let desc = IOPSGetPowerSourceDescription(blob, ps);
            if desc.is_null() {
                continue;
            }
            // `desc` is a get (no ownership transfer); borrow it as a dictionary.
            let dict = &*(desc as *const CFDictionary);
            if power_source_is_battery(dict) {
                drop(blob_owned);
                drop(list_owned);
                return true;
            }
        }
        drop(blob_owned);
        drop(list_owned);
        false
    }
}

/// Check the kIOPSPowerSourceStateKey ("Power Source State") for "Battery Power".
fn power_source_is_battery(dict: &CFDictionary) -> bool {
    unsafe {
        let key = CFString::from_static_str("Power Source State");
        let key_ptr = (&*key) as *const CFString as *const c_void;
        let mut value: *const c_void = std::ptr::null();
        let found = CFDictionaryGetValueIfPresent(
            dict as *const CFDictionary,
            key_ptr,
            &mut value as *mut *const c_void,
        );
        if found == 0 || value.is_null() {
            return false;
        }
        let state = &*(value as *const CFString);
        let battery = CFString::from_static_str("Battery Power");
        cf_string_eq(state, &battery)
    }
}

extern "C" {
    fn CFDictionaryGetValueIfPresent(
        the_dict: *const CFDictionary,
        key: *const c_void,
        value: *mut *const c_void,
    ) -> u8;
}

fn cf_string_eq(a: &CFString, b: &CFString) -> bool {
    a.to_string() == b.to_string()
}
