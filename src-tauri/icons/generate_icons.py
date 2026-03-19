#!/usr/bin/env python3
"""
Vori icon generator — Tauri 2 app icons
Uses supersampling (4x) for clean antialiasing on all sizes.
"""

import math
from pathlib import Path
from PIL import Image, ImageDraw, ImageFilter

ICONS_DIR = Path(__file__).parent


# ---------------------------------------------------------------------------
# Core drawing helpers
# ---------------------------------------------------------------------------

def make_rounded_rect_mask(size: int, radius: int) -> Image.Image:
    """Return an RGBA image that is a white filled rounded rectangle."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    draw.rounded_rectangle([0, 0, size - 1, size - 1], radius=radius, fill=(255, 255, 255, 255))
    return img


def radial_gradient(size: int, center_color: tuple, edge_color: tuple) -> Image.Image:
    """Paint a circular radial gradient onto an RGB canvas."""
    img = Image.new("RGB", (size, size), edge_color)
    cx = cy = size / 2
    max_r = math.hypot(cx, cy)

    pixels = img.load()
    for y in range(size):
        for x in range(size):
            d = math.hypot(x - cx, y - cy) / max_r          # 0 at centre, ~1 at corner
            d = min(d, 1.0)
            r = int(center_color[0] * (1 - d) + edge_color[0] * d)
            g = int(center_color[1] * (1 - d) + edge_color[1] * d)
            b = int(center_color[2] * (1 - d) + edge_color[2] * d)
            pixels[x, y] = (r, g, b)
    return img


def draw_v_shape(draw: ImageDraw.ImageDraw, size: int, color: tuple, stroke_w: int) -> None:
    """
    Draw a bold geometric V using two line-cap rounded polygons.
    The V occupies roughly 58% of the canvas, centred.
    """
    margin = size * 0.18          # outer padding
    cx = size / 2

    # Top-left and top-right anchor points
    x_left  = margin
    x_right = size - margin
    y_top   = size * 0.22

    # Bottom tip of the V
    x_tip = cx
    y_tip = size * 0.76

    # Draw with a thick line — use rounded joints by drawing overlapping circles
    # We approximate a thick stroke by drawing filled polygons (trapezoids) for
    # each arm, then capping with circles at the endpoints.

    half = stroke_w / 2

    def arm_polygon(ax, ay, bx, by):
        """Return a filled polygon for a thick line from (ax,ay) to (bx,by)."""
        dx = bx - ax
        dy = by - ay
        length = math.hypot(dx, dy)
        # Unit perpendicular
        nx = -dy / length * half
        ny =  dx / length * half
        return [
            (ax + nx, ay + ny),
            (ax - nx, ay - ny),
            (bx - nx, by - ny),
            (bx + nx, by + ny),
        ]

    # Left arm: top-left → tip
    draw.polygon(arm_polygon(x_left, y_top, x_tip, y_tip), fill=color)
    # Right arm: top-right → tip
    draw.polygon(arm_polygon(x_right, y_top, x_tip, y_tip), fill=color)

    # Round caps — circles at each endpoint
    for (ex, ey) in [(x_left, y_top), (x_right, y_top), (x_tip, y_tip)]:
        draw.ellipse(
            [ex - half, ey - half, ex + half, ey + half],
            fill=color,
        )


def draw_teal_glow(img: Image.Image, size: int, cx: float, tip_y: float) -> Image.Image:
    """Add a subtle teal blur glow behind the V symbol."""
    glow_layer = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    gd = ImageDraw.Draw(glow_layer)

    # Soft oval below the V tip
    gw = size * 0.45
    gh = size * 0.28
    gx = cx - gw / 2
    gy = tip_y - gh * 0.4

    gd.ellipse([gx, gy, gx + gw, gy + gh], fill=(0, 200, 150, 80))
    glow_layer = glow_layer.filter(ImageFilter.GaussianBlur(radius=size * 0.12))
    img = Image.alpha_composite(img.convert("RGBA"), glow_layer)
    return img


# ---------------------------------------------------------------------------
# Master icon builder (renders at `render_size`, output as `out_size`)
# ---------------------------------------------------------------------------

def build_icon(out_size: int, supersample: int = 4) -> Image.Image:
    s = out_size * supersample          # working resolution

    # 1 — background: radial gradient dark navy
    center_col = (30, 41, 59)           # #1e293b
    edge_col   = (17, 24, 39)           # #111827
    bg = radial_gradient(s, center_col, edge_col)
    icon = bg.convert("RGBA")

    # 2 — draw the V symbol with teal glow underlay
    stroke = int(s * 0.115)             # thickness of each arm
    tip_y  = s * 0.76

    icon = draw_teal_glow(icon, s, s / 2, tip_y)

    draw = ImageDraw.Draw(icon)

    # Subtle teal drop-shadow for the V (offset slightly down)
    shadow_offset = int(s * 0.018)
    draw_v_shape(draw, s, (0, 150, 110, 160), stroke + shadow_offset)

    # Main white V
    draw_v_shape(draw, s, (229, 249, 243, 255), stroke)  # #E5F9F3

    # 3 — apply rounded-rectangle mask (clip to shape)
    radius = int(s * 0.22)
    mask = make_rounded_rect_mask(s, radius)
    icon.putalpha(mask.split()[3])

    # 4 — downsample with high-quality Lanczos
    icon = icon.resize((out_size, out_size), Image.LANCZOS)
    return icon


def build_tray_icon(out_size: int = 22, supersample: int = 8) -> Image.Image:
    """White V on fully transparent background."""
    s = out_size * supersample

    img = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    stroke = int(s * 0.13)
    draw_v_shape(draw, s, (255, 255, 255, 255), stroke)

    return img.resize((out_size, out_size), Image.LANCZOS)


# ---------------------------------------------------------------------------
# Generate all required files
# ---------------------------------------------------------------------------

def main():
    specs = [
        ("icon.png",        1024, 4),
        ("32x32.png",         32, 8),
        ("128x128.png",      128, 6),
        ("128x128@2x.png",   256, 5),
    ]

    for filename, size, ss in specs:
        out_path = ICONS_DIR / filename
        icon = build_icon(size, supersample=ss)
        icon.save(str(out_path), "PNG", optimize=True)
        print(f"  {filename:20s}  {size}x{size}  →  {out_path.stat().st_size // 1024} KB")

    # Tray icon
    tray = build_tray_icon(22, supersample=8)
    tray_path = ICONS_DIR / "tray_icon.png"
    tray.save(str(tray_path), "PNG", optimize=True)
    print(f"  {'tray_icon.png':20s}  22x22   →  {tray_path.stat().st_size} B")


if __name__ == "__main__":
    print("Generating Vori icons …")
    main()
    print("Done.")
