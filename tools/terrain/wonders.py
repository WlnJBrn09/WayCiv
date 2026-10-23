"""Natural wonder sprites (cell order = WONDERS in terrain.rs), plus one
generic marker (cell 20).  Each is an oblique heightfield scene drawn with
the shared renderer, with a thin golden ring under it so wonders stand out
on the map at a glance."""
import numpy as np
from scipy import ndimage as ndi
from texlib import *
import relief
from features import hexmask, scatter, add_tree

WOFF = -0.3


def cyw():
    return CANVAS / 2 - WOFF * P


def put(H, alb, h, color, mask=None):
    """Raise the scene to h where it is higher (mask optional), painting color."""
    up = h > H
    if mask is not None:
        up &= mask
    H[:] = np.where(up, h, H)
    c = np.asarray(color, np.float32)
    alb[:] = np.where(up[..., None], np.broadcast_to(c, alb.shape) if c.ndim == 1 else c, alb)


def new_scene():
    return np.zeros_like(GX), np.zeros(GX.shape + (3,), np.float32), np.zeros_like(GX)


def cone(cx, cy, R, Ht, pw=1.0, ry=0.85):
    d = np.sqrt(((GX - cx) / R) ** 2 + ((GY - cy) / (R * ry)) ** 2)
    return Ht * np.clip(1 - d, 0, 1) ** pw


def column(cx, cy, R, Ht, soft=0.12, ry=0.9):
    d = np.sqrt(((GX - cx) / R) ** 2 + ((GY - cy) / (R * ry)) ** 2)
    return Ht * (1 - smoothstep(1 - soft, 1.0, d))


def aura(cell):
    """Composite a golden hex ring under the sprite cell (256x256 RGBA)."""
    n = CELL
    yy, xx = np.mgrid[0:n, 0:n].astype(np.float32)
    z = n / (2 * EXT)
    cx, cy = n / 2, n / 2 - WOFF * z
    x, y = (xx - cx) / z, (yy - cy) / z
    d = np.maximum.reduce([np.abs(x), np.abs(x * 0.5 + y * 0.866), np.abs(x * 0.5 - y * 0.866)]) / 0.866
    ring = np.exp(-((d - 0.9) / 0.05) ** 2) * 0.75 + np.exp(-((d - 0.82) / 0.09) ** 2) * 0.25
    glow = np.clip(1 - d, 0, 1) ** 2 * 0.10
    a = np.clip(ring + glow, 0, 1)
    col = np.array([1.0, 0.82, 0.38], np.float32)
    ring_rgba = np.concatenate([np.broadcast_to(col, (n, n, 3)), a[..., None]], -1)
    # over
    ao = cell[..., 3:4]
    ab = ring_rgba[..., 3:4]
    oa = ao + ab * (1 - ao)
    rgb = (cell[..., :3] * ao + ring_rgba[..., :3] * ab * (1 - ao)) / np.maximum(oa, 1e-4)
    return np.concatenate([rgb, oa], -1)


def scene(H, alb, alpha, ky=0.7, kh=1.0, ring=True, **kw):
    args = dict(shadow_len=70, shadow_strength=0.6, ambient=0.54, sun=1.08, hscale=1.25)
    args.update(kw)
    lit = shade(H, alb, **args)
    img = render_height(H, lit, alpha, cyw(), ky=ky, kh=kh)
    cell = finish(img)
    return aura(cell) if ring else cell


def overlay(cell, layer_rgba):
    """Alpha-composite a CELL-sized RGBA float image over `cell`."""
    ao, ab = layer_rgba[..., 3:4], cell[..., 3:4]
    oa = ao + ab * (1 - ao)
    rgb = (layer_rgba[..., :3] * ao + cell[..., :3] * ab * (1 - ao)) / np.maximum(oa, 1e-4)
    return np.concatenate([rgb, oa], -1)


def cell_xy():
    n = CELL
    yy, xx = np.mgrid[0:n, 0:n].astype(np.float32)
    z = n / (2 * EXT)
    return (xx - n / 2) / z, (yy - (n / 2 - WOFF * z)) / z  # ground units, screen y


def ROCKC(h):
    return hexc(h)


# ------------------------------------------------------------------ scenes

def reef(seed=100):
    r = np.random.default_rng(seed)
    H, alb, alpha = new_scene()
    mask = hexmask(0.84, 1.0, 0.5, grid_noise(seed + 1, 2.6))
    alb[:] = hexc("5cd0d0")
    alpha[:] = mask * 0.22
    cols = ["e0508a", "f08a3c", "8c52c8", "34c8b0", "f0d050", "e86a6a"]
    cn = fft_noise(GRID, 1.2, seed + 2)
    ids, d1, d2 = voronoi(GRID, seed + 3, 150)
    cellnoise = np.random.default_rng(seed + 4).uniform(0, 1, 160)[ids]
    for (x, y) in scatter(seed + 5, 9, 0.62, 0.62, 0.34):
        rx, ry = r.uniform(0.18, 0.3), r.uniform(0.14, 0.24)
        d = np.sqrt(((GX - x) / rx) ** 2 + ((GY - y) / ry) ** 2) * (1 + (grid_noise(seed + int(x * 100 + 7), 2.8) - 0.5) * 0.7)
        m = 1 - smoothstep(0.7, 1.0, d)
        c = hexc(cols[r.integers(len(cols))])
        h = m * (0.03 + 0.05 * cellnoise + 0.03 * smoothstep(0, 3, d2 - d1))
        col = c * (0.8 + 0.4 * cellnoise[..., None])
        up = h > H
        H[:] = np.where(up, h, H)
        alb[:] = np.where(up[..., None] & (m > 0.2)[..., None], col, alb)
        alpha[:] = np.maximum(alpha, smoothstep(0.1, 0.5, m) * 0.97)
    return scene(H, alb, alpha, ky=1.0, kh=1.2, shadows=False, ao=0.2)


