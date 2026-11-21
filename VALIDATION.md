# computercraft — validation ladder evidence

All commands ran through `ports/tools/build-slot.sh` (one lease per line unless
noted), `CARGO_INCREMENTAL=0`, no `--target-dir`, at the campaign pin:
nightly **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** / mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**. Raw logs:
`../_shared/evidence/computercraft/`, game legs
`../_shared/evidence/computercraft-e2e/`. Every log line is stamped with the
package sha256 and size it ran against (**47,938 B**, sha256
`e364023e7be61bcc142f79ea7084aaa7b07f935ebb2c118a9fe517e764304c3d`).

The pin moved from `0b14f8a` (the migration's pin) to `0a875da` for slice 2.
`[native] horizon` was updated **and** `.horizon/sdk` deleted and re-fetched, so
the build used the new surface (`SDK 0a875da … in ./.horizon/sdk`);
`HANDOVER` §9d.17a is the trap this avoids.

## 1. The port's own crate (`cargo`)

```
$ cargo fmt --check                                              # sdk-fmt.log
fmt_exit=0

$ cargo clippy --all-targets --offline -- -D warnings             # sdk-clippy.log
Finished `dev` profile [unoptimized + debuginfo] target(s)
clippy_exit=0

$ cargo test --offline                                            # sdk-test.log
running 56 tests
test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

- **31 of the 56 are `runtime`'s**, on `src/runtime.rs`
  (`sha256 e11eebb6af48ba51b118ee23c638a8b837f96cccafc2a348b4d135d7e160a017`, the
  same file in the tree and in the archive: `verify-sdk-package` compares them).
  It is the retired `.hnmod` file (`7f9465ab…`) plus 0.2.1's `MAX_STRING_BYTES`
  budget and its test.
- **6 new (`family`) + 1 new `commands`** in slice 2, plus the four `commands`
  tests the `screen_json(block, …)` signature touched. TDD: the bodies were
  `todo!()` first and the run showed
  `test result: FAILED. 47 passed; 9 failed`, each with
  `not yet implemented: …` — quoted verbatim in
  `../_shared/evidence/computercraft/red-0.2.2-family-readout.log`; the run
  above is the green one.

## 2. The package

```
$ cargo horizon-mod build                                         # two runs, cmp
SDK 0a875da (build 0a875da91dbb0fdcb7ba8c42c11341aa229af51a) in ./.horizon/sdk
Built ./target/horizon-mod/computercraft.hmod
e364023e7be61bcc142f79ea7084aaa7b07f935ebb2c118a9fe517e764304c3d  (both builds)

$ cargo horizon-mod check computercraft.hmod                      # sdk-check.log
computercraft 0.2.2 (native-mod, format native-1/0a875da91dbb0fdcb7ba8c42c11341aa229af51a/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  47938 bytes, SHA-256 e364023e7be61bcc142f79ea7084aaa7b07f935ebb2c118a9fe517e764304c3d, unsigned
  native source: unsandboxed; not executed by the registry check. …
check_exit=0

$ cargo horizon-mod fits computercraft.hmod --build 0a875da       # sdk-fits.log
CC: Tweaked:
0a875da  made for
fits_exit=0

$ cargo horizon-mod fits computercraft.hmod --build 0b14f8a       # sdk-fits-other.log
CC: Tweaked:
0b14f8a  fits
fits_exit=0
```

The first `fits` line is the **tautology** (`--build` equals the package's own
pin, so the comparison is skipped); the second is the real gate. 44 entries:
`horizon-mod.toml`, `Cargo.toml`, `LICENSE`, `LICENSE-NOTICE.md`, `README.md`,
`uses.json`, `assets/computercraft/**` (23), `data/computercraft/**` (10),
`src/{lib,family,runtime,commands,screen}.rs` — and **no `Cargo.lock`**
(`rm -f Cargo.lock` before each build).

Reproducibility: two builds `cmp` → identical bytes; the zip's entry timestamps
are normalised to 1980-01-01. The documentation travels **inside** the package
(`README.md`, `LICENSE`, `LICENSE-NOTICE.md`), so editing any of them changes the
hash and needs a repackage — the 0.2.2 hash above is after the final README and
licence-notice edits.

## 3. Game runs (launcher-built game, leased)

```
$ ports/tools/launcher-game.sh build computercraft     # Compiling computercraft (1 of 1)
$ eval "$(ports/tools/launcher-game.sh env computercraft)"
$ build-slot.sh run --job W1-computercraft --port ports/computercraft --step e2e -- \
      bash ports/computercraft/tools/e2e.sh
PASS: all three computers (normal, advanced, command) … 
EXIT:0
```

- **modded leg**: `computercraft-e2e/e2e-launcher.log` → `PASS` + `EXIT:0`;
  every phase asserts a parsed `ctl` value (see `notes.md` §"Slice 2" for the
  transcript). `--offscreen`, because the recipe phase opens a crafting-table
  menu.
- **control leg**: the same command with `GAME=$CONTROL_GAME` (the same Nightly
  without the package) → `FAIL: the package did not provide
  computercraft:computer_normal` + `EXIT:1` — the differential that the ids and
  the behaviour come from this package.
- **selftest**: `SELFTEST=1` inverts the advanced computer's terminal assertion →
  `FAIL: selftest advanced terminal: expected [selftest], got [CraftOS 1.9|no
  ROM: see|/computercraft]` + `EXIT:1` — the counterfactual that the assertion
  can fail.
- `python3 ports/tools/check-e2e-logs.py computercraft-e2e --strict` →
  `logs=3 ok=3 … exit 0`.

`tools/visual.sh` (the screenshot leg) still targets the retired `.hnmod` build
and was not re-run; the two screenshots in `../_shared/evidence/computercraft/`
are from 0.1.0 and are labelled as such.

## 4. Gates

```
$ python3 ports/tools/verify-sdk-package.py computercraft      # 1/1
  ok   package  47938 B sha256 e364023e…
  ok   runtime byte-identical  (tree == archive)
  ok   no packed Cargo.lock  44 entries
  ok   tests  56 passed, 0 failed  (sdk-test.log)
  ok   clippy  0 warning(s)  (sdk-clippy.log)
  ok   check  sdk-check.log
  ok   fits  sdk-fits.log
  ok   skills gate  PASS

$ python3 ports/tools/check-skills.py --notes ports/computercraft
```

`python3 ports/tools/upstream_ready.py computercraft --copyleft` is **not**
expected to pass from this job: `NEEDS_GRANT` in `ports/tools/upstream_ready.py`
lists `LGPL/GPL/AGPL/MPL/EUPL` but not `CCPL` (a one-line tool fix in an
orchestrator-owned file), and the ledger row is the orchestrator's to record.
Both are reported in the hand-back.
