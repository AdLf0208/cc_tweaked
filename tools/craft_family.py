#!/usr/bin/env python3
"""Craft the three computer recipes in a real crafting table, and prove a wrong one makes nothing.

Called by `tools/e2e.sh` with the open game's control socket. The harness opens a crafting table as
a screen whose 3x3 grid is slots 100..108 (row-major) and whose result is 200; the player's own
slots keep their inventory numbers (hotbar 0..8).

What it proves, in this order:

1. the jar's `computer_normal` recipe (`###/#R#/#G#` over stone) makes exactly one
   `computercraft:computer_normal`, and its seven stone, one redstone and one glass pane are
   consumed;
2. the jar's `computer_advanced` recipe (the same shape over gold ingots) makes exactly one
   `computercraft:computer_advanced`, and its ingredients are consumed;
3. the jar's `computer_command` recipe (gold ingots around a command block) makes exactly one
   `computercraft:computer_command`, and its ingredients are consumed;
4. the *control*: the advanced pattern with one gold ingot replaced by stone leaves the result
   empty and consumes nothing, so a recipe matches the pattern it declares and nothing else.

Exit 1 with the observed result on the first mismatch.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys

# The harness's crafting screen: the table's grid and its result.
GRID = list(range(100, 109))
RESULT = 200
HOTBAR = 0

# The jar's three recipes, as 3x3 layouts of item ids (`None` = an empty cell). The patterns are
# `###/#R#/#G#` with `#` the case material and the two middle cells the wire and the pane; the
# command computer puts a command block where the other two put redstone.
STONE = "minecraft:stone"
GOLD = "minecraft:gold_ingot"
REDSTONE = "minecraft:redstone"
PANE = "minecraft:glass_pane"
COMMAND_BLOCK = "minecraft:command_block"

NORMAL = [
    [STONE, STONE, STONE],
    [STONE, REDSTONE, STONE],
    [STONE, PANE, STONE],
]
ADVANCED = [
    [GOLD, GOLD, GOLD],
    [GOLD, REDSTONE, GOLD],
    [GOLD, PANE, GOLD],
]
COMMAND = [
    [GOLD, GOLD, GOLD],
    [GOLD, COMMAND_BLOCK, GOLD],
    [GOLD, PANE, GOLD],
]
# The control: the advanced pattern with the top-left ingot replaced by stone.
WRONG = [
    [STONE, GOLD, GOLD],
    [GOLD, REDSTONE, GOLD],
    [GOLD, PANE, GOLD],
]

SELFTEST = os.environ.get("SELFTEST") == "1"


def cells(layout: list[list[str | None]]) -> dict[int, str]:
    """The grid slots of one 3x3 layout, row-major."""
    out = {}
    for row in range(3):
        for column in range(3):
            item = layout[row][column]
            if item is not None:
                out[GRID[row * 3 + column]] = item
    return out


def main() -> int:
    sock, ctl, x, y, z = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5]

    def run(*args: str) -> dict:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def wait(ticks: int = 4) -> None:
        run("wait", str(ticks))

    def slots() -> list[dict]:
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"no menu is open: {screen}")
        return screen["slots"]

    def count_of(item: str) -> int:
        return run("inventory")["counts"].get(item, 0)

    def find(item: str) -> int:
        for slot in slots():
            if slot.get("item") == item:
                return slot["slot"]
        raise SystemExit(f"missing ingredient {item} (inventory: {run('inventory')['counts']})")

    def result() -> dict | None:
        found = [s for s in slots() if s["slot"] == RESULT and s.get("item")]
        return found[0] if found else None

    def grid_items() -> dict[int, str]:
        return {s["slot"]: s["item"] for s in slots() if s["slot"] in GRID and s.get("item")}

    def place(item: str, targets: list[int]) -> None:
        """Pick the item up, put one in each target, put the rest back."""
        slot = find(item)
        run("click-slot", str(slot))
        wait(1)
        for target in targets:
            run("click-slot", str(target), "right")
            wait(1)
        run("click-slot", str(slot))
        wait(2)

    def setup(ingredients: dict[str, int], cell: dict[int, str], clear: bool = True) -> None:
        """A fresh table, exactly `ingredients` in the inventory, and `cell` in the grid."""
        try:
            run("close")
        except SystemExit:
            pass
        wait(5)
        run("setblock", str(x), str(y), str(z), "minecraft:air")
        run("setblock", str(x), str(y), str(z), "minecraft:crafting_table")
        wait(2)
        run("command", f"tp @s {float(x) - 0.5} {float(y)} {float(z) + 0.5}")
        wait(3)
        if clear:
            run("command", "clear @s")
        for item, count in ingredients.items():
            run("give", item, str(count))
        wait(3)
        run("interact", str(x), str(y), str(z))
        wait(5)
        by_item: dict[str, list[int]] = {}
        for slot, item in sorted(cell.items()):
            by_item.setdefault(item, []).append(slot)
        for item, targets in by_item.items():
            place(item, targets)
        # Every cell the layout names holds what it names, or the grid is not the recipe's.
        got = grid_items()
        want = dict(sorted(cell.items()))
        if got != want:
            raise SystemExit(f"FAIL: the grid is {got}, expected {want}")

    def take() -> None:
        run("click-slot", str(RESULT))
        wait(2)
        run("click-slot", str(HOTBAR))
        wait(2)
        run("close")
        wait(3)

    def craft(label: str, layout: list[list[str | None]], item_id: str) -> None:
        cell = cells(layout)
        ingredients: dict[str, int] = {}
        for item in cell.values():
            ingredients[item] = ingredients.get(item, 0) + 1
        setup(ingredients, cell)
        got = result()
        print(f"  {label} -> {json.dumps(got)}")
        if not got:
            raise SystemExit(f"FAIL: {label}: the table made nothing")
        if got.get("item") != item_id or got.get("count") != 1:
            raise SystemExit(f"FAIL: {label}: expected 1 {item_id}, got {got}")
        take()
        for item, count in ingredients.items():
            left = count_of(item)
            if left != 0:
                raise SystemExit(f"FAIL: {label}: {left} of {item} left after the craft")
        if count_of(item_id) != 1:
            raise SystemExit(f"FAIL: {label}: the crafted {item_id} is not in the inventory")
        print(f"  {label}: 1 {item_id}, every ingredient consumed")

    def wrong_makes_nothing() -> None:
        cell = cells(WRONG)
        ingredients: dict[str, int] = {}
        for item in cell.values():
            ingredients[item] = ingredients.get(item, 0) + 1
        setup(ingredients, cell)
        got = result()
        print(f"  the advanced pattern with stone in place of gold -> {json.dumps(got)}")
        if got is not None:
            raise SystemExit(f"FAIL: the wrong pattern made {got}")
        run("close")
        wait(3)
        print("  the wrong pattern made nothing")

    craft("normal", NORMAL, "computercraft:computer_normal")
    craft("advanced", ADVANCED, "computercraft:computer_advanced")
    craft("command", COMMAND, "computercraft:computer_command")
    if SELFTEST:
        # The flipped expectation: the wrong pattern must NOT make nothing. The run has to fail
        # here, which is what proves the assertion above can.
        cell = cells(ADVANCED)
        ingredients: dict[str, int] = {}
        for item in cell.values():
            ingredients[item] = ingredients.get(item, 0) + 1
        setup(ingredients, cell)
        if result() is not None:
            raise SystemExit("FAIL: selftest: the correct pattern made a result, the flipped "
                             "expectation fails as it must")
        print("  selftest: the flipped expectation failed as it must")
        return 1
    wrong_makes_nothing()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
