"""Feature sprites (forest, rainforest, taiga, marsh, floodplains, dunes,
glacier, oasis, sea ice).  Cell layout (8 columns):
 0-2 forest  3-5 jungle  6-8 taiga  9-11 marsh  12-13 floodplains (desert)
 14-15 floodplains (grass)  16-17 floodplains (plains)  18-20 dunes
 21-23 glacier  24-25 oasis  26-28 sea ice"""
import numpy as np
from texlib import *

FEAT_OFF = -0.12     # sprite centre offset (z units) used by the game


def hexmask(R, ky, soft=0.15, wob=None, wob_amp=0.35):
    x = GX
    y = GY * ky
    d = np.maximum.reduce([np.abs(x), np.abs(x * 0.5 + y * 0.866), np.abs(x * 0.5 - y * 0.866)]) / (0.866 * R)
    if wob is not None:
        d = d * (1 + (wob - 0.5) * wob_amp)
    return 1 - smoothstep(1 - soft, 1.0, d)


def cy_feat():
    return CANVAS / 2 - FEAT_OFF * P


def scatter(seed, n, rx, ry, min_d, tries=4000):
    r = np.random.default_rng(seed)
    pts = []
    for _ in range(tries):
        x, y = r.uniform(-rx, rx), r.uniform(-ry, ry)
        if (x / rx) ** 2 + (y / ry) ** 2 > 1:
            continue
        if all((x - a) ** 2 + (y - b) ** 2 > min_d ** 2 for a, b, *_ in pts):
            pts.append((x, y))
        if len(pts) >= n:
            break
    pts.sort(key=lambda p: p[1])
    return pts


def add_tree(H, alb, x, y, R, Ht, kind, col, r, snow=0.0):
    d2 = ((GX - x) / R) ** 2 + ((GY - y) / (R * 0.9)) ** 2
    m = d2 < 1.0
    t = np.sqrt(np.where(m, d2, 1.0))
    if kind == "cone":
        h = Ht * (1 - t)
        saw = (h / Ht * 5.5) % 1.0
        h = h * (0.88 + 0.12 * saw)
    else:
        h = Ht * np.power(np.clip(1 - d2, 0, 1), 0.55)
        bump = fft_noise(GRID, 2.0, int(r.integers(1, 1 << 30)))
        h = h * (0.9 + 0.2 * bump)
    h = np.where(m, h, 0)
    up = h > H
    H[:] = np.where(up, h, H)
    c = np.asarray(col, np.float32)
    shade_v = (0.82 + 0.35 * (h / Ht))[..., None]
    treec = np.broadcast_to(c, alb.shape) * shade_v
    if snow > 0:
        sm = smoothstep(0.55, 0.9, h / Ht)[..., None] * snow
        treec = treec * (1 - sm) + np.array([0.93, 0.96, 1.0]) * sm
    alb[:] = np.where((up & m)[..., None], treec, alb)


