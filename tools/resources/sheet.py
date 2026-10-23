#!/usr/bin/env python3
"""Contact sheet of all generated resource sprites, icons and improvements."""
import os
import sys

from PIL import Image, ImageDraw

root, out = sys.argv[1], sys.argv[2]
bg = (118, 150, 92, 255)
groups = []
for sub, title, cell in (("sprites", "Resource sprites", 112), ("icons", "Resource icons", 112), ("improvements", "Improvements and camps", 112)):
    d = os.path.join(root, sub)
    if os.path.isdir(d):
        groups.append((title, [os.path.join(d, f) for f in sorted(os.listdir(d)) if f.endswith(".png")], cell))
cols = 14
pad = 4
W = cols * (112 + pad) + pad
H = pad
rows_per = [((len(fs) + cols - 1) // cols, title) for title, fs, _ in groups]
H += sum(r * (112 + 14 + pad) + 22 for r, _ in rows_per)
sheet = Image.new("RGBA", (W, H), bg)
dr = ImageDraw.Draw(sheet)
y = pad
for (title, files, cell) in groups:
    dr.text((pad, y + 4), title, fill=(255, 255, 255, 255))
    y += 22
    for i, f in enumerate(files):
        x = pad + (i % cols) * (112 + pad)
        yy = y + (i // cols) * (112 + 14 + pad)
        im = Image.open(f).convert("RGBA").resize((112, 112), Image.LANCZOS)
        sheet.alpha_composite(im, (x, yy))
        name = os.path.basename(f)[:-4].replace("resource_", "").replace("imp_", "")
        dr.text((x + 2, yy + 112), name[:20], fill=(255, 255, 255, 255))
    y += ((len(files) + cols - 1) // cols) * (112 + 14 + pad)
sheet.convert("RGB").save(out)
print("sheet:", out, sheet.size)
