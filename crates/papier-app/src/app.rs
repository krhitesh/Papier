//! Application coordinator: the `NSApplication` (as an `LSUIElement`/accessory
//! agent), the `PapierDelegate` that owns mutable app state, and the wiring that
//! connects every module:
//!
//! - builds overlay panels per screen,
//! - builds the menu-bar status item,
//! - observes screen-parameter changes (rebuild panels) and app-activation
//!   (per-app exclusion),
//! - polls the power source on a repeating timer (battery pause),
//! - applies menu actions (toggle, intensity, texture, snooze, login item).
//!
//! State lives in a `RefCell<AppState>` ivar on the delegate. The delegate is
//! `MainThreadOnly` (it implements `NSApplicationDelegate`), so all access is
//! single-threaded — no locking needed.

use std::cell::RefCell;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSPanel, NSStatusItem,
    NSWorkspace, NSWorkspaceActiveSpaceDidChangeNotification,
    NSWorkspaceDidActivateApplicationNotification,
};
use objc2_foundation::{
    NSNotification, NSNotificationCenter, NSObject, NSObjectProtocol, NSTimer,
};

use crate::grain::CATALOG;
use crate::settings::{clamp_intensity, clamp_warmth, Settings};
use crate::{exclusion, loginitem, overlay, power};

/// How often (seconds) we poll the power source for battery state.
const POWER_POLL_INTERVAL: f64 = 30.0;

/// All mutable runtime state, held behind a `RefCell` ivar.
pub struct AppState {
    pub settings: Settings,
    pub panels: Vec<Retained<NSPanel>>,
    pub status_item: Option<Retained<NSStatusItem>>,
    /// True while snoozed (overlay temporarily forced hidden).
    pub snoozed: bool,
    /// True while suppressed by an excluded frontmost app.
    pub suppressed_by_app: bool,
    /// True while paused due to running on battery.
    pub paused_by_battery: bool,
}

impl AppState {
    /// The overlay should be visible iff enabled and none of the suppressors
    /// (snooze, exclusion, battery) are active.
    fn should_show(&self) -> bool {
        self.settings.enabled
            && !self.snoozed
            && !self.suppressed_by_app
            && !self.paused_by_battery
    }
}

