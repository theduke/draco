
// Depend on draco_macros and re-export everything in it.
// This allows users to just depend on draco and get access
// to the proc macros.
#[allow(unused_imports)]
#[macro_use]
extern crate draco_macros;
#[doc(hidden)]
pub use draco_macros::*;

#[macro_use]
pub mod console;
pub mod app;
pub mod element;
pub mod fetch;
pub mod html;
pub mod mailbox;
pub mod node;
pub mod router;
pub mod subscription;
pub mod svg;
pub mod text;

pub use self::app::{start, App, Instance};
pub use self::element::{h, s};
pub use self::element::{Element, KeyedElement, NonKeyedElement};
pub use self::mailbox::Mailbox;
pub use self::node::Node;
pub use self::subscription::{Subscription, Unsubscribe};
pub use self::text::Text;
use std::borrow::Cow;

pub type S = Cow<'static, str>;

pub fn select(selector: &str) -> Option<web_sys::Element> {
    web_sys::window()?
        .document()?
        .query_selector(selector)
        .ok()?
}

pub fn set_panic_hook() {
    use std::sync::Once;

    static PANIC_HOOK: Once = Once::new();

    PANIC_HOOK.call_once(|| {
        std::panic::set_hook(Box::new(|panic| {
            crate::console::error(&panic.to_string());
        }));
    });
}
