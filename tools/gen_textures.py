#!/usr/bin/env python3
"""Generate the port's four block textures — its own art, from a named palette.

CC: Tweaked's textures are ComputerCraft Public License 1.0.0, and section 7 of
that licence forbids using the mod's assets "in other games", which is the
reviewer's finding on the 0.2.0 package. Not one upstream pixel is read or
copied here: only the block's **shape** (a 16x16 front, side and top of a case
with a screen) and its **ids** (`computer_normal_front`, `_side`, `_top`,
`computer_on`) are followed, and every pixel comes from the palette below. The
precedent is MineColonies' regenerated art (`ARCHITECTURE.md` §8).

The output is deterministic: the PNGs use stored (uncompressed) deflate blocks,
so the same palette always gives the same bytes, and `--check` re-renders into
memory and compares byte for byte.

    python3 tools/gen_textures.py            # write assets/computercraft/textures/block/
    python3 tools/gen_textures.py --check    # exit 1 if a file differs from what it would write

Palette (one named colour per part; every texture below is composed from these only):

    CASE_LIGHT  #c6c6c6   the case's lit top edge
    CASE        #9a9a9a   the case itself
    CASE_DARK   #6f6f6f   its shadowed edge and the vents
    ADVANCED_*  gold      the advanced computer's case (its own palette, not upstream's)
    COMMAND_*   purple    the command computer's case (its own palette, not upstream's)
    BEZEL       #3a3f44   the screen's bezel
    SCREEN_OFF  #101416   the powered-off screen
    SCREEN_ON   #2e6f5e   the lit screen
    SCREEN_GLOW #58c39a   the lit screen's scanlines

The case colour is the only thing that tells the three computers apart in the jar's own art, so it
is the one thing each palette changes: the family is drawn from the same shapes with three cases.
"""

from __future__ import annotations

import argparse
from pathlib import Path
import struct
import sys
import zlib

ROOT = Path(__file__).resolve().parents[1]
TEXTURES = ROOT / "assets" / "computercraft" / "textures" / "block"

PALETTE = {
    "case_light": (0xC6, 0xC6, 0xC6),
    "case": (0x9A, 0x9A, 0x9A),
    "case_dark": (0x6F, 0x6F, 0x6F),
    "advanced_light": (0xF0, 0xDE, 0xA6),
    "advanced": (0xD6, 0xB4, 0x5C),
    "advanced_dark": (0x9A, 0x7B, 0x2C),
    "command_light": (0xC0, 0xB0, 0xDC),
    "command": (0x8A, 0x76, 0xAD),
    "command_dark": (0x59, 0x48, 0x78),
    "bench_light": (0xA8, 0xD8, 0xD0),
    "bench": (0x6F, 0xA9, 0xA0),
    "bench_dark": (0x47, 0x72, 0x6C),
    "bezel": (0x3A, 0x3F, 0x44),
    "screen_off": (0x10, 0x14, 0x16),
    "screen_on": (0x2E, 0x6F, 0x5E),
    "screen_glow": (0x58, 0xC3, 0x9A),
}


def rgba(name: str) -> tuple[int, int, int, int]:
    r, g, b = PALETTE[name]
    return (r, g, b, 255)


def png(width: int, height: int, pixels: list[tuple[int, int, int, int]]) -> bytes:
    """An RGBA PNG with stored deflate blocks: same input, same bytes, on any Python."""
    raw = bytearray()
    for y in range(height):
        raw.append(0)  # filter: none
        for x in range(width):
            raw.extend(pixels[y * width + x])

    def chunk(kind: bytes, payload: bytes) -> bytes:
        return (
            struct.pack(">I", len(payload))
            + kind
            + payload
            + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)
        )

    deflate = bytearray(b"\x78\x01")
    data = bytes(raw)
    for start in range(0, len(data), 0xFFFF):
        block = data[start : start + 0xFFFF]
        final = 1 if start + 0xFFFF >= len(data) else 0
        deflate.append(final)
        deflate.extend(struct.pack("<HH", len(block), len(block) ^ 0xFFFF))
        deflate.extend(block)
    deflate.extend(struct.pack(">I", zlib.adler32(data) & 0xFFFFFFFF))
    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", bytes(deflate)) + chunk(b"IEND", b"")


