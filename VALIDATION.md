# computercraft — validation ladder evidence

All commands ran through `ports/tools/build-slot.sh` (one lease per line unless
noted), `CARGO_INCREMENTAL=0`, no `--target-dir`, at the SDK format's pin:
nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** / mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**. Raw logs:
`../_shared/evidence/computercraft/`.

## 1. The port's own crate (`cargo`)

```
$ cargo fmt --check                                              # fmt.log
fmt_exit=0

$ cargo clippy --all-targets --offline -- -D warnings             # clippy.log
Finished `dev` profile [unoptimized + debuginfo] target(s)
clippy_exit=0

$ cargo test --offline                                            # unit-tests.log
running 49 tests
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

- **31 of the 49 are `runtime`'s**, on the byte-identical `src/runtime.rs`
  (`sha256 7f9465aba8b94b4c7480a3d0d655bec200726401bd90dbd058d20617a874d2be`,
  `cmp` against `retired-hnmod/mixins/computercraft/runtime.rs`: identical).
- **18 are new** (`commands` 11, `screen` 7). TDD: both modules were written with
  `todo!()` bodies first and the run showed
  `test result: FAILED. 31 passed; 18 failed`, each failing with
  `not yet implemented: …`; the bodies were then written and the run above is the
  green one.

## 2. The package

```
$ cargo horizon-mod build                                         # build.log
SDK 0b14f8a (build 0b14f8acfd97fa2c750e42795249dcc981178ca4) in ./.horizon/sdk
Built ./target/horizon-mod/computercraft.hmod

$ cargo horizon-mod check computercraft.hmod                      # check.log
computercraft 0.2.0 (native-mod, format native-1/0b14f8acfd97fa2c750e42795249dcc981178ca4/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  40694 bytes, SHA-256 830d815ef6cedb1786df671d7e00080ec65a05f3a78d5e636acc68d12daf7d5d, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.

$ cargo horizon-mod fits computercraft.hmod                       # fits.log
CC: Tweaked:
0b14f8a  made for
```

24 entries (`package.log`): `horizon-mod.toml`, `Cargo.toml`, `Cargo.lock`,
`LICENSE`, `LICENSE-NOTICE.md`, `README.md`, `uses.json`,
`assets/computercraft/**` (9), `data/computercraft/**` (4),
`src/{lib,runtime,commands,screen}.rs`.

Reproducibility: `cargo horizon-mod build` twice, `cmp` → identical bytes (both
`830d815e…7d5d`); the zip's entry timestamps are normalised to 1980-01-01. The
documentation travels **inside** the package (`README.md`, `LICENSE`,
`LICENSE-NOTICE.md`), so editing any of them changes the hash and needs a
repackage — that is exactly how the first recorded hash (`67e6d334…`) became
stale after a `README.md` edit here.

`check` does **not** compile the native source (it says so), so the compile
evidence is section 1 against the same `.horizon/sdk` the launcher builds with.

## 3. Game runs — rewritten, not re-run here

`tools/e2e.sh` and `tools/visual.sh` are rewritten for the SDK layout: one
`.hmod`, no `--dev-mod` content leg (the code mod is compiled into the game),
`GAME=` naming a `horizon-launcher native-build` output, `SELFTEST=1` still
inverting two assertions. **They were not run in this job**: `horizon-launcher`
is not installed on this box, so there is no game binary built with this package.
Both scripts stop with `missing GAME: …` rather than run an unmodified game.

The scenario they assert (unchanged from the `.hnmod` build, where it passed):

1. the item exists and the block places with the state the setblock asked for;
2. the default program boots to `CraftOS 1.9|no ROM: see|/computercraft`;
3. `/computercraft load … print(1+1)` then `screen` gives `2`;
4. a loop and a concatenation give `item 1|item 2|item 3`;
5. the block reads `state=on` while it ran;
6. **control** `/computercraft power … off` empties the terminal and the same
   program prints nothing through a load and ten ticks;
7. **control** a stone block answers `No computer at 5,64,2.`;
8. powering it back on runs the loaded program.

Prior evidence for that scenario is still in the folder, from the retired
`.hnmod` build: `e2e-default.log`, `e2e-embedded.log`, `e2e-selftest.log`,
`visual.log`, `ladder-check.log`, `ladder-test.log` and the two screenshots. It
proves the behaviour of the *same* Lua subset and the same NBT record; it does
**not** prove the new glue, which is what a `native-build` run must confirm.

## 4. Gates

```
$ python3 ports/tools/check-skills.py --notes ports/computercraft
$ python3 ports/tools/check-skills.py --brief ports/_shared/queue/brief-computercraft-sdk.md --vendored
```

Both are green (Skills line present; the brief carries the boot markers and both
skill blocks).

`python3 ports/tools/upstream_ready.py computercraft --copyleft` is **not**
expected to pass from this job: `NEEDS_GRANT` in `ports/tools/upstream_ready.py`
lists `LGPL/GPL/AGPL/MPL/EUPL` but not `CCPL` (a one-line tool fix in an
orchestrator-owned file), and the ledger row is the orchestrator's to record.
Both are reported in the hand-back.
