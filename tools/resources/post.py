#!/usr/bin/env python3
"""Turns the raw Blender renders into the game's assets.

  python post.py RAW_DIR OUT_DIR       (OUT_DIR = assets/resources)

  sprites/<id>.png        map sprites of resources        (224 px, RGBA, dark rim)
  icons/<id>.png          class-coloured circular badges  (128 px, RGBA)
  improvements/<id>.png   improvement and camp sprites    (160 px; field decals 224 px)
Needs Pillow + numpy.
"""
import os
import sys
import zlib

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


SPRITE_PX = 224


def outlined(sprite):
    """A thin dark rim under the model (not its soft ground shadow), so a
    resource stands out from busy terrain."""
    a = np.asarray(sprite)[..., 3]
    solid = Image.fromarray(np.where(a > 150, 255, 0).astype(np.uint8), "L")
    rim = solid.filter(ImageFilter.MaxFilter(5)).filter(ImageFilter.GaussianBlur(1.0))
    lay = Image.new("RGBA", sprite.size, (24, 20, 14, 0))
    lay.putalpha(rim.point(lambda v: int(v * 0.6)))
    out = Image.new("RGBA", sprite.size, (0, 0, 0, 0))
    out.alpha_composite(lay)
    out.alpha_composite(sprite)
    return out


def noise_field(W, seed, cells=(5, 11, 23)):
    """Smooth value noise in [-1, 1], a few octaves."""
    rng = np.random.default_rng(seed)
    out = np.zeros((W, W), np.float32)
    amp = 1.0
    for n in cells:
        g = Image.fromarray((rng.random((n, n)) * 255).astype(np.uint8), "L").resize((W, W), Image.BICUBIC)
        out += (np.asarray(g).astype(np.float32) / 127.5 - 1.0) * amp
        amp *= 0.5
    return out / sum(0.5 ** i for i in range(len(cells)))


def soften_field(im, seed, inner=0.74, outer=0.98, warp=0.13):
    """Fade a top-down field decal out toward (and past) the hex edge along a
    wobbly, noise-warped outline so it grows out of the terrain."""
    W = im.width
    yy, xx = np.mgrid[0:W, 0:W].astype(np.float32)
    x = (xx - W / 2) / (W / 2)
    y = -(yy - W / 2) / (W / 2)
    d = np.maximum(np.abs(x), np.maximum(np.abs(0.5 * x + 0.866 * y), np.abs(0.5 * x - 0.866 * y))) / 0.866
    d = d * 0.82 + np.hypot(x, y) * 0.18  # rounds the corners
    d = d + noise_field(W, seed) * warp
    m = np.clip((outer - d) / (outer - inner), 0, 1)
    m = m * m * (3 - 2 * m)
    a = np.asarray(im).astype(np.float32)
    a[..., 3] *= m
    return Image.fromarray(a.astype(np.uint8), "RGBA")


def main():
    for sub in ("sprites", "icons", "improvements"):
        os.makedirs(os.path.join(OUT, sub), exist_ok=True)
    names = sorted(f[:-4] for f in os.listdir(RAW) if f.endswith(".png"))
    for name in names:
        raw = Image.open(os.path.join(RAW, name + ".png")).convert("RGBA")
        if name.startswith("resource_"):
            outlined(premul_resize(raw, SPRITE_PX)).save(os.path.join(OUT, "sprites", name + ".png"), optimize=True)
            make_icon(raw, klass(name)).save(os.path.join(OUT, "icons", name + ".png"), optimize=True)
        else:
            if name.startswith(("imp_farm_", "imp_plantation_")):
                im = premul_resize(soften_field(raw, zlib.crc32(name.encode())), 224)
            else:
                im = premul_resize(raw, SPRITE_PX)
            im.save(os.path.join(OUT, "improvements", name + ".png"), optimize=True)
    print("post: wrote", len(names), "assets")


main()
