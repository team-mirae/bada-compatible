//! `export!` with everything: data buffers, profiles, a process-wide dictionary. A small IME made
//! for this test (prefix `full`), so every branch of the macro is compiled and run. It also shows
//! how to use data: `P_data_new` gets the buffers the host loads (a manifest's `data`).

use std::ffi::{CStr, CString, c_char};
use std::sync::{Arc, RwLock};

use serde_json::{Value, json};
use shout::compat::{Ime, KeyEvent, Mode, NoSettings, Output};

/// Words to complete, from buffer 0; a greeting, from buffer 1.
#[derive(Default)]
struct Words {
    words: Vec<String>,
    greeting: String,
}

static DICTIONARY: RwLock<Vec<String>> = RwLock::new(Vec::new());

struct Full {
    data: Arc<Words>,
    profile: String,
    staged: String,
}

impl Full {
    /// Dictionary words, then data words, that start with what's staged.
    fn menu(&self) -> Vec<String> {
        if self.staged.is_empty() {
            return vec![];
        }
        let mut menu: Vec<String> = DICTIONARY.read().unwrap().clone();
        menu.extend(self.data.words.iter().cloned());
        menu.retain(|w| w.starts_with(&self.staged));
        menu
    }

    fn shown(&self) -> String {
        self.menu().into_iter().next().unwrap_or_else(|| self.staged.clone())
    }
}

impl Ime for Full {
    type Data = Words;
    type Settings = NoSettings;

    fn new(data: Arc<Words>) -> Self {
        Full { data, profile: "mac-kana".into(), staged: String::new() }
    }

    fn data(buffers: &[&[u8]]) -> Option<Words> {
        let text = |b: &[u8]| String::from_utf8(b.to_vec()).ok();
        let words = text(buffers[0])?.lines().map(str::to_string).collect();
        if buffers[0] == b"fail" {
            return None;
        }
        Some(Words { words, greeting: text(buffers[1])? })
    }

    fn key(&mut self, e: &KeyEvent) -> Output {
        match e.key().letter() {
            Some(l) => {
                self.staged.push(l);
                let menu = self.menu();
                let preedit = self.shown();
                Output { preedit, menu, consumed: true, ..Output::default() }
            }
            None => self.flush(),
        }
    }

    fn set_mode(&mut self, _mode: Mode) -> Output {
        self.flush()
    }

    fn flush(&mut self) -> Output {
        let shown = self.shown();
        self.staged.clear();
        Output::pass(shown)
    }

    fn set_profile(&mut self, name: &str) -> Output {
        self.profile = if ["mac-kana", "henkan", "katakana"].contains(&name) { name } else { "mac-kana" }.into();
        let mut out = self.flush();
        out.notes = vec![self.profile.clone(), self.data.greeting.clone()];
        out
    }

    fn dictionary_load(text: &str) -> Value {
        let words: Vec<String> = text.split_whitespace().map(str::to_string).collect();
        let n = words.len();
        *DICTIONARY.write().unwrap() = words;
        json!({"ok": true, "error": null, "positive": n, "negative": 0})
    }
}

shout::export! {
    prefix: "full",
    ime: Full,
    data: [(words, words_len), (greeting, greeting_len)],
    optional: [set_profile, dictionary_load],
}

fn out(p: *const c_char) -> Output {
    serde_json::from_str(unsafe { CStr::from_ptr(p) }.to_str().unwrap()).unwrap()
}

#[test]
fn everything() {
    assert_eq!(ffi::PREFIX, "full");
    let words = b"apple\napricot\nbanana";
    let hi = b"hello";
    let data = ffi::data_new(words.as_ptr(), words.len(), hi.as_ptr(), hi.len());
    assert!(!data.is_null());
    let failed = ffi::data_new(b"fail".as_ptr(), 4, std::ptr::null(), 0);
    assert!(failed.is_null(), "Ime::data returning None is NULL");

    let e = ffi::new(data);
    // Two engines on one Data: each holds its own reference, each has its own state.
    let e2 = ffi::new(data);
    let k = CString::new(r#"{"key":"KeyA"}"#).unwrap();
    let p = CString::new(r#"{"key":"KeyP"}"#).unwrap();
    out(ffi::key(e, k.as_ptr()));
    let o = out(ffi::key(e, p.as_ptr()));
    assert_eq!(o.menu, ["apple", "apricot"]);

    let name = CString::new("katakana").unwrap();
    let o = out(ffi::set_profile(e, name.as_ptr()));
    assert_eq!((o.commit.as_str(), o.notes.as_slice()), ("apple", &["katakana".to_string(), "hello".into()][..]));
    let bad = CString::new("nope").unwrap();
    assert_eq!(out(ffi::set_profile(e2, bad.as_ptr())).notes[0], "mac-kana");

    let dict = CString::new("apex").unwrap();
    let summary = ffi::dictionary_load(dict.as_ptr());
    let v: Value = serde_json::from_str(unsafe { CStr::from_ptr(summary) }.to_str().unwrap()).unwrap();
    ffi::string_free(summary);
    assert_eq!(v["positive"], 1);
    out(ffi::key(e2, k.as_ptr()));
    assert_eq!(out(ffi::key(e2, p.as_ptr())).menu, ["apex", "apple", "apricot"]);

    ffi::free(e);
    ffi::free(e2);
    ffi::data_free(data);
}