def dover(seed=101):
    H, alb, alpha = new_scene()
    d = np.sqrt(((GX - 0.0) / 0.78) ** 2 + ((GY - 0.0) / 0.52) ** 2) * (1 + (blurn(grid_noise(seed, 2.8), 6) - 0.5) * 0.5)
    top = 1 - smoothstep(0.9, 1.0, d)
    H = top * 0.5 + top * 0.04 * grid_noise(seed + 1, 2.0)
    H = blurn(H, 2.0)
    grass = ramp(grid_noise(seed + 2, 2.2), [(0, hexc("4f9a3c")), (1, hexc("86c050"))])
    dy, dx = np.gradient(H, STEP)
    slope = np.sqrt(dx * dx + dy * dy)
    chalk = ramp(grid_noise(seed + 3, 1.2), [(0, hexc("e8e6dc")), (1, hexc("fbfaf4"))])
    streak = 0.5 + 0.5 * np.sin(H * 90 + grid_noise(seed + 4, 2.0) * 6)
    chalk = chalk * (0.9 + 0.08 * streak[..., None])
    cl = smoothstep(0.7, 1.6, slope)
    alb = mix(grass, chalk, cl)
    # grey flint bands
    alb = alb * (1 - 0.1 * (streak > 0.93)[..., None] * cl[..., None])
    alpha = smoothstep(0.02, 0.06, H)
    # sea foam at the base
    return scene(H, alb, alpha, ky=0.68, kh=1.0)


def crater(seed=102):
    H, alb, alpha = new_scene()
    d = np.sqrt((GX / 0.78) ** 2 + (GY / 0.6) ** 2)
    nn = blurn(grid_noise(seed, 2.8), 3)
    d = d * (1 + (nn - 0.5) * 0.3)
    rim = np.exp(-((d - 0.72) / 0.26) ** 2) * 0.62 * (0.8 + 0.4 * blurn(ridged(GRID, seed + 1, 2.8), 2.0))
    outer = np.clip(1 - d, 0, 1) ** 0.8 * 0.1
    H = rim + outer
    H = np.where(d < 0.44, 0.07, H)
    H += (grid_noise(seed + 2, 2.6) - 0.5) * 0.02
    rock = ramp(grid_noise(seed + 3, 2.0), [(0, hexc("7a6c60")), (1, hexc("b8a894"))])
    green = ramp(grid_noise(seed + 4, 2.0), [(0, hexc("3e7a38")), (1, hexc("6a9c48"))])
    alb = mix(rock, green, (1 - smoothstep(0.1, 0.32, H)) * 0.9)
    snow = smoothstep(0.42, 0.55, H + (nn - 0.5) * 0.12)
    alb = mix(alb, np.broadcast_to(hexc("f4f8ff"), alb.shape), snow * 0.8)
    # the lake
    lake = 1 - smoothstep(0.40, 0.46, d)
    wcol = ramp(np.clip(1 - d / 0.44, 0, 1), [(0, hexc("123c9a")), (0.5, hexc("1c60c0")), (1, hexc("3c90d4"))])
    alb = alb * (1 - lake[..., None]) + wcol * lake[..., None]
    # Wizard Island
    isl = cone(0.12, 0.02, 0.15, 0.2, 1.1)
    up = isl > H
    H = np.where(up, isl + 0.07, H)
    alb = np.where(up[..., None], mix(hexc("6a5c50"), hexc("4a7a40"), smoothstep(0.02, 0.1, isl) * 0.6)[..., None][..., 0, :] if False else np.broadcast_to(hexc("5a6a48"), alb.shape), alb)
    alpha = smoothstep(0.02, 0.07, H + lake * 0.07)
    return scene(H, alb, alpha, ky=0.66, kh=1.0)


def dead_sea(seed=103):
    H, alb, alpha = new_scene()
    ky = 0.9
    d = np.sqrt(((GX + 0.02) / 0.66) ** 2 + ((GY - 0.05) / (0.4 / ky)) ** 2)
    d = d * (1 + (blurn(grid_noise(seed, 2.8), 5) - 0.5) * 0.55)
    lake = 1 - smoothstep(0.82, 0.92, d)
    crust = (1 - smoothstep(0.92, 1.25, d)) * smoothstep(0.7, 0.9, d)
    sand = ramp(grid_noise(seed + 1, 2.4), [(0, hexc("d8bc82")), (1, hexc("efd9a2"))])
    alb = sand
    water = ramp(np.clip(1 - d, 0, 1), [(0, hexc("3aa6a0")), (0.45, hexc("64c8be")), (1, hexc("a8e6d8"))])
    stripes = 0.5 + 0.5 * np.sin(GX * 14 + grid_noise(seed + 2, 2.4) * 9)
    water = water * (0.94 + 0.08 * stripes[..., None])
    alb = alb * (1 - crust[..., None]) + np.array([0.97, 0.97, 0.94]) * crust[..., None]
    alb = alb * (1 - lake[..., None]) + water * lake[..., None]
    # ridge of arid hills behind
    H = np.maximum(cone(-0.1, -0.5, 0.7, 0.24, 1.2, 0.5), cone(0.45, -0.42, 0.4, 0.2, 1.2, 0.5)) * (1 - lake)
    H = blurn(H, 3)
    alb = np.where((H > 0.02)[..., None], mix(sand, np.broadcast_to(hexc("b89868"), sand.shape), 0.35 * smoothstep(0.02, 0.2, H)), alb)
    alpha = np.maximum(np.maximum(lake, crust), smoothstep(0.02, 0.07, H))
    return scene(H, alb, alpha, ky=ky, kh=1.0)


def everest(seed=104):
    peaks = [(0.0, 0.0, 0.95, 1.5), (-0.55, 0.2, 0.55, 0.85), (0.52, 0.16, 0.6, 0.95)]
    H, E = relief.mountain_height(seed, peaks, jag=1.1)
    alb = relief.mountain_albedo("TUNDRA", H, E, seed, snow_override=(0.22, 1.0))
    alpha = smoothstep(0.012, 0.07, H)
    cell = scene(H, alb, alpha, ky=0.62, kh=1.0, ambient=0.58, shadow_len=100, hscale=1.3)
    # wind-blown plume off the summit
    x, y = cell_xy()
    top_y = -1.5 * 1.0 * 0.9
    plume = np.zeros((CELL, CELL), np.float32)
    for k in range(8):
        px, py = 0.08 + 0.12 * k, -1.35 * 0.98 + 0.03 * k
        plume = np.maximum(plume, np.exp(-(((x - px) / (0.1 + 0.03 * k)) ** 2 + ((y - py) / 0.05) ** 2)) * (0.55 - 0.05 * k))
    rgba = np.concatenate([np.ones((CELL, CELL, 3), np.float32), plume[..., None]], -1)
    return overlay(cell, rgba)


