"""Render ui_gallery's actual cell buffers. Optional QA dependency: Pillow.

python scripts/render-gallery.py target/ui-gallery
"""
import json
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

COLORS = dict(zip(
    ["Black", "Red", "Green", "Yellow", "Blue", "Magenta", "Cyan", "Gray",
     "DarkGray", "LightRed", "LightGreen", "LightYellow", "LightBlue",
     "LightMagenta", "LightCyan", "White"],
    ["#101820", "#e56f74", "#88c08c", "#e1ba75", "#739ed0", "#b68acf",
     "#78c7d0", "#c8ced6", "#536171", "#ff9396", "#a6dcaa", "#f1d692",
     "#9bb9e2", "#d0a8e9", "#a0e1e6", "#f4f6fb"],
))
FONT_PATHS = ["/System/Library/Fonts/Menlo.ttc",
              "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"]
font_path = next((p for p in FONT_PATHS if Path(p).exists()), "DejaVuSansMono.ttf")
font = ImageFont.truetype(font_path, 16)
bold = ImageFont.truetype(font_path, 16, index=1 if font_path.endswith(".ttc") else 0)
for path in Path(sys.argv[1]).glob("*.json"):
    data = json.loads(path.read_text())
    if "cells" not in data:
        continue
    width, height = data["width"], data["height"]
    image = Image.new("RGB", (width * 10 + 40, height * 21 + 75), "#101820")
    draw = ImageDraw.Draw(image)
    caption = f'TERSH / {path.stem} / DEMO'
    draw.text((20, 12), caption[:width], font=font, fill="#91a4b8")
    for index, cell in enumerate(data["cells"]):
        x, y = 20 + index % width * 10, 45 + index // width * 21
        fg, bg = COLORS.get(cell["fg"], "#d6dfe9"), COLORS.get(cell["bg"], "#101820")
        if cell["reverse"]:
            fg, bg = bg, fg
        draw.rectangle((x, y, x + 10, y + 21), fill=bg)
        draw.text((x, y), cell["text"], font=bold if cell["bold"] else font, fill=fg)
    image.save(path.with_suffix(".png"))
