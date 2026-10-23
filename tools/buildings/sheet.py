"""Contact sheet of the sprites on a hex grid (needs Pillow).

    python3 tools/buildings/sheet.py assets/buildings out.png [prefix ...] [--cols N] [--scale S] [--tint]

--tint paints each *_tint layer in a sample owner colour on top of the sprite.
"""
import math
import os
import sys

from PIL import Image, ImageDraw


def hexagon(draw, cx, cy, r, fill, outline):
    pts = [(cx + r * math.cos(math.radians(60 * i - 30)), cy + r * math.sin(math.radians(60 * i - 30))) for i in range(6)]
    draw.polygon(pts, fill=fill, outline=outline)


def main():
    args = sys.argv[1:]
    src, out = args[0], args[1]
    cols, scale, tint = 6, 0.7, False
    prefixes = []
    i = 2
    while i < len(args):
        if args[i] == '--cols':
            cols = int(args[i + 1]); i += 2
        elif args[i] == '--scale':
            scale = float(args[i + 1]); i += 2
        elif args[i] == '--tint':
            tint = True; i += 1
        else:
            prefixes.append(args[i]); i += 1
    names = sorted(f[:-4] for f in os.listdir(src) if f.endswith('.png') and not f.endswith('_tint.png'))
    if prefixes:
        names = [n for n in names if any(n.startswith(p) for p in prefixes)]
    sw, sh = int(256 * scale), int(320 * scale)
    r = 1.0 * sw / 2.2  # hex circumradius in px
    cellw, cellh = sw * 0.92, sh * 0.78
    rows = (len(names) + cols - 1) // cols
    W = int(cellw * cols) + 50
    H = int(cellh * rows + sh) + 40
    img = Image.new('RGBA', (W, H), (40, 60, 40, 255))
    d = ImageDraw.Draw(img)
    tints = [(200, 40, 40), (40, 90, 200), (230, 170, 30), (60, 160, 70)]
    for k, n in enumerate(names):
        row, col = divmod(k, cols)
        cx = 20 + cellw * (col + 0.5) + cellw * 0.04
        cy = 20 + cellh * row + 0.6 * sh * 0.9
        cy = 30 + cellh * row + 0.6 * sh
        hexagon(d, cx, cy, r, (92, 140, 70, 255), (30, 50, 30, 255))
    for k, n in enumerate(names):
        row, col = divmod(k, cols)
        cx = 20 + cellw * (col + 0.5) + cellw * 0.04
        cy = 30 + cellh * row + 0.6 * sh
        im = Image.open(os.path.join(src, n + '.png')).convert('RGBA').resize((sw, sh), Image.LANCZOS)
        img.alpha_composite(im, (int(cx - sw / 2), int(cy - 0.6 * sh)))
        tp = os.path.join(src, n + '_tint.png')
        if tint and os.path.exists(tp):
            t = Image.open(tp).convert('RGBA').resize((sw, sh), Image.LANCZOS)
            c = tints[k % len(tints)]
            px = t.load()
            for yy in range(sh):
                for xx in range(sw):
                    p = px[xx, yy]
                    px[xx, yy] = (p[0] * c[0] // 255, p[1] * c[1] // 255, p[2] * c[2] // 255, p[3])
            img.alpha_composite(t, (int(cx - sw / 2), int(cy - 0.6 * sh)))
        d.text((cx - sw / 2 + 4, cy + r * 0.95), n.replace('district_', '').replace('wonder_', ''), fill=(255, 255, 255, 255))
    img.convert('RGB').save(out)
    print(out, img.size)


main()