def galapagos(seed=105):
    r = np.random.default_rng(seed)
    H, alb, alpha = new_scene()
    ky = 0.9
    foam = np.zeros_like(H)
    for (x, y, R, Ht) in [(-0.35, 0.05, 0.3, 0.34), (0.3, -0.08, 0.34, 0.42), (0.05, 0.36, 0.22, 0.22)]:
        R2 = R * (1 + 0.25 * (blurn(grid_noise(seed + int(x * 50), 2.8), 4) - 0.5))
        d = np.sqrt(((GX - x) / R) ** 2 + ((GY - y) / (R * 0.8)) ** 2) * (1 + (blurn(grid_noise(seed + int(y * 70) + 3, 2.8), 5) - 0.5) * 0.4)
        h = Ht * np.clip(1 - d, 0, 1) ** 0.8
        h = np.where(d < 0.28, h - (0.28 - d) * 0.35, h)  # crater
        h = np.maximum(h, 0)
        beach = (1 - smoothstep(1.0, 1.3, d)) * smoothstep(0.6, 0.95, d)
        foam = np.maximum(foam, (1 - smoothstep(1.2, 1.55, d)) * smoothstep(0.85, 1.05, d))
        land = h > 0.01
        rock = ramp(grid_noise(seed + 9, 1.8), [(0, hexc("463e3a")), (1, hexc("8a7458"))])
        green = hexc("7a9a48")
        col = mix(rock, np.broadcast_to(green, rock.shape), smoothstep(0.1, 0.9, grid_noise(seed + 10, 2.0)) * smoothstep(0.35, 0.05, h) * 0.8)
        col = np.where((beach > 0.2)[..., None], np.broadcast_to(hexc("ecdca8"), col.shape), col)
        up = (h > H) | ((beach > 0.2) & (H < 0.005))
        H = np.where(h > H, h, H)
        alb = np.where(up[..., None], col, alb)
        alpha = np.maximum(alpha, np.maximum(smoothstep(0.01, 0.05, h), beach))
    alb = np.where((alpha < 0.05)[..., None], np.broadcast_to(hexc("ffffff"), alb.shape), alb)
    alpha = np.maximum(alpha, foam * 0.55)
    return scene(H, alb, alpha, ky=ky, kh=1.0, shadow_len=40)


def kilimanjaro(seed=106):
    H, alb, alpha = new_scene()
    d = np.sqrt((GX / 0.95) ** 2 + (GY / 0.7) ** 2) * (1 + (blurn(grid_noise(seed, 2.6), 4) - 0.5) * 0.15)
    prof = np.clip(1 - d, 0, 1) ** 1.25
    H = np.minimum(prof * 1.35, 0.98)
    H = np.where(d < 0.22, 0.9 - (0.22 - d) * 0.5, H)  # crater
    # Mawenzi: second, jagged peak
    mp, _ = relief.mountain_height(seed + 5, [(-0.62, 0.18, 0.36, 0.7)], jag=1.2)
    H = np.maximum(H, mp)
    H *= 0.92 + 0.12 * blurn(ridged(GRID, seed + 6, 2.8), 2.0)
    H = blurn(H, 1.5)
    n = grid_noise(seed + 7, 2.0)
    savanna = ramp(n, [(0, hexc("b8a050")), (1, hexc("d8c070"))])
    forest = ramp(n, [(0, hexc("2e6c3a")), (1, hexc("4a8a42"))])
    rock = ramp(n, [(0, hexc("6c5c52")), (1, hexc("9a8878"))])
    a = mix(savanna, forest, smoothstep(0.08, 0.2, H))
    a = mix(a, rock, smoothstep(0.42, 0.62, H + (n - 0.5) * 0.1))
    snow = smoothstep(0.68, 0.8, H + (n - 0.5) * 0.12)
    alb = mix(a, np.broadcast_to(hexc("f8fbff"), a.shape), snow)
    alpha = smoothstep(0.01, 0.06, H)
    return scene(H, alb, alpha, ky=0.66, kh=1.0, shadow_len=90)


def pantanal(seed=107):
    r = np.random.default_rng(seed)
    H, alb, alpha = new_scene()
    ky = 0.9
    mask = hexmask(0.82, ky, 0.4, grid_noise(seed, 2.6))
    n = blurn(grid_noise(seed + 1, 3.0), 3)
    pool = smoothstep(0.46, 0.52, n) * mask
    grass = ramp(grid_noise(seed + 2, 1.8), [(0, hexc("4e9a3c")), (0.5, hexc("78bc48")), (1, hexc("a6d85c"))])
    water = ramp(grid_noise(seed + 3, 2.6), [(0, hexc("2f7a9c")), (0.6, hexc("4ea0bc")), (1, hexc("9ad0dc"))])
    alb = grass * (1 - pool[..., None]) + water * pool[..., None]
    # water lilies
    lil = dots(GRID, seed + 4, 90, 3.0) > 0.35
    alb = np.where((lil & (pool > 0.5))[..., None], np.broadcast_to(hexc("f4a0c8"), alb.shape), alb)
    # reeds & trees
    for (x, y) in scatter(seed + 5, 14, 0.7, 0.62, 0.22):
        for k in range(4):
            ox, oy = r.normal(0, 0.03, 2)
            dd = np.sqrt((GX - x - ox) ** 2 + (GY - y - oy) ** 2)
            h = np.where(dd < 0.02, r.uniform(0.08, 0.16) * (1 - dd / 0.02), 0)
            m = h > H
            H = np.where(m, h, H)
            alb = np.where(m[..., None], hexc("8cae48") * (0.85 + 0.3 * (h / 0.16)), alb)
    for (x, y) in scatter(seed + 6, 3, 0.55, 0.5, 0.45):
        add_tree(H, alb, x, y, r.uniform(0.15, 0.19), r.uniform(0.28, 0.34), "dome", hexc("2e8c46"), r)
    # birds: pink and white
    for (x, y) in scatter(seed + 7, 6, 0.55, 0.5, 0.2):
        c = hexc("f290b0") if r.random() < 0.5 else hexc("fafafa")
        for dx in (-0.015, 0.015):
            d = np.sqrt((GX - x - dx) ** 2 + (GY - y) ** 2)
            h = np.where(d < 0.02, 0.1 * (1 - d / 0.02) + 0.03, 0)
            m = h > H
            H = np.where(m, h, H)
            alb = np.where(m[..., None], c, alb)
    alpha = np.maximum(mask * 0.88, (H > 0.01) * 1.0)
    return scene(H, alb, alpha, ky=ky, kh=1.0, shadow_len=30)


