#!/usr/bin/env python3
"""Regenerate all icon assets from src-tauri/icons/icon.svg.

Outputs:
  - src-tauri/icons/icon.png  (1024×1024 master)
  - src-tauri/icons/32x32.png
  - src-tauri/icons/128x128.png
  - src-tauri/icons/128x128@2x.png  (256×256)
  - src-tauri/icons/Square{30,44,71,89,107,142,150,284,310}x*Logo.png  (Windows Store)
  - src-tauri/icons/StoreLogo.png  (50×50)
  - src-tauri/icons/icon.ico  (multi-size ICO for Windows)
  - src-tauri/icons/icon.icns  (macOS bundle icon, via iconutil)
  - public/nootle-icon.png  (1024×1024 in-app sidebar logo)
  - public/nootle-icon.svg  (synced to source)
  - site/public/nootle-icon.png  (1024×1024 marketing)
  - site/public/nootle-icon.svg  (synced to source)
  - src-tauri/dmg/background.png  (1320×800 @ 144 DPI macOS installer background)

Requires: cairosvg, Pillow. iconutil is needed only when regenerating
src-tauri/icons/icon.icns (macOS only); otherwise it is skipped.
"""

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import cairosvg
from PIL import Image, ImageDraw, ImageFont

REPO = Path(__file__).resolve().parent.parent
SRC_SVG = REPO / "src-tauri/icons/icon.svg"
ICON_DIR = REPO / "src-tauri/icons"
PUBLIC_DIR = REPO / "public"
SITE_PUBLIC_DIR = REPO / "site/public"
DMG_DIR = REPO / "src-tauri/dmg"
DMG_BG_SVG = DMG_DIR / "background.svg"
DMG_FONT_OUTFIT = DMG_DIR / "fonts/Outfit-Variable.ttf"
DMG_FONT_DM_SANS = DMG_DIR / "fonts/DMSans-Variable.ttf"
DMG_BG_WIDTH = 660
DMG_BG_HEIGHT = 400


def render(svg_path: Path, out_path: Path, size: int) -> None:
    cairosvg.svg2png(url=str(svg_path), write_to=str(out_path), output_width=size, output_height=size)


def set_weight(font: ImageFont.FreeTypeFont, weight: int) -> None:
    """Set the Weight axis on a variable font while leaving other axes
    (such as DM Sans's Optical Size) at their defaults. Passing a bare
    list to set_variation_by_axes maps positionally, so DM Sans would
    receive the weight value on its Optical Size axis instead."""
    try:
        axes = font.get_variation_axes()
    except (AttributeError, OSError):
        return
    if not axes:
        return
    values = []
    for axis in axes:
        name = axis.get("name", b"").decode("ascii", "ignore").lower()
        if name == "weight":
            values.append(weight)
        elif "default" in axis:
            values.append(axis["default"])
        else:
            return
    try:
        font.set_variation_by_axes(values)
    except (AttributeError, OSError):
        pass


