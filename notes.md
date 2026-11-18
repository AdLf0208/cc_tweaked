# CC: Tweaked — port notes

Ported from **CC: Tweaked 1.21.1-forge-1.121.0** (`cc-tweaked/CC-Tweaked`),
Minecraft Java 1.21.1 NeoForge, **content / `deep`**, core, priority P0.

**Format: the flat SDK package (mod API `0.2`), version `0.2.0`.** The retired
`.hnmod` bundle (0.1.0, submitted as **PR #788**) was closed in the format purge
(Horizon retired `.hnmod`, #17992). The code mod is now one `computercraft.hmod`
carrying `horizon-mod.toml` + `Cargo.toml` + `src/` + `assets/` + `data/`,
compiled by the launcher against **Horizon nightly
`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (2026-11-18) and **mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**. `runtime.rs` is the old file
byte for byte.

## Licence — CCPL-1.0, rank 1 (copyleft-style), publishable under the same licence

Read in full from the URL the jar's manifest names
(`https://raw.githubusercontent.com/dan200/ComputerCraft/master/LICENSE`,
98 lines, SHA-256 `f72a6699…2935f`), stored verbatim as `LICENSE`; see
`LICENSE-NOTICE.md` for where every notice was read. Both files travel **inside**
the package. Redistribution is allowed provided **(a)** the complete source is
obtainable at no cost and **(b)** every distribution stays under the CCPL. This
port therefore ships its complete source beside the package (`src/` is in the
`.hmod`) and is **not** relicensed; publishing still needs the campaign
maintainer's approval (`ARCHITECTURE.md` §8), and the upstream submission carries
the reviewer note that `MOD-POLICY.md` lists only permissive licences for new
projects.

## Class + tier and the approach

`content` / **`deep`** (`horizon/docs/modpacks.md`): the mod has block entities,
a runtime and peripherals, so the slice must prove **a program runs and its
output is observable**, not that a block exists. Approach: **carry-over** for
the block's assets/data (four upstream textures, the blockstate shape, the
recipe shape) and an **agent port** for behaviour (the runtime and the terminal).

## Decision ladder — landed on rung (B), no Lua dependency

The brief's ladder, worked in order, with the measurements:

| Rung | Outcome | Evidence |
|---|---|---|
| **(A) real Lua** (`piccolo`, pure-Rust VM) | **refused by the retired planner** | `retired-hnmod/…` in `../_shared/evidence/computercraft/`: `E_CRATE_DEPENDENCY: computercraft needs piccolo ^0.3, which isn't in Horizon's Cargo.lock` (exit 1); the old pin's lock had **0** Lua packages in **565**, for the injected module. The VM itself built in a scratch crate (`ladder-a-piccolo-standalone.log`) |
| | **re-openable at the SDK** | the SDK's own `Cargo.toml` comment says *"Any other registry dependency must be pinned with `horizon-launcher native-lock .` before building"* — a launcher tool that did not exist at the old pin. So a real Lua VM is no longer blocked by the lock, only by a dependency decision + a slice; that is a **later slice**, not this migration (see Gaps) |
| **(B) documented Lua subset** | **implemented and unchanged by the migration** | `src/runtime.rs`: lexer, precedence-climbing parser and evaluator, **31 unit tests**, byte-identical to the `.hnmod` version (`sha256 7f9465ab…74d2be`) |
| **(C) neither** | not reached | the readout below makes the output observable, so a computer that does something could be shipped |

## The slice boundary

One block, one program, one observable:

