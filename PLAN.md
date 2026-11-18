# computercraft — migration plan (SDK format, version 0.2.0)

**Goal:** `computercraft` becomes one flat `.hmod` (`api = "0.2"`) at nightly
`0b14f8a`, with `runtime.rs` byte-identical and its 31 tests green, every old
helper mapped to a `Hooks`/Part method, and the package passing
`cargo horizon-mod check` + `fits`.

**Decision carried over:** ladder rung **(B)** — the port ships a documented
Lua-5.2 subset (`src/runtime.rs`). The SDK documents
`horizon-launcher native-lock .` for registry dependencies, so rung (A) is no
longer refused by the lock; a real VM is a later slice.

**Files**
- `src/runtime.rs` — the old file, byte for byte (sha256 `7f9465ab…`).
- `src/commands.rs` — new, pure: `asked()`, `screen_json()`, JSON escaping.
- `src/screen.rs` — new, pure: `pose()`, `facing_of()`, `line_origin()`.
- `src/lib.rs` — the `Computer` NBT record, `ComputerPart` (`BlockPart` +
  `BlockEntityPart`), `impl Hooks` (`register`, `register_commands`,
  `run_mod_command`) and `act()`/`computer_at()`.
- `horizon-mod.toml`, `Cargo.toml`, `assets/`, `data/`, `LICENSE`,
  `LICENSE-NOTICE.md`.

**Tasks (each ends green before the next)**
1. Scaffold at `/tmp/opencode/cc-new`, read the SDK's `Hooks`, Part traits and
   host areas; search every hook with `cargo horizon-mod hooks`.
2. Carry `runtime.rs` byte-identical; add the two new pure modules **test-first**
   (18 tests seen failing with `not yet implemented`, then green).
3. Rewrite the glue as one `impl Hooks` + one `BlockEntityPart`; map every old
   helper in the table in `notes.md` (9 kept, 1 recorded gap).
4. Carry the content into the root `assets/`+`data/`, keep `LICENSE` and
   `LICENSE-NOTICE.md` inside the package, move the retired tree out of the port.
5. `fmt` + `clippy -D warnings` + `cargo test` + `cargo horizon-mod build` +
   `check` + `fits`, all through a lease; rebuild twice for determinism.
6. `tools/e2e.sh` and `tools/visual.sh` rewritten for a launcher-built game;
   `notes.md` §Migration, gaps, evidence and the `Skills:` line updated.
