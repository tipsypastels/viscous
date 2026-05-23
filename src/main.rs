use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, rc::Retained,
    runtime::ProtocolObject, sel,
};
use objc2_app_kit::{
    NSApplication, NSApplicationDelegate, NSEventModifierFlags, NSMenu, NSMenuItem, NSStatusBar,
    NSStatusItem,
};
use objc2_foundation::{NSNotification, NSObject, NSObjectProtocol, ns_string};
use std::cell::OnceCell;

fn main() {
    let mtm = MainThreadMarker::new().unwrap();
    let application = NSApplication::sharedApplication(mtm);

    let delegate = AppDelegate::new(mtm);
    let delegate = ProtocolObject::from_ref(&*delegate);

    application.setDelegate(Some(delegate));
    application.run();
}

#[derive(Debug, Default)]
struct AppDelegateIvars {
    status_item: OnceCell<Retained<NSStatusItem>>,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = AppDelegateIvars]
    struct AppDelegate;

    unsafe impl NSObjectProtocol for AppDelegate {}
    unsafe impl NSApplicationDelegate for AppDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn did_finish_launching(&self, notification: &NSNotification) {
            let _ = notification;

            let mtm = self.mtm();
            let status_item = NSStatusBar::systemStatusBar().statusItemWithLength(-1.0);

            let quit_item = unsafe {
                NSMenuItem::initWithTitle_action_keyEquivalent(
                    NSMenuItem::alloc(mtm),
                    ns_string!("Quit Viscous"),
                    Some(sel!(terminate:)),
                    ns_string!("q"),
                )
            };
            quit_item.setKeyEquivalentModifierMask(NSEventModifierFlags::Command);

            let menu = NSMenu::new(mtm);
            menu.setTitle(ns_string!("Viscous"));
            menu.addItem(&quit_item);

            status_item.setMenu(Some(&menu));

            let status_button = status_item.button(mtm).unwrap();
            status_button.setTitle(ns_string!("🌯"));

            self.ivars().status_item.set(status_item).unwrap();
        }
    }
}

impl AppDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(AppDelegateIvars::default());
        unsafe { msg_send![super(this), init] }
    }
}
