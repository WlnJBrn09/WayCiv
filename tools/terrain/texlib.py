"""Shared helpers for the WayCiv terrain art generators: noise, colour
ramps, and a small oblique heightfield renderer used for hills, mountains,
trees, dunes and natural wonders.  Everything here is original procedural
art (numpy only); nothing is derived from any game assets."""
import numpy as np
from scipy import ndimage as ndi

CELL = 256          # px per atlas cell in the shipped PNGs
SS = 2              # supersampling of sprites
CANVAS = CELL * SS  # sprite canvas (px)
EXT = 1.3           # sprite canvas covers [-EXT, EXT] z-units horizontally
P = CANVAS / (2 * EXT)  # canvas px per z-unit

# ---------------------------------------------------------------- noise


def fft_noise(n, beta, seed, lo=None, hi=None):
    """Tileable 1/f^beta noise, normalised to 0..1 (n x n)."""
    r = np.random.default_rng(seed)
    w = r.standard_normal((n, n))
    f = np.fft.fftfreq(n)
    fx, fy = np.meshgrid(f, f)
    k = np.sqrt(fx * fx + fy * fy)
    k[0, 0] = 1
    spec = np.fft.fft2(w) / (k ** (beta / 2.0))
    spec[0, 0] = 0
    out = np.real(np.fft.ifft2(spec))
    lo = np.percentile(out, 1) if lo is None else lo
    hi = np.percentile(out, 99) if hi is None else hi
    return np.clip((out - lo) / (hi - lo), 0, 1)


def fbm(n, seed, base_beta=2.2):
    return fft_noise(n, base_beta, seed)


def ridged(n, seed, beta=2.0):
    a = fft_noise(n, beta, seed)
    return 1.0 - np.abs(a * 2 - 1)


def smoothstep(a, b, x):
    t = np.clip((x - a) / (b - a + 1e-9), 0, 1)
    return t * t * (3 - 2 * t)


def blur(a, s):
    return ndi.gaussian_filter(a, s, mode="wrap")


def blurn(a, s):
    return ndi.gaussian_filter(a, s, mode="nearest")


def lerp(a, b, t):
    return a + (b - a) * t


def hexc(h):
    h = h.lstrip("#")
    return np.array([int(h[i:i + 2], 16) for i in (0, 2, 4)], dtype=np.float32) / 255.0


def mix(c0, c1, t):
    """Colour mix. t is an HxW array in 0..1, c0/c1 are RGB triples."""
    t = t[..., None]
    return np.asarray(c0) * (1 - t) + np.asarray(c1) * t


def ramp(t, stops):
    """Piecewise-linear colour ramp. stops = [(pos, rgb), ...]"""
    out = np.zeros(t.shape + (3,), np.float32)
    ps = [s[0] for s in stops]
    cs = [np.asarray(s[1], np.float32) for s in stops]
    t = np.clip(t, ps[0], ps[-1])
    for ch in range(3):
        out[..., ch] = np.interp(t, ps, [c[ch] for c in cs])
    return out


def strokes(n, seed, count, length, width, angle=None, jitter=0.6, wrap=True):
    """Soft 'brush stroke' coverage map (n x n, 0..1): short oriented
    line segments splatted with a gaussian profile."""
    r = np.random.default_rng(seed)
    img = np.zeros((n, n), np.float32)
    xs = r.uniform(0, n, count)
    ys = r.uniform(0, n, count)
    if angle is None:
        ang = r.uniform(0, np.pi, count)
    else:
        ang = angle + r.normal(0, jitter, count)
    ln = r.uniform(0.5, 1.0, count) * length
    steps = max(3, int(length / max(width, 1)))
    for s in range(steps + 1):
        t = (s / steps - 0.5)
        px = (xs + np.cos(ang) * ln * t).astype(int) % n
        py = (ys + np.sin(ang) * ln * t).astype(int) % n
        np.add.at(img, (py, px), 1.0)
    img = blur(img, width)
    img /= max(img.max(), 1e-6)
    return np.clip(img * 3.0, 0, 1)


def dots(n, seed, count, radius):
    r = np.random.default_rng(seed)
    img = np.zeros((n, n), np.float32)
    np.add.at(img, (r.integers(0, n, count), r.integers(0, n, count)), 1.0)
    img = blur(img, radius)
    return np.clip(img / max(img.max(), 1e-6), 0, 1)