1. `computercraft:computer_normal` — `facing` (4) × `state` (`off`/`on`/`blinking`)
   = 12 states on the `minecraft:stone` template, upstream's model shape
   (`minecraft:block/orientable` with `computer_normal_front`/`_side`/`_top`;
   the running state uses upstream's lit `computer_on` face), self-drop loot,
   the upstream crafting shape rewritten to 26.3 ingredients, one creative tab.
2. A block entity (`computercraft:computer`, declared in code) keeping
   `Program`, `Buffer`, `Running`, `Powered` in its saved compound; it ticks,
   boots its program once, lights `state=on`, and sends the terminal to clients.
3. A **Lua 5.2 subset** in `runtime.rs`: `print`, numbers (int/float, `e`
   exponent), strings (single/double quotes, `\n \t \r \a \b \f \v \\ \" \'`),
   `..`, `+ - * / % ^`, `< <= > >= == ~=`, `and`/`or`/`not`, `#` (bytes),
   `local`, assignment (nearest scope, else global), `if`/`elseif`/`else`,
   `while`, numeric `for` with step, `do … end`, `return`, `--` and `--[[ ]]`
   comments, blocks and scopes, Lua's precedence table (`..` right-associative,
   `^` right-associative and above unary), `print` tab-separated.
4. The terminal is a 16×10 buffer (the block's screen); clients draw it with the
   font atlas.
5. Four commands, now **registered in the server's command tree** (they complete
   in chat and are permission-checked): `/computercraft load <pos> <program>`,
   `run <pos>`, `screen <pos>`, `power <pos> on|off`.

## Migration to the SDK format

### File map