def forest_like(kind, variant, seed):
    r = np.random.default_rng(seed)
    H = np.zeros_like(GX)
    alb = np.zeros(GX.shape + (3,), np.float32)
    ky = 0.8
    if kind == "forest":
        pts = scatter(seed, 13, 0.72, 0.62 / ky * 0.9, 0.27)
        palette = ["2f7a3a", "3c8a3e", "4c9a42", "5ea84a", "2a6a36"]
        floorc = "2c5a2a"
    elif kind == "jungle":
        pts = scatter(seed, 15, 0.74, 0.66 / ky * 0.9, 0.25)
        palette = ["1f8a4a", "2a9a52", "3aa858", "168040", "56b05c"]
        floorc = "17502c"
    else:
        pts = scatter(seed, 20, 0.76, 0.68 / ky * 0.9, 0.2)
        palette = ["2a6a5a", "336f5c", "245a4c", "3c7a62"]
        floorc = "1c4640"
    floor = hexmask(0.78, ky, 0.4, grid_noise(seed + 50, 2.6)) * 0.62
    fcol = np.broadcast_to(hexc(floorc), alb.shape) * (0.8 + 0.4 * grid_noise(seed + 51, 2.4))[..., None]
    alb[:] = fcol
    for (x, y) in pts:
        if kind == "forest":
            if r.random() < 0.3:
                add_tree(H, alb, x, y, r.uniform(0.11, 0.14), r.uniform(0.34, 0.42), "cone", hexc("2a6a48"), r)
            else:
                add_tree(H, alb, x, y, r.uniform(0.15, 0.2), r.uniform(0.28, 0.36), "dome", hexc(palette[r.integers(len(palette))]), r)
        elif kind == "jungle":
            add_tree(H, alb, x, y, r.uniform(0.17, 0.24), r.uniform(0.3, 0.42), "dome", hexc(palette[r.integers(len(palette))]), r)
        else:
            add_tree(H, alb, x, y, r.uniform(0.1, 0.13), r.uniform(0.38, 0.52), "cone", hexc(palette[r.integers(len(palette))]), r, snow=0.35)
    if kind == "jungle":
        fl = dots(GRID, seed + 60, 60, 1.6)
        alb[:] = alb * (1 - (fl[..., None] > 0.5) * 0.7) + np.array([0.95, 0.35, 0.45]) * ((fl[..., None] > 0.5) * 0.7) * (H[..., None] > 0.1)
    alpha = np.maximum(smoothstep(0.015, 0.05, H), floor)
    lit = shade(H, alb, shadow_len=40, shadow_strength=0.6, ambient=0.5, sun=1.0, hscale=1.2)
    img = render_height(H, lit, alpha, cy_feat(), ky=ky, kh=1.0)
    return finish(img)


def marsh(variant, seed):
    r = np.random.default_rng(seed)
    ky = 0.9
    H = np.zeros_like(GX)
    mask = hexmask(0.8, ky, 0.5, grid_noise(seed + 1, 2.6))
    n = grid_noise(seed + 2, 3.0)
    pool = smoothstep(0.44, 0.50, n) * mask
    mud = hexc("5e6a3a")
    grassc = hexc("6a8a3e")
    alb = np.broadcast_to(mud, GX.shape + (3,)).copy()
    gn = grid_noise(seed + 3, 1.8)
    alb = mix(alb, grassc, smoothstep(0.4, 0.7, gn))
    water = ramp(grid_noise(seed + 4, 2.6), [(0, hexc("3f7e98")), (0.6, hexc("56a0b4")), (1, hexc("9ccdd6"))])
    alb = alb * (1 - pool[..., None]) + water * pool[..., None]
    # reeds / tussocks
    for (x, y) in scatter(seed + 7, 18, 0.7, 0.6, 0.2):
        R = r.uniform(0.04, 0.07)
        d = np.sqrt((GX - x) ** 2 + ((GY - y) / 0.9) ** 2)
        for k in range(5):
            ox, oy = r.normal(0, 0.03, 2)
            dd = np.sqrt((GX - x - ox) ** 2 + ((GY - y - oy)) ** 2)
            h = np.where(dd < 0.02, (r.uniform(0.07, 0.15)) * (1 - dd / 0.02), 0)
            m = h > H
            H = np.where(m, h, H)
            c = hexc("a8b04e") if r.random() < 0.6 else hexc("6a8e3a")
            alb = np.where(m[..., None], c * (0.85 + 0.3 * (h / 0.15))[..., None], alb)
    alpha = np.maximum(mask * 0.85, pool)
    lit = shade(H, alb, shadow_len=30, shadow_strength=0.5, ambient=0.55, sun=0.9)
    # sky glint on the water
    gl = smoothstep(0.72, 0.9, grid_noise(seed + 9, 1.2)) * pool
    lit = lit * (1 - gl[..., None] * 0.5) + np.array([0.95, 0.98, 1.0]) * gl[..., None] * 0.5
    img = render_height(H, lit, alpha, cy_feat(), ky=ky, kh=1.0)
    return finish(img)


