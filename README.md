# bada-compatible

A template for building **bada-compatible IME engines**: input method cores, in Rust, that speak
the same C ABI as [bada ime](https://github.com/team-mirae/bada-ime)'s engine. Headless: key events
in, JSON out, no I/O, no GUI. Any host that runs bada's engine (its macOS shell, its web simulator,
[superkeyboard](https://github.com/team-mirae/superkeyboard)) can run yours.

**bada-compatible v1** is bada ime 0.2.0's ABI: [docs/bada-compatible-v1.md](docs/bada-compatible-v1.md).

The template's IME is **shout** (superkeyboard's example IME, grown up a little): henkan mode
composes letters as CAPITALS, with lowercase (and optionally Title case) in the menu; 半角/全角 in
muhenkan shouts the selection.

```sh
cargo test                       # shout's behaviour, the v1 conformance checklist, export!
cargo build --release            # target/release/libshout.{dylib,so,a}
```

## Layout

| Path | What | For a new IME |
|---|---|---|
| `src/compat/` | The v1 layer: `Output`, `KeyEvent`, `Mode`, the `Ime` trait, the `export!` macro, a headless test host | Keep as is |
| `src/shout.rs` | The IME: `impl Ime for Shout` | **Replace** |
| `src/lib.rs` | `export! { prefix, ime, data, optional }`: the C ABI | Change the prefix and lists |
| `include/shout.h` | The C header | Rename, list your optional functions |
| `tests/shout.rs` | Behaviour, typed through `compat::host::Host` | Replace |
| `tests/conformance.rs` | The v1 checklist through the C ABI | Keep; change its two marked lines |
| `tests/export.rs` | Every `export!` option (data buffers, profiles, dictionary), with a tiny IME | Keep; it's also the data example |
| `imes/shout.json` | superkeyboard manifest | Rename |

## Make a new IME

1. Copy this repository. In `Cargo.toml`, rename the package and `[lib] name`.
2. Write your engine: a type that implements `compat::Ime`.

   ```rust
   impl Ime for MyIme {
       type Data = ();                 // or your dictionaries, built by fn data(buffers)
       type Settings = NoSettings;     // or a serde struct with #[serde(default)]
       fn new(data: Arc<()>) -> Self { … }
       fn key(&mut self, e: &KeyEvent) -> Output { … }
       fn set_mode(&mut self, mode: Mode) -> Output { … }   // commit what's staged
       fn flush(&mut self) -> Output { … }                  // commit what's staged
       // optional: data, set_settings + settings, set_profile, host_selection, catalog,
       // dictionary_load; and impl Report (staged) for `state`, which wrappers expose (§12)
   }
   ```

3. Export it in `src/lib.rs`:

   ```rust
   export! {
       prefix: "myime",                              // symbols myime_new, myime_key, …
       ime: crate::MyIme,
       data: [(words, words_len), (lm, lm_len)],     // myime_data_new's buffers, in order; [] for none
       optional: [set_settings, catalog],            // only what the IME really does
   }
   ```

4. In `tests/conformance.rs`, point `abi` at your `ffi` module and `STAGE` at keys that compose
   something; drop the checks for optional functions you don't export. `cargo test`.
5. Update `include/`, `imes/`, this README and `CLAUDE.md`.

Every exported function catches panics; an engine that panics answers `{}` (nothing consumed) for
that call.

## Run it in superkeyboard

```sh
cargo build --release
superkeyboard imes/shout.json
curl -s localhost:8790/run -H 'content-type: application/json' \
  -d '{"ime": "shout", "keys": "h e l l o Space w o r l d Tab Enter"}' | jq -r .view
# HELLO world
# │
```

## Embed it

- **Dynamic** (superkeyboard, any `dlopen` host): `target/release/libshout.dylib` and a manifest.
- **Static** (a native shell, as bada's macOS shell links `libbada.a`): `target/release/libshout.a`
  and `include/shout.h`.
- **Rust**: depend on the crate and drive `compat::Ime` directly, or `compat::host::Host` for a
  text field around it.

## License

[CC0 1.0](LICENSE): public domain, and so are the example IMEs. IMEs built from this template may
use any license. bada ime itself is MIT.
