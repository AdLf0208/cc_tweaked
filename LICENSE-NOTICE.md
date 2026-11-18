# Licence notice — CC: Tweaked port

**Ported from:** CC: Tweaked 1.21.1-forge-1.121.0 (`cc-tweaked/CC-Tweaked`),
Minecraft Java 1.21.1 NeoForge. Copyright © Daniel Ratcliffe, Aaron Mills and
SquidDev; the Lua ROM and the mod are under the **ComputerCraft Public License
1.0.0** (`LICENSES/LicenseRef-CCPL.txt` in the upstream repository).

## What was read, and from where

| Item | Source |
|---|---|
| Licence name and URL | `META-INF/neoforge.mods.toml` in the jar: `license="ComputerCraft Public License (https://raw.githubusercontent.com/dan200/ComputerCraft/master/LICENSE)"` |
| Licence text | that URL, fetched 2026-11-11: 98 lines, SHA-256 `f72a6699caccf1c4dc6098b50e647dba5c9d2b9f7e8fd237fd7d79e03be2935f`, stored verbatim as `LICENSE` |
| Per-file licence headers | every bundled Lua file: `SPDX-License-Identifier: LicenseRef-CCPL`; the build files: `MPL-2.0` |
| Assets carried | the four block textures in `content/assets/computercraft/textures/block/` (`computer_normal_front`, `_side`, `_top`, `computer_on`), from the jar, CCPL |
| Code | none: this is a clean-room Rust rewrite written against the observed behaviour (the shipped data, the public wiki and in-game checks). No Java was decompiled, read or translated |

## The obligation this port takes on (CCPL 1.0.0)

The licence **allows redistribution**, including modified forms, on two
conditions that this port meets deliberately:

1. **Complete corresponding source available at no cost.** The port ships its
   complete source in the same tree as the package: `mixins/computercraft/`
   (`runtime.rs`, `support.rs`, one helper per injection, the plan), `content/`
   (the block, models, textures, data), `tools/` (the ladder, packager and e2e)
   and this notice with the full licence text. Anyone who receives
   `computercraft.hnmod` can obtain exactly that source from the same
   repository.
2. **All distributions stay under the CCPL.** This port and any redistribution
   of it stay under the ComputerCraft Public License 1.0.0; it is **not**
   relicensed. Reusing this port's code or assets in another mod is allowed
   only non-commercially, with the reused source remaining CCPL.

The licence also states the software is provided "as is", requires the user to
own a legally acquired copy of Minecraft, and forbids commercial use without
the original author's permission.

## Publishing

Rank **1** (copyleft-style): publishable under the same licence with complete
source. Publication still needs the campaign maintainer's approval
(`ARCHITECTURE.md` §8), and a submission to `horizon-rust/horizon-mods` carries
the reviewer note that `MOD-POLICY.md` names only permissive licences for new
projects, so acceptance is the maintainers' decision.
