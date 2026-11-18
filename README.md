# CC: Tweaked — a Horizon port

**CC: Tweaked** is a Minecraft mod. Just another ComputerCraft fork. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [cc-tweaked/CC-Tweaked](https://github.com/cc-tweaked/CC-Tweaked) — licence **CCPL-1.0**.

| | |
|---|---|
| Upstream mod | CC: Tweaked ([source](https://github.com/cc-tweaked/CC-Tweaked)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `computercraft.hmod` (install it with the Horizon launcher) |
| Licence | **CCPL-1.0** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

### What this slice implements

- `computercraft:computer_normal` — `facing` × `state` (off/on/blinking) on the
  stone template, upstream's blockstate model chain (`minecraft:block/orientable`
  with the computer's own front/side/top textures, the lit screen texture while
  running), a block entity declared in code, self-drop loot, the upstream
  crafting shape with 26.3 ingredients, and a creative tab.
- A **Lua 5.2 subset** interpreter (`src/runtime.rs`): `print`, numbers, strings,
  `..`, `+ - * / % ^`, comparisons, `and`/`or`/`not`, `#`, `local`, assignment,
  `if`/`elseif`/`else`, `while`, numeric `for`, comments, `return`, blocks and
  scopes — 31 unit tests.
- The terminal is kept in the block entity's saved compound and drawn on the
  block's front face with the font atlas; `/computercraft screen` returns it as
  one line of JSON for machines.

## Not yet ported

- **The ROM**: 105 Lua files (`bios.lua`, 20 `rom/apis`, 23 `rom/modules`, 60
- **Language features**: tables, functions and closures, `break`, `repeat`,
- **Peripherals and the world**: wired/wireless modems and `rednet`, monitors,
- **Block behaviour**: `computer_advanced`, `computer_command`, the other 15
- **A `BlockState`'s property read**: the pin reads a *position's* property and
- **Placement**: the facing rule is unit-tested (`facing_for_placement`) and
- **The in-game legs**: `tools/e2e.sh` and `tools/visual.sh` are rewritten for

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `computercraft.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

CC: Tweaked was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **CCPL-1.0**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/cc-tweaked/CC-Tweaked

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run computercraft` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/computercraft/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
