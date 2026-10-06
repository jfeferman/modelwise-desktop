#!/usr/bin/env python3
"""Draws the app's source images from the Modelwise mark: a 3x3 grid of dots
with the diagonal lit and joined (apps/web/src/components/Logo.tsx).

  src-tauri/icons/tray.png            36px, black on clear; macOS tints it for the menu bar
                                      the diagonal lit and joined: working
  src-tauri/icons/tray-attention.png  the diagonal lit but not joined: needs attention
  src-tauri/icons/tray-broken.png     nothing lit: not working
  app-icon.png                        1024px, the mark in the brand orange on a dark rounded square

Run `npm run tauri icon app-icon.png` afterwards for the sizes a bundle needs.
"""
import math
import struct
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The mark's own units: centres at 10, 44 and 78, radius 10, in an 88-unit square.
CENTRES = (10, 44, 78)
RADIUS = 10
DIAGONAL = ((10, 10), (44, 44), (78, 78))
# The web's brand tokens.
ACCENT = (232, 80, 15)
INK = (22, 18, 15)


def png(size, pixel):
    rows = b"".join(b"\x00" + b"".join(bytes(pixel(x, y)) for x in range(size)) for y in range(size))

    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))

    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(rows, 9))
        + chunk(b"IEND", b"")
    )


def bezier(p0, p1, p2, p3, t):
    s = 1 - t
    return tuple(s**3 * a + 3 * s**2 * t * b + 3 * s * t**2 * c + t**3 * d for a, b, c, d in zip(p0, p1, p2, p3))


# The joiner between two lit dots, in a frame along the diagonal from the
# first centre: a bar 3.6 wide, flaring into each dot. Its edge is the
# logo's path, sampled into a table of half-widths by distance along the bar.
FLARE = [bezier((7.53, 6.58), (10.19, 3.54), (14.03, 1.8), (18.07, 1.8), i / 64) for i in range(65)]
SPAN = 34 * math.sqrt(2)


def joiner_half_width(along):
    if along < 7.53 or along > SPAN - 7.53:
        return None
    if along > SPAN / 2:
        along = SPAN - along
    if along >= 18.07:
        return 1.8
    for (x0, y0), (x1, y1) in zip(FLARE, FLARE[1:]):
        if x0 <= along <= x1:
            return y0 + (y1 - y0) * (along - x0) / (x1 - x0)
    return 1.8


def in_joiner(u, v, start):
    """Whether the point is on the joiner that starts at the lit dot `start`."""
    dx, dy = u - start[0], v - start[1]
    along = (dx + dy) / math.sqrt(2)
    across = abs(dx - dy) / math.sqrt(2)
    half = joiner_half_width(along)
    return half is not None and across <= half


def layer(u, v, joined, lit):
    """Which part of the mark the point (in mark units) falls on: 'lit', 'dim' or None."""
    for cx in CENTRES:
        for cy in CENTRES:
            if (u - cx) ** 2 + (v - cy) ** 2 <= RADIUS**2:
                return "lit" if lit and cx == cy else "dim"
    if lit and joined and any(in_joiner(u, v, start) for start in DIAGONAL[:2]):
        return "lit"
    return None


def sampled(size, inset, joined, lit, colours, samples=3):
    """A pixel function drawing the mark, scaled to `size - 2 * inset`, with
    `colours` mapping each layer to an RGBA tuple; antialiased by sampling."""
    scale = 88 / (size - 2 * inset)

    def pixel(x, y):
        total = [0, 0, 0, 0]
        for i in range(samples):
            for j in range(samples):
                u = (x + (i + 0.5) / samples - inset) * scale
                v = (y + (j + 0.5) / samples - inset) * scale
                colour = colours.get(layer(u, v, joined, lit))
                if colour:
                    for k in range(4):
                        total[k] += colour[k]
        n = samples * samples
        return tuple(round(c / n) for c in total)

    return pixel


def tray(joined, lit):
    return sampled(36, 2, joined, lit, {"lit": (0, 0, 0, 255), "dim": (0, 0, 0, 90)})


def app_icon(x, y):
    size, inset, radius = 1024, 100, 185
    edge = size - inset
    if not (inset <= x < edge and inset <= y < edge):
        return (0, 0, 0, 0)
    # Round the corners.
    cx = min(max(x, inset + radius), edge - radius)
    cy = min(max(y, inset + radius), edge - radius)
    if (x - cx) ** 2 + (y - cy) ** 2 > radius**2:
        return (0, 0, 0, 0)
    return app_icon.mark(x, y)


# The dim dots: the web's 16% ink on paper, here 22% white on ink, already blended.
DIM_ON_INK = tuple(round(c + 0.22 * (255 - c)) for c in INK)
app_icon.mark = sampled(
    1024, 280, True, True, {"lit": (*ACCENT, 255), "dim": (*DIM_ON_INK, 255), None: (*INK, 255)}, samples=2
)

(ROOT / "src-tauri/icons").mkdir(parents=True, exist_ok=True)
(ROOT / "src-tauri/icons/tray.png").write_bytes(png(36, tray(joined=True, lit=True)))
(ROOT / "src-tauri/icons/tray-attention.png").write_bytes(png(36, tray(joined=False, lit=True)))
(ROOT / "src-tauri/icons/tray-broken.png").write_bytes(png(36, tray(joined=False, lit=False)))
(ROOT / "app-icon.png").write_bytes(png(1024, app_icon))