define_class!(
    // SAFETY:
    // - Superclass NSObject has no subclassing requirements.
    // - PapierDelegate does not implement Drop.
    // - Marked MainThreadOnly because it implements NSApplicationDelegate.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "PapierDelegate"]
    #[ivars = RefCell<AppState>]
    struct PapierDelegate;

    impl PapierDelegate {
        #[unsafe(method(toggleEnabled:))]
        fn toggle_enabled(&self, _sender: Option<&AnyObject>) {
            {
                let mut st = self.ivars().borrow_mut();
                st.settings.enabled = !st.settings.enabled;
            }
            self.persist();
            self.reconcile_visibility();
            self.rebuild_menu();
        }

        #[unsafe(method(intensityChanged:))]
        fn intensity_changed(&self, sender: Option<&AnyObject>) {
            // `sender` is the NSSlider; read its doubleValue.
            let value: f64 = match sender {
                Some(s) => unsafe { msg_send![s, doubleValue] },
                None => return,
            };
            let clamped = clamp_intensity(value as f32);
            {
                let mut st = self.ivars().borrow_mut();
                st.settings.intensity = clamped;
                overlay::set_intensity(&st.panels, clamped);
            }
            self.persist();
        }

        #[unsafe(method(warmthChanged:))]
        fn warmth_changed(&self, sender: Option<&AnyObject>) {
            let value: f64 = match sender {
                Some(s) => unsafe { msg_send![s, doubleValue] },
                None => return,
            };
            let clamped = clamp_warmth(value as f32);
            {
                let mut st = self.ivars().borrow_mut();
                st.settings.warmth = clamped;
            }
            // Re-bake the grain pattern in place (no panel teardown).
            {
                let st = self.ivars().borrow();
                overlay::set_pattern(&st.panels, &st.settings.texture, clamped);
            }
            self.persist();
        }

        #[unsafe(method(pickTexture:))]
        fn pick_texture(&self, sender: Option<&AnyObject>) {
            let tag: isize = match sender {
                Some(s) => unsafe { msg_send![s, tag] },
                None => return,
            };
            if let Some(entry) = CATALOG.get(tag as usize) {
                let (texture, warmth) = {
                    let mut st = self.ivars().borrow_mut();
                    st.settings.texture = entry.key.to_string();
                    (st.settings.texture.clone(), st.settings.warmth)
                };
                // Re-bake the grain pattern in place (no panel teardown).
                {
                    let st = self.ivars().borrow();
                    overlay::set_pattern(&st.panels, &texture, warmth);
                }
                self.persist();
                self.rebuild_menu();
            }
        }

        #[unsafe(method(snooze:))]
        fn snooze(&self, sender: Option<&AnyObject>) {
            let minutes: isize = match sender {
                Some(s) => unsafe { msg_send![s, tag] },
                None => return,
            };
            {
                let mut st = self.ivars().borrow_mut();
                st.snoozed = true;
            }
            self.reconcile_visibility();
            // Schedule un-snooze.
            unsafe {
                let _timer = NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                    (minutes as f64) * 60.0,
                    self,
                    sel!(endSnooze:),
                    None,
                    false,
                );
            }
        }

        #[unsafe(method(endSnooze:))]
        fn end_snooze(&self, _timer: Option<&AnyObject>) {
            {
                let mut st = self.ivars().borrow_mut();
                st.snoozed = false;
            }
            self.reconcile_visibility();
        }

        #[unsafe(method(togglePauseOnBattery:))]
        fn toggle_pause_on_battery(&self, _sender: Option<&AnyObject>) {
            {
                let mut st = self.ivars().borrow_mut();
                st.settings.pause_on_battery = !st.settings.pause_on_battery;
            }
            self.persist();
            self.poll_power();
            self.rebuild_menu();
        }

        #[unsafe(method(toggleLoginItem:))]
        fn toggle_login_item(&self, _sender: Option<&AnyObject>) {
            let desired = {
                let st = self.ivars().borrow();
                !st.settings.login_item
            };
            let achieved = loginitem::set_enabled(desired);
            {
                let mut st = self.ivars().borrow_mut();
                st.settings.login_item = achieved;
            }
            self.persist();
            self.rebuild_menu();
        }

        #[unsafe(method(quit:))]
        fn quit(&self, _sender: Option<&AnyObject>) {
            let mtm = self.mtm();
            NSApplication::sharedApplication(mtm).terminate(None);
        }

        // --- power timer target ---
        #[unsafe(method(pollPower:))]
        fn poll_power_timer(&self, _timer: Option<&AnyObject>) {
            self.poll_power();
        }

        // --- workspace notification: frontmost app changed ---
        #[unsafe(method(appActivated:))]
        fn app_activated(&self, _note: &NSNotification) {
            let bundle = exclusion::frontmost_bundle_id();
            let suppress = {
                let st = self.ivars().borrow();
                exclusion::should_suppress(bundle.as_deref(), &st.settings.exclusion_list)
            };
            {
                let mut st = self.ivars().borrow_mut();
                st.suppressed_by_app = suppress;
            }
            self.reconcile_visibility();
        }

        // --- workspace notification: active Space changed (three-finger swipe) ---
        #[unsafe(method(activeSpaceChanged:))]
        fn active_space_changed(&self, _note: &NSNotification) {
            // canJoinAllSpaces already shows the panels on every Space, but a
            // screensaver-level overlay can lag the switch animation and pop in
            // late. Re-assert the panels front the instant the Space changes so
            // the grain is present immediately on the incoming Space.
            self.reconcile_visibility();
        }
    }

    unsafe impl NSObjectProtocol for PapierDelegate {}

    unsafe impl NSApplicationDelegate for PapierDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn did_finish_launching(&self, _note: &NSNotification) {
            self.start();
        }

        #[unsafe(method(applicationDidChangeScreenParameters:))]
        fn did_change_screen_parameters(&self, _note: &NSNotification) {
            self.rebuild_panels();
        }
    }
);

impl PapierDelegate {
    fn new(mtm: MainThreadMarker, settings: Settings) -> Retained<Self> {
        let state = AppState {
            settings,
            panels: Vec::new(),
            status_item: None,
            snoozed: false,
            suppressed_by_app: false,
            paused_by_battery: false,
        };
        let this = mtm.alloc::<Self>().set_ivars(RefCell::new(state));
        unsafe { msg_send![super(this), init] }
    }