def voronoi(n, seed, count):
    """Returns (cell id map, distance to nearest feature point, distance
    to second nearest) in 0..1-ish units of the grid."""
    r = np.random.default_rng(seed)
    pts = r.uniform(0, n, (count, 2))
    yy, xx = np.mgrid[0:n, 0:n].astype(np.float32)
    d1 = np.full((n, n), 1e9, np.float32)
    d2 = np.full((n, n), 1e9, np.float32)
    ids = np.zeros((n, n), np.int32)
    for i, (px, py) in enumerate(pts):
        dx = np.abs(xx - px)
        dy = np.abs(yy - py)
        dx = np.minimum(dx, n - dx)
        dy = np.minimum(dy, n - dy)
        d = np.sqrt(dx * dx + dy * dy)
        closer = d < d1
        d2 = np.where(closer, d1, np.minimum(d2, d))
        ids = np.where(closer, i, ids)
        d1 = np.where(closer, d, d1)
    return ids, d1, d2


# --------------------------------------------------------- image output


def to_u8(rgb):
    return (np.clip(rgb, 0, 1) * 255 + 0.5).astype(np.uint8)


def downsample_premul(rgba, f):
    """Box-downsample an RGBA float image (unpremultiplied) by integer f
    using premultiplied alpha; returns unpremultiplied float RGBA."""
    h, w, _ = rgba.shape
    a = rgba[..., 3:4]
    pm = np.concatenate([rgba[..., :3] * a, a], axis=-1)
    pm = pm.reshape(h // f, f, w // f, f, 4).mean(axis=(1, 3))
    a = pm[..., 3:4]
    rgb = np.where(a > 1e-4, pm[..., :3] / np.maximum(a, 1e-4), 0)
    # bleed colour into fully transparent pixels to keep mips clean
    return np.concatenate([rgb, a], axis=-1)


def bleed(rgba, iters=6):
    """Spread colour of opaque-ish pixels into transparent neighbours so
    bilinear filtering never mixes in black."""
    rgb = rgba[..., :3].copy()
    a = rgba[..., 3]
    known = a > 0.02
    for _ in range(iters):
        w = known.astype(np.float32)
        num = ndi.uniform_filter(rgb * w[..., None], size=(3, 3, 1)) * 9
        den = ndi.uniform_filter(w, size=3) * 9
        fill = (~known) & (den > 0)
        rgb[fill] = (num[fill] / den[fill][:, None])
        known = known | fill
    out = rgba.copy()
    out[..., :3] = rgb
    return out


def save_rgba(path, img):
    from PIL import Image
    Image.fromarray(to_u8(img), "RGBA").save(path, optimize=True)


def save_rgb(path, img):
    from PIL import Image
    Image.fromarray(to_u8(img), "RGB").save(path, optimize=True)


# ------------------------------------------------- heightfield renderer

GRID = CANVAS
gxs = np.linspace(-EXT, EXT, GRID, dtype=np.float32)
GX, GY = np.meshgrid(gxs, gxs)  # GY grows toward the viewer (south)
STEP = (2 * EXT) / GRID


def shade(H, albedo, light=(-0.55, -0.62, 0.62), ambient=0.46, sun=0.78,
          shadows=True, shadow_len=70, shadow_strength=0.55, hscale=1.0,
          ao=0.55, warm=(1.0, 0.95, 0.84), cool=(0.80, 0.88, 1.02)):
    """Lambert + cast shadows + cheap ambient occlusion for a heightfield
    given in z-units. Returns lit RGB."""
    Hs = H * hscale
    dy, dx = np.gradient(Hs, STEP)
    n = np.stack([-dx, -dy, np.ones_like(dx)], -1)
    n /= np.linalg.norm(n, axis=-1, keepdims=True)
    L = np.array(light, np.float32)
    L /= np.linalg.norm(L)
    lam = np.clip((n * L).sum(-1), 0, 1)
    sh = np.ones_like(H)
    if shadows:
        lxy = np.array([L[0], L[1]])
        lxy /= np.linalg.norm(lxy)
        tan_e = L[2] / np.sqrt(L[0] ** 2 + L[1] ** 2)
        shadow = np.zeros_like(H)
        for s in range(2, shadow_len, 3):
            ox = int(round(-lxy[0] * s))
            oy = int(round(-lxy[1] * s))
            # sample toward the light: H at (x + lxy*s)
            shifted = np.roll(np.roll(Hs, -int(round(lxy[1] * s)), 0),
                              -int(round(lxy[0] * s)), 1)
            occl = shifted - Hs - s * STEP * tan_e
            shadow = np.maximum(shadow, smoothstep(0.0, 0.05, occl))
        sh = 1.0 - shadow * shadow_strength
    if ao > 0:
        bl = blurn(Hs, 10)
        occ = np.clip((bl - Hs) * 3.0, 0, 1)
        sh = sh * (1 - occ * ao)
    sunl = (sun * lam * sh)[..., None] * np.array(warm, np.float32)
    amb = ambient * np.array(cool, np.float32) * (0.75 + 0.25 * sh[..., None])
    ref = sun * L[2] * np.array(warm, np.float32) + ambient * np.array(cool, np.float32)
    return albedo * (sunl + amb) / ref


def render_height(H, rgb, alpha, cy, ky=0.62, kh=1.0):
    """Oblique heightfield renderer.  H in z-units; rgb lit colour per
    grid sample; alpha footprint 0..1.  The ground plane point (gx, gy)
    lands at canvas x = col, y = cy + gy*P*ky - H*P*kh.  Returns an RGBA
    float image (CANVAS x CANVAS, unpremultiplied)."""
    out = np.zeros((CANVAS, CANVAS, 4), np.float32)
    ymin = np.full(CANVAS, CANVAS, np.int64)
    cols = np.arange(CANVAS)
    for row in range(GRID - 1, -1, -1):
        gy = gxs[row]
        base = cy + gy * P * ky
        a = alpha[row]
        live = a > 0.02
        if not live.any():
            continue
        top = np.floor(base - H[row] * P * kh).astype(np.int64)
        bot = np.minimum(ymin, int(base) + 2)
        top = np.clip(top, 0, CANVAS)
        n = np.where(live, np.maximum(bot - top, 0), 0)
        tot = int(n.sum())
        if tot:
            cidx = np.repeat(cols, n)
            starts = np.repeat(top, n)
            offs = np.arange(tot) - np.repeat(np.cumsum(n) - n, n)
            ys = starts + offs
            col = np.concatenate([rgb[row], a[:, None]], axis=-1)
            out[ys, cidx] = col[cidx]
        ymin = np.where(live, np.minimum(ymin, top), ymin)
    return out


def finish(rgba, x_shift=0, y_shift=0):
    """Supersampled canvas -> 256x256 cell (unpremultiplied RGBA)."""
    rgba = bleed(rgba, 3)
    small = downsample_premul(rgba, SS)
    return small


def blob(cx, cy, rx, ry, soft=0.25, wob=None):
    """Elliptical footprint alpha over the grid."""
    d = np.sqrt(((GX - cx) / rx) ** 2 + ((GY - cy) / ry) ** 2)
    if wob is not None:
        d = d * (1 + (wob - 0.5) * 0.5)
    return 1 - smoothstep(1 - soft, 1.0, d)


def grid_noise(seed, beta=2.2):
    return fft_noise(GRID, beta, seed)


# ---------------------------------------------------- Blender rendering


def render_many(jobs, samples=48):
    """Render sprite jobs with Cycles (see blrender.py) and return RGBA
    float cells (CELL x CELL, unpremultiplied).  Each job is a dict with
    `H` (z-units), `rgb` (unlit sRGB albedo) and `alpha` over the sprite
    canvas, plus `cy` (canvas row of the ground origin), `ky`, `kh` and
    optional `rough`, `spec`, `sun`, `sky`, `step`."""
    import os, subprocess, tempfile
    tmp = tempfile.mkdtemp(prefix="wayciv_terrain_")
    for i, j in enumerate(jobs):
        d = dict(rough=0.85, spec=0.2, sun=3.4, sky=0.75, step=1, ky=0.62, kh=1.0)
        d.update(j)
        np.savez(os.path.join(tmp, "job_%d.npz" % i), **{k: np.asarray(v) for k, v in d.items()})
    here = os.path.dirname(os.path.abspath(__file__))
    subprocess.run(["blender", "-b", "--factory-startup", "-P", os.path.join(here, "blrender.py"),
                    "--", tmp, str(len(jobs)), str(samples)],
                   check=True, stdout=subprocess.DEVNULL)
    out = []
    for i in range(len(jobs)):
        a = np.load(os.path.join(tmp, "out_%d.npy" % i))
        out.append(bleed(downsample_premul(a, SS), 3))
    return out
