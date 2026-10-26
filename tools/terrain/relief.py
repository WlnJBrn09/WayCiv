"""Hills and mountains: oblique heightfield sprites, 3 variants for each of
grass / plains / desert / tundra / snow.  Cells 0-14 hills, 15-44 mountains
(6 variants per biome)."""
import os
import numpy as np
from scipy import ndimage as ndi
import ground
from texlib import *

MOUNTAINS = 6   # variants per biome
BIOMES = ["GRASS", "PLAINS", "DESERT", "TUNDRA", "SNOW"]
GFN = {"GRASS": ground.grass, "PLAINS": ground.plains, "DESERT": ground.desert,
       "TUNDRA": ground.tundra, "SNOW": ground.snow}

HILL_OFF = 0.0       # sprite centre offset (z units) used by the game
MTN_OFF = -0.55


def ground_grid(biome, seed):
    c = GFN[biome](seed)
    z = ndi.zoom(c, (GRID / c.shape[0], GRID / c.shape[1], 1), order=1)
    return z.astype(np.float32)


def gauss(cx, cy, sx, sy, a):
    return a * np.exp(-(((GX - cx) / sx) ** 2 + ((GY - cy) / sy) ** 2))


def hills_job(biome, variant):
    seed = 5000 + 100 * BIOMES.index(biome) + variant
    r = np.random.default_rng(seed)
    alb = ground_grid(biome, 1000 * (ground.BIOME_NAMES.index(biome) + 1) + 37 * variant)
    alb = ndi.gaussian_filter(alb, (1.3, 1.3, 0))
    n1 = grid_noise(seed + 1, 2.6)
    n2 = grid_noise(seed + 2, 1.4)
    H = np.zeros_like(GX)
    k = [3, 2, 3][variant]
    if variant == 0:
        spots = [(-0.30, 0.08, 0.50, 0.42, 0.46), (0.36, -0.04, 0.46, 0.38, 0.40), (0.02, 0.40, 0.42, 0.28, 0.26)]
    elif variant == 1:
        spots = [(0.0, -0.02, 0.66, 0.52, 0.54), (-0.5, 0.3, 0.34, 0.26, 0.25)]
    else:
        spots = [(-0.38, -0.08, 0.42, 0.36, 0.36), (0.32, 0.14, 0.52, 0.42, 0.46), (0.0, -0.36, 0.36, 0.26, 0.26)]
    for (x, y, sx, sy, a) in spots:
        H = np.maximum(H, gauss(x, y, sx, sy, a)) * 0.0 + H + gauss(x, y, sx, sy, a) * 0.0
        H = np.maximum(H, gauss(x, y, sx, sy, a))
    n3 = grid_noise(seed + 9, 3.6)
    H = H * (0.88 + 0.24 * n1) + (n3 - 0.5) * 0.07 * smoothstep(0.02, 0.2, H)
    H = blurn(np.clip(H, 0, None), 4.0)
    alpha = smoothstep(0.02, 0.045, H)
    # texture variation on the slopes: a little darker rock / earth
    dy, dx = np.gradient(H, STEP)
    slope = np.sqrt(dx * dx + dy * dy)
    if biome == "GRASS":
        tint = np.array([0.62, 0.62, 0.45]); amt = 0.2
    elif biome == "PLAINS":
        tint = np.array([0.7, 0.62, 0.4]); amt = 0.25
    elif biome == "DESERT":
        tint = np.array([0.78, 0.55, 0.36]); amt = 0.35
    elif biome == "TUNDRA":
        tint = np.array([0.5, 0.52, 0.5]); amt = 0.35
    else:
        tint = np.array([0.72, 0.8, 0.94]); amt = 0.3
    rock = smoothstep(0.45, 1.1, slope) * (0.5 + 0.5 * n2)
    alb = alb * (1 - (rock * amt)[..., None]) + tint * (rock * amt)[..., None]
    # slightly lighter crown
    alb = alb * (1 + 0.12 * smoothstep(0.1, 0.35, H)[..., None])
    return dict(H=H.astype(np.float32), rgb=alb.astype(np.float32), alpha=alpha.astype(np.float32),
                cy=CANVAS / 2 - HILL_OFF * P, ky=0.78, kh=1.3, rough=0.95, spec=0.1, sun=4.2, sky=0.6, step=2)


