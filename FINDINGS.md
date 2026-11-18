# CC: Tweaked — jar inventory and behaviour findings

Ported from **CC: Tweaked 1.21.1-forge-1.121.0**
(`cc-tweaked/CC-Tweaked` — the source map row gives
`https://github.com/cc-tweaked/CC-Tweaked/`, licence **CCPL-1.0**), Minecraft
Java 1.21.1 NeoForge. Jar as evidence:
`ports/_shared/licence-cache/cc-tweaked-1.21.1-forge-1.121.0.jar`
(SHA-256 `bf8dd119701560ea60085cf8c1076d32ffc99a1be5f8b99f6f3b174a88a208f8`,
3,209,662 B).

This file is the **measured** inventory, not the pack's description. Two of the
brief's numbers disagree with the artefact and the artefact wins (see
"Corrections to the brief").

## 1. Inventory (`unzip -l`)

| Area | Measured |
|---|---|
| Entries | **2,334** (6,909,369 B uncompressed) |
| Java classes | 1,393 under `dan200/computercraft/` |
| `assets/computercraft/` | 398 files |
| `data/computercraft/` | 477 files: 250 under `lua/`, 88 recipe, 86 advancement, 21 tags, 19 loot_table, 12 `data/computercraft/computercraft/` |
| Blockstates | **16** under `assets/computercraft/blockstates/` (one empty directory entry makes `unzip -l` print 17 lines) |
| Bundled Lua | **105** real `.lua` files: `data/computercraft/lua/bios.lua` + 104 under `lua/rom/` (20 `rom/apis`, 23 `rom/modules`, 60 `rom/programs`, `rom/startup.lua`) |
| Other assets | `assets/minecraft/` 10 overrides, `data/minecraft/` 18 overrides, `resourcepacks/turtle_storage/`, `banner.png`, `pack.png` |

The 16 blockstates are: `computer_normal`, `computer_advanced`, `computer_command`,
`disk_drive`, `lectern`, `monitor_normal`, `monitor_advanced`, `printer`,
`redstone_relay`, `speaker`, `turtle_normal`, `turtle_advanced`, `cable`,
`wired_modem_full`, `wireless_modem_normal`, `wireless_modem_advanced`.

### Corrections to the brief (the artefact wins)

- The brief says "**250 bundled Lua files**"; measured: **105** `.lua` files.
  250 is the file count of `data/computercraft/lua/` *including* the non-Lua
  assets stored there (`fun/advanced/levels/*.dat` level files and the
  `rom/apis/colors` data). Recorded so the gap list is honest about what the
  ROM really is.
