# CC: Tweaked — port notes

Ported from **CC: Tweaked 1.21.1-forge-1.121.0** (`cc-tweaked/CC-Tweaked`),
Minecraft Java 1.21.1 NeoForge, **content / `deep`**, core, priority P0.

**Format: the flat SDK package (mod API `0.2`), version `0.2.2`.** The retired
`.hnmod` bundle (0.1.0, submitted as **PR #788**) was closed in the format purge
(Horizon retired `.hnmod`, #17992). The code mod is now one `computercraft.hmod`
carrying `horizon-mod.toml` + `Cargo.toml` + `src/` + `assets/` + `data/`,
compiled by the launcher against **Horizon nightly
`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (2026-11-20) and **mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**. `runtime.rs` is the old file
byte for byte. Version history: `0.2.0` the migration, `0.2.1` the review fixes,
**`0.2.2` slice 2 — the computer block family** (§"Slice 2" below).

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
| **(B) documented Lua subset** | **implemented and unchanged by the migration** | `src/runtime.rs`: lexer, precedence-climbing parser and evaluator, **31 unit tests**, byte-identical to the `.hnmod` version at the migration (`sha256 7f9465ab…74d2be`; 0.2.1 added the `MAX_STRING_BYTES` budget, now `sha256 e11eebb6…a017`, and its test) |
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
| `mixins/computercraft/runtime.rs` (46,546 B) | **`src/runtime.rs`, byte-identical** at the migration (`sha256 7f9465aba8b94b4c7480a3d0d655bec200726401bd90dbd058d20617a874d2be`). **From 0.2.1 it is `sha256 e11eebb6af48ba51b118ee23c638a8b837f96cccafc2a348b4d135d7e160a017`**: the review's `MAX_STRING_BYTES` budget (one constant, one guard, one test) is the only change since, and the file's logic is otherwise the 0.1.0 file |
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
| Block template numbers | the stone template's hardness/resistance/sounds, for all three computers | upstream's `ComputerBlock` properties (Java, not read) |
| The three computers' terminals | one terminal and one ink for all three: the advanced computer's colour palette and the command computer's `commands.exec` are not modelled | the advanced terminal prints in colour and a command computer runs commands from Lua |
| The command computer's recipe | gold ingots around `minecraft:command_block` (26.3 has no `c:` tags) | the same shape with the `c:ingots/gold` tag |
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
- **Block behaviour — the three computers are DONE (slice 2, 0.2.2).** What is
  left of this bullet: the jar's **other thirteen block types** (16 blockstates
  hold three computers; the rest are `disk_drive`, `lectern`,
  `monitor_normal|advanced`, `printer`, `redstone_relay`, `speaker`,
  `turtle_normal|advanced`, `cable`, `wired_modem_full`,
  `wireless_modem_normal|advanced`); the **blinking animation** (the state is
  carried and proven to be accepted/refused, but nothing sets it); the
  computer's **GUI/menu** (the pin does have
  `BlockEntityPart::screen`/`screen_draw`, and a *content menu type* is the
  route — a block entity's own menu opens with zero slots in single player);
  the terminal's cursor, colours and `term.setTextColour`; the computer's real
  loot conditions; `computercraft:*` component types (a computer keeps its ID).
- **A `BlockState`'s property read**: the pin reads a *position's* property and
  asks a `BlockMetadata` for a state's; a hook is given neither for a `BlockState`
  it was handed. `screen::facing_of` parses the state string instead. Issue
  shape: *"Read a `BlockState`'s property in a hook without a world lookup
  (`native_block_state_property`, which the retired API had)."*
- **Placement — DONE in slice 2.** A computer is now placed **by hand**
  (`ctl place`) from two yaws and its facing read back (north at yaw 0, east at
  yaw 90), which is the `state_for_placement` block moment in game.
- **The in-game legs**: `tools/e2e.sh` now runs on a launcher-built game and is
  green (`../_shared/evidence/computercraft-e2e/`: `PASS` + `EXIT:0`, control
  `EXIT:1`, `SELFTEST=1` `EXIT:1`, `--strict` 3/3). `tools/visual.sh` (the
  screenshot leg) is still written for the `.hnmod` build and was **not**
  re-run: the two screenshots in the evidence folder are from 0.1.0.

## Review fixes (2026-11-18) — registry review #1166

| Finding | Fix | Test / proof |
|---|---|---|
| `src/lib.rs:252` — the four level-0 usages read, reprogram or power-cycle any loaded computer by coordinates | all four are registered with `Command::op(usage, 2)` through `commands::usages()`, so they are gamemaster tools as `/data` is; the handlers keep their own block guard | `commands::tests::every_usage_acts_on_a_computer_by_coordinates_and_is_operator_only`; the e2e makes its player an operator through the console |
| `src/runtime.rs:1198` — `..` allocates without a byte or memory budget | `MAX_STRING_BYTES` (64 KiB) bounds every built string and the joined `print` line; the interpreter stops with `string is too long …` before the step cap can be reached | `runtime::tests::a_program_cannot_grow_one_string_without_a_byte_budget` (watched failing first) |
| the four carried textures are CCPL, whose section 7 forbids reuse "in other games" | the textures are now **generated** by `tools/gen_textures.py` from its documented palette; `--check` re-renders and compares byte for byte, and `LICENSE-NOTICE.md` records that no upstream pixel is carried | `python3 tools/gen_textures.py --check` exit 0 |
| `authors` credited only the port | the shipped manifest credits Daniel Ratcliffe, Aaron Mills, SquidDev and AdLf0208; the registry entry's authors/description are the orchestrator's (entry-level) | `unzip -p computercraft.hmod horizon-mod.toml` |

The two licence bullets are the reviewer's entry-level items: the **entry** licence
(`CCPL-1.0.0`) and the missing `icons/computercraft.png` are the orchestrator's, and the
package's own `authors` were already upstream-complete. The asset carry was a package
problem and is fixed above.

**Found while proving the leg:** the scratch e2e line that made the harness player an
operator was removed — this pin has no `/op @a` (the command's argument is a game profile
name, and the single-player owner is already level 4 as the level-2 usages prove). **Ladder:**
51 tests, clippy `-D warnings` 0, `fmt --check` clean, two builds byte-identical, `check` and
`fits` exit 0; `launcher-game.sh run computercraft` → `PASS` + `E2E EXIT:0`, the same command
with `GAME=$CONTROL_GAME` → `FAIL: the package did not provide computercraft:computer_normal`
+ `EXIT:1` (`../_shared/evidence/computercraft-e2e/`).

Package 0.2.1: **39,283 B**, SHA-256 `a92fc5ff2338dd3530049360b84b41f68eec37b40fbd3333a29b0058b3ecf4ff`;
a second build is byte-identical (`cmp`), `check` and `fits` exit 0.

## Slice 2 (0.2.2) — the computer block family

**The gap line it comes from** (`## Gaps`, "Block behaviour"): *"`computer_advanced`,
`computer_command`, the other 15 blockstates; the blinking animation; the computer's GUI/menu; …"*.
The jar ships **16 blockstates for 16 blocks**; three of them are computers, and slice 1 registered
one. This slice is the whole computer family: the two missing computers as full members.

**Why it fits the tier and needs no Lua runtime.** The block entity, the tick, the terminal, the
renderer and the four commands already existed; the family asks *which blocks carry them*. The
Lua-VM gaps (the ROM, the language, the peripherals) are untouched, and nothing here invents a
substitute interpreter. `family.rs` is the one new pure module; `commands::screen_json` gained the
block it read (it hardcoded `computer_normal`, which would have answered for a computer that is
not there).

**What it adds** (`data/` and `assets/` per block, all four files of the jar's own shape):

| Area | 0.2.1 | 0.2.2 |
|---|---|---|
| Blocks | `computer_normal` | + `computer_advanced`, `computer_command` (same 12-state blockstate: `facing` × `state`) |
| Block entity | one type over `computer_normal` | one type over `family::PATHS` (all three) |
| Textures | 4, generated | **10**, generated — each computer's own case palette (advanced gold, command purple) |
| Recipes | stone + redstone + pane | + gold + redstone + pane, and gold + **command block** + pane |
| Loot | self-drop | three self-drop tables (same 26.3 shape) |
| Readout | always named `computer_normal` | names the block it read (`commands::screen_json(block, …)`) |
| Commands | guard `block.id == BLOCK` | guard `family::is_computer` — the commands act on any of the three |

**Red first (verbatim, `../_shared/evidence/computercraft/red-0.2.2-family-readout.log`).** The two
new pure rules were written with `todo!()` bodies, then:

```
---- commands::tests::the_readout_names_the_block_it_read stdout ----
thread 'commands::tests::the_readout_names_the_block_it_read' (1868575) panicked at src/commands.rs:111:5:
not yet implemented: the readout names the block it read
---- family::tests::another_block_is_not_a_member stdout ----
thread 'family::tests::another_block_is_not_a_member' (1868577) panicked at src/family.rs:27:5:
not yet implemented: the family predicate
---- family::tests::the_two_spellings_line_up stdout ----
thread 'family::tests::the_two_spellings_line_up' (1868579) panicked at src/family.rs:33:5:
not yet implemented: the family predicate
test result: FAILED. 47 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Green: **56 tests** (31 `runtime` byte-identical, 12 `commands`, 7 `screen`, 6 `family`), clippy
`-D warnings` 0, `fmt --check` clean.

**Trap checked before the content was written (the `constructionstick` class):** a carried recipe
whose 3×3 grid resolves to a **vanilla 26.3** recipe is shadowed and uncraftable. Every shaped
vanilla recipe in the prepared 26.3 `data/` was resolved to its item grid and compared with all
three computer grids — **0 collisions** (3782 recipe files). The legs then craft all three anyway.

**In game** (`../_shared/evidence/computercraft-e2e/e2e-launcher.log`, `--offscreen`, The End arena
at x=200 on a stone floor, one SDK package at `0a875da`):

```
  placed computercraft:computer_normal: computercraft:computer_normal[facing=south,state=on]
  placed computercraft:computer_advanced: computercraft:computer_advanced[facing=south,state=on]
  placed computercraft:computer_command: computercraft:computer_command[facing=south,state=on]
  state=blinking accepted: Changed the block at 209, 101, 0
  state=dancing refused: ...uth,state=dancing]<--[HERE] | Block computercraft:computer_normal does not accept 'dancing' for state property
  normal boot terminal: CraftOS 1.9|no ROM: see|/computercraft
  advanced boot terminal: CraftOS 1.9|no ROM: see|/computercraft
  command boot terminal: CraftOS 1.9|no ROM: see|/computercraft
  advanced after its own program: 4
  normal unchanged by the advanced load: CraftOS 1.9|no ROM: see|/computercraft
  command unchanged by the advanced load: CraftOS 1.9|no ROM: see|/computercraft
  readout at 200,0 names computercraft:computer_normal
  readout at 203,0 names computercraft:computer_advanced
  readout at 206,0 names computercraft:computer_command
  normal -> {"count": 1, "item": "computercraft:computer_normal", "slot": 200}
  advanced -> {"count": 1, "item": "computercraft:computer_advanced", "slot": 200}
  command -> {"count": 1, "item": "computercraft:computer_command", "slot": 200}
  the wrong pattern with stone in place of gold made nothing
  computercraft:computer_normal: broke; 0 on the ground, 1 picked up
  computercraft:computer_advanced: broke; 0 on the ground, 1 picked up
  computercraft:computer_command: broke; 0 on the ground, 1 picked up
  a stone block answers: No computer at 209,101,0. | Changed the block at 209, 101, 0
  an unregistered id is refused
  computercraft:computer_advanced hand-placed facing north
  computercraft:computer_command hand-placed facing east
PASS: … (+ EXIT:0)
```

The last two lines close the old "Placement" gap: `/setblock` never runs a block moment, so the
item was **placed by hand** (`ctl place`) from two different yaws and the facing read back —
north from yaw 0, east from yaw 90, the two rows of `runtime::facing_for_placement`'s own table. That
is also what exercises the changed `state_for_placement` guard (it now accepts all three blocks).
The two negative legs: `GAME=$CONTROL_GAME` → `FAIL: the package did not provide
computercraft:computer_normal` + `EXIT:1`; `SELFTEST=1` → `FAIL: selftest advanced terminal …` +
`EXIT:1`. `check-e2e-logs.py computercraft-e2e --strict` → `logs=3 ok=3`.

**Coverage of the jar's computer family**

| family | state | how |
|---|---|---|
| `computer_normal` block + item + blockstate (12 states) | **tested** | placed, state read back, `blinking` accepted / `dancing` refused, hand-placed facing |
| `computer_advanced` block + item + blockstate | **tested** | same, plus its own terminal and its own recipe |
| `computer_command` block + item + blockstate | **tested** | same, plus power-off/power-on |
| the three recipes | **tested** | crafted in a real table, output id + count, ingredients consumed, a wrong ingredient makes nothing |
| the three loot tables | **tested** | each block mined in survival, exactly one dropped |
| the terminal (boot, `print`, loop, concat) | **tested** | `/computercraft screen` parsed with `jq` |
| the advanced computer's **colour terminal** | **not ported** | needs the Lua `term` palette (the VM gap) |
| the command computer's **command API** (`commands.exec`) | **not ported** | planned; needs the VM and an op-permission bridge |
| the `blinking` **animation** (blink phase) | **not ported** | the state exists (accept/refuse proven) but the code never sets it: upstream blinks when idle-but-busy, which the subset scheduler does not model |
| the other 13 block types (monitor, disk drive, turtle, cable, modem, printer, speaker, lectern, relay) | **not ported** | each is its own family; see `## Gaps` |
| the GUI/menu | **not ported** | `BlockEntityPart::screen` exists at the pin, but a block entity's own menu opens with zero slots in single player (`HANDOVER` §9d.21); a content menu type would be the route |

**Delivered package 0.2.2:** **47,938 B**, SHA-256
`e364023e7be61bcc142f79ea7084aaa7b07f935ebb2c118a9fe517e764304c3d`, 44 entries, no packed
`Cargo.lock`; two builds `cmp`-identical.

## Verification (evidence in `../_shared/evidence/computercraft/`)

All at the pin `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` / mixins `e59da909…`, package 0.2.2
(**47,938 B**, sha256 `e364023e…4c3d`), every log stamped with that sha and size.

| Step | Command | Result |
|---|---|---|
| Format | `cargo fmt --check` | **exit 0** (`sdk-fmt.log`) |
| Lints | `cargo clippy --all-targets --offline -- -D warnings` | **exit 0**, 0 warnings (`sdk-clippy.log`) |
| Unit tests | `cargo test --offline` | **56 passed; 0 failed** (`sdk-test.log`): 31 `runtime` (`sha256 e11eebb6…a017`, the 0.1.0 file plus 0.2.1's byte budget) + 12 `commands` + 7 `screen` + 6 `family`; TDD: the 6 new tests (and the 4 readout tests the signature change touched) failed with `not yet implemented` first (`red-0.2.2-family-readout.log`) |
| Package | `cargo horizon-mod build` | **exit 0**, `SDK 0a875da … Built ./target/horizon-mod/computercraft.hmod`; **two builds `cmp`-identical** |
| Registry check | `cargo horizon-mod check computercraft.hmod` | **exit 0**: `computercraft 0.2.2 (native-mod, format native-1/0a875da91…/e59da909…, for 26.3)`, 47 938 bytes, sha256 `e364023e…4c3d`, unsigned (`sdk-check.log`) |
| Fits (pin) | `cargo horizon-mod fits computercraft.hmod --build 0a875da` | **exit 0**: `CC: Tweaked: 0a875da  made for` (`sdk-fits.log`) — a tautology, kept as the record |
| Fits (other build) | `cargo horizon-mod fits computercraft.hmod --build 0b14f8a` | **exit 0**: `CC: Tweaked: 0b14f8a  fits` (`sdk-fits-other.log`) — the real gate |
| Package verifier | `python3 ports/tools/verify-sdk-package.py computercraft` | **1/1**: package, `runtime.rs` byte-identical in the archive, no packed `Cargo.lock` (44 entries), tests, clippy, check, fits, skills gate |
| In game | `ports/tools/launcher-game.sh build computercraft` + a leased `tools/e2e.sh` | **PASS + EXIT:0** (`computercraft-e2e/e2e-launcher.log`); control `GAME=$CONTROL_GAME` **FAIL + EXIT:1**; `SELFTEST=1` **FAIL + EXIT:1**; `check-e2e-logs.py computercraft-e2e --strict` → **logs=3 ok=3** |

Retired-format evidence (kept, still valid for the `.hnmod` build it describes;
the old crate's own `unit-tests.log` was replaced by this migration's run):
`ladder-check.log`, `ladder-test.log`, `e2e-default.log`, `e2e-embedded.log`,
`e2e-selftest.log`, `visual.log` and the two screenshots, plus the whole old
tree in `retired-hnmod/` (see `../_shared/evidence/computercraft/README.md`).

Deliverable: **`computercraft.hmod`** (47,938 B, 44 entries, SHA-256
`e364023e7be61bcc142f79ea7084aaa7b07f935ebb2c118a9fe517e764304c3d`) holding
`horizon-mod.toml`, `Cargo.toml`, `LICENSE`, `LICENSE-NOTICE.md`, `README.md`,
`uses.json`, `assets/computercraft/**` (23 files, 10 of them generated textures),
`data/computercraft/**` (10 files) and
`src/{lib,family,runtime,commands,screen}.rs`. No `Cargo.lock` is packed.

### Changelog for the resubmission (supersedes #788)

> Supersedes #788, closed because Horizon retired `.hnmod` (#17992), and 0.2.1
> (PR #1166). Changes in 0.2.2: the jar's other two computers
> (`computer_advanced`, `computer_command`) as full members — blocks, 12-state
> blockstates, generated textures and models, item models, recipes, loot tables,
> creative-tab entries — over one block entity type covering the family; the
> `/computercraft screen` readout now names the block it read; the family guard
> accepts all three so the commands act on any computer. `runtime.rs` and its 31
> tests are unchanged. Now pinned to `0a875da` / `e59da90`.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port