ROCK = {
    # (shadow rock, lit rock, snowline as a fraction of the peak, snow amount)
    "GRASS": ("6e675c", "b4a894", 0.40, 0.97),
    "PLAINS": ("7a6a54", "c2ae8a", 0.44, 0.95),
    "DESERT": ("9a5a3c", "e0976a", 0.92, 0.6),
    "TUNDRA": ("5e666e", "aab4be", 0.28, 1.0),
    "SNOW": ("8c9cb8", "e0eaf6", 0.10, 1.0),
}


def peaks_for(variant, r):
    return [
        [(0.0, 0.02, 0.95, 1.12), (-0.52, 0.18, 0.5, 0.6), (0.5, 0.14, 0.54, 0.66)],
        [(-0.25, 0.0, 0.78, 1.0), (0.32, 0.1, 0.72, 0.9), (0.0, 0.32, 0.42, 0.42)],
        [(0.08, 0.0, 0.9, 1.08), (-0.46, 0.24, 0.46, 0.52), (0.52, 0.27, 0.4, 0.46)],
        [(0.0, 0.05, 0.72, 1.28)],
        [(-0.58, 0.12, 0.5, 0.78), (0.0, 0.0, 0.62, 1.1), (0.58, 0.14, 0.5, 0.74)],
        [(-0.22, 0.1, 0.88, 0.86), (0.38, 0.16, 0.6, 0.7), (-0.5, 0.34, 0.36, 0.4)],
    ][variant]


def mountain_height(seed, peaks):
    """Pyramidal peaks with radial arêtes, eroded by ridged detail that
    grows with altitude and is softened so no single-pixel spikes remain."""
    r = np.random.default_rng(seed)
    X = GX + (grid_noise(seed + 1, 3.2) - 0.5) * 0.30
    Y = GY + (grid_noise(seed + 2, 3.2) - 0.5) * 0.30
    E = np.zeros_like(GX)
    for (x, y, R, A) in peaks:
        dx, dy = (X - x) / R, (Y - y) / (R * 0.85)
        d = np.sqrt(dx * dx + dy * dy)
        th = np.arctan2(dy, dx)
        k = int(r.integers(3, 6))
        wob = grid_noise(seed + 5 + k, 3.0)
        arete = 0.5 + 0.5 * np.cos(k * th + r.uniform(0, 6.28) + 3.0 * wob)
        prof = np.clip(1 - d, 0, 1)
        e = A * prof ** 1.15 * (0.62 + 0.55 * arete * (1 - 0.4 * prof))
        E = np.maximum(E, e) * 0.0 + np.power(np.power(E, 6) + np.power(e, 6), 1 / 6)
    lo = smoothstep(0.25, 0.95, blurn(ridged(GRID, seed + 3, 3.6), 3.5))
    mid = smoothstep(0.25, 0.95, blurn(ridged(GRID, seed + 4, 3.0), 2.2))
    hi = smoothstep(0.30, 0.95, blurn(ridged(GRID, seed + 6, 2.4), 1.4))
    detail = 0.60 + 0.50 * lo + 0.22 * mid + 0.07 * hi
    H = E * detail * smoothstep(0.0, 0.25, E) + E * 0.62 * (1 - smoothstep(0.0, 0.25, E))
    H = blurn(H, 1.6)
    return np.clip(H, 0, None), E


def lap_norm(H, s):
    lap = ndi.laplace(blurn(H, s))
    return lap / (np.percentile(np.abs(lap), 97) + 1e-9)


