//! rtl: the example of a right-to-left IME (spec §6 `direction`, v1.2). It types English, right to
//! left.
//!
//! Henkan mode composes letters in the preedit, with a menu of two rows:
//!
//! - **RTL** (shown first): the letters as typed, with `direction: "rtl"`. A host that honours
//!   `direction` draws them right to left.
//! - **LTR**: the letters wrapped in RIGHT-TO-LEFT OVERRIDE … POP DIRECTIONAL FORMATTING
//!   (U+202E … U+202C), with `direction: "ltr"`, so any bidi-aware field draws them right to left,
//!   even one that knows nothing of `direction`.
//!
//! Every Output's `direction` is the selected row's. Keys are shout's: Tab / ↓ / かな down, ⇧ and ↑
//! up, no wrap; Space, Enter, Esc and any other key commit what shows, then act; ⇧Enter commits and
//! does nothing else; Backspace edits. Muhenkan consumes nothing.
//!
//! The text is always in logical order (spec §4): the letters are never reversed.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::compat::{Direction, Ime, Key, KeyEvent, Mode, NoSettings, Output, Report};

/// RIGHT-TO-LEFT OVERRIDE.
pub const RLO: char = '\u{202E}';
/// POP DIRECTIONAL FORMATTING: ends the override.
pub const PDF: char = '\u{202C}';

/// The menu's rows, top first: how each writes the letters, and the direction it reports.
const ROWS: [(&str, Direction); 2] = [("RTL", Direction::Rtl), ("LTR", Direction::Ltr)];

pub struct Rtl {
    mode: Mode,
    /// The staged letters.
    letters: String,
    /// The menu row showing.
    row: usize,
}

impl Rtl {
    fn write(&self, row: usize) -> String {
        match ROWS[row].1 {
            Direction::Rtl => self.letters.clone(),
            Direction::Ltr => format!("{RLO}{}{PDF}", self.letters),
        }
    }

    /// An Output for the current state, after `commit`. With nothing staged, the direction of the
    /// row last used, so a commit is drawn the way it showed.
    fn out(&self, commit: String, consumed: bool, direction: Direction) -> Output {
        if self.letters.is_empty() {
            return Output { commit, consumed, direction, ..Output::default() };
        }
        Output {
            commit,
            preedit: self.write(self.row),
            menu: (0..ROWS.len()).map(|r| self.write(r)).collect(),
            notes: ROWS.iter().map(|(note, _)| note.to_string()).collect(),
            selected: self.row,
            consumed,
            direction: ROWS[self.row].1,
            ..Output::default()
        }
    }

    /// Commits what shows: the text, and the direction it showed in.
    fn take(&mut self) -> (String, Direction) {
        let taken = (self.staged(), ROWS[self.row].1);
        self.letters.clear();
        self.row = 0;
        taken
    }

    fn commit(&mut self, consumed: bool) -> Output {
        let (commit, direction) = self.take();
        let direction = if commit.is_empty() { ROWS[0].1 } else { direction };
        self.out(commit, consumed, direction)
    }

    fn moved(&mut self, row: usize) -> Output {
        self.row = row;
        self.out(String::new(), true, ROWS[0].1)
    }
}

impl Report for Rtl {
    /// What shows: the selected row's text.
    fn staged(&self) -> String {
        if self.letters.is_empty() { String::new() } else { self.write(self.row) }
    }
}

impl Ime for Rtl {
    type Data = ();
    type Settings = NoSettings;

    fn new(_data: Arc<()>) -> Self {
        Rtl { mode: Mode::Henkan, letters: String::new(), row: 0 }
    }

    fn key(&mut self, e: &KeyEvent) -> Output {
        let key = e.key();
        let m = e.mods;
        if self.mode == Mode::Muhenkan {
            return Output::default();
        }
        let staged = !self.letters.is_empty();
        let last = ROWS.len() - 1;
        match key {
            k if k.letter().is_some() && m.is_plain() => {
                self.letters.extend(k.letter());
                self.moved(0)
            }
            Key::Backspace if staged && m.is_plain() => {
                self.letters.pop();
                self.moved(0)
            }
            Key::Tab | Key::Lang1 if staged && !m.shift => self.moved((self.row + 1).min(last)),
            Key::ArrowDown if staged => self.moved((self.row + 1).min(last)),
            Key::Tab | Key::Lang1 | Key::ArrowUp if staged => self.moved(self.row.saturating_sub(1)),
            Key::Enter if staged && m.shift => self.commit(true),
            Key::Lang1 => Output { consumed: true, direction: ROWS[0].1, ..Output::default() },
            _ => self.commit(false),
        }
    }

    fn set_mode(&mut self, mode: Mode) -> Output {
        let out = self.commit(false);
        self.mode = mode;
        out
    }

    fn flush(&mut self) -> Output {
        self.commit(false)
    }

    fn catalog(&self) -> Value {
        let rows: Vec<_> = ROWS.iter().map(|(note, d)| json!({"note": note, "direction": d})).collect();
        json!({"name": "rtl", "rows": rows})
    }
}