def fjord_walls(seed, left_h, right_h, channel_x, channel_w, tilt=0.0, flat_top=False):
    n = blurn(grid_noise(seed, 2.8), 4)
    cx = channel_x + tilt * GY
    d = np.abs(GX - cx) - channel_w * (0.75 + 0.25 * (GY + 1.0) / 2)
    d = d + (n - 0.5) * 0.12
    wall = smoothstep(0.0, 0.06, d)
    side = np.where(GX < cx, left_h, right_h)
    base = side * wall
    if flat_top:
        base = np.minimum(base, 0.62) * (0.96 + 0.04 * n)
    else:
        base = base * (0.7 + 0.5 * blurn(ridged(GRID, seed + 1, 2.6), 2))
    mask = hexmask(0.9, 0.7, 0.2)
    base = base * mask
    return base, d, mask


def piopiotahi(seed=108):
    H, d, mask = fjord_walls(seed, 0.8, 0.7, 0.03, 0.16, tilt=0.12)
    mp = cone(-0.45, -0.3, 0.42, 1.15, 1.1, 0.8)
    H = np.maximum(H, mp * mask)
    H = blurn(H, 1.5)
    n = grid_noise(seed + 2, 2.0)
    green = ramp(n, [(0, hexc("2e6a3a")), (1, hexc("5a8c4a"))])
    rock = ramp(n, [(0, hexc("6a7068")), (1, hexc("a4a8a0"))])
    dy, dx = np.gradient(H, STEP)
    slope = np.sqrt(dx * dx + dy * dy)
    alb = mix(green, rock, smoothstep(0.7, 1.6, slope) * 0.8 + smoothstep(0.55, 0.8, H) * 0.5)
    alb = mix(alb, np.broadcast_to(hexc("f4f8ff"), alb.shape), smoothstep(0.85, 1.0, H) * 0.9)
    water = ramp(grid_noise(seed + 3, 2.4), [(0, hexc("1c6a82")), (1, hexc("3a9ab0"))])
    wm = (1 - smoothstep(0.0, 0.03, d)) * mask
    alb = alb * (1 - wm[..., None]) + water * wm[..., None]
    # waterfall: bright vertical streak on the left wall
    wf = np.exp(-((GX + 0.3) / 0.018) ** 2) * smoothstep(0.1, 0.5, H) * (H > 0.15) * (GY > -0.2) * (GY < 0.3)
    alb = alb * (1 - wf[..., None] * 0.85) + np.array([0.96, 0.99, 1.0]) * wf[..., None] * 0.85
    alpha = np.maximum(smoothstep(0.01, 0.06, H), wm * 0.95)
    return scene(H, alb, alpha, ky=0.7, kh=1.0)


def torres(seed=109):
    r = np.random.default_rng(seed)
    H, alb, alpha = new_scene()
    mask = hexmask(0.86, 0.72, 0.3, grid_noise(seed, 2.6))
    steppe = ramp(grid_noise(seed + 1, 2.2), [(0, hexc("8a8650")), (1, hexc("b0a468"))])
    alb = steppe
    base = cone(0.0, -0.1, 0.9, 0.28, 1.1, 0.7) * mask
    base = blurn(base * (0.8 + 0.4 * grid_noise(seed + 2, 2.6)), 3)
    H = base
    alb = mix(alb, np.broadcast_to(hexc("6a6a58"), alb.shape), smoothstep(0.1, 0.28, base) * 0.5)
    for (x, y, R, Ht) in [(-0.3, -0.05, 0.12, 0.95), (0.03, -0.12, 0.13, 1.15), (0.34, -0.02, 0.11, 0.88)]:
        t = column(x, y, R, Ht, 0.35)
        t = t * (0.97 + 0.03 * np.sin(GX * 90))
        top = t > 0.55 * Ht
        up = t > H
        H = np.where(up, t + base * 0.3, H)
        gran = hexc("c0aaa0") * (0.85 + 0.25 * (0.5 + 0.5 * np.sin(GX * 70 + 3 * x)))[..., None]
        cap = smoothstep(0.78 * Ht, 0.86 * Ht, t)[..., None]
        coln = gran * (1 - cap) + hexc("3a3836") * cap
        # snow dusting on the cap tops
        coln = coln * (1 - 0.0) 
        alb = np.where(up[..., None], coln, alb)
    # milky turquoise lake in front
    d = np.sqrt(((GX - 0.0) / 0.55) ** 2 + ((GY - 0.52) / 0.2) ** 2) * (1 + (blurn(grid_noise(seed + 4, 2.8), 5) - 0.5) * 0.4)
    lake = 1 - smoothstep(0.88, 1.0, d)
    wcol = ramp(np.clip(1 - d, 0, 1), [(0, hexc("46b8c8")), (1, hexc("90e0e0"))])
    alb = alb * (1 - lake[..., None]) + wcol * lake[..., None]
    H = H * (1 - lake)
    alpha = np.maximum(mask * 0.85, np.maximum(lake, smoothstep(0.02, 0.06, H)))
    return scene(H, alb, alpha, ky=0.72, kh=1.0)