def floodplain(kind, variant, seed):
    ky = 1.0
    r = np.random.default_rng(seed)
    mask = hexmask(0.84, ky, 0.4, grid_noise(seed + 1, 2.6))
    warp = grid_noise(seed + 2, 2.8)
    ang = [0.35, -0.3][variant % 2]
    u = (GX * np.cos(ang) + GY * np.sin(ang))
    v = (-GX * np.sin(ang) + GY * np.cos(ang))
    ph = v * 7.5 + warp * 3.2
    s = 0.5 + 0.5 * np.sin(ph)
    chan = smoothstep(0.88, 0.97, s)
    fld = grid_noise(seed + 3, 2.0)
    if kind == "desert":
        a, b, bank, w = hexc("5c9a44"), hexc("8cbc54"), hexc("b09a68"), hexc("4a9cb8")
    elif kind == "grass":
        a, b, bank, w = hexc("6aa840"), hexc("a0c24e"), hexc("9a8a58"), hexc("4a98b4")
    else:
        a, b, bank, w = hexc("a8a24e"), hexc("d0c070"), hexc("9a8458"), hexc("4a94b0")
    alb = mix(np.broadcast_to(a, GX.shape + (3,)), b, smoothstep(0.3, 0.8, fld * 0.6 + s * 0.4))
    alb = mix(alb, bank, smoothstep(0.78, 0.9, s) * 0.5)
    alb = mix(alb, w, chan * 0.0)
    alb = alb * (1 - 0.10 * chan[..., None])
    # field plots: faint darker rectangles
    plots = 0.5 + 0.5 * np.sin(u * 9 + warp * 4)
    alb = alb * (0.92 + 0.12 * plots[..., None])
    strength = 1.0 if kind == "desert" else 0.78
    H = chan * -0.012 + (1 - chan) * 0.0
    lit = shade(H + s * 0.01, alb, shadows=False, ao=0, ambient=0.5, sun=0.8)
    alpha = mask * strength * 0.92
    img = render_height(np.zeros_like(H), lit, alpha, cy_feat(), ky=ky, kh=1.0)
    return finish(img)


def dunes(variant, seed):
    ky = 0.85
    mask = hexmask(0.86, ky, 0.35, grid_noise(seed + 1, 2.6))
    warp = blurn(grid_noise(seed + 2, 3.0), 10)
    warp2 = blurn(grid_noise(seed + 3, 2.0), 6)
    ang = [0.55, -0.45, 1.0][variant]
    ph = (GX * np.cos(ang) + GY * np.sin(ang)) * 2 * np.pi * 1.25 + (warp - 0.5) * 5.0 + (warp2 - 0.5) * 1.5
    s = np.sin(ph)
    H = (0.055 * (s + 0.35 * np.sin(2 * ph + 1.0)) + 0.07) * mask * smoothstep(0.0, 0.6, mask)
    H = blurn(H, 2.0)
    H *= 0.7 + 0.6 * blurn(grid_noise(seed + 4, 2.4), 5)
    alb = ramp(grid_noise(seed + 5, 2.4), [(0, hexc("e0bd78")), (0.5, hexc("ecd198")), (1, hexc("f6e4b6"))])
    # crests lighter
    crest = smoothstep(0.7, 1.0, s)
    alb = alb * (1 + 0.1 * crest[..., None])
    lit = shade(H, alb, shadow_len=40, shadow_strength=0.45, ambient=0.52, sun=1.0, hscale=2.2, ao=0.1)
    alpha = mask * 0.9
    img = render_height(H, lit, alpha, cy_feat(), ky=ky, kh=1.0)
    return finish(img)


