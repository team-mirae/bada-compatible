//! rtl's behaviour (src/rtl_ime.rs): typed through the headless host, and through its C ABI
//! (prefix `rtl`, in the same library as shout). A new IME deletes this file.

use std::ffi::{CStr, CString, c_char};

use serde_json::Value;
use shout::compat::host::Host;
use shout::compat::{Direction, Mode, Output, Report};
use shout::rtl::{PDF, RLO, Rtl, ffi as abi};

fn host() -> Host<Rtl> {
    Host::default()
}

fn wrapped(text: &str) -> String {
    format!("{RLO}{text}{PDF}")
}

#[test]
fn types_english_right_to_left() {
    let mut h = host();
    h.type_keys("h e l l o").unwrap();
    assert_eq!(h.preedit(), "hello", "logical order: the letters are never reversed");
    assert_eq!(h.direction(), Direction::Rtl);
    assert_eq!(h.menu(), [String::from("hello"), wrapped("hello")]);
    assert_eq!(h.last().unwrap().notes, ["RTL", "LTR"]);
    h.type_keys("Space").unwrap();
    assert_eq!(h.committed(), "hello ");
    assert_eq!(h.direction(), Direction::Rtl, "the commit is drawn as it showed");
}

#[test]
fn the_ltr_row_wraps_in_an_override() {
    let mut h = host();
    h.type_keys("h i Tab").unwrap();
    assert_eq!((h.preedit(), h.direction()), (wrapped("hi"), Direction::Ltr));
    assert_eq!(h.ime.staged(), wrapped("hi"));
    h.type_keys("Enter").unwrap();
    assert_eq!(h.committed(), format!("{}\n", wrapped("hi")));
    assert_eq!(h.direction(), Direction::Ltr);
}

#[test]
fn the_menu_moves_without_wrapping() {
    let mut h = host();
    h.type_keys("a ArrowUp").unwrap();
    assert_eq!((h.entry(), h.direction()), (0, Direction::Rtl));
    h.type_keys("Tab Tab").unwrap();
    assert_eq!((h.entry(), h.direction()), (1, Direction::Ltr));
    h.type_keys("S-Tab").unwrap();
    assert_eq!(h.entry(), 0);
    // Typing on starts again at the RTL row.
    h.type_keys("Tab b").unwrap();
    assert_eq!((h.entry(), h.preedit().as_str()), (0, "ab"));
}

#[test]
fn backspace_and_shift_enter() {
    let mut h = host();
    h.type_keys("a b BS").unwrap();
    assert_eq!(h.preedit(), "a");
    h.type_keys("S-Enter").unwrap();
    assert_eq!(h.committed(), "a", "⇧Enter commits and does nothing else");
    h.type_keys("BS").unwrap();
    assert_eq!(h.committed(), "", "nothing staged: Backspace goes to the app");
}

#[test]
fn muhenkan_consumes_nothing() {
    let mut h = Host::<Rtl>::new(Default::default(), Mode::Muhenkan);
    h.type_keys("h i").unwrap();
    assert_eq!((h.committed().as_str(), h.direction()), ("hi", Direction::Ltr));
}

fn s(text: &str) -> CString {
    CString::new(text).unwrap()
}

fn json(ptr: *const c_char) -> Value {
    serde_json::from_str(unsafe { CStr::from_ptr(ptr) }.to_str().unwrap()).unwrap()
}

#[test]
fn through_the_c_abi() {
    assert_eq!(abi::PREFIX, "rtl");
    let e = abi::new(std::ptr::null());
    json(abi::set_mode(e, 1));
    let out: Output = serde_json::from_value(json(abi::key(e, s(r#"{"key":"KeyA"}"#).as_ptr()))).unwrap();
    assert_eq!((out.preedit.as_str(), out.direction), ("a", Direction::Rtl));
    assert_eq!(json(abi::key(e, s(r#"{"key":"KeyA"}"#).as_ptr()))["direction"], "rtl", "written on the wire");
    let state = json(abi::state(e));
    assert_eq!((&state["direction"], &state["staged"]), (&Value::from("rtl"), &Value::from("aa")));
    json(abi::key(e, s(r#"{"key":"Tab"}"#).as_ptr()));
    let state = json(abi::state(e));
    assert!(state.get("direction").is_none(), "LTR isn't written");
    assert_eq!(json(abi::flush(e))["commit"], state["staged"], "staged is what P_flush commits");
    assert!(json(abi::catalog(e))["rows"].is_array());
    abi::free(e);
}
