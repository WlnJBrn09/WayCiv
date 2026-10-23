"""Post-process raw Blender renders into game sprites (needs Pillow + numpy).

  python post.py RAW_DIR OUT_DIR [--size 192] [UNIT_ID...]

For every <id>.png / <id>_mask.png in RAW_DIR writes
  OUT_DIR/<id>.png            beauty sprite (RGBA)
  OUT_DIR/<id>_mask.png       team-colour layer: grey = shading, alpha = team parts
  OUT_DIR/portraits/<id>.png  128 px round portrait
"""
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

TEAM_ALBEDO = 0.55


def down(img, size):
    return img.convert("RGBa").resize((size, size), Image.LANCZOS).convert("RGBA")


def make_sprite(raw, mask_raw, size):
    beauty = Image.open(raw).convert("RGBA")
    mask = Image.open(mask_raw).convert("RGBA")
    b = np.asarray(beauty).astype(np.float32) / 255.0
    m = np.asarray(mask)[:, :, 3].astype(np.float32) / 255.0
    # shading under the team parts, normalised so the lit side reaches 1.0
    lum = b[:, :, :3].mean(axis=2) / (TEAM_ALBEDO * 0.78)
    lum = np.clip(lum, 0.0, 1.0)
    a = np.minimum(m, b[:, :, 3])
    g = (lum * 255).astype(np.uint8)
    mk = np.stack([g, g, g, (a * 255).astype(np.uint8)], axis=2)
    mk_img = Image.fromarray(mk, "RGBA")
    return down(beauty, size), down(mk_img, size)


def tint_over(sprite, mask, rgb):
    t = np.asarray(mask).astype(np.float32) / 255.0
    col = np.array(rgb, np.float32) / 255.0
    layer = np.zeros(t.shape, np.float32)
    layer[:, :, :3] = t[:, :, :3] * col
    layer[:, :, 3] = t[:, :, 3]
    lay = Image.fromarray((layer * 255).astype(np.uint8), "RGBA")
    out = sprite.copy()
    out.alpha_composite(lay)
    return out


def crop_tight(img, mask, pad=2):
    bbox = img.getchannel("A").point(lambda v: 255 if v > 6 else 0).getbbox()
    return bbox


def portrait(sprite, mask, kind, size=128):
    """Round portrait: sky/sea/grass backdrop, the unit enlarged, soft shadow."""
    S = size * 2
    bg = Image.new("RGBA", (S, S))
    px = bg.load()
    if kind == "sea":
        top, bot = (150, 200, 225), (40, 100, 140)
    elif kind == "air":
        top, bot = (90, 150, 215), (210, 228, 240)
    else:
        top, bot = (150, 195, 225), (92, 128, 70)
    for y in range(S):
        t = y / (S - 1)
        c = tuple(int(top[i] * (1 - t) + bot[i] * t) for i in range(3)) + (255,)
        for x in range(S):
            px[x, y] = c
    if kind == "land":
        dr = ImageDraw.Draw(bg)
        dr.ellipse((-S * 0.3, S * 0.62, S * 1.3, S * 1.5), fill=(104, 138, 74, 255))
    spr = tint_over(sprite, mask, (168, 58, 44)) if mask is not None else sprite
    bbox = spr.getchannel("A").point(lambda v: 255 if v > 8 else 0).getbbox()
    if bbox:
        spr = spr.crop(bbox)
        w, h = spr.size
        k = min(S * 0.82 / w, S * 0.78 / h)
        spr = spr.resize((max(1, int(w * k)), max(1, int(h * k))), Image.LANCZOS)
        w, h = spr.size
        x, y = (S - w) // 2, int(S * 0.52 - h / 2) if kind != "land" else int(S * 0.9 - h)
        if kind == "land":
            y = int(S * 0.92 - h)
        sh = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        ImageDraw.Draw(sh).ellipse((x + w * 0.12, y + h * 0.84, x + w * 0.88, y + h * 1.0), fill=(0, 0, 0, 90))
        sh = sh.filter(ImageFilter.GaussianBlur(S * 0.02))
        bg.alpha_composite(sh)
        bg.alpha_composite(spr, (x, y))
    # round mask and rim
    circ = Image.new("L", (S, S), 0)
    ImageDraw.Draw(circ).ellipse((2, 2, S - 3, S - 3), fill=255)
    out = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    out.paste(bg, (0, 0), circ)
    ImageDraw.Draw(out).ellipse((2, 2, S - 3, S - 3), outline=(230, 200, 120, 255), width=5)
    return out.resize((size, size), Image.LANCZOS)


def main():
    args = sys.argv[1:]
    size = 192
    if "--size" in args:
        i = args.index("--size")
        size = int(args[i + 1])
        del args[i:i + 2]
    raw, out = args[0], args[1]
    only = [a.lower() for a in args[2:]]
    kinds, manifest = {}, []
    for f in sorted(os.listdir(raw)):
        if f.endswith(".meta"):
            uid, ortho, anchor, kind = open(os.path.join(raw, f)).read().split()
            kinds[uid] = kind
            manifest.append((uid, ortho, anchor))
    os.makedirs(os.path.join(out, "portraits"), exist_ok=True)
    for f in sorted(os.listdir(raw)):
        if not f.endswith(".png") or f.endswith("_mask.png"):
            continue
        uid = f[:-4]
        if only and uid not in only:
            continue
        mask_path = os.path.join(raw, uid + "_mask.png")
        if not os.path.exists(mask_path):
            continue
        spr, mk = make_sprite(os.path.join(raw, f), mask_path, size)
        spr.save(os.path.join(out, uid + ".png"), optimize=True)
        # grey + alpha keeps the mask small
        la = Image.merge("LA", (mk.getchannel("R"), mk.getchannel("A")))
        la.save(os.path.join(out, uid + "_mask.png"), optimize=True)
        portrait(spr, mk, kinds.get(uid, "land")).save(os.path.join(out, "portraits", uid + ".png"), optimize=True)
        print("ok", uid)
    if manifest and not only:
        with open(os.path.join(out, "manifest.tsv"), "w") as f:
            f.write("# id\tortho_width_world_units\tanchor_y_fraction (ground point), generated by tools/units\n")
            for m in manifest:
                f.write("\t".join(m) + "\n")


if __name__ == "__main__":
    main()
