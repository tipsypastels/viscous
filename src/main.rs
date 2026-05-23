use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, rc::Retained,
    runtime::ProtocolObject, sel,
};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSAutoresizingMaskOptions,
    NSBackingStoreType, NSColor, NSFont, NSMenu, NSMenuItem, NSTextAlignment, NSTextField,
    NSWindow, NSWindowDelegate, NSWindowStyleMask,
};
use objc2_foundation::{
    NSNotification, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, ns_string,
};
use std::cell::OnceCell;

fn main() {
    let mtm = MainThreadMarker::new().unwrap();
    let application = NSApplication::sharedApplication(mtm);

    let delegate = AppDelegate::new(mtm);
    let delegate = ProtocolObject::from_ref(&*delegate);

    application.setDelegate(Some(delegate));
    application.run();
}

struct AppDelegateIvars {
    window: OnceCell<Retained<NSWindow>>,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = AppDelegateIvars]
    struct AppDelegate;

    unsafe impl NSObjectProtocol for AppDelegate {}
    unsafe impl NSApplicationDelegate for AppDelegate {
            // SAFETY: The signature is correct.
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn did_finish_launching(&self, notification: &NSNotification) {
            let mtm = self.mtm();

            // Retrieve the NSApplication.
            let app = notification
                .object()
                .unwrap()
                .downcast::<NSApplication>()
                .unwrap();

            // Create a minimal menubar with a "Quit" entry.
            let menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), ns_string!(""));
            let menu_app_item = unsafe {
                NSMenuItem::initWithTitle_action_keyEquivalent(
                    NSMenuItem::alloc(mtm),
                    ns_string!(""),
                    None,
                    ns_string!(""),
                )
            };
            let menu_app_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), ns_string!(""));
            unsafe {
                menu_app_menu.addItemWithTitle_action_keyEquivalent(
                    ns_string!("Quit"),
                    Some(sel!(terminate:)),
                    ns_string!("q"),
                )
            };
            menu_app_item.setSubmenu(Some(&menu_app_menu));
            menu.addItem(&menu_app_item);
            app.setMainMenu(Some(&menu));

            // Create a window and display a text field inside that.
            let text_field = {
                let text_field = NSTextField::labelWithString(ns_string!("Hello, World!"), mtm);
                text_field.setFrame(NSRect::new(
                    NSPoint::new(5.0, 100.0),
                    NSSize::new(290.0, 100.0),
                ));
                text_field.setTextColor(Some(&NSColor::colorWithSRGBRed_green_blue_alpha(
                    0.0, 0.5, 0.0, 1.0,
                )));
                text_field.setAlignment(NSTextAlignment::Center);
                text_field.setFont(Some(&NSFont::systemFontOfSize(45.0)));
                text_field.setAutoresizingMask(
                    NSAutoresizingMaskOptions::ViewWidthSizable
                        | NSAutoresizingMaskOptions::ViewHeightSizable,
                );
                text_field
            };

            // SAFETY: We disable releasing when closed below.
            let window = unsafe {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    NSWindow::alloc(mtm),
                    NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(300.0, 300.0)),
                    NSWindowStyleMask::Titled
                        | NSWindowStyleMask::Closable
                        | NSWindowStyleMask::Miniaturizable
                        | NSWindowStyleMask::Resizable,
                    NSBackingStoreType::Buffered,
                    false,
                )
            };
            // SAFETY: Disable auto-release when closing windows.
            // This is required when creating `NSWindow` outside a window
            // controller.
            unsafe { window.setReleasedWhenClosed(false) };

            // Set various window properties.
            window.setTitle(ns_string!("A window"));
            let view = window.contentView().expect("window must have content view");
            view.addSubview(&text_field);
            window.center();
            window.setContentMinSize(NSSize::new(300.0, 300.0));
            window.setDelegate(Some(ProtocolObject::from_ref(self)));

            // Show the window.
            window.makeKeyAndOrderFront(None);

            // Store the window in the delegate.
            self.ivars().window.set(window).unwrap();

            app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

            // Activate the application.
            // Required when launching unbundled (as is done with Cargo).
            #[allow(deprecated)]
            app.activateIgnoringOtherApps(true);
        }
    }

    unsafe impl NSWindowDelegate for AppDelegate {
        #[unsafe(method(windowWillClose:))]
        fn window_will_close(&self, _notification: &NSNotification) {
            // Quit the application when the window is closed.
            let app = NSApplication::sharedApplication(self.mtm());
            app.terminate(None);
        }
    }
}

impl AppDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(AppDelegateIvars {
            window: OnceCell::new(),
        });
        unsafe { msg_send![super(this), init] }
    }
}
