#!/usr/bin/env python3
"""Builds WayCiv's Earth maps from real elevation and climate data.

Inputs (10 arc-minute grids, 1080 x 2160):
  etopo10.npy           ETOPO1 elevation/bathymetry in meters, south row first
                        (NOAA, public domain; fetched with OPeNDAP stride 10)
  wc2.1_10m_bio_1.tif   WorldClim 2.1 annual mean temperature (C), north first
  wc2.1_10m_bio_10.tif  WorldClim 2.1 mean temperature of the warmest quarter
  wc2.1_10m_bio_12.tif  WorldClim 2.1 annual precipitation (mm)

Usage: build_earth.py <data dir> <out dir> [width ...]

Writes earth_<W>x<H>.txt for each width: a header line, then H rows each of
terrain codes, feature codes, and flags (see crates/wayciv-core/src/earth.rs).
Also writes a preview PNG next to it (needs ImageMagick).
"""
import heapq
import math
import subprocess
import sys

import numpy as np
import tifffile

LAT_TOP, LAT_BOTTOM = 84.0, -78.0
# The map's west edge: the Bering Strait, so the wrap seam falls in the Pacific.
LON_WEST = -169.0
N, M = 1080, 2160


def miller(lat_deg):
    return 1.25 * np.log(np.tan(np.pi / 4 + 0.4 * np.radians(lat_deg)))


def inv_miller(y):
    return np.degrees(2.5 * np.arctan(np.exp(0.8 * y)) - 0.625 * np.pi)


def load(data):
    etopo = np.load(f"{data}/etopo10.npy")[::-1].astype(np.float64)  # north first
    def tif(n):
        a = tifffile.imread(f"{data}/wc2.1_10m_bio_{n}.tif").astype(np.float64)
        a[a < -1e30] = np.nan
        return a
    return etopo, tif(1), tif(10), tif(12)


def flow_accumulation(land, elev, precip, lat):
    """Priority-flood drainage on the data grid: each land cell drains to the
    neighbor it was reached from, flooding up from the sea and lakes. Returns
    precipitation-weighted upstream flow per cell."""
    receiver = np.full(N * M, -1, dtype=np.int64)
    done = ~land.ravel().copy()
    heap = []
    flat_elev = elev.ravel()
    # Seed with water cells next to land.
    water = ~land
    edge = np.zeros_like(land)
    for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        edge |= water & np.roll(np.roll(land, dy, 0), dx, 1)
    for i in np.flatnonzero(edge.ravel()):
        heap.append((-1e9, int(i)))
    heapq.heapify(heap)
    order = []
    nbrs = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)]
    while heap:
        level, i = heapq.heappop(heap)
        r, c = divmod(i, M)
        for dr, dc in nbrs:
            rr = r + dr
            if rr < 0 or rr >= N:
                continue
            j = rr * M + (c + dc) % M
            if done[j]:
                continue
            done[j] = True
            receiver[j] = i
            order.append(j)
            heapq.heappush(heap, (max(level, flat_elev[j]), j))
    area = np.cos(np.radians(lat))[:, None] * np.ones((1, M))
    acc = (np.nan_to_num(precip) * area).ravel()
    for j in reversed(order):
        acc[receiver[j]] += acc[j]
    acc[~land.ravel()] = 0
    return acc.reshape(N, M), receiver


_ACC = None


def tile_hash(x, y, salt):
    h = (x * 73856093) ^ (y * 19349663) ^ (salt * 83492791)
    h = (h ^ (h >> 13)) * 1274126177
    return ((h ^ (h >> 16)) & 0xFFFFFF) / float(0xFFFFFF)