Case = tuple[str, str, str]

#: The ordinary computer's case: light, body, dark.
CASE: Case = ("case_light", "case", "case_dark")


def case_pixel(x: int, y: int, case: Case = CASE) -> tuple[int, int, int, int]:
    """The case: light along the top and left, dark along the bottom and right."""
    light, body, dark = case
    if x == 0 or y == 0:
        return rgba(light)
    if x == 15 or y == 15:
        return rgba(dark)
    return rgba(body)


def front_texture(lit: bool, case: Case = CASE) -> bytes:
    """A case with a bevelled screen: 2 px of bezel, 11x10 of screen, 1 px of case."""
    pixels = []
    for y in range(16):
        for x in range(16):
            inside_screen = 3 <= x <= 13 and 3 <= y <= 12
            bezel = 2 <= x <= 14 and 2 <= y <= 13
            if inside_screen:
                # Two scanlines on the lit screen, one on the dark one, so the two
                # states read differently at a glance without any upstream pixel.
                if lit and y in (5, 9):
                    pixels.append(rgba("screen_glow"))
                elif not lit and y == 12:
                    pixels.append(rgba("bezel"))
                else:
                    pixels.append(rgba("screen_on" if lit else "screen_off"))
            elif bezel:
                pixels.append(rgba("bezel"))
            else:
                pixels.append(case_pixel(x, y, case))
    return png(16, 16, pixels)


def side_texture(case: Case = CASE) -> bytes:
    """The case with two pairs of vents down the middle."""
    pixels = []
    for y in range(16):
        for x in range(16):
            if 5 <= x <= 10 and (y in (6, 7, 9, 10)):
                pixels.append(rgba("case_dark"))
            else:
                pixels.append(case_pixel(x, y, case))
    return png(16, 16, pixels)


def top_texture(case: Case = CASE) -> bytes:
    """The case with one seam across it and a small drive bay."""
    pixels = []
    for y in range(16):
        for x in range(16):
            if y == 8:
                pixels.append(rgba("case_dark"))
            elif 3 <= x <= 12 and 3 <= y <= 5:
                pixels.append(rgba("bezel"))
            else:
                pixels.append(case_pixel(x, y, case))
    return png(16, 16, pixels)


ADVANCED: Case = ("advanced_light", "advanced", "advanced_dark")
COMMAND: Case = ("command_light", "command", "command_dark")
#: The peripheral probe's case: a colour none of the computers uses, so a screenshot tells which
#: block answered.
BENCH: Case = ("bench_light", "bench", "bench_dark")

TEXTURES_TO_WRITE = {
    "computer_normal_front.png": front_texture(lit=False),
    "computer_on.png": front_texture(lit=True),
    "computer_normal_side.png": side_texture(),
    "computer_normal_top.png": top_texture(),
    "computer_advanced_front.png": front_texture(lit=False, case=ADVANCED),
    "computer_advanced_side.png": side_texture(case=ADVANCED),
    "computer_advanced_top.png": top_texture(case=ADVANCED),
    "computer_command_front.png": front_texture(lit=False, case=COMMAND),
    "computer_command_side.png": side_texture(case=COMMAND),
    "computer_command_top.png": top_texture(case=COMMAND),
    # The peripheral probe (slice 3): a case with vents, in its own colour.
    "peripheral_harness.png": side_texture(case=BENCH),
}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="compare instead of writing")
    args = parser.parse_args()
    failed = False
    for name, data in TEXTURES_TO_WRITE.items():
        path = TEXTURES / name
        if args.check:
            current = path.read_bytes() if path.exists() else b""
            if current != data:
                print(f"{path}: differs from the palette's output", file=sys.stderr)
                failed = True
        else:
            path.write_bytes(data)
    if args.check:
        print(
            f"{len(TEXTURES_TO_WRITE)} textures: "
            + ("byte-identical to the palette's output" if not failed else "STALE"),
            file=sys.stderr,
        )
        return 1 if failed else 0
    print(f"wrote {len(TEXTURES_TO_WRITE)} textures under {TEXTURES}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
