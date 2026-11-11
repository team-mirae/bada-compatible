# bada-compatible

**bada-compatible** is the standard interface between an IME engine and the program that runs it,
taken from [bada ime](https://github.com/team-mirae/bada-ime)'s engine. This repository holds the
standard and a Rust template for writing engines that follow it.

An engine is headless: key events go in as JSON, and what to type comes out as JSON. It does no I/O
and has no GUI. The program around it, the **wrapper** or host, owns the text field, the windows,
the language keys and Caps Lock. Any wrapper that runs bada's engine can run yours.

## Examples

| Repository | Role |
|---|---|
| [team-mirae/bada-ime](https://github.com/team-mirae/bada-ime) | **IME**: bada ime's engine (`engine/`, prefix `bada`), the reference the standard is taken from. It also ships wrappers: the macOS shell, which links `libbada.a`, and a web simulator |
| [team-mirae/pado-ime](https://github.com/team-mirae/pado-ime) | **IME**: pado ime, the Windows fork of bada ime |
| [team-mirae/bada-enigma](https://github.com/team-mirae/bada-enigma) | **IME**: a joke engine whose henkan mode types through an Enigma machine, also compiled to WebAssembly for a [web page](https://enigma.sjun.me/) |
| [team-mirae/bada-windows](https://github.com/team-mirae/bada-windows) | **Wrapper**: a Windows text service that runs bada-compatible engines |
| [team-mirae/bada-linux](https://github.com/team-mirae/bada-linux) | **Wrapper**: a Fcitx5 addon and an IBus engine that run bada-compatible engines on Linux |
| [team-mirae/superkeyboard](https://github.com/team-mirae/superkeyboard) | **Wrapper**: a headless typing robot that loads engines from a manifest and drives them over REST and MCP |
| this repository's `shout` | **IME**: the template's example engine ([below](#the-template)) |

## The standard

[docs/bada-compatible-v1.md](docs/bada-compatible-v1.md) is the contract. If the code and that
document disagree, one of them is fixed in the same change.

| Version | What it is |
|---|---|
| **v1** | The C ABI of bada ime 0.2.0's engine: five required functions, plus optional ones a wrapper looks up with `dlsym` |
| **v1.1** | v1, plus the optional, read-only `P_state` (§12) |

What every engine and wrapper should know, in short:

- **Functions** are named `<prefix>_<name>`: `shout_key`, `bada_key`. Required: `P_new`, `P_free`,
  `P_key`, `P_set_mode`, `P_flush`. Optional: `P_data_new` / `P_data_free`, `P_set_settings`,
  `P_set_profile`, `P_host_selection`, `P_catalog`, `P_dictionary_load` / `P_string_free`, and
  `P_state` (§3).
- **Key events** are W3C `KeyboardEvent.code`s on a JIS keyboard, such as
  `{"key": "KeyA", "shift": true}` (§5).
- **Outputs** say what to commit, what to mark as the preedit, the candidate menu, and whether the
  key was consumed (§6). The wrapper applies them all the same way (§7.1).
- **Two modes**: `0` muhenkan (keys type directly or pass to the app) and `1` henkan (keys compose,
  then commit). The wrapper switches them with its language keys (§7.4).
- **Caps Lock belongs to the wrapper.** The engine never sees it: events carry no Caps Lock state,
  and the wrapper types passed-on keys in capitals (§7.7).
- **State** (v1.1): `P_state` tells the wrapper the mode, the profile, the menu, and what is
  **staged**, meaning the exact text `P_flush` would commit now. Asking changes nothing (§12).

### Exposing state to scripts

Wrappers choose whether and how to expose `P_state` to other programs. Exposing it is recommended
(§12.1), so that tools such as Hammerspoon, Karabiner-Elements and AutoHotkey can follow the IME.
A wrapper that does:

- publishes the object as is, with its own fields under `host`;
- is read-only from outside;
- blanks typed text in password fields;
- uses a channel the platform's scripting tools can read without a plugin, such as a JSON file plus
  a notification on macOS, or a file plus a window message or named pipe on Windows.

```json
{"format": "bada-compatible-state/1", "mode": "henkan", "profile": "mac-kana", "staged": "한국어",
 "preedit": "한국어", "menu": ["한국어", "韓國語"], "selected": 0, "zero": 0, "horizontal": false,
 "need_selection": null}
```

## The template

The example IME is **shout**. In henkan mode it composes letters as CAPITALS, with lowercase (and,
if configured, Title case) in the menu. In muhenkan mode, 半角/全角 shouts the selection. It
exports settings, the catalog, the host's selection and its state.

```sh
cargo test                     # shout's behaviour, the conformance checklist, every export! option
cargo clippy --all-targets     # stays clean
cargo build --release          # target/release/libshout.{dylib,so,a}
```

| Path | What | For a new IME |
|---|---|---|
| `docs/bada-compatible-v1.md` | The standard | Read; don't edit (changes go upstream) |
| `src/compat/` | The contract layer: `Output`, `KeyEvent`, `Mode`, `State`, the `Ime` and `Report` traits, `export!`, a headless test host | Keep as is; a change here is a change to the standard |
| `src/shout.rs` | The IME: `impl Ime for Shout`, `impl Report for Shout` | **Replace** |
| `src/lib.rs` | `export! { prefix, ime, data, optional }`, which generates the C ABI | Change the prefix and lists |
| `include/shout.h` | The C header | Rename; list your optional functions |
| `tests/shout.rs` | Behaviour, typed through `compat::host::Host` | Replace |
| `tests/conformance.rs` | The checklist (§10), through the C ABI | Keep; change its two marked lines |
| `tests/export.rs` | Every `export!` option, with a small IME | Keep; it's also the example for data |
| `imes/shout.json` | superkeyboard's manifest (§9) | Rename |

## Make a new IME

1. Copy this repository. In `Cargo.toml`, rename the package and `[lib] name`.
2. Implement `compat::Ime`, and `compat::Report` if you export `state`:

   ```rust
   impl Ime for MyIme {
       type Data = ();                 // or your dictionaries, built by fn data(buffers)
       type Settings = NoSettings;     // or a serde struct with #[serde(default)]
       fn new(data: Arc<()>) -> Self { … }
       fn key(&mut self, e: &KeyEvent) -> Output { … }
       fn set_mode(&mut self, mode: Mode) -> Output { … }   // commit what's staged, then switch
       fn flush(&mut self) -> Output { … }                  // commit what's staged
       // optional: data, set_settings + settings, set_profile, host_selection, catalog,
       // dictionary_load
   }

   impl Report for MyIme {
       fn staged(&self) -> String { … }                     // what flush would commit; changes nothing
       fn profile(&self) -> Option<String> { … }            // optional
   }
   ```

   The contract layer fills in the state's `mode` and display fields; your IME only says what's
   staged.

3. Export it in `src/lib.rs`:

   ```rust
   export! {
       prefix: "myime",                              // myime_new, myime_key, …
       ime: crate::MyIme,
       data: [(words, words_len), (lm, lm_len)],     // myime_data_new's buffers, in order; [] for none
       optional: [set_settings, catalog, state],     // only what the IME really does
   }
   ```

4. In `tests/conformance.rs`, point `abi` at your `ffi` module and `STAGE` at keys that compose
   something. Remove the checks for optional functions you don't export. Run `cargo test`.
5. Update `include/`, `imes/`, this README and `CLAUDE.md`.

Every exported function catches panics. An engine that panics answers `{}` for that call, which
means nothing is consumed, and nothing unwinds into the wrapper.

## Run it

**In superkeyboard:**

```sh
cargo build --release
superkeyboard imes/shout.json
curl -s localhost:8790/run -H 'content-type: application/json' \
  -d '{"ime": "shout", "keys": "h e l l o Space w o r l d Tab Enter"}' | jq -r .view
# HELLO world
# │
```

**Embedded:**

- **Dynamic** (superkeyboard, any `dlopen` / `LoadLibrary` wrapper): the `.dylib`, `.so` or
  `.dll`, plus a manifest.
- **Static** (a native shell, as bada's macOS shell links `libbada.a`): the `.a` and
  `include/shout.h`.
- **Rust**: depend on the crate and drive `compat::Ime` directly, or use `compat::host::Host` for a
  text field around it.

## License

[CC0 1.0](LICENSE): the standard, the template and the example IMEs are in the public domain.
IMEs built from this template may use any license. bada ime itself is MIT.
