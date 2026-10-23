"""Contact sheet: python preview.py OUT.png SPRITE_DIR [ids...] — sprites tinted on grass."""
import os, sys
from PIL import Image, ImageDraw
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from post import tint_over

out, d = sys.argv[1], sys.argv[2]
ids = [a.lower() for a in sys.argv[3:]] or sorted(f[:-4] for f in os.listdir(d) if f.endswith(".png") and not f.endswith("_mask.png"))
cols = int(os.environ.get("COLS", 8))
cell = int(os.environ.get("CELL", 192))
colors = [(200, 40, 40), (40, 90, 200), (230, 190, 30), (40, 150, 70), (150, 60, 180), (230, 120, 30)]
rows = (len(ids) + cols - 1) // cols
sheet = Image.new("RGBA", (cols * cell, rows * (cell + 14)), (96, 128, 72, 255))
dr = ImageDraw.Draw(sheet)
for i, uid in enumerate(ids):
    sp = Image.open(os.path.join(d, uid + ".png")).convert("RGBA")
    mp = os.path.join(d, uid + "_mask.png")
    if os.path.exists(mp):
        sp = tint_over(sp, Image.open(mp).convert("RGBA"), colors[i % len(colors)])
    sp = sp.resize((cell, cell), Image.LANCZOS)
    x, y = (i % cols) * cell, (i // cols) * (cell + 14)
    sheet.alpha_composite(sp, (x, y))
    dr.text((x + 3, y + cell), uid.replace("unit_", ""), fill=(255, 255, 255, 255))
sheet.convert("RGB").save(out)