def glacier(variant, seed):
    ky = 0.9
    mask = hexmask(0.8, ky, 0.45, blurn(grid_noise(seed + 1, 2.6), 6), wob_amp=1.1)
    ids, d1, d2 = voronoi(GRID, seed + 2, 12 + 4 * variant)
    base = blurn(grid_noise(seed + 5, 3.0), 7)
    # broken ice: most seams are shallow, a few are deep blue crevasses
    r = np.random.default_rng(seed)
    deep = r.uniform(0, 1, 64)[ids] < 0.35
    seam = smoothstep(7, 0, d2 - d1)
    H = (0.05 + 0.1 * base) * mask
    H = H - seam * 0.03 * mask
    H = blurn(H, 1.5)
    alb = ramp(grid_noise(seed + 4, 2.4), [(0, hexc("e2f0fa")), (0.6, hexc("f4faff")), (1, hexc("ffffff"))])
    alb = mix(alb, hexc("b4d8f0"), smoothstep(0.55, 0.2, base) * 0.45)
    alb = mix(alb, hexc("5aa0d4"), seam * deep * 0.8)
    alb = mix(alb, hexc("9cc8e8"), seam * (~deep) * 0.5)
    lit = shade(H, alb, shadow_len=36, shadow_strength=0.35, ambient=0.6, sun=0.9, hscale=3.0, ao=0.1)
    img = render_height(H, np.clip(lit, 0, 1.05), mask, cy_feat(), ky=ky, kh=1.0)
    return finish(img)


def palm(H, alb, x, y, r, Ht):
    # trunk
    for k in range(10):
        t = k / 9
        px, py = x + 0.05 * np.sin(t * 2), y - 0.0
        d = np.sqrt((GX - px - 0.04 * t) ** 2 + (GY - py) ** 2)
        h = np.where(d < 0.018, Ht * t, 0)
        m = h > H
        H[:] = np.where(m, h, H)
        alb[:] = np.where(m[..., None], hexc("8a6a44") * (0.8 + 0.3 * t), alb)
    cx, cyy = x + 0.04 + 0.05 * np.sin(2), y
    for a in np.linspace(0, 2 * np.pi, 9, endpoint=False) + r.uniform(0, 1):
        L = r.uniform(0.17, 0.23)
        dx, dy = np.cos(a), np.sin(a) * 0.8
        px = (GX - cx) * dx + (GY - cyy) * dy / 0.8 * 0.8
        t = ((GX - cx) * dx + (GY - cyy) * dy) / (L * (dx * dx + dy * dy))
        perp = np.abs(-(GX - cx) * dy + (GY - cyy) * dx) / np.sqrt(dx * dx + dy * dy)
        m = (t > 0) & (t < 1) & (perp < 0.017 * (1 - t * 0.5))
        h = Ht - 0.26 * t * t + 0.03
        up = m & (h > H)
        H[:] = np.where(up, h, H)
        alb[:] = np.where(up[..., None], hexc("3a9a3c") * (0.85 + 0.3 * (1 - t))[..., None], alb)


