"""Hills and mountains: oblique heightfield sprites, 3 variants for each of
grass / plains / desert / tundra / snow.  Cells 0-14 hills, 15-29 mountains."""
import numpy as np
from scipy import ndimage as ndi
import ground
from texlib import *

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


def hills(biome, variant):
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
    lit = shade(H, alb, shadow_len=60, shadow_strength=0.55, ambient=0.40, sun=1.05, hscale=1.4)
    img = render_height(H, lit, alpha, CANVAS / 2 - HILL_OFF * P, ky=0.78, kh=1.0)
    return finish(img)


ROCK = {
    "GRASS": ("8a8474", "c4b8a0", 0.36, 0.95),
    "PLAINS": ("94826a", "ccb690", 0.44, 0.9),
    "DESERT": ("b06a44", "e49c68", 0.80, 0.8),
    "TUNDRA": ("78808a", "b8c0c8", 0.28, 1.0),
    "SNOW": ("a0aec8", "e4eef8", 0.08, 1.0),
}


def peaks_for(variant, r):
    if variant == 0:
        return [(0.0, 0.02, 0.95, 1.12), (-0.52, 0.18, 0.5, 0.6), (0.5, 0.14, 0.54, 0.66)]
    if variant == 1:
        return [(-0.25, 0.0, 0.78, 1.0), (0.32, 0.1, 0.72, 0.9), (0.0, 0.32, 0.42, 0.42)]
    return [(0.08, 0.0, 0.9, 1.08), (-0.46, 0.24, 0.46, 0.52), (0.52, 0.27, 0.4, 0.46)]


def mountain_height(seed, peaks, jag=1.0):
    r1 = blurn(ridged(GRID, seed + 1, 3.0), 2.0)
    r2 = blurn(ridged(GRID, seed + 2, 2.4), 1.5)
    r1 = smoothstep(0.2, 0.95, r1); r2 = smoothstep(0.2, 0.95, r2)
    n = grid_noise(seed + 3, 3.0)
    E = np.zeros_like(GX)
    for (x, y, R, A) in peaks:
        d = np.sqrt(((GX - x) / R) ** 2 + ((GY - y) / (R * 0.82)) ** 2)
        e = A * np.clip(1 - d, 0, 1) ** 1.35
        E = np.maximum(E, e)
    detail = 0.5 + 0.75 * r1 * jag + 0.3 * r2 * jag
    H = E * detail * (0.9 + 0.2 * n)
    H += (grid_noise(seed + 4, 1.6) - 0.5) * 0.025 * smoothstep(0.02, 0.2, E)
    return np.clip(H, 0, None), E


def mountain_albedo(biome, H, E, seed, snow_override=None):
    ca, cb, snowline, snowamt = ROCK[biome]
    if snow_override is not None:
        snowline, snowamt = snow_override
    n = grid_noise(seed + 10, 2.0)
    n2 = grid_noise(seed + 11, 0.9)
    alb = ramp(np.clip(n * 0.7 + n2 * 0.3, 0, 1), [(0, hexc(ca)), (1, hexc(cb))])
    # strata
    band = 0.5 + 0.5 * np.sin(H * 55 + n * 7)
    alb = alb * (0.88 + 0.16 * band[..., None])
    dy, dx = np.gradient(H, STEP)
    slope = np.sqrt(dx * dx + dy * dy)
    # snow cap
    top = E.max()
    sl = snowline * top
    s = smoothstep(sl - 0.08, sl + 0.05, H + (n - 0.5) * 0.18 - slope * 0.10)
    s = s * snowamt
    alb = alb * (1 - s[..., None]) + np.array([0.96, 0.98, 1.0]) * s[..., None]
    # foothills take the colour of the ground
    g = ground_grid(biome, 1000 * (ground.BIOME_NAMES.index(biome) + 1) + 37 * (seed % 3))
    low = 1 - smoothstep(0.04, 0.26, H + (n - 0.5) * 0.1)
    alb = alb * (1 - low[..., None] * 0.85) + g * low[..., None] * 0.85
    return alb


def mountain(biome, variant):
    seed = 7000 + 100 * BIOMES.index(biome) + variant
    r = np.random.default_rng(seed)
    peaks = peaks_for(variant, r)
    H, E = mountain_height(seed, peaks)
    alb = mountain_albedo(biome, H, E, seed)
    alpha = smoothstep(0.012, 0.07, H)
    lit = shade(H, alb, shadow_len=90, shadow_strength=0.6, ambient=0.56, sun=1.1, hscale=1.25)
    cy = CANVAS / 2 - MTN_OFF * P
    img = render_height(H, lit, alpha, cy, ky=0.62, kh=1.0)
    return finish(img)


def build_sheet():
    cols, rows = 8, 4
    sheet = np.zeros((rows * CELL, cols * CELL, 4), np.float32)
    cells = []
    for b in BIOMES:
        for v in range(3):
            cells.append(hills(b, v))
    for b in BIOMES:
        for v in range(3):
            cells.append(mountain(b, v))
    for i, c in enumerate(cells):
        r, cc = divmod(i, cols)
        sheet[r * CELL:(r + 1) * CELL, cc * CELL:(cc + 1) * CELL] = c
    return sheet


if __name__ == "__main__":
    import sys
    save_rgba(sys.argv[1], build_sheet())
