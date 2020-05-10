//! macOS-specific code

// TODO: localize

use cocoa::appkit::{NSApp, NSApplication, NSEventModifierFlags, NSMenu, NSMenuItem};
use cocoa::base::{nil};
use cocoa::foundation::{NSAutoreleasePool, NSString, NSUInteger};

use objc::*;

#[allow(non_upper_case_globals)]
const NSEventModifierFlagOption: NSUInteger = 1 << 19;

#[allow(non_upper_case_globals)]
const NSEventModifierFlagCommand: NSUInteger = 1 << 20;

// create macOS menu-bar
pub(crate) unsafe fn create_menu_bar() {
    // assume that an NSAutoreleasePool is generated by winit-rs.
    let app = NSApp();

    let menubar = NSMenu::new(nil).autorelease();
    
    let app_menu_item = NSMenuItem::new(nil).autorelease();
    let file_menu_item = NSMenuItem::new(nil).autorelease();
    let window_menu_item = NSMenuItem::new(nil).autorelease();

    menubar.addItem_(app_menu_item);
    menubar.addItem_(file_menu_item);
    menubar.addItem_(window_menu_item);

    app.setMainMenu_(menubar);

    // ReizeiinTouka
    //  - About ReizeiinTouka...
    //  -
    //  - Hide ReizeiinTouka
    //  - Hide others
    //  - Show All
    //  - --
    //  - Quit ReizeiinTouka
    let app_menu = NSMenu::new(nil).autorelease();

    app_menu.addItem_(
        NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(
                NSString::alloc(nil).init_str("冷泉院桐香を非表示\0"),
                sel![hide:],
                NSString::alloc(nil).init_str("h\0"),
            )
            .autorelease(),
    );

    let hide_others = NSMenuItem::alloc(nil)
        .initWithTitle_action_keyEquivalent_(
            NSString::alloc(nil).init_str("ほかを非表示\0"),
            sel![hideOtherApplications:],
            NSString::alloc(nil).init_str("h\0"),
        )
        .autorelease();

    hide_others.setKeyEquivalentModifierMask_(
        NSEventModifierFlags::from_bits(NSEventModifierFlagOption | NSEventModifierFlagCommand).unwrap(),
    );

    app_menu.addItem_(hide_others);

    app_menu.addItem_(NSMenuItem::alloc(nil)
    .initWithTitle_action_keyEquivalent_(
        NSString::alloc(nil).init_str("すべてを表示\0"),
        sel![unhideAllApplications:],
        NSString::alloc(nil).init_str("\0"),
    )
    .autorelease());

    app_menu.addItem_(NSMenuItem::separatorItem(nil).autorelease());

    let services = NSMenuItem::alloc(nil)
        .initWithTitle_action_keyEquivalent_(
            NSString::alloc(nil).init_str("サービス\0"),
            std::mem::zeroed(), // null-selector
            NSString::alloc(nil).init_str("\0"),
        )
        .autorelease();

    let services_menu = NSMenu::new(nil).autorelease();
    let _: () = msg_send![app, setServicesMenu: services_menu];
    services.setSubmenu_(services_menu);
    
    app_menu.addItem_(services);

    app_menu.addItem_(NSMenuItem::separatorItem(nil).autorelease());

    // NOTE: the actual "Quit" should be done by winit.
    //       currently, this performed by closing window.
    let quit_item = NSMenuItem::alloc(nil)
        .initWithTitle_action_keyEquivalent_(
            NSString::alloc(nil).init_str("冷泉院桐香を終了\0"),
            sel![performClose:],
            NSString::alloc(nil).init_str("q\0"),
        )
        .autorelease();
    app_menu.addItem_(quit_item);

    app_menu_item.setSubmenu_(app_menu);

    // File
    //  - Close
    let file_menu = NSMenu::new(nil).autorelease();
    let _: () = msg_send![file_menu, setTitle: (NSString::alloc(nil).init_str("ファイル\0"))];

    file_menu.addItem_(
        NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(
                NSString::alloc(nil).init_str("閉じる\0"),
                sel![performClose:],
                NSString::alloc(nil).init_str("w\0"),
            )
            .autorelease(),
    );

    file_menu_item.setSubmenu_(file_menu);

    // File
    //  - Close
    let window_menu = NSMenu::new(nil).autorelease();
    let _: () = msg_send![window_menu, setTitle: (NSString::alloc(nil).init_str("ウインドウ\0"))];
    let _: () = msg_send![app, setWindowsMenu: window_menu];

    window_menu_item.setSubmenu_(window_menu);
}
