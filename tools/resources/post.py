#!/usr/bin/env python3
"""Turns the raw Blender renders into the game's assets.

  python post.py RAW_DIR OUT_DIR       (OUT_DIR = assets/resources)

  sprites/<id>.png        map sprites of resources        (160 px, RGBA)
  icons/<id>.png          class-coloured circular badges  (128 px, RGBA)
  improvements/<id>.png   improvement and camp sprites    (160/192 px)
Needs Pillow + numpy.
"""
import math
import os
import sys

import numpy as np
from PIL import Image, ImageChops, ImageDraw, ImageFilter

RAW, OUT = sys.argv[1], sys.argv[2]

CLASS_COL = {
    "bonus": ((96, 190, 84), (36, 110, 44), (232, 255, 214)),
    "luxury": ((178, 108, 214), (82, 36, 120), (246, 226, 255)),
    "strategic": ((226, 86, 72), (120, 28, 26), (255, 226, 214)),
    "artifact": ((226, 184, 84), (120, 84, 24), (255, 244, 206)),
}
BONUS = "bananas cattle copper crabs deer fish rice sheep stone wheat maize".split()
STRATEGIC = "aluminum coal horses iron niter oil uranium".split()
ARTIFACT = "antiquity_site shipwreck".split()


def klass(name):
    n = name.replace("resource_", "")
    if n in BONUS:
        return "bonus"
    if n in STRATEGIC:
        return "strategic"
    if n in ARTIFACT:
        return "artifact"
    return "luxury"


def premul_resize(im, size):
    a = np.asarray(im.convert("RGBA")).astype(np.float32) / 255.0
    a[..., 3] = np.clip((a[..., 3] - 0.06) / 0.94, 0, 1)  # drop denoiser noise
    a[..., :3] *= a[..., 3:4]
    pm = Image.fromarray((a * 255).astype(np.uint8), "RGBA")
    pm = pm.resize((size, size), Image.LANCZOS) if isinstance(size, int) else pm.resize(size, Image.LANCZOS)
    b = np.asarray(pm).astype(np.float32) / 255.0
    al = b[..., 3:4]
    b[..., :3] = np.where(al > 1e-4, b[..., :3] / np.maximum(al, 1e-4), 0)
    return Image.fromarray((np.clip(b, 0, 1) * 255).astype(np.uint8), "RGBA")


def object_bbox(im, thresh=170):
    a = np.asarray(im)[..., 3]
    ys, xs = np.where(a > thresh)
    if len(xs) == 0:
        return (0, 0, im.width, im.height)
    return (xs.min(), ys.min(), xs.max() + 1, ys.max() + 1)


def make_icon(raw, klass_name, size=128):
    """A round badge: class-coloured gradient, rim, the model centred."""
    S = size * 4
    base, dark, light = CLASS_COL[klass_name]
    yy, xx = np.mgrid[0:S, 0:S].astype(np.float32)
    cx = cy = S / 2
    r = np.hypot(xx - cx, yy - cy) / (S / 2)
    # radial gradient, lighter top-left like a glass button
    gl = np.hypot(xx - S * 0.38, yy - S * 0.32) / (S * 0.8)
    t = np.clip(gl, 0, 1)[..., None]
    col = np.array(light, np.float32) * (1 - t) + np.array(base, np.float32) * t * 1.0
    t2 = np.clip((r - 0.55) / 0.45, 0, 1)[..., None]
    col = col * (1 - t2 * 0.5) + np.array(dark, np.float32) * t2 * 0.5
    img = np.zeros((S, S, 4), np.float32)
    img[..., :3] = col
    img[..., 3] = 255
    badge = Image.fromarray(img.astype(np.uint8), "RGBA")
    # circular alpha mask
    mask = Image.new("L", (S, S), 0)
    d = ImageDraw.Draw(mask)
    d.ellipse([2, 2, S - 3, S - 3], fill=255)
    # rim
    rim = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    rd = ImageDraw.Draw(rim)
    w = max(4, S // 22)
    rd.ellipse([2, 2, S - 3, S - 3], outline=(*[int(v * 0.55) for v in dark], 255), width=w)
    rd.ellipse([2 + w, 2 + w, S - 3 - w, S - 3 - w], outline=(*light, 150), width=max(2, w // 3))
    # the model, cropped to its bbox and fitted
    x0, y0, x1, y1 = object_bbox(raw)
    bw, bh = x1 - x0, y1 - y0
    side = max(bw, bh) * 1.12
    mx, my = (x0 + x1) / 2, (y0 + y1) / 2 + side * 0.03
    crop = raw.crop((int(mx - side / 2), int(my - side / 2), int(mx + side / 2), int(my + side / 2)))
    inner = int(S * 0.74)
    crop = premul_resize(crop, inner)
    badge.paste(crop, ((S - inner) // 2, (S - inner) // 2), crop)
    badge.alpha_composite(rim)
    badge.putalpha(ImageChops.multiply(badge.getchannel("A"), mask))
    return premul_resize(badge, size)


def soften_hex(im, inset=0.0, feather=10):
    """Fade the edge of a top-down hex decal into the terrain."""
    W = im.width
    S = W * 2
    # pointy-top hex of radius R (px), ortho 2.0 -> R = W/2 * mask
    R = W / 2 * inset if inset else W / 2 * 0.9
    m = Image.new("L", (W, W), 0)
    pts = [(W / 2 + R * math.cos(math.radians(30 + 60 * i)), W / 2 - R * math.sin(math.radians(30 + 60 * i))) for i in range(6)]
    ImageDraw.Draw(m).polygon(pts, fill=255)
    m = m.filter(ImageFilter.GaussianBlur(feather))
    out = im.copy()
    out.putalpha(ImageChops.multiply(im.getchannel("A"), m))
    return out


def main():
    for sub in ("sprites", "icons", "improvements"):
        os.makedirs(os.path.join(OUT, sub), exist_ok=True)
    names = sorted(f[:-4] for f in os.listdir(RAW) if f.endswith(".png"))
    for name in names:
        raw = Image.open(os.path.join(RAW, name + ".png")).convert("RGBA")
        if name.startswith("resource_"):
            premul_resize(raw, 160).save(os.path.join(OUT, "sprites", name + ".png"), optimize=True)
            make_icon(raw, klass(name)).save(os.path.join(OUT, "icons", name + ".png"), optimize=True)
        else:
            fieldlike = name in ("imp_farm", "imp_plantation")
            if fieldlike:
                im = premul_resize(soften_hex(raw, 0.93 if name == "imp_farm" else 0.88, 5), 192)
            else:
                im = premul_resize(raw, 160)
            im.save(os.path.join(OUT, "improvements", name + ".png"), optimize=True)
    print("post: wrote", len(names), "assets")


main()