def tsingy(seed=110):
    r = np.random.default_rng(seed)
    H, alb, alpha = new_scene()
    ky = 0.8
    mask = hexmask(0.8, ky, 0.4, grid_noise(seed, 2.6))
    alb = ramp(grid_noise(seed + 1, 2.0), [(0, hexc("2a5a34")), (1, hexc("5a8844"))])
    pts = scatter(seed + 2, 95, 0.74, 0.7, 0.12, tries=9000)
    for (x, y) in pts:
        R = r.uniform(0.04, 0.075)
        Ht = r.uniform(0.2, 0.55) * (1.0 - 0.3 * (abs(x) / 0.8))
        h = cone(x, y, R, Ht, 0.8, 0.9)
        up = h > H
        H = np.where(up, h, H)
        c = ramp(np.full(GX.shape, 0.5) + 0.0, [(0, hexc("8c8a82")), (1, hexc("c8c4b6"))]) * 1.0
        alb = np.where(up[..., None], hexc("9a988c") * (0.8 + 0.35 * (h / Ht))[..., None], alb)
    alpha = np.maximum(mask * 0.9, (H > 0.01) * 1.0)
    return scene(H, alb, alpha, ky=ky, kh=1.0, shadow_len=50, ao=0.4)


def yosemite(seed=111):
    r = np.random.default_rng(seed)
    H, d, mask = fjord_walls(seed, 0.9, 0.4, 0.0, 0.2, tilt=0.0)
    # Half Dome on the right: sheer west face, rounded back
    dx = (GX - 0.5) / 0.34
    dy_ = (GY + 0.05) / 0.3
    dome = np.sqrt(np.clip(1 - dx * dx * np.where(dx < 0, 4.0, 1.0) - dy_ * dy_, 0, 1)) * 0.95
    H = np.maximum(H, dome * mask)
    H = blurn(H, 1.2)
    n = grid_noise(seed + 2, 1.8)
    gran = ramp(n, [(0, hexc("9c968a")), (1, hexc("d4cec0"))])
    streak = 0.5 + 0.5 * np.sin(GX * 55 + grid_noise(seed + 3, 2.0) * 8)
    gran = gran * (0.88 + 0.16 * streak[..., None])
    green = ramp(n, [(0, hexc("3c7a38")), (1, hexc("6aa848"))])
    dy2, dx2 = np.gradient(H, STEP)
    slope = np.sqrt(dx2 * dx2 + dy2 * dy2)
    alb = mix(green, gran, smoothstep(0.5, 1.4, slope) * 0.95 + smoothstep(0.25, 0.5, H) * 0.3)
    meadow = ramp(grid_noise(seed + 4, 2.0), [(0, hexc("62a844")), (1, hexc("9acc58"))])
    wm = (1 - smoothstep(0.0, 0.04, d)) * mask
    alb = alb * (1 - wm[..., None]) + meadow * wm[..., None]
    # waterfall on the left wall
    wf = np.exp(-((GX + 0.24) / 0.016) ** 2) * (H > 0.3) * (GY > -0.35) * (GY < 0.25)
    alb = alb * (1 - wf[..., None] * 0.9) + np.array([0.97, 0.99, 1.0]) * wf[..., None] * 0.9
    # valley river
    rv = np.exp(-((GX - 0.02 * np.sin(GY * 6)) / 0.04) ** 2) * wm
    alb = alb * (1 - rv[..., None] * 0.7) + hexc("4a9ec4") * rv[..., None] * 0.7
    for (x, y) in scatter(seed + 5, 9, 0.18, 0.55, 0.2):
        add_tree(H, alb, x + 0.02, y, 0.07, 0.2, "cone", hexc("2a6a42"), r)
    alpha = np.maximum(smoothstep(0.01, 0.06, H), wm * 0.95)
    return scene(H, alb, alpha, ky=0.7, kh=1.0)


def uluru(seed=112):
    H, alb, alpha = new_scene()
    ky = 0.72
    mask = hexmask(0.84, ky, 0.4, grid_noise(seed, 2.6))
    sand = ramp(grid_noise(seed + 1, 2.4), [(0, hexc("d89a60")), (1, hexc("ecc088"))])
    alb = sand
    n = blurn(grid_noise(seed + 2, 2.8), 5)
    d = np.sqrt((GX / 0.7) ** 2 + (GY / 0.3) ** 2) * (1 + (n - 0.5) * 0.35)
    body = 1 - smoothstep(0.86, 1.0, d)
    dome = np.sqrt(np.clip(1 - np.minimum(d, 1) ** 2, 0, 1))
    H = body * (0.28 + 0.26 * dome ** 0.6)
    H = H * (0.96 + 0.04 * np.sin(GX * 38 + n * 6))
    H = blurn(H, 1.2)
    streak = 0.5 + 0.5 * np.sin(GX * 44 + grid_noise(seed + 3, 2.0) * 9)
    rock = ramp(grid_noise(seed + 4, 1.6), [(0, hexc("a8481e")), (0.5, hexc("c8602c")), (1, hexc("e08a48"))])
    rock = rock * (0.85 + 0.2 * streak[..., None])
    dy, dx = np.gradient(H, STEP)
    slope = np.sqrt(dx * dx + dy * dy)
    rock = rock * (1 - 0.35 * smoothstep(0.9, 2.0, slope))[..., None]
    alb = alb * (1 - body[..., None]) + rock * body[..., None]
    # scrub at the foot
    for (x, y) in scatter(seed + 6, 14, 0.7, 0.5, 0.17):
        if np.sqrt((x / 0.62) ** 2 + (y / 0.3) ** 2) > 1.0:
            h = cone(x, y, 0.045, 0.06, 0.9)
            up = h > H
            H = np.where(up, h, H)
            alb = np.where(up[..., None], hexc("6a8a3c"), alb)
    alpha = np.maximum(mask * 0.55, smoothstep(0.01, 0.05, H))
    return scene(H, alb, alpha, ky=ky, kh=1.0, shadow_len=70)