def render_dmg_background(out_path: Path) -> None:
    """Render the DMG background at 2× resolution with 144 DPI metadata
    so macOS Finder displays it crisply on Retina while still fitting
    the 660×400 point DMG window. Text is drawn via Pillow so the
    Outfit/DM Sans typefaces match the rest of the Nootle brand
    regardless of the host system's installed fonts."""
    scale = 2
    width = DMG_BG_WIDTH * scale
    height = DMG_BG_HEIGHT * scale
    cairosvg.svg2png(
        url=str(DMG_BG_SVG),
        write_to=str(out_path),
        output_width=width,
        output_height=height,
    )
    image = Image.open(out_path).convert("RGBA")
    draw = ImageDraw.Draw(image)

    title = ImageFont.truetype(str(DMG_FONT_OUTFIT), 34 * scale)
    set_weight(title, 700)
    subtitle = ImageFont.truetype(str(DMG_FONT_DM_SANS), 14 * scale)
    set_weight(subtitle, 400)
    instruction = ImageFont.truetype(str(DMG_FONT_DM_SANS), 14 * scale)
    set_weight(instruction, 500)
    badge = ImageFont.truetype(str(DMG_FONT_DM_SANS), 11 * scale)
    set_weight(badge, 600)

    def centered(text: str, font: ImageFont.FreeTypeFont, y: int, fill: tuple[int, int, int]) -> None:
        left, top, right, bottom = font.getbbox(text)
        text_width = right - left
        draw.text(((width - text_width) // 2 - left, y * scale - top), text, font=font, fill=fill)

    centered("Nootle", title, 30, (59, 7, 100))
    centered("Your AI meeting recorder and assistant", subtitle, 75, (109, 40, 217))
    centered("Drag Nootle to your Applications folder to install", instruction, 320, (91, 33, 182))
    centered("LOCAL  ·  PRIVATE  ·  ON-DEVICE", badge, 360, (109, 40, 217))

    image.save(out_path, format="PNG", dpi=(72 * scale, 72 * scale))


def main() -> None:
    if not SRC_SVG.exists():
        sys.exit(f"missing source: {SRC_SVG}")

    print(f"source: {SRC_SVG}")

    render(SRC_SVG, ICON_DIR / "icon.png", 1024)
    render(SRC_SVG, ICON_DIR / "32x32.png", 32)
    render(SRC_SVG, ICON_DIR / "128x128.png", 128)
    render(SRC_SVG, ICON_DIR / "128x128@2x.png", 256)

    windows_sizes = {
        "Square30x30Logo.png": 30,
        "Square44x44Logo.png": 44,
        "Square71x71Logo.png": 71,
        "Square89x89Logo.png": 89,
        "Square107x107Logo.png": 107,
        "Square142x142Logo.png": 142,
        "Square150x150Logo.png": 150,
        "Square284x284Logo.png": 284,
        "Square310x310Logo.png": 310,
        "StoreLogo.png": 50,
    }
    for name, size in windows_sizes.items():
        render(SRC_SVG, ICON_DIR / name, size)

    ico_path = ICON_DIR / "icon.ico"
    ico_sizes = [16, 24, 32, 48, 64, 128, 256]
    ico_images = []
    with tempfile.TemporaryDirectory() as td:
        for size in ico_sizes:
            tmp = Path(td) / f"{size}.png"
            render(SRC_SVG, tmp, size)
            ico_images.append(Image.open(tmp).convert("RGBA"))
        ico_images[0].save(ico_path, format="ICO", sizes=[(s, s) for s in ico_sizes])

    icns_path = ICON_DIR / "icon.icns"
    iconset_sizes = [
        ("icon_16x16.png", 16),
        ("icon_16x16@2x.png", 32),
        ("icon_32x32.png", 32),
        ("icon_32x32@2x.png", 64),
        ("icon_128x128.png", 128),
        ("icon_128x128@2x.png", 256),
        ("icon_256x256.png", 256),
        ("icon_256x256@2x.png", 512),
        ("icon_512x512.png", 512),
        ("icon_512x512@2x.png", 1024),
    ]
    if shutil.which("iconutil"):
        with tempfile.TemporaryDirectory() as td:
            iconset = Path(td) / "icon.iconset"
            iconset.mkdir()
            for name, size in iconset_sizes:
                render(SRC_SVG, iconset / name, size)
            subprocess.run(["iconutil", "-c", "icns", str(iconset), "-o", str(icns_path)], check=True)
    else:
        print(f"skipping {icns_path.name}: iconutil not available (macOS only)")

    PUBLIC_DIR.mkdir(parents=True, exist_ok=True)
    SITE_PUBLIC_DIR.mkdir(parents=True, exist_ok=True)
    render(SRC_SVG, PUBLIC_DIR / "nootle-icon.png", 1024)
    render(SRC_SVG, SITE_PUBLIC_DIR / "nootle-icon.png", 1024)
    shutil.copyfile(SRC_SVG, PUBLIC_DIR / "nootle-icon.svg")
    shutil.copyfile(SRC_SVG, SITE_PUBLIC_DIR / "nootle-icon.svg")

    if DMG_BG_SVG.exists() and DMG_FONT_OUTFIT.exists() and DMG_FONT_DM_SANS.exists():
        DMG_DIR.mkdir(parents=True, exist_ok=True)
        render_dmg_background(DMG_DIR / "background.png")
        (DMG_DIR / "background@2x.png").unlink(missing_ok=True)

    print("done")


if __name__ == "__main__":
    main()