    /// One-time startup wiring (called from `applicationDidFinishLaunching:`).
    fn start(&self) {
        self.rebuild_panels();
        self.build_status_item();
        self.register_observers();
        self.start_power_timer();
        self.poll_power();
        self.reconcile_visibility();
    }

    fn persist(&self) {
        let st = self.ivars().borrow();
        let _ = st.settings.save(); // best-effort; failures are non-fatal.
    }

    /// Tear down and recreate the overlay panels (e.g. on screen change or
    /// texture switch).
    fn rebuild_panels(&self) {
        let mtm = self.mtm();
        let (texture, intensity, warmth) = {
            let st = self.ivars().borrow();
            (
                st.settings.texture.clone(),
                st.settings.intensity,
                st.settings.warmth,
            )
        };
        let panels = overlay::build_for_all_screens(mtm, &texture, intensity, warmth);
        {
            let mut st = self.ivars().borrow_mut();
            // Hide the old panels BEFORE dropping them. Dropping a still-ordered-in
            // NSPanel can leave an orphaned window on screen (or flicker), so order
            // them out first, then replace (drop releases our reference).
            overlay::set_visible(&st.panels, false);
            st.panels = panels;
        }
        self.reconcile_visibility();
    }

    /// Apply the current `should_show()` decision to the panels.
    fn reconcile_visibility(&self) {
        let st = self.ivars().borrow();
        overlay::set_visible(&st.panels, st.should_show());
    }

    fn build_status_item(&self) {
        let mtm = self.mtm();
        let target: &AnyObject = self.as_ref();
        let item = {
            let st = self.ivars().borrow();
            crate::menubar::build_status_item(mtm, target, &st.settings)
        };
        let mut st = self.ivars().borrow_mut();
        st.status_item = Some(item);
    }

    /// Rebuild the menu in place to reflect current settings (checkmarks etc.).
    fn rebuild_menu(&self) {
        let mtm = self.mtm();
        let target: &AnyObject = self.as_ref();
        let st = self.ivars().borrow();
        if let Some(item) = &st.status_item {
            let menu = crate::menubar::build_menu(mtm, target, &st.settings);
            item.setMenu(Some(&menu));
        }
    }

    fn register_observers(&self) {
        // Workspace notifications (app activation) live on the *workspace*
        // notification center, not the default one.
        let workspace = NSWorkspace::sharedWorkspace();
        let center = workspace.notificationCenter();
        unsafe {
            center.addObserver_selector_name_object(
                self,
                sel!(appActivated:),
                Some(NSWorkspaceDidActivateApplicationNotification),
                None,
            );
            // Re-assert the overlay the moment the active Space changes.
            center.addObserver_selector_name_object(
                self,
                sel!(activeSpaceChanged:),
                Some(NSWorkspaceActiveSpaceDidChangeNotification),
                None,
            );
        }
        let _ = NSNotificationCenter::defaultCenter(); // (screen-change handled via delegate method)
    }

    fn start_power_timer(&self) {
        unsafe {
            let _timer = NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                POWER_POLL_INTERVAL,
                self,
                sel!(pollPower:),
                None,
                true,
            );
        }
    }

    /// Poll battery state and update the pause flag if pause-on-battery is on.
    fn poll_power(&self) {
        let (enabled_pause, on_battery) = {
            let st = self.ivars().borrow();
            (st.settings.pause_on_battery, power::on_battery())
        };
        let paused = enabled_pause && on_battery;
        {
            let mut st = self.ivars().borrow_mut();
            st.paused_by_battery = paused;
        }
        self.reconcile_visibility();
    }
}

/// Entry point: configure the shared application as an accessory agent, install
/// the delegate, and run the main loop. Never returns under normal operation.
pub fn run() {
    let mtm = MainThreadMarker::new().expect("must run on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    // LSUIElement agent: no Dock icon, no menu bar app menu.
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let settings = Settings::load();
    let delegate = PapierDelegate::new(mtm, settings);
    let proto: &ProtocolObject<dyn NSApplicationDelegate> = ProtocolObject::from_ref(&*delegate);
    app.setDelegate(Some(proto));

    // Keep the delegate alive for the lifetime of the app.
    std::mem::forget(delegate);

    app.run();
}