def ha_long(seed=113):
    r = np.random.default_rng(seed)
    H, alb, alpha = new_scene()
    ky = 0.85
    mask = hexmask(0.9, ky, 0.4)
    foam = np.zeros_like(H)
    for (x, y) in scatter(seed + 1, 8, 0.72, 0.62, 0.34):
        R = r.uniform(0.09, 0.16)
        Ht = r.uniform(0.3, 0.7)
        d = np.sqrt(((GX - x) / R) ** 2 + ((GY - y) / (R * 0.85)) ** 2) * (1 + (blurn(grid_noise(seed + int(x * 90) + 5, 2.8), 3) - 0.5) * 0.3)
        body = (1 - smoothstep(0.82, 1.0, d))
        h = body * Ht * (0.9 + 0.2 * np.clip(1 - d, 0, 1))
        top = np.clip((h - 0.8 * Ht) / (0.2 * Ht), 0, 1)
        lime = ramp(grid_noise(seed + 9, 1.6), [(0, hexc("b0a894")), (1, hexc("e0d8c0"))]) * (0.85 + 0.2 * (0.5 + 0.5 * np.sin(GX * 60 + x * 5)))[..., None]
        col = lime * (1 - top[..., None]) + hexc("3a8a46") * top[..., None]
        # waterline stain
        col = col * (1 - 0.3 * (1 - smoothstep(0.0, 0.08, h))[..., None] * (h > 0.005)[..., None])
        up = h > H
        H = np.where(up, h, H)
        alb = np.where(up[..., None], col, alb)
        foam = np.maximum(foam, (1 - smoothstep(1.0, 1.5, d)) * smoothstep(0.75, 1.0, d))
        alpha = np.maximum(alpha, smoothstep(0.01, 0.05, h))
    alb = np.where((alpha < 0.05)[..., None], np.broadcast_to(hexc("ffffff"), alb.shape), alb)
    alpha = np.maximum(alpha, foam * 0.5)
    return scene(H, alb, alpha, ky=ky, kh=1.0, shadow_len=60)


def eyja(seed=114):
    H, alb, alpha = new_scene()
    d = np.sqrt((GX / 0.9) ** 2 + (GY / 0.66) ** 2) * (1 + (blurn(grid_noise(seed, 2.6), 4) - 0.5) * 0.15)
    prof = np.clip(1 - d, 0, 1) ** 1.15
    H = np.minimum(prof * 1.2, 0.8) * (0.9 + 0.2 * blurn(ridged(GRID, seed + 1, 2.8), 2.0))
    H = np.where(d < 0.14, 0.78 - (0.14 - d) * 0.8, H)
    H = blurn(H, 1.5)
    n = grid_noise(seed + 2, 2.0)
    rock = ramp(n, [(0, hexc("34302e")), (1, hexc("6a625c"))])
    ice = ramp(n, [(0, hexc("dbe8f6")), (1, hexc("ffffff"))])
    s = smoothstep(0.34, 0.5, H + (n - 0.5) * 0.15)
    alb = mix(rock, ice, s)
    ash = (1 - smoothstep(0.1, 0.3, H)) * 0.0
    alb = mix(alb, np.broadcast_to(hexc("56503e"), alb.shape), (1 - smoothstep(0.0, 0.14, H)) * 0.6)
    # lava glow at the crater
    lava = np.exp(-(((GX) / 0.07) ** 2 + ((GY) / 0.05) ** 2))
    alb = mix(alb, np.broadcast_to(hexc("ff6a1c"), alb.shape), lava * (d < 0.2))
    alpha = smoothstep(0.01, 0.06, H)
    cell = scene(H, alb, alpha, ky=0.66, kh=1.0, shadow_len=80)
    x, y = cell_xy()
    plume = np.zeros((CELL, CELL), np.float32)
    top_y = -0.8 * 0.9
    for k in range(14):
        t = k / 13
        px, py = 0.0 + 0.55 * t * t + 0.05 * np.sin(k * 2.0), top_y - 0.1 - 0.7 * t
        rad = 0.1 + 0.2 * t
        plume = np.maximum(plume, np.exp(-(((x - px) ** 2 + (y - py) ** 2) / rad ** 2) * 1.3) * (0.78 - 0.35 * t))
    nz = fft_noise(CELL, 2.0, seed + 8)
    col = np.stack([0.32 + 0.3 * nz, 0.3 + 0.28 * nz, 0.3 + 0.28 * nz], -1)
    rgba = np.concatenate([col, (plume * (0.75 + 0.4 * nz))[..., None]], -1)
    rgba[..., 3] = np.clip(rgba[..., 3], 0, 0.92)
    return overlay(cell, rgba)


def lysefjord(seed=115):
    H, d, mask = fjord_walls(seed, 0.7, 0.62, 0.0, 0.15, tilt=-0.18, flat_top=True)
    # Preikestolen: a square platform jutting from the left wall
    pk = (np.abs(GX + 0.2) < 0.1) & (np.abs(GY - 0.12) < 0.09)
    H = np.where(pk, 0.55, H)
    H = blurn(H, 1.2)
    n = grid_noise(seed + 2, 2.0)
    rock = ramp(n, [(0, hexc("7e8078")), (1, hexc("b8b8ac"))])
    green = ramp(n, [(0, hexc("3e7040")), (1, hexc("68985a"))])
    dy, dx = np.gradient(H, STEP)
    slope = np.sqrt(dx * dx + dy * dy)
    alb = mix(green, rock, smoothstep(0.6, 1.4, slope))
    alb = mix(alb, np.broadcast_to(hexc("f0f4fa"), alb.shape), smoothstep(0.55, 0.62, H) * 0.35)
    water = ramp(grid_noise(seed + 3, 2.4), [(0, hexc("1c5a84")), (1, hexc("3a8ab4"))])
    wm = (1 - smoothstep(0.0, 0.03, d)) * mask
    alb = alb * (1 - wm[..., None]) + water * wm[..., None]
    alpha = np.maximum(smoothstep(0.01, 0.06, H), wm * 0.95)
    return scene(H, alb, alpha, ky=0.7, kh=1.0)


