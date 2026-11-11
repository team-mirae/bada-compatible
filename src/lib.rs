//! shout, a bada-compatible v1 IME engine (docs/bada-compatible-v1.md), and the template for new
//! ones. Headless: event in → output out, no I/O.
//!
//! - [`compat`]: the contract layer. Don't edit it for your IME.
//! - [`shout`](crate::Shout): the IME. Replace it with yours.
//! - [`export!`] below: the C ABI, `include/shout.h`.
//! - [`rtl`]: a second example, a right-to-left IME (v1.2 `direction`), exported from the same
//!   library under its own prefix (`src/rtl_ime.rs`, `include/rtl.h`). A new IME deletes it.

pub mod compat;
mod rtl_ime;
mod shout;

pub use shout::{Form, Settings, Shout};

export! {
    prefix: "shout",
    ime: crate::Shout,
    data: [],
    optional: [set_settings, host_selection, catalog, state],
}

/// rtl: types English right to left. One library can export several IMEs, each with its own
/// prefix, by calling `export!` in a module of its own.
pub mod rtl {
    pub use crate::rtl_ime::{PDF, RLO, Rtl};

    crate::export! {
        prefix: "rtl",
        ime: Rtl,
        data: [],
        optional: [catalog, state],
    }
}