| `.hnmod` (retired) | `.hmod` 0.2 (now) |
|---|---|
| `computercraft.hnmod` (23,130 B) | `computercraft.hmod` (**40,694 B**, sha256 `830d815e…7d5d`, 24 entries) |
| `horizon-native.json` + `compatibility.json` | `[native]` in `horizon-mod.toml` (exact `game`/`horizon`/`mixins`); `compatibility.json` deleted |
| `mixins/computercraft/horizon-mixin.toml` (plan, 9 injections) | **gone**: hooks are methods of one `impl Hooks` / one `impl BlockEntityPart` |
| `mixins/computercraft/{block_entity,block_entity_ticks,…}.rs` (8 helpers) | **gone**: the same bodies are the Part's methods in `src/lib.rs` |
| `mixins/computercraft/support.rs` (13,668 B glue) | `src/lib.rs` (the Part + `Hooks` + the NBT record) — ~15 KB, no per-hook split |
| `mixins/computercraft/runtime.rs` (46,546 B) | **`src/runtime.rs`, byte-identical** (`sha256 7f9465aba8b94b4c7480a3d0d655bec200726401bd90dbd058d20617a874d2be`) |
| `mixins/computercraft/data/computercraft-content.hmod` + `embedded_content.rs` (AE2 pattern) | **gone**: `assets/` and `data/` sit beside `src/` in the one package; no embedding and no `content_root` hook |
| `content/horizon-mod.toml` (id/version 0.1.0) | **one** `horizon-mod.toml` (`api = "0.2"`, `capabilities = ["registries"]`, version **0.2.0**) |
| — | `src/commands.rs` + `src/screen.rs`: the pure logic the migration *added* (below) |
| `rust-toolchain.toml`, per-hook `Cargo.toml` | deleted (the launcher owns the toolchain; `Cargo.toml` is the port's own crate) |

`retired-hnmod/` (the whole old tree, the two old packages and the old tooling)
is kept as evidence at `ports/_shared/evidence/computercraft/retired-hnmod/`,
**outside** the port directory: `cargo horizon-mod build` refuses a tree that
still contains a `horizon-mixin.toml` (`error: unexpected package path: …`), so
the retiree cannot live beside the new package.

### Helper → hook table

| old helper (`.hnmod`) | new `Hooks`/Part method, host call | verdict |
|---|---|---|
| `block_entity.rs` (`block_entity`, API 15) | `BlockEntityPart::block_entity` → `Option<Compound>` | **kept** |
| `block_entity_ticks.rs` (`block_entity_ticks`) | `BlockEntityPart::block_entity_ticks` → `event.ticks = true` | **kept** |
| `block_entity_tick.rs` (`block_entity_tick`) | `BlockEntityPart::block_entity_tick` + `host.set_block_property(pos,"state","on")` + `host.block_entity_sync(pos)` | **kept** |
| `block_entity_update_tag.rs` (`block_entity_update_tag`, 45) | `BlockEntityPart::block_entity_update_tag` → `Option<Compound>` (`Buffer` + `Powered`) | **kept** |
| `block_entity_submit.rs` (`block_entity_submit`, 71) | `BlockEntityPart::block_entity_submit` + `draw.{push,translate,rotate_about,scale,text,pop}` | **kept** |
| `state_for_placement.rs` (`state_for_placement`, 76) | `BlockPart::state_for_placement` + `LevelHost::state_block` / `state_with_property` | **kept** (a block moment: a Part method, not a global hook) |
| `register.rs` (`register_commands`, 8) | `Hooks::register_commands` → `event.commands.push(Command::new(usage))` | **kept** |
| `perform.rs` (`perform_command`, 8) | `Hooks::run_mod_command` → `Flow::Cancel`, args from `CommandInvocation::argument`, replies via `host.commands().feedback` | **kept, and upgraded** to typed arguments |
| `embedded_content.rs` (`content_root` + AE2 pattern) | — | **not needed**: the format has no second file to embed |
| `support.rs`: `game.world.block_entities.get_mut` | `event.entity.nbt` in the Part; `host.block_entity(level,pos)` + `EntityRef::data`/`merge_data` in the commands | **kept** |
| `game.native_feedback` | `host.commands().feedback(Text::literal(…))` | **kept** |
| `game.native_set_block_property` | `host.set_block_property` (`BlockStatesServerHost`) | **kept** |
| `game.native_block_entity_sync` | `host.block_entity_sync` (`BlockEntitiesServerHost`) | **kept** |
| `game.native_draw_text(draw, text, at, colour, shadow, bg)` | `BlockEntityDrawContext::text(&Text, [f32;2], 0xRRGGBB, shadow, Option<u32> outline)` | **kept** (the colour format is `0xRRGGBB`, so `0x00d8d8d8` keeps meaning grey; the old `0u32` background became `None`) |
| `game.native_block_state_property(state, "facing")` | **no equivalent for a `BlockState`**: `LevelRef::block_property(host,pos,key)` reads a *position*, and `BlockState::property` needs a `&dyn BlockMetadata` a hook is not given | **worked around, recorded**: `screen::facing_of` parses `host.block_state_string(state)` (`…[facing=east,state=off]`), unit-tested |
| `game.native_state_block` | `LevelHost::state_block` | **kept** |

**The old `perform_command` question, answered:** the SDK exposes its exact
NeoForge counterpart — `Hooks::player_command` (`CommandEvent`, `contract =
cancel`, `out = canceled`, `replies: &mut Vec<Text>`), the hook a raw typed line
would be answered from. The port does **not** need it: `register_commands` puts
the four usages in the tree (chat completion, permission levels) and
`run_mod_command` (a Horizon addition; NeoForge's counterpart is *"part of the
Brigadier node added in RegisterCommandsEvent"*) hands over `CommandInvocation`
with `BlockPos` and `String` arguments already parsed. The e2e's
`ctl chat "/computercraft screen …"` reaches it on the same player path as
before. The old chat plumbing therefore **survives, and is better typed**; nothing
about it is a gap.

### New pure modules, and why `runtime.rs` did not grow

`runtime.rs` had to stay byte-identical, so the two pieces of logic the migration
needed went to their own pure modules, test-first (18 new tests, seen failing
with `todo!()` first):

- `src/commands.rs` — `asked(sub, program, switch) -> Asked` (the command surface,
  its two usage lines) and `screen_json(at, powered, running, lines)` (the one
  JSON line the e2e parses, with the port's own JSON escaping: the port has no
  JSON crate).
- `src/screen.rs` — `pose(facing)` (the corner and quarter-turn per face),
  `facing_of(state_string)` and `line_origin(index)`.

### Manifest mapping

`horizon-mod.toml` is the only manifest: `id = "computercraft"` (also the
resource namespace), `version = "0.2.0"`, `api = "0.2"`,
`capabilities = ["registries"]` (the block + the block entity type),
`side = "both"` (a client needs the screen; the server needs the tick),
`license = "CCPL-1.0"`, `[native] game = "26.3"` + the two exact commits. The
content half's old `content/horizon-mod.toml` is gone; its assets and data are
the root `assets/` and `data/`.

### Build and check commands (all through a lease)

```sh
ports/tools/build-slot.sh run --job sdk-cc --port ports/computercraft --step test -- \
    bash -c 'cd ports/computercraft && cargo fmt --check && cargo clippy --all-targets --offline -- -D warnings && cargo test --offline'
ports/tools/build-slot.sh run --job sdk-cc --port ports/computercraft --step build -- \
    bash -c 'cd ports/computercraft && cargo horizon-mod build'      # -> target/horizon-mod/computercraft.hmod
ports/tools/build-slot.sh run --job sdk-cc --port ports/computercraft --step check -- \
    bash -c 'cd ports/computercraft && cargo horizon-mod check computercraft.hmod && cargo horizon-mod fits computercraft.hmod'
```

### Traps the next migration must know

1. **`cargo horizon-mod build` walks the whole port directory.** Any legacy
   `horizon-mixin.toml` left behind stops it with
   `error: unexpected package path: <path>` — move the retired tree out (this
   port keeps it in `ports/_shared/evidence/<id>/retired-hnmod/`).
2. **The package is built, not the binary.** `cargo horizon-mod check` reports
   *"native source: unsandboxed; not executed by the registry check"*, so the
   check is a format/manifest check only. The compile evidence is `cargo
   clippy --all-targets`/`cargo test` against the same `.horizon/sdk` the
   launcher uses; an in-game run needs `horizon-launcher native-build`
   (not installed on this box — see Verification).
3. **`build-slot.sh run` starts in the caller's directory**, so the command must
   `cd` into the port itself; `--port` labels the lease and links the caches, it
   does not change the working directory.
4. **`build-slot.sh run` forwards `test`/`check`/`build` to the second machine**,
   whose synced copy of a *new* port does not exist yet: it fails with
   `cd: /home/deepseek/atm10/computercraft: No such file or directory`. Run with
   `BUILD_LOCAL=1` (or `remote-slot.sh run`, which syncs first) for a new port.
5. **`Compound` is `BTreeMap<JavaString, Tag>`.** There is no `get(&str)`; use
   `nbt.get(&"Program".into())` / `nbt.insert("Program".into(), …)` and let
   inference name the key type (`JavaString` is not re-exported by `mod_api`).
6. **A block moment on a block entity Part must be written on the `BlockPart`
   impl.** `state_for_placement` is a `BlockPart` method; a block entity Part is
   also registered as a block Part for its blocks, so one struct implements both
   (`#[mod_api::horizon_part]` on each impl).
7. **`state_with_property` returns `Option<BlockState>`, not a nested `Option`.**
   The Part method's own answer is `Option<Option<BlockState>>`: `Some(Some(s))`
   places, `Some(None)` refuses, `None` leaves it to vanilla.
8. **Host area traits need no import for a `dyn ServerHost`/`dyn ClientHost`** —
   calling `host.set_block_property(…)` compiles without `use
   mod_api::prelude::*` (and the glob then trips `-D unused-imports`).
9. **`build-slot.sh` clears `stage/modded` per lease; nothing else is shared.**
   Two builds of the same sources produced byte-identical packages
   (`cmp`-verified, same sha256), timestamps normalised to 1980-01-01 inside the
   zip.
10. **`README.md`, `LICENSE` and `LICENSE-NOTICE.md` are packed**, so any edit to
    them changes the package hash: rebuild and re-record after documentation
    changes (this cost one stale hash here).

## Observability: the route this slice uses, and what is *not* verified

- **Asserted by machines: `/computercraft screen <x> <y> <z>`**, one line of
  JSON (`{"block","pos","powered","running","lines"}`) through
  `Commands::feedback`. `tools/e2e.sh` parses it with `jq` and asserts exact
  lines.
- **Also verified before the migration: the on-screen drawing.**
  `block_entity_submit` draws the buffer with `draw.text` on the block's
  `facing` face (origin and rotation per facing, 6-px glyphs at 1/96 of a block,
  9-px lines). `tools/visual.sh` is the rendering leg; the two screenshots from
  the `.hnmod` build are still the only image evidence
  (`visual-screen-loaded.png`, `visual-screen-off.png` in the evidence folder,
  taken with the identical pose and colour code that `src/screen.rs` now owns).
  The migration **did not re-run** the visual leg: it needs a launcher-built
  game.

### A finding from the visual leg (kept in the code)

A block entity's tag reaches clients only when its chunk is sent or its **block**
changes, so a run that changed only the saved compound never refreshed the
screen. Fixed by calling `block_entity_sync` after every terminal write — in the
tick and in all four commands. The two screenshots are the proof.

## Deviations from upstream (deliberate, recorded)

| Area | This port | Upstream |
|---|---|---|
| Terminal | 16 columns × 10 rows (a one-block screen) | 51 × 19 basic terminal |
| Number formatting | whole numbers print as integers, others as Rust's shortest round-trip form | `%.14g` (so `1e20` prints as `100000000000000000000`, not `1e+20`) |
| Number ↔ string | no coercion: `"10" + 1` is an error | Lua coerces |
| Program sources | loaded by `/computercraft load` (chat cannot carry newlines: separate statements with `;`) | `fs`/`shell`/`edit` from the ROM |
| Runtime limit | a **step budget** (`RUN_STEP_LIMIT` = 100 000) stops a program with `error: … too long running …` | "Too long without yielding" after a configurable wall-clock time, with coroutines to yield |
| Boot program | a port-owned three-line program (`CraftOS 1.9`, `no ROM: see`, `/computercraft`) | `bios.lua` → `shell.lua` prompt |
| Computer power | `/computercraft power … on\|off` is a port diagnostic (also the e2e's broken-computer control) | no such command; a computer needs no fuel |
| Block template numbers | the stone template's hardness/resistance/sounds | upstream's `ComputerBlock` properties (Java, not read) |
| Loot | self-drop with `survives_explosion` | the jar's table is gated on `computercraft:block_named` / `has_id` / `player_creative`, three custom conditions 26.3 does not have (recorded gap: a named/ID'd computer keeps its label upstream) |
| Recipe | stone ×7 + `minecraft:redstone` + `minecraft:glass_pane` | the same shape with `c:dusts/redstone` and `c:glass_panes` (26.3 ships no `c:` tags) |
| `state=blinking` | shares the lit model; never set by the code | the server's blink phase when a computer is idle-but-busy |
| Command parsing | the server's own dispatcher (`block_pos`, greedy `text`, `word`), so a bad position is refused by the game with vanilla's message | a Brigadier tree with a `block_pos` argument, the same shape |

## Upstream defects fixed

**None found in this slice.** The two places where the artefact disagrees with
26.3 are an API gap (the loot table's custom conditions) and a tag naming (`c:`
tags absent in 26.3); both are handled by the closest supported form and
recorded above, not by reproducing a broken behaviour.

## Gaps (what a later slice must do)

- **The ROM**: 105 Lua files (`bios.lua`, 20 `rom/apis`, 23 `rom/modules`, 60
  `rom/programs`) and the shell. This needs a real Lua 5.2 VM. Rung (A) is no
  longer refused by the lock at this pin — the SDK documents
  `horizon-launcher native-lock .` for registry dependencies — but that tool is
  not installed here and adding a VM is a slice of its own.
- **Language features**: tables, functions and closures, `break`, `repeat`,
  `in`/generic `for`, multiple returns and multiple assignment, metamethods,
  `pcall`/`error`, string/table/math/os libraries, `require`/`dofile`,
  coroutines (the basis of every CC API), varargs, `#` on tables, integer
  division `//`, `goto`, long strings `[[…]]`, hex literals, `\ddd` escapes.
- **Peripherals and the world**: wired/wireless modems and `rednet`, monitors,
  disk drives and floppies, printers, speakers, the lectern, cables
  (`cable`/`wired_modem_full`), the redstone relay, pocket computers, turtles,
  `gps`, `http`, `peripheral`/`os.pullEvent`.
- **Block behaviour**: `computer_advanced`, `computer_command`, the other 15
  blockstates; the blinking animation; the computer's GUI/menu (the pin does
  have `BlockEntityPart::screen`/`screen_draw`, so a menu is now expressible);
  the terminal's cursor, colours and `term.setTextColour`; the computer's real
  loot conditions; `computercraft:*` component types (a computer keeps its ID).
- **A `BlockState`'s property read**: the pin reads a *position's* property and
  asks a `BlockMetadata` for a state's; a hook is given neither for a `BlockState`
  it was handed. `screen::facing_of` parses the state string instead. Issue
  shape: *"Read a `BlockState`'s property in a hook without a world lookup
  (`native_block_state_property`, which the retired API had)."*
- **Placement**: the facing rule is unit-tested (`facing_for_placement`) and
  wired through `state_for_placement`, but no in-game run yet places a computer
  by hand and reads its facing back.
- **The in-game legs**: `tools/e2e.sh` and `tools/visual.sh` are rewritten for
  the SDK layout and stop at `missing GAME` here, because
  `horizon-launcher` is not installed on this box; until a
  `horizon-launcher native-build` run exists, the behaviour claims of this
  migration rest on the compile + 49 unit tests, not on a `ctl` transcript.

## Verification (evidence in `../_shared/evidence/computercraft/`)

| Step | Command | Result |
|---|---|---|
| Format | `cargo fmt --check` | **exit 0** (`fmt.log`) |
| Lints | `cargo clippy --all-targets --offline -- -D warnings` | **exit 0**, 0 warnings (`clippy.log`) |
| Unit tests | `cargo test --offline` | **49 passed; 0 failed** (`unit-tests.log`): 31 `runtime` (byte-identical file) + 18 new (`commands` 11, `screen` 7); TDD: the 18 failed with `not yet implemented` first |
| Package | `cargo horizon-mod build` | **exit 0**, `SDK 0b14f8a … Built ./target/horizon-mod/computercraft.hmod` (`build.log`) |
| Registry check | `cargo horizon-mod check computercraft.hmod` | **exit 0**: `computercraft 0.2.0 (native-mod, format native-1/0b14f8ac…/e59da909…, for 26.3)`, 40 694 bytes, sha256 `830d815e…7d5d`, unsigned (`check.log`) |
| Fits | `cargo horizon-mod fits computercraft.hmod` | **exit 0**: `CC: Tweaked: 0b14f8a  made for` (`fits.log`) |
| Reproducibility | `cargo horizon-mod build` twice | **identical bytes** (`cmp`), same sha256 (`package.log`) |
| In-game (`e2e.sh`, `visual.sh`) | needs `horizon-launcher native-build` | **not run here**: `horizon-launcher` is not installed; the scripts now stop at `missing GAME` instead of running an unmodified game |

Retired-format evidence (kept, still valid for the `.hnmod` build it describes;
the old crate's own `unit-tests.log` was replaced by this migration's run):
`ladder-check.log`, `ladder-test.log`, `e2e-default.log`, `e2e-embedded.log`,
`e2e-selftest.log`, `visual.log` and the two screenshots, plus the whole old
tree in `retired-hnmod/` (see `../_shared/evidence/computercraft/README.md`).

Deliverable: **`computercraft.hmod`** (40,694 B, 24 entries, SHA-256
`830d815ef6cedb1786df671d7e00080ec65a05f3a78d5e636acc68d12daf7d5d`) holding
`horizon-mod.toml`, `Cargo.toml`, `Cargo.lock`, `LICENSE`,
`LICENSE-NOTICE.md`, `README.md`, `uses.json`, `assets/computercraft/**` (9
files), `data/computercraft/**` (4 files) and `src/{lib,runtime,commands,screen}.rs`.

### Changelog for the resubmission (supersedes #788)

> Supersedes #788, closed because Horizon retired `.hnmod` (#17992). Changes:
> flat SDK package with `api = "0.2"` and `[native]` pinned to `0b14f8a`; hooks
> rewritten as `impl mod_api::Hooks` methods against typed hosts/events (8 hooks
> mapped, 0 recorded as gaps); `runtime.rs` and its tests unchanged.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port