def giants(seed=116):
    r = np.random.default_rng(seed)
    H, alb, alpha = new_scene()
    ky = 0.8
    # hexagonal columns on a lattice
    s = 0.105
    x, y = GX, GY / ky * ky
    q = (x * np.sqrt(3) / 3 - y / 3) / s
    rr = (2 / 3 * y) / s
    cx_, cz_ = np.round(q), np.round(rr)
    cy_ = np.round(-q - rr)
    dq, dr, ds = np.abs(cx_ - q), np.abs(cz_ - rr), np.abs(cy_ - (-q - rr))
    fix_x = (dq > dr) & (dq > ds)
    fix_y = (~fix_x) & (ds > dr)
    cx_ = np.where(fix_x, -cy_ - cz_, cx_)
    cz_ = np.where(~fix_x & ~fix_y, -cx_ - cy_, cz_)
    cid = (cx_ * 131 + cz_ * 17).astype(np.int64) & 0xFFFF
    hh = np.random.default_rng(seed).uniform(0, 1, 65536)[cid]
    cxw = s * np.sqrt(3) * (cx_ + cz_ / 2)
    cyw_ = s * 1.5 * cz_
    # distance to cell centre in hex metric -> edges
    ex = np.abs(x - cxw) / (s * 0.866)
    ey = np.abs(y - cyw_) / s
    hexd = np.maximum(ex, (ex * 0.5 + ey * 0.866) / 0.866 * 0.866 + 0.0)
    hexd = np.maximum(ex, ex * 0.5 + ey * 0.866)
    mask = (1 - smoothstep(0.78, 0.9, np.sqrt((GX / 0.78) ** 2 + (GY / 0.55) ** 2)))
    # height profile: highest at the back (north), stepping to sea level
    front = (GY + 0.55) / 1.1
    prof = np.clip(0.42 * (1 - front) ** 1.1 + (hh - 0.5) * 0.08, 0.0, None)
    steps = np.floor(prof / 0.045) * 0.045
    Hh = steps * mask
    edge = smoothstep(0.82, 0.96, hexd)
    top = hexc("7a7676") * (0.85 + 0.3 * hh)[..., None]
    side = hexc("3c3a3c")
    alb = top * (1 - edge[..., None]) + side * edge[..., None]
    alb = alb * np.array([1.0, 0.97, 0.94])
    H = Hh
    alpha = smoothstep(0.005, 0.03, H) * 1.0
    alb = np.where((H < 0.02)[..., None], hexc("5a5a5c"), alb)
    # grass on the cliff top behind
    grass = (GY < -0.38) * mask
    alb = alb * (1 - 0.8 * grass[..., None]) + hexc("5a9a40") * 0.8 * grass[..., None]
    return scene(H, alb, alpha, ky=ky, kh=1.0, shadow_len=40, hscale=1.4, ao=0.2)


def bermuda(seed=117):
    H, alb, alpha = new_scene()
    n = CELL
    x, y = cell_xy()
    ang = np.arctan2(y, x)
    rad = np.sqrt(x * x + y * y) + 1e-3
    nz = fft_noise(n, 2.4, seed)
    nz2 = fft_noise(n, 1.4, seed + 1)
    spiral = np.sin(3 * ang - 6.5 * np.log(rad + 0.15) + nz * 2.6)
    arm = smoothstep(-0.1, 0.9, spiral)
    fade = (1 - smoothstep(0.55, 0.92, rad)) * smoothstep(0.03, 0.18, rad)
    a = np.clip(arm * fade * (0.55 + 0.6 * nz2), 0, 0.88)
    bright = 0.62 + 0.34 * nz2 - 0.2 * (1 - arm)
    col = np.stack([bright * 0.82, bright * 0.9, bright * 0.96], -1)
    # dark teal eye
    eye = np.exp(-(rad / 0.15) ** 2)
    col = col * (1 - eye[..., None] * 0.5) + np.array([0.08, 0.2, 0.28]) * eye[..., None] * 0.5
    a = np.maximum(a, eye * 0.6)
    # faint glowing triangle
    tri = np.zeros((n, n), np.float32)
    pts = [(-0.55, 0.45), (0.55, 0.45), (0.0, -0.6)]
    for (p0, p1) in [(pts[0], pts[1]), (pts[1], pts[2]), (pts[2], pts[0])]:
        e = np.array(p1) - np.array(p0)
        t = np.clip(((x - p0[0]) * e[0] + (y - p0[1]) * e[1]) / (e @ e), 0, 1)
        dist = np.sqrt((x - p0[0] - t * e[0]) ** 2 + (y - p0[1] - t * e[1]) ** 2)
        tri = np.maximum(tri, np.exp(-(dist / 0.022) ** 2))
    tri *= 0.75
    col = col * (1 - tri[..., None]) + np.array([0.6, 0.9, 1.0]) * tri[..., None]
    a = np.maximum(a, tri * 0.9)
    rgba = np.concatenate([col, a[..., None]], -1).astype(np.float32)
    return aura(rgba)


