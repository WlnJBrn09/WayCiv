#!/usr/bin/env python3
"""Rebuild assets/terrain/*.png.  Usage: python build.py [outdir] [sheet ...]
Needs numpy, scipy and pillow.  Sheets: ground relief features wonders water."""
import os, sys
here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from texlib import save_rgb, save_rgba

out = sys.argv[1] if len(sys.argv) > 1 else os.path.join(here, "..", "..", "assets", "terrain")
which = sys.argv[2:] or ["ground", "relief", "features", "wonders", "water"]
os.makedirs(out, exist_ok=True)
for name in which:
    mod = __import__(name)
    if name == "ground":
        save_rgb(os.path.join(out, "ground.png"), mod.build_atlas())
    elif name == "water":
        save_rgb(os.path.join(out, "water.png"), mod.build_sheet())
        save_rgb(os.path.join(out, "glint.png"), mod.build_glint())
    else:
        save_rgba(os.path.join(out, name + ".png"), mod.build_sheet())
    print("wrote", name)