- The brief says "**17 blockstates**"; measured **16** (the 17th line is the
  directory's own entry).
- The brief's Step 0 item 5 names the source-map row for `tiab` and
  `ports/_shared/licence-cache/tiab-neoforge-6.5.4.jar`. That is a copy-paste
  error from another brief: this job is `computercraft`, its row and its jar are
  `cc-tweaked-1.21.1-forge-1.121.0.jar` (both read), and the `tiab` files were
  not touched.
- The brief says the jar carries `LICENSES/LicenseRef-CCPL.txt`; the jar
  carries **no** licence file at all. Its `META-INF/neoforge.mods.toml` says
  `license="ComputerCraft Public License
  (https://raw.githubusercontent.com/dan200/ComputerCraft/master/LICENSE)"`,
  its build files are `SPDX-License-Identifier: MPL-2.0` and every bundled Lua
  file is `SPDX-License-Identifier: LicenseRef-CCPL`. The CCPL text in this port
  (`LICENSE`) was fetched from that URL the manifest names (98 lines,
  SHA-256 `f72a6699caccf1c4dc6098b50e647dba5c9d2b9f7e8fd237fd7d79e03be2935f`).

## 2. What a computer actually is (from the mod's own shipped data)

Read: `data/computercraft/lua/bios.lua`, `.../rom/startup.lua`,
`.../rom/apis/term.lua`, `.../rom/apis/fs.lua`, `.../rom/apis/os.lua` (via
`bios.lua`'s definitions), `assets/computercraft/blockstates/computer_normal.json`,
`assets/computercraft/models/block/computer_normal_{on,off,blinking}.json`,
`data/computercraft/recipe/computer_normal.json`,
`data/computercraft/loot_table/blocks/computer_normal.json`,
`assets/computercraft/lang/en_us.json`.

- **The ROM is data, shipped in the jar.** `bios.lua` opens
  `rom/modules/main/cc/expect.lua`, `loadstring`s it and starts the shell. The
  ROM is a Lua **5.2** program: `bit32` is present and aliased into a legacy
  `bit` table (`bios.lua:25-34`), `loadstring` exists, coroutines are the basis
  of every API (`os.pullEventRaw` is `coroutine.yield`), and
  `os.version()` returns `"CraftOS 1.9"` (`bios.lua:38-40`). So a faithful
  computer needs a **Lua 5.2 VM plus coroutines**, not a small expression
  evaluator; this slice records that as the central gap.
- **A computer is a block entity with a terminal.** The block state is
  `facing` × `state` (`off`/`on`/`blinking`) — 12 states — and the model is
  `minecraft:block/orientable` with `front`/`side`/`top` textures; the `on`
  model adds an emissive `cursor` overlay (`computer_on.json`,
  `computercraft:emissive_texture`). The terminal itself is **not** a model:
  upstream draws it as text into the block entity renderer.
- **Drops**: the loot table copies the block entity's components and is gated
  on `computercraft:block_named` / `computercraft:has_id` /
  `computercraft:player_creative` — three custom loot conditions 26.3 does not
  have, so the port drops the block itself (recorded gap: a named/ID'd
  computer keeps its label upstream).
- **Recipe**: shaped `###`/`#R#`/`#G#` with `#` stone, `R` the
  `c:dusts/redstone` tag and `G` the `c:glass_panes` tag. 26.3 ships no `c:`
  tags (campaign finding, `dyenamics`/`farmersdelight`), so the port writes the
  same shape with `minecraft:redstone` and `minecraft:glass_pane`.
- **Language**: `block.computercraft.computer_normal` = "Computer".

## 3. Upstream source analysis

No clone was made. The behaviour this slice needs is in the mod's **shipped
data** (the ROM and the blockstate above), which the jar carries verbatim; the
Java side (a `ComputerBlockEntity` with a `Computer` holding a Lua machine, a
`ServerTerminal`, and packet/frame plumbing) is not something a clean-room
subset can copy anyway, and the campaign forbids reading it. The repo URL is
recorded above for the next slice (turtles/peripherals will need the Java
behaviour: `ServerComputer`, `TurtleAPI`, `WiredModemPeripheral`).

## 4. Measured platform constraint (the ladder's rung A)

`grep -ci lua ports/_shared/pins/horizon-1d63e0a/Cargo.lock` → **0** across
**565** locked packages. A pure-Rust Lua VM (`piccolo`) therefore cannot be
linked: see `notes.md` §"Decision ladder" for the two measurements
(`E_CRATE_DEPENDENCY` from the mixin planner, and the fact that the injected
module compiles inside `horizon-client`, whose manifest the port cannot edit).

## 5. Commands used

```sh
unzip -l  ports/_shared/licence-cache/cc-tweaked-1.21.1-forge-1.121.0.jar | tail -2
unzip -l  … | awk '{print $4}' | grep -c '\.lua$'                    # 105
unzip -l  … | awk '{print $4}' | grep '^assets/computercraft/blockstates/'   # 16 + dir
unzip -p  … data/computercraft/lua/bios.lua | head -40
unzip -p  … assets/computercraft/blockstates/computer_normal.json
unzip -p  … data/computercraft/recipe/computer_normal.json
unzip -p  … data/computercraft/loot_table/blocks/computer_normal.json
unzip -p  … META-INF/neoforge.mods.toml | head -40
sha256sum ports/_shared/licence-cache/cc-tweaked-1.21.1-forge-1.121.0.jar
```
