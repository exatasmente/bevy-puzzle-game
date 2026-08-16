#!/usr/bin/env python3
"""Regenerates the PWA icons in docs/icons/.

Pure standard library, like tools/make_sounds.py and for the same reason: the
repo owns its assets outright, with nothing to re-source and no licence to
track. A PNG is a zlib stream in a handful of chunks, which is little enough
code to be worth writing rather than taking a dependency for.

The icon is the favicon's four squares on the game's ground — the same mark,
so an installed app and a browser tab are recognisably the same thing.

    python3 tools/make_icons.py
"""

import pathlib
import struct
import zlib

OUT = pathlib.Path(__file__).resolve().parent.parent / "docs" / "icons"

# theme.rs: BACKGROUND, then PRIMARY / SUCCESS / LIME / ACCENT.
GROUND = (0x0E, 0x0C, 0x16)
QUADRANTS = [
    (0xA8, 0x55, 0xF7),
    (0x4A, 0xDE, 0x80),
    (0xA3, 0xC6, 0x14),
    (0xEC, 0x48, 0x99),
]


def chunk(kind: bytes, payload: bytes) -> bytes:
    """One PNG chunk: length, type, payload, CRC of type+payload."""
    return (
        struct.pack(">I", len(payload))
        + kind
        + payload
        + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)
    )


def png(size: int, maskable: bool) -> bytes:
    """An RGB PNG of the mark at `size` square.

    `maskable` leaves a wider margin. Android crops a maskable icon to whatever
    shape the launcher uses, and anything inside the middle 80% is guaranteed to
    survive that crop — squares drawn to the edge would come back with their
    corners shaved off.
    """
    margin = size * (0.22 if maskable else 0.12)
    gap = size * 0.045
    cell = (size - 2 * margin - gap) / 2

    rows = []
    for y in range(size):
        # A filter byte per scanline; 0 means "no filtering", which costs a few
        # bytes and saves needing an encoder.
        row = bytearray(b"\x00")
        for x in range(size):
            colour = GROUND
            for index, quad in enumerate(QUADRANTS):
                left = margin + (cell + gap) * (index % 2)
                top = margin + (cell + gap) * (index // 2)
                if left <= x < left + cell and top <= y < top + cell:
                    colour = quad
                    break
            row += bytes(colour)
        rows.append(bytes(row))

    raw = b"".join(rows)
    header = struct.pack(">IIBBBBB", size, size, 8, 2, 0, 0, 0)

    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)

    # 192 and 512 are what the manifest spec and every install prompt ask for.
    for size in (192, 512):
        (OUT / f"icon-{size}.png").write_bytes(png(size, maskable=False))
    (OUT / "icon-maskable-512.png").write_bytes(png(512, maskable=True))

    for path in sorted(OUT.iterdir()):
        print(f"{path.name}: {path.stat().st_size} bytes")


if __name__ == "__main__":
    main()
