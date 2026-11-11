//! shout: the example IME. **This is the file a new IME replaces.**
//!
//! Henkan mode composes letters as CAPITALS in the preedit, with a menu of the letters' forms
//! (CAPITALS, lowercase by default; Title case on request). Tab / ↓ / かな move down the menu, ⇧ and
//! ↑ up, without wrapping. Space, Enter, Esc and any other key commit what shows, then act; ⇧Enter
//! commits and does nothing else; Backspace edits. In muhenkan nothing is consumed, except 半角/全角,
//! which shouts the selection (the host's selection, through `need_selection`).
//!
//! It uses no data, and of the optional functions: settings, the catalog and the host's selection.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::compat::{Ime, Key, KeyEvent, Mode, Output};

/// One way to write the letters: one menu row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Form {
    Upper,
    Lower,
    Title,
}

impl Form {
    pub const ALL: [Form; 3] = [Form::Upper, Form::Lower, Form::Title];

    fn write(self, letters: &str) -> String {
        match self {
            Form::Upper => letters.to_uppercase(),
            Form::Lower => letters.to_lowercase(),
            Form::Title => {
                let mut c = letters.chars();
                c.next().map(|f| f.to_uppercase().chain(c.flat_map(char::to_lowercase)).collect()).unwrap_or_default()
            }
        }
    }

    fn label(self) -> &'static str {
        match self {
            Form::Upper => "CAPITALS",
            Form::Lower => "lowercase",
            Form::Title => "Title case",
        }
    }
}

/// `{"forms": ["upper", "lower"]}`: the menu's rows, top first. The first shows while typing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub forms: Vec<Form>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { forms: vec![Form::Upper, Form::Lower] }
    }
}

pub struct Shout {
    mode: Mode,
    settings: Settings,
    /// The staged letters, lowercase.
    letters: String,
    /// The menu row showing.
    row: usize,
    /// Waiting for the host's selection (半角/全角 in muhenkan).
    waiting: bool,
}

impl Shout {
    fn shown(&self) -> String {
        self.settings.forms[self.row].write(&self.letters)
    }

    /// An Output for the current state, after `commit`.
    fn out(&self, commit: String, consumed: bool) -> Output {
        if self.letters.is_empty() {
            return Output { commit, consumed, ..Output::default() };
        }
        Output {
            commit,
            preedit: self.shown(),
            menu: self.settings.forms.iter().map(|f| f.write(&self.letters)).collect(),
            selected: self.row,
            consumed,
            ..Output::default()
        }
    }

    /// What shows, unstaged.
    fn take(&mut self) -> String {
        let shown = if self.letters.is_empty() { String::new() } else { self.shown() };
        self.letters.clear();
        self.row = 0;
        shown
    }
}

impl Ime for Shout {
    type Data = ();
    type Settings = Settings;

    fn new(_data: Arc<()>) -> Self {
        Shout { mode: Mode::Henkan, settings: Settings::default(), letters: String::new(), row: 0, waiting: false }
    }

    fn key(&mut self, e: &KeyEvent) -> Output {
        let key = e.key();
        let m = e.mods;
        if self.mode == Mode::Muhenkan {
            if key == Key::Backquote && m.is_plain() {
                self.waiting = true;
                return Output { consumed: true, need_selection: Some("shout".into()), ..Output::default() };
            }
            return Output::default();
        }
        let staged = !self.letters.is_empty();
        let last = self.settings.forms.len() - 1;
        match key {
            k if k.letter().is_some() && m.is_plain() => {
                self.letters.extend(k.letter());
                self.row = 0;
                self.out(String::new(), true)
            }
            Key::Backspace if staged && m.is_plain() => {
                self.letters.pop();
                self.row = 0;
                self.out(String::new(), true)
            }
            Key::Tab | Key::Lang1 if staged && !m.shift => {
                self.row = (self.row + 1).min(last);
                self.out(String::new(), true)
            }
            Key::ArrowDown if staged => {
                self.row = (self.row + 1).min(last);
                self.out(String::new(), true)
            }
            Key::Tab | Key::Lang1 | Key::ArrowUp if staged => {
                self.row = self.row.saturating_sub(1);
                self.out(String::new(), true)
            }
            Key::Enter if staged && m.shift => {
                let commit = self.take();
                self.out(commit, true)
            }
            Key::Lang1 => Output::consumed(),
            _ => {
                let commit = self.take();
                self.out(commit, false)
            }
        }
    }

    fn set_mode(&mut self, mode: Mode) -> Output {
        let commit = self.take();
        self.mode = mode;
        self.waiting = false;
        self.out(commit, false)
    }

    fn flush(&mut self) -> Output {
        let commit = self.take();
        self.out(commit, false)
    }

    fn set_settings(&mut self, mut settings: Settings) {
        let mut seen = Vec::new();
        settings.forms.retain(|f| {
            !seen.contains(f) && {
                seen.push(*f);
                true
            }
        });
        if settings.forms.is_empty() {
            settings = Settings::default();
        }
        self.settings = settings;
        self.row = self.row.min(self.settings.forms.len() - 1);
    }

    fn settings(&self) -> Settings {
        self.settings.clone()
    }

    fn host_selection(&mut self, text: &str) -> Output {
        let waiting = std::mem::take(&mut self.waiting);
        if !waiting || text.is_empty() {
            return Output::consumed();
        }
        Output { commit: text.to_uppercase(), replace_selection: true, consumed: true, ..Output::default() }
    }

    fn catalog(&self) -> Value {
        let forms: Vec<_> = Form::ALL.iter().map(|f| json!({"id": f, "label": f.label()})).collect();
        json!({"name": "shout", "forms": forms})
    }
}