def mountain_albedo(biome, H, E, seed):
    ca, cb, snowline, snowamt = ROCK[biome]
    n = grid_noise(seed + 10, 2.0)
    n2 = grid_noise(seed + 11, 0.9)
    rock = ramp(np.clip(n * 0.65 + n2 * 0.35, 0, 1), [(0, hexc(ca)), (1, hexc(cb))])
    band = 0.5 + 0.5 * np.sin(H * 42 + n * 6)
    rock = rock * (0.90 + 0.14 * band[..., None])
    lap = lap_norm(H, 1.6)
    conc = np.clip(lap, 0, 1)      # gullies
    conv = np.clip(-lap, 0, 1)     # ridges and crests
    rock = rock * (1 - 0.30 * conc[..., None]) * (1 + 0.18 * conv[..., None])
    dy, dx = np.gradient(H, STEP)
    slope = np.sqrt(dx * dx + dy * dy)
    top = max(E.max(), 0.2)
    sl = snowline * top
    s = smoothstep(sl - 0.10, sl + 0.07, H + (n - 0.5) * 0.24 + conc * 0.10 - conv * 0.04 - slope * 0.07)
    s = np.clip(s * snowamt, 0, 1)
    snow = np.array([0.97, 0.985, 1.0]) * (0.9 + 0.1 * grid_noise(seed + 12, 1.6))[..., None]
    alb = rock * (1 - s[..., None]) + snow * s[..., None]
    # foothills take the colour of the ground, with a belt of dark trees
    g = ground_grid(biome, 1000 * (ground.BIOME_NAMES.index(biome) + 1) + 37 * (seed % 3))
    low = 1 - smoothstep(0.05, 0.30, H + (n - 0.5) * 0.12)
    alb = alb * (1 - low[..., None] * 0.9) + g * low[..., None] * 0.9
    if biome in ("GRASS", "PLAINS", "TUNDRA"):
        belt = smoothstep(0.03, 0.09, H) * (1 - smoothstep(0.16, 0.34, H + (n - 0.5) * 0.1)) * (1 - smoothstep(0.7, 1.4, slope))
        trees = np.clip(dots(GRID, seed + 13, 2600, 1.6) * 3.0, 0, 1) * belt
        tc = np.array([0.13, 0.30, 0.16]) if biome != "TUNDRA" else np.array([0.14, 0.26, 0.22])
        alb = alb * (1 - trees[..., None] * 0.8) + tc * (trees[..., None] * 0.8)
    return alb


def mountain_job(biome, variant):
    seed = 7000 + 100 * BIOMES.index(biome) + 7 * variant
    r = np.random.default_rng(seed)
    peaks = peaks_for(variant, r)
    H, E = mountain_height(seed, peaks)
    alb = mountain_albedo(biome, H, E, seed)
    alpha = smoothstep(0.006, 0.05, H)
    return dict(H=H.astype(np.float32), rgb=alb.astype(np.float32), alpha=alpha.astype(np.float32),
                cy=CANVAS / 2 - MTN_OFF * P, ky=0.62, kh=1.0, rough=0.9, spec=0.15)


def build_sheet():
    cols, rows = 8, 6
    sheet = np.zeros((rows * CELL, cols * CELL, 4), np.float32)
    jobs = []
    for b in BIOMES:
        for v in range(3):
            jobs.append(hills_job(b, v))
    for b in BIOMES:
        for v in range(MOUNTAINS):
            jobs.append(mountain_job(b, v))
    first = 0
    if os.environ.get("RELIEF_MOUNTAINS_ONLY"):
        # quick iteration: keep the hills already in the shipped sheet
        from PIL import Image
        old = os.path.join(os.path.dirname(__file__), "..", "..", "assets", "terrain", "relief.png")
        sheet[:] = np.asarray(Image.open(old), np.float32) / 255.0
        first = 15
    for i, c in enumerate(render_many(jobs[first:]), first):
        r, cc = divmod(i, cols)
        sheet[r * CELL:(r + 1) * CELL, cc * CELL:(cc + 1) * CELL] = c
    return sheet


if __name__ == "__main__":
    import sys
    save_rgba(sys.argv[1], build_sheet())
