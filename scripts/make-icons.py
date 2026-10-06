#!/usr/bin/env python3
"""Draws the app's source images: three rising bars.

  src-tauri/icons/tray.png            44px, black on clear; macOS tints it for the menu bar
  src-tauri/icons/tray-attention.png  the same with a badge, for a connection needing attention
  src-tauri/icons/tray-broken.png     the bars hollow, for a connection that is not working
  app-icon.png                        1024px, white bars on a dark rounded square

Run `npm run tauri icon app-icon.png` afterwards for the sizes a bundle needs.
"""
import struct
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


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


def in_bars(u, v):
    """Whether the point, in 0..1 across the drawing, falls on one of three bars."""
    for left, top in ((0.14, 0.58), (0.41, 0.36), (0.68, 0.12)):
        if left <= u < left + 0.18 and top <= v < 0.88:
            return True
    return False


def tray(x, y):
    return (0, 0, 0, 255) if in_bars((x + 0.5) / 44, (y + 0.5) / 44) else (0, 0, 0, 0)


def tray_attention(x, y):
    """The bars, and a badge in the top left corner where the short bar leaves room."""
    u, v = (x + 0.5) / 44, (y + 0.5) / 44
    if (u - 0.23) ** 2 + (v - 0.24) ** 2 < 0.13**2:
        return (0, 0, 0, 255)
    return tray(x, y)


def tray_broken(x, y):
    """The bars as outlines only."""
    u, v = (x + 0.5) / 44, (y + 0.5) / 44
    if not in_bars(u, v):
        return (0, 0, 0, 0)
    edge = 2.2 / 44
    inside = all(in_bars(u + du, v + dv) for du in (-edge, 0, edge) for dv in (-edge, 0, edge))
    return (0, 0, 0, 0) if inside else (0, 0, 0, 255)


def app(x, y):
    size, inset, radius = 1024, 100, 185
    edge = size - inset
    if not (inset <= x < edge and inset <= y < edge):
        return (0, 0, 0, 0)
    # Round the corners.
    cx = min(max(x, inset + radius), edge - radius)
    cy = min(max(y, inset + radius), edge - radius)
    if (x - cx) ** 2 + (y - cy) ** 2 > radius**2:
        return (0, 0, 0, 0)
    u = (x - inset - 150) / (edge - inset - 300)
    v = (y - inset - 150) / (edge - inset - 300)
    return (255, 255, 255, 255) if 0 <= u < 1 and 0 <= v < 1 and in_bars(u, v) else (24, 28, 38, 255)


(ROOT / "src-tauri/icons").mkdir(parents=True, exist_ok=True)
(ROOT / "src-tauri/icons/tray.png").write_bytes(png(44, tray))
(ROOT / "src-tauri/icons/tray-attention.png").write_bytes(png(44, tray_attention))
(ROOT / "src-tauri/icons/tray-broken.png").write_bytes(png(44, tray_broken))
(ROOT / "app-icon.png").write_bytes(png(1024, app))
