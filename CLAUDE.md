# bada-compatible

- A template for bada-compatible IME engines: Rust, headless, the C ABI of bada ime 0.2.0
  (`../bada-ime/engine/include/bada.h`, `engine/src/ffi.rs`). The contract is
  `docs/bada-compatible-v1.md`; if code and that doc disagree, fix one in the same change.
- `src/compat/` is the contract layer, shared by every IME made from the template: a change there is a
  contract change (versioning rules in the doc's §1). `src/shout.rs` is the example IME; new IMEs
  replace it.
- Keep it compatible with superkeyboard (`../superkeyboard`, `docs/ime-contract.md` there):
  `imes/shout.json` must load there.
- Names: bada (lowercase); muhenkan / henkan mode.
- License: CC0, repository and example IMEs alike.

## Building

- `cargo test` (shout behaviour, v1 conformance through the C ABI, every `export!` option).
- `cargo clippy --all-targets` stays clean.
- `cargo build --release` → `target/release/libshout.{dylib,a}`; then
  `../superkeyboard/target/release/superkeyboard imes/shout.json` for an end-to-end check.