def build(data, width):
    etopo, temp, warm, precip = load(data)
    lat = 90 - (np.arange(N) + 0.5) / 6
    lon = -180 + (np.arange(M) + 0.5) / 6
    land = ~np.isnan(temp)
    # Antarctica's interior and a few lakes aside, WorldClim's mask is land.
    hex_w = 2 * np.pi / width
    size = hex_w / math.sqrt(3)
    row_h = 1.5 * size
    y_top = miller(LAT_TOP)
    height = int(round((y_top - miller(LAT_BOTTOM)) / row_h))
    height += height % 2

    # Hex of every data cell (pointy-top, odd rows shifted right).
    lon_g, lat_g = np.meshgrid(lon, lat)
    px = np.radians((lon_g - LON_WEST) % 360) - hex_w / 2
    py = (y_top - miller(np.clip(lat_g, -89, 89))) - row_h / 2
    q = (math.sqrt(3) / 3 * px - py / 3) / size
    r = (2 / 3 * py) / size
    # Cube rounding.
    cx, cz = q, r
    cy = -cx - cz
    rx, ry, rz = np.round(cx), np.round(cy), np.round(cz)
    dx, dy, dz = abs(rx - cx), abs(ry - cy), abs(rz - cz)
    fix_x = (dx > dy) & (dx > dz)
    fix_y = ~fix_x & (dy > dz)
    rx = np.where(fix_x, -ry - rz, rx)
    ry = np.where(fix_y, -rx - rz, ry)
    rz = np.where(~fix_x & ~fix_y, -rx - ry, rz)
    row = rz.astype(int)
    col = (rx.astype(int) + (row - (row & 1)) // 2) % width
    inside = (row >= 0) & (row < height)
    tile = np.where(inside, row * width + col, -1)

    T = height * width
    weight = np.cos(np.radians(lat_g))
    sel = inside.ravel()
    t = tile.ravel()[sel]
    w = weight.ravel()[sel]
    lnd = land.ravel()[sel]

    def mean(values, mask):
        v = np.nan_to_num(values.ravel()[sel])
        num = np.bincount(t[mask], weights=(v * w)[mask], minlength=T)
        den = np.bincount(t[mask], weights=w[mask], minlength=T)
        return num / np.maximum(den, 1e-9), den

    total = np.bincount(t, weights=w, minlength=T)
    land_w = np.bincount(t[lnd], weights=w[lnd], minlength=T)
    land_frac = land_w / np.maximum(total, 1e-9)
    elev, _ = mean(etopo, lnd)
    elev_sq, _ = mean(etopo ** 2, lnd)
    relief = np.sqrt(np.maximum(elev_sq - elev ** 2, 0))
    depth, _ = mean(etopo, ~lnd)
    tmean, _ = mean(temp, lnd)
    twarm, _ = mean(warm, lnd)
    prec, _ = mean(precip, lnd)

    print(f"  flow accumulation for {width}x{height}...", flush=True)
    global _ACC
    if _ACC is None:
        _ACC = flow_accumulation(land, etopo, precip, lat)
    acc, receiver = _ACC
    flow = np.zeros(T)
    np.maximum.at(flow, t[lnd], acc.ravel()[sel][lnd])

    is_land = land_frac >= 0.42
    # Keep notable small islands: any tile that is a third land, surrounded by sea.
    is_land |= (land_frac >= 0.3) & (elev > 50)

    def neighbors(i):
        y, x = divmod(i, width)
        qq = x - (y - (y & 1)) // 2
        for dq, dr in ((1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)):
            ny = y + dr
            if 0 <= ny < height:
                nx = (qq + dq + (ny - (ny & 1)) // 2) % width
                yield ny * width + nx

    # Relief classes by quantile of a ruggedness score over land.
    score = relief + 0.12 * np.maximum(elev, 0)
    land_scores = score[is_land]
    mountain_cut = np.quantile(land_scores, 0.91)
    hill_cut = np.quantile(land_scores, 0.66)
    river_cut = np.quantile(flow[is_land], 0.91)
    big_river_cut = np.quantile(flow[is_land], 0.975)

    # Downstream neighbor of each hex: follow the drainage from the hex's
    # biggest-flow cell until it leaves the hex.
    tile_flat = tile.ravel()
    acc_flat = acc.ravel()
    cells = np.flatnonzero(inside.ravel() & land.ravel())
    order = np.lexsort((acc_flat[cells], tile_flat[cells]))
    sorted_cells = cells[order]
    sorted_tiles = tile_flat[sorted_cells]
    last = np.r_[sorted_tiles[1:] != sorted_tiles[:-1], True]
    main_cell = dict(zip(sorted_tiles[last].tolist(), sorted_cells[last].tolist()))

    def axial(i):
        y, x = divmod(i, width)
        return x - (y - (y & 1)) // 2, y

    def hex_distance(a, b):
        (aq, ar), (bq, br) = axial(a), axial(b)
        best = 1 << 30
        for shift in (-width, 0, width):
            dq, dr = aq - (bq + shift), ar - br
            best = min(best, (abs(dq) + abs(dr) + abs(dq + dr)) // 2)
        return best

    def downstream(i):
        j = main_cell.get(i)
        if j is None:
            return None
        for _ in range(400):
            k = receiver[j]
            if k < 0:
                return None
            if tile_flat[k] != i:
                d = tile_flat[k]
                if d < 0:
                    return None
                nbrs = list(neighbors(i))
                return min(range(len(nbrs)), key=lambda n: hex_distance(nbrs[n], d))
            j = k
        return None

    terrain = [""] * T
    feature = ["."] * T
    flags = ["."] * T
    lat_c = np.zeros(T)
    for i in range(T):
        y, x = divmod(i, width)
        lat_c[i] = inv_miller(y_top - (y + 0.5) * row_h)
    coastal = np.zeros(T, bool)
    for i in range(T):
        if is_land[i]:
            coastal[i] = any(not is_land[n] for n in neighbors(i))

    for i in range(T):
        y, x = divmod(i, width)
        h = lambda salt: tile_hash(x, y, salt)
        if not is_land[i]:
            near_land = any(is_land[n] for n in neighbors(i))
            d = depth[i] if total[i] > 0 else -4000
            if near_land or d > -180:
                terrain[i] = "c"
            elif d > -3200:
                terrain[i] = "o"
            else:
                terrain[i] = "x"
            polar = lat_c[i] > 77 or lat_c[i] < -64
            if polar and h(1) < 0.85:
                feature[i] = "i"
            continue
        Tm, Tw, P = tmean[i], twarm[i], prec[i]
        river = flow[i] >= river_cut
        big_river = flow[i] >= big_river_cut
        relief_class = "M" if score[i] >= mountain_cut else "H" if score[i] >= hill_cut else ""
        arid = max(20 * Tm + 140, 100)
        feat = "."
        if Tw < 3:
            base = "s"
            if not coastal[i] and (elev[i] > 600 or Tm < -18):
                feat = "G"
        elif Tw < 10:
            base = "t"
        elif P < 0.5 * arid and P < 450:
            base = "d"
            if P < 70 and relief[i] < 160 and relief_class != "M":
                feat = "u"
            elif P < 160 and h(2) < 0.05 and relief_class == "":
                feat = "o"
        elif P < arid:
            base = "p"
        elif Tm < 2:
            base = "t" if Tm < -3 else "p"
            if h(3) < min(1.0, (P - 200) / 250) * 0.85:
                feat = "T"
        elif Tm >= 20 and P > 1650:
            base = "g"
            if h(4) < 0.85:
                feat = "j"
        elif Tm >= 20 and P > 1200:
            base = "g"
            if h(4) < 0.45:
                feat = "j"
        elif Tm >= 18:
            base = "g" if P > 1100 else "p"
        else:
            base = "g" if P > 650 else "p"
            if P > 700 and h(5) < 0.5:
                feat = "f"
        flat = relief_class == "" and elev[i] < 160 and relief[i] < 40
        if flat and P > 380 and base in "gpt" and feat in ".fT" and (river or (Tm < 3 and Tw >= 10)):
            if h(6) < (0.55 if river else 0.3):
                feat = "m"
        if big_river and relief_class == "" and elev[i] < 400 and feat == ".":
            if base == "d" or h(7) < 0.7:
                feat = "F"
        if relief_class == "M":
            code = {"g": "1", "p": "2", "d": "3", "t": "4", "s": "5"}[base]
            feat = "." if feat in "Gufo" else feat
            if feat in "TjmF":
                feat = "."
        else:
            code = base.upper() if relief_class == "H" else base
            if relief_class == "H" and feat in "mFuo":
                feat = "."
        terrain[i] = code
        feature[i] = feat
        steep = coastal[i] and relief_class != "" and relief[i] > 140
        direction = downstream(i) if river and code not in "12345" and feat != "G" else None
        if direction is None:
            flags[i] = "c" if steep else "."
        else:
            flags[i] = chr((ord("A") if steep else ord("0")) + direction)

    lines = [f"WAYCIV-EARTH {width} {height} {LAT_TOP} {LAT_BOTTOM} {LON_WEST}"]
    for grid in (terrain, feature, flags):
        for y in range(height):
            lines.append("".join(grid[y * width:(y + 1) * width]))
    return width, height, lines, terrain, feature, flags


PREVIEW = {
    "g": (90, 160, 60), "p": (180, 175, 95), "d": (225, 205, 140), "t": (140, 150, 120), "s": (235, 240, 245),
    "c": (90, 150, 210), "o": (35, 80, 160), "x": (15, 40, 100),
}


def preview(path, width, height, terrain, feature, flags):
    img = np.zeros((height, width, 3), np.uint8)
    for i, code in enumerate(terrain):
        y, x = divmod(i, width)
        base = code.lower() if code.isalpha() else "gpdts"[int(code) - 1]
        c = np.array(PREVIEW[base], float)
        if code.isupper():
            c *= 0.8
        if code.isdigit():
            c = np.array((110, 100, 95), float)
        f = feature[i]
        tint = {"f": (40, 110, 40), "T": (30, 85, 75), "j": (20, 90, 30), "m": (70, 120, 110), "u": (240, 190, 100),
                "o": (60, 170, 120), "F": (130, 170, 60), "G": (250, 252, 255), "i": (220, 235, 250)}.get(f)
        if tint:
            c = 0.35 * c + 0.65 * np.array(tint)
        if flags[i] not in ".c":
            c = 0.5 * c + 0.5 * np.array((40, 90, 230))
        img[y, x] = np.clip(c, 0, 255)
    ppm = path.replace(".png", ".ppm")
    with open(ppm, "wb") as f:
        f.write(f"P6 {width} {height} 255\n".encode())
        f.write(img.tobytes())
    subprocess.run(["magick", ppm, "-scale", "800%", path], check=True)
    subprocess.run(["rm", ppm])


def main():
    data, out = sys.argv[1], sys.argv[2]
    widths = [int(w) for w in sys.argv[3:]] or [104, 128, 160]
    for width in widths:
        w, h, lines, terrain, feature, flags = build(data, width)
        path = f"{out}/earth_{w}x{h}.txt"
        with open(path, "w") as f:
            f.write("\n".join(lines) + "\n")
        print(f"wrote {path}")
        preview(f"{out}/earth_{w}x{h}.png", w, h, terrain, feature, flags)


if __name__ == "__main__":
    main()