def oasis(variant, seed):
    r = np.random.default_rng(seed)
    ky = 0.9
    H = np.zeros_like(GX)
    mask = hexmask(0.78, ky, 0.5, grid_noise(seed + 1, 2.6))
    px, py = [(-0.05, 0.12), (0.06, 0.1)][variant]
    d = np.sqrt(((GX - px) / 0.44) ** 2 + ((GY - py) / (0.30 / 0.9)) ** 2)
    d *= 1 + (grid_noise(seed + 2, 3.0) - 0.5) * 0.4
    pond = 1 - smoothstep(0.85, 1.0, d)
    lush = (1 - smoothstep(0.9, 1.9, d)) * mask
    alb = np.broadcast_to(hexc("e0c47c"), GX.shape + (3,)).copy()
    alb = mix(alb, hexc("5aa044"), lush * (0.55 + 0.45 * grid_noise(seed + 3, 1.8)))
    shore = smoothstep(0.7, 1.05, d) * (1 - smoothstep(1.05, 1.25, d))
    alb = mix(alb, hexc("d8c488"), shore * 0.7)
    water = ramp(np.clip(1 - d, 0, 1), [(0, hexc("2c82b4")), (0.35, hexc("46a8d0")), (1, hexc("8ad4e0"))])
    alb = alb * (1 - pond[..., None]) + water * pond[..., None]
    shift = [(-0.38, -0.12), (0.4, -0.05), (0.2, -0.32), (-0.2, -0.36)]
    for (x, y) in shift[:3 + variant]:
        palm(H, alb, x, y, r, r.uniform(0.34, 0.42))
    lit = shade(H, alb, shadow_len=40, shadow_strength=0.5, ambient=0.55, sun=1.0, hscale=1.3)
    alpha = np.maximum(mask * 0.7, np.maximum(pond, (H > 0.01) * 1.0))
    img = render_height(H, lit, alpha, cy_feat(), ky=ky, kh=1.0)
    return finish(img)


def sea_ice(variant, seed):
    ky = 1.0
    r = np.random.default_rng(seed)
    wob = blurn(grid_noise(seed + 2, 2.8), 3)
    floe = np.zeros_like(GX, bool)
    for (x, y) in scatter(seed + 5, 6 + variant, 0.62, 0.62, 0.42):
        rx, ry = r.uniform(0.22, 0.38), r.uniform(0.18, 0.3)
        d = np.sqrt(((GX - x) / rx) ** 2 + ((GY - y) / ry) ** 2) * (1 + (wob - 0.5) * 0.7)
        floe |= d < 1
    fl = blurn(floe.astype(np.float32), 1.6)
    alpha = smoothstep(0.35, 0.6, fl)
    edge = smoothstep(0.0, 14.0, ndi_dist(floe))
    H = alpha * (0.05 + 0.04 * edge) + (grid_noise(seed + 3, 2.4) - 0.5) * 0.02 * alpha
    alb = ramp(grid_noise(seed + 4, 2.4), [(0, hexc("dcecf8")), (0.6, hexc("f0f8ff")), (1, hexc("ffffff"))])
    alb = mix(alb, hexc("8cc8e6"), (1 - edge) * 0.55)
    lit = shade(H, alb, shadow_len=30, shadow_strength=0.35, ambient=0.6, sun=0.85, hscale=1.6, ao=0.2)
    img = render_height(H, lit, alpha, cy_feat(), ky=ky, kh=1.0)
    return finish(img)


def ndi_dist(mask):
    from scipy import ndimage as ndi
    return ndi.distance_transform_edt(mask).astype(np.float32)


def build_sheet():
    cols, rows = 8, 4
    sheet = np.zeros((rows * CELL, cols * CELL, 4), np.float32)
    cells = []
    for v in range(3):
        cells.append(forest_like("forest", v, 11000 + v))
    for v in range(3):
        cells.append(forest_like("jungle", v, 12000 + v))
    for v in range(3):
        cells.append(forest_like("taiga", v, 13000 + v))
    for v in range(3):
        cells.append(marsh(v, 14000 + v))
    for k in ("desert", "grass", "plains"):
        for v in range(2):
            cells.append(floodplain(k, v, 15000 + 10 * v + len(k)))
    for v in range(3):
        cells.append(dunes(v, 16000 + v))
    for v in range(3):
        cells.append(glacier(v, 17000 + v))
    for v in range(2):
        cells.append(oasis(v, 18000 + v))
    for v in range(3):
        cells.append(sea_ice(v, 19000 + v))
    for i, c in enumerate(cells):
        rr, cc = divmod(i, cols)
        sheet[rr * CELL:(rr + 1) * CELL, cc * CELL:(cc + 1) * CELL] = c
    return sheet


if __name__ == "__main__":
    import sys
    save_rgba(sys.argv[1], build_sheet())