def fountain(seed=118):
    r = np.random.default_rng(seed)
    H, alb, alpha = new_scene()
    ky = 0.8
    mask = hexmask(0.8, ky, 0.45, grid_noise(seed, 2.6))
    lawn = ramp(grid_noise(seed + 1, 2.0), [(0, hexc("4a9c3c")), (1, hexc("86c852"))])
    alb = lawn
    d = np.sqrt((GX / 0.42) ** 2 + (GY / (0.3 / ky)) ** 2)
    ringm = (d < 1.0) & (d > 0.78)
    inner = d <= 0.78
    H = np.where(ringm, 0.07, 0.0) + np.where(inner, 0.025, 0.0)
    alb = np.where(ringm[..., None], hexc("d8d2c4") * (0.9 + 0.2 * grid_noise(seed + 2, 1.5))[..., None], alb)
    water = ramp(np.clip(1 - d / 0.78, 0, 1), [(0, hexc("5ad8f0")), (0.5, hexc("3aa8e0")), (1, hexc("2478c4"))])
    alb = np.where(inner[..., None], water, alb)
    # central column of spray
    for k in range(34):
        t = k / 33
        a = t * 11
        rad = 0.02 + 0.09 * t
        px, py = rad * np.cos(a) * (1 if t < 0.5 else 1.0), 0.0 + rad * np.sin(a) * 0.5
        hh = 0.12 + 0.55 * np.sin(np.pi * min(1, t * 1.15)) ** 0.7
        dd = np.sqrt((GX - px) ** 2 + (GY - py) ** 2)
        h = np.where(dd < 0.025, hh + 0.05 * (1 - dd / 0.025), 0)
        up = h > H
        H = np.where(up, h, H)
        alb = np.where(up[..., None], np.array([0.9, 0.98, 1.0]), alb)
    # flowers and trees around
    for (x, y) in scatter(seed + 3, 22, 0.7, 0.62, 0.15):
        if d[int((y + EXT) / (2 * EXT) * GRID), int((x + EXT) / (2 * EXT) * GRID)] > 1.2:
            c = [hexc("f2c04a"), hexc("f27a9a"), hexc("fafafa")][r.integers(3)]
            dd = np.sqrt((GX - x) ** 2 + (GY - y) ** 2)
            h = np.where(dd < 0.025, 0.04, 0)
            up = h > H
            H = np.where(up, h, H)
            alb = np.where(up[..., None], c, alb)
    for (x, y) in [(-0.55, -0.25), (0.55, -0.2)]:
        add_tree(H, alb, x, y, 0.17, 0.36, "dome", hexc("2e8c46"), r)
    alpha = np.maximum(mask * 0.85, (H > 0.01) * 1.0)
    cell = scene(H, alb, alpha, ky=ky, kh=1.0, shadow_len=40)
    # golden sparkles
    x, y = cell_xy()
    sp = dots(CELL, seed + 9, 26, 1.0)
    sp = np.clip(sp * 3.0, 0, 1) * (np.sqrt((x / 0.8) ** 2 + ((y + 0.2) / 0.9) ** 2) < 1)
    rgba = np.concatenate([np.broadcast_to(np.array([1.0, 0.92, 0.5], np.float32), (CELL, CELL, 3)), sp[..., None]], -1)
    return overlay(cell, rgba)


def paititi(seed=119):
    r = np.random.default_rng(seed)
    H, alb, alpha = new_scene()
    ky = 0.72
    mask = hexmask(0.82, ky, 0.4, grid_noise(seed, 2.6))
    floorc = hexc("1f5a32")
    alb[:] = floorc * (0.8 + 0.4 * grid_noise(seed + 1, 2.4))[..., None]
    # stepped pyramid
    tiers = [(0.58, 0.26, 0.12), (0.46, 0.21, 0.24), (0.34, 0.16, 0.36), (0.22, 0.11, 0.48)]
    stone = hexc("c8a860")
    for (rx, ry, ht) in tiers:
        m = (np.abs(GX) < rx) & (np.abs(GY + 0.05) < ry)
        up = m & (ht > H)
        H = np.where(up, ht, H)
        gold = 0.5 + 0.5 * np.sin(GX * 40 + GY * 7)
        c = stone * (0.85 + 0.15 * gold)[..., None]
        alb = np.where(up[..., None], c, alb)
    # golden shrine on top
    shrine = (np.abs(GX) < 0.09) & (np.abs(GY + 0.05) < 0.07)
    H = np.where(shrine, 0.62, H)
    alb = np.where(shrine[..., None], hexc("f4cc4a"), alb)
    # stair on the front
    stair = (np.abs(GX) < 0.07) & (GY > -0.05 + 0.07) & (GY < 0.26 + 0.0) & (H > 0)
    alb = np.where(stair[..., None], hexc("e8d8a0"), alb)
    # vines / jungle around the base
    for (x, y) in scatter(seed + 3, 14, 0.78, 0.68, 0.2):
        inside = abs(x) < 0.62 and abs(y + 0.05) < 0.3
        if not inside:
            add_tree(H, alb, x, y, r.uniform(0.14, 0.2), r.uniform(0.26, 0.4), "dome", hexc(["1f8a4a", "2a9a52", "168040"][r.integers(3)]), r)
    alpha = np.maximum(mask * 0.75, (H > 0.01) * 1.0)
    return scene(H, alb, alpha, ky=ky, kh=1.0, shadow_len=50)


def generic(seed=120):
    r = np.random.default_rng(seed)
    H, alb, alpha = new_scene()
    ky = 0.85
    mask = hexmask(0.8, ky, 0.5, grid_noise(seed, 2.6))
    alb[:] = ramp(grid_noise(seed + 1, 2.0), [(0, hexc("b8a468")), (1, hexc("e0cc90"))])
    alpha = mask * 0.5
    for k in range(6):
        a = k / 6 * 2 * np.pi
        x, y = 0.42 * np.cos(a), 0.42 * np.sin(a) / ky * 0.8
        h = column(x, y, 0.07, r.uniform(0.35, 0.55), 0.3)
        up = h > H
        H = np.where(up, h, H)
        alb = np.where(up[..., None], hexc("b0a898") * (0.85 + 0.3 * (h / 0.5))[..., None], alb)
    h = cone(0, 0, 0.14, 0.7, 0.8)
    up = h > H
    H = np.where(up, h, H)
    alb = np.where(up[..., None], hexc("f2c64a"), alb)
    alpha = np.maximum(alpha, (H > 0.01) * 1.0)
    return scene(H, alb, alpha, ky=ky, kh=1.0)


SCENES = [reef, dover, crater, dead_sea, everest, galapagos, kilimanjaro, pantanal, piopiotahi,
          torres, tsingy, yosemite, uluru, ha_long, eyja, lysefjord, giants, bermuda, fountain,
          paititi, generic]


def build_sheet(only=None):
    cols, rows = 8, 3
    sheet = np.zeros((rows * CELL, cols * CELL, 4), np.float32)
    for i, fn in enumerate(SCENES):
        if only is not None and i not in only:
            continue
        rr, cc = divmod(i, cols)
        sheet[rr * CELL:(rr + 1) * CELL, cc * CELL:(cc + 1) * CELL] = fn()
        print("  wonder", i, fn.__name__, flush=True)
    return sheet


if __name__ == "__main__":
    import sys
    save_rgba(sys.argv[1], build_sheet())
