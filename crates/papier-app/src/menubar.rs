//! Menu-bar UI: an `NSStatusItem` with a menu wiring the user controls.
//!
//! Menu structure (spec §6):
//! - On/Off toggle (checkmark reflects `enabled`)
//! - Intensity slider (an `NSSlider` hosted in a menu item's custom view,
//!   clamped to 15–30%)
//! - Texture picker (one checked item per catalog entry)
//! - Snooze submenu (15 / 30 / 60 minutes)
//! - Quit
//!
//! All actions target the `PapierDelegate` (passed in as `target`). Parameterized
//! actions carry their argument in the menu item's `tag` (texture index, snooze
//! minutes).

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Sel};
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{
    NSControlStateValueOff, NSControlStateValueOn, NSMenu, NSMenuItem, NSSlider, NSStatusBar,
    NSStatusItem, NSVariableStatusItemLength,
};
use objc2_foundation::{NSRect, NSString};

use crate::grain::CATALOG;
use crate::settings::{Settings, MAX_INTENSITY, MIN_INTENSITY};

/// Tag used on the intensity slider so the action handler can find it.
pub const SLIDER_TAG: isize = 9001;

/// Build the status item + menu. The returned `NSStatusItem` must be kept alive
/// for the menu to remain in the menu bar.
pub fn build_status_item(
    mtm: MainThreadMarker,
    target: &AnyObject,
    settings: &Settings,
) -> Retained<NSStatusItem> {
    let bar = NSStatusBar::systemStatusBar();
    let item = bar.statusItemWithLength(NSVariableStatusItemLength);

    if let Some(button) = item.button(mtm) {
        button.setTitle(&NSString::from_str("P"));
    }

    let menu = build_menu(mtm, target, settings);
    item.setMenu(Some(&menu));
    item
}

fn add_action_item(
    mtm: MainThreadMarker,
    menu: &NSMenu,
    title: &str,
    action: Sel,
    target: &AnyObject,
    state_on: bool,
    tag: isize,
) -> Retained<NSMenuItem> {
    unsafe {
        let item = NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc::<NSMenuItem>(),
            &NSString::from_str(title),
            Some(action),
            &NSString::from_str(""),
        );
        item.setTarget(Some(target));
        item.setTag(tag);
        item.setState(if state_on {
            NSControlStateValueOn
        } else {
            NSControlStateValueOff
        });
        menu.addItem(&item);
        item
    }
}

/// Build the full menu tree. Selectors must match methods defined on
/// `PapierDelegate` (see `app.rs`).
pub fn build_menu(
    mtm: MainThreadMarker,
    target: &AnyObject,
    settings: &Settings,
) -> Retained<NSMenu> {
    let menu = NSMenu::new(mtm);

    // On/Off toggle.
    add_action_item(
        mtm,
        &menu,
        "Enabled",
        sel!(toggleEnabled:),
        target,
        settings.enabled,
        0,
    );

    menu.addItem(&NSMenuItem::separatorItem(mtm));

    // Intensity slider, hosted in a custom-view menu item.
    unsafe {
        let label = NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc::<NSMenuItem>(),
            &NSString::from_str("Intensity"),
            None,
            &NSString::from_str(""),
        );
        label.setEnabled(false);
        menu.addItem(&label);

        let frame = NSRect::new(
            objc2_foundation::NSPoint::new(0.0, 0.0),
            objc2_foundation::NSSize::new(180.0, 24.0),
        );
        let slider = NSSlider::initWithFrame(mtm.alloc::<NSSlider>(), frame);
        slider.setMinValue(MIN_INTENSITY as f64);
        slider.setMaxValue(MAX_INTENSITY as f64);
        slider.setDoubleValue(settings.intensity as f64);
        slider.setTag(SLIDER_TAG);
        slider.setTarget(Some(target));
        slider.setAction(Some(sel!(intensityChanged:)));

        let slider_item = NSMenuItem::new(mtm);
        slider_item.setView(Some(&slider));
        menu.addItem(&slider_item);
    }

    menu.addItem(&NSMenuItem::separatorItem(mtm));

    // Texture picker — one checked item per catalog entry; tag == catalog index.
    for (idx, entry) in CATALOG.iter().enumerate() {
        add_action_item(
            mtm,
            &menu,
            entry.display,
            sel!(pickTexture:),
            target,
            entry.key == settings.texture,
            idx as isize,
        );
    }

    menu.addItem(&NSMenuItem::separatorItem(mtm));

    // Snooze submenu (minutes carried in the tag).
    unsafe {
        let snooze_root = NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc::<NSMenuItem>(),
            &NSString::from_str("Snooze"),
            None,
            &NSString::from_str(""),
        );
        let snooze_menu = NSMenu::new(mtm);
        for minutes in [15isize, 30, 60] {
            add_action_item(
                mtm,
                &snooze_menu,
                &format!("{minutes} minutes"),
                sel!(snooze:),
                target,
                false,
                minutes,
            );
        }
        snooze_root.setSubmenu(Some(&snooze_menu));
        menu.addItem(&snooze_root);
    }

    // Pause-on-battery toggle.
    add_action_item(
        mtm,
        &menu,
        "Pause on Battery",
        sel!(togglePauseOnBattery:),
        target,
        settings.pause_on_battery,
        0,
    );

    // Launch-at-login toggle.
    add_action_item(
        mtm,
        &menu,
        "Launch at Login",
        sel!(toggleLoginItem:),
        target,
        settings.login_item,
        0,
    );

    menu.addItem(&NSMenuItem::separatorItem(mtm));

    // Quit.
    add_action_item(mtm, &menu, "Quit Papier", sel!(quit:), target, false, 0);

    menu
}
