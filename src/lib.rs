//! shout, a bada-compatible v1 IME engine (docs/bada-compatible-v1.md), and the template for new
//! ones. Headless: event in → output out, no I/O.
//!
//! - [`compat`]: the contract layer. Don't edit it for your IME.
//! - [`shout`](crate::Shout): the IME. Replace it with yours.
//! - [`export!`] below: the C ABI, `include/shout.h`.

pub mod compat;
mod shout;

pub use shout::{Form, Settings, Shout};

export! {
    prefix: "shout",
    ime: crate::Shout,
    data: [],
    optional: [set_settings, host_selection, catalog],
}
