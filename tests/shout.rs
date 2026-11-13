//! shout's behaviour, typed through the headless host (src/compat/host.rs). A new IME replaces this
//! file with its own.

use shout::compat::Mode;
use shout::compat::host::Host;
use shout::{Form, Settings, Shout};

fn host() -> Host<Shout> {
    Host::default()
}

#[test]
fn superkeyboards_shout_test() {
    // superkeyboard's tests/shout.rs, unchanged in meaning.
    let mut h = host();
    h.type_keys("h e l l o").unwrap();
    assert_eq!(h.preedit(), "HELLO");
    assert_eq!(h.menu()[1], "hello");
    h.type_keys("Space w o r l d Tab Enter").unwrap();
    assert_eq!(h.committed(), "HELLO world\n");
    h.type_keys("eisu a b c Backspace").unwrap();
    assert_eq!(h.mode(), Mode::Muhenkan);
    assert_eq!(h.committed(), "HELLO world\nab");
}

#[test]
fn the_menu_moves_without_wrapping() {
    let mut h = host();
    h.type_keys("h i S-Tab").unwrap();
    assert_eq!((h.entry(), h.preedit().as_str()), (0, "HI"));
    h.type_keys("Tab Tab").unwrap();
    assert_eq!((h.entry(), h.preedit().as_str()), (1, "hi"));
    h.type_keys("ArrowUp").unwrap();
    assert_eq!(h.preedit(), "HI");
    // かな in henkan with a menu is "next entry" (the host sends Lang1 to the engine).
    h.type_keys("kana").unwrap();
    assert_eq!(h.preedit(), "hi");
    // Typing on starts again at entry 0.
    h.type_keys("m").unwrap();
    assert_eq!((h.entry(), h.preedit().as_str()), (0, "HIM"));
}

#[test]
fn control_kana_fixes_what_shows() {
    // ⌃かな (v1.3) is sent to the engine in henkan; shout has nothing to fix, so nothing changes.
    let mut h = host();
    h.type_keys("h i Tab C-kana").unwrap();
    assert_eq!((h.entry(), h.preedit().as_str()), (1, "hi"));
    h.type_keys("C-henkan C-katakana").unwrap();
    assert_eq!((h.entry(), h.preedit().as_str()), (1, "hi"));
    // In muhenkan it does nothing, and doesn't enter henkan.
    h.type_keys("S-Enter eisu C-kana a").unwrap();
    assert_eq!(h.mode(), Mode::Muhenkan);
    assert_eq!(h.text(), "hia");
}

#[test]
fn commit_then_act() {
    let mut h = host();
    h.type_keys("a b Period c d Esc e f S-Enter").unwrap();
    // Period and Esc commit, then act (Period types ".", Esc nothing); ⇧Enter only commits.
    assert_eq!(h.text(), "AB.CDEF");
    h.type_keys("x C-a").unwrap();
    // A shortcut commits, and goes to the app (which the host doesn't model).
    assert_eq!(h.text(), "AB.CDEFX");
    assert_eq!(h.preedit(), "");
}

#[test]
fn backspace_edits_then_deletes() {
    let mut h = host();
    h.type_keys("o k Space n o BS BS BS").unwrap();
    assert_eq!(h.text(), "OK");
}

#[test]
fn settings_choose_the_forms() {
    let mut h = host();
    let s = h.set_settings(Settings { forms: vec![Form::Title, Form::Title, Form::Lower] });
    assert_eq!(s.forms, vec![Form::Title, Form::Lower]);
    h.type_keys("s e o u l").unwrap();
    assert_eq!(h.menu(), ["Seoul", "seoul"]);
    assert_eq!(h.set_settings(Settings { forms: vec![] }), Settings::default());
}

#[test]
fn zenkaku_shouts_the_selection_in_muhenkan() {
    let mut h = Host::<Shout>::new(Default::default(), Mode::Muhenkan);
    h.type_keys("q u i e t Space p l e a s e").unwrap();
    h.select(6, 12);
    h.type_keys("zenkaku").unwrap();
    assert_eq!(h.text(), "quiet PLEASE");
    // Nothing selected: nothing happens.
    h.type_keys("zenkaku").unwrap();
    assert_eq!(h.text(), "quiet PLEASE");
}

#[test]
fn mode_switches_commit() {
    let mut h = host();
    h.type_keys("h e y eisu").unwrap();
    assert_eq!((h.text().as_str(), h.preedit().as_str()), ("HEY", ""));
    h.type_keys("kana y o").unwrap();
    assert_eq!(h.mode(), Mode::Henkan);
    h.set_text("replaced");
    assert_eq!(h.text(), "replaced");
}
