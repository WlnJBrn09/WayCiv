"""Ground (biome) texture cells: 256x256 painterly tiles, 3 variants per
biome.  The game maps one cell onto a 2.5-hex-radius square centred on the
tile, so the hex sits in the middle with spare texture around it for edge
feathering."""
import numpy as np
from texlib import *

N = CELL


def grass(seed):
    low = fft_noise(N, 2.6, seed)
    mid = fft_noise(N, 1.4, seed + 1)
    t = np.clip(low * 0.65 + mid * 0.35, 0, 1)
    col = ramp(t, [(0, hexc("3f8a34")), (0.45, hexc("5aa23c")), (0.75, hexc("78b844")), (1, hexc("a2cf58"))])
    col = mix(col, hexc("2f7a3a"), smoothstep(0.55, 0.9, fft_noise(N, 2.2, seed + 2)) * 0.35)
    lite = strokes(N, seed + 3, 3200, 9, 0.9, angle=np.pi / 2, jitter=0.5)
    dark = strokes(N, seed + 4, 2200, 8, 0.8, angle=np.pi / 2, jitter=0.5)
    col = mix(col, hexc("b6dc66"), lite * 0.5)
    col = mix(col, hexc("2d6a2c"), dark * 0.4)
    fl = dots(N, seed + 5, 45, 0.9)
    col = mix(col, hexc("f2e27a"), np.clip(fl * 2, 0, 1) * 0.55)
    return col


def plains(seed):
    low = fft_noise(N, 2.5, seed)
    mid = fft_noise(N, 1.3, seed + 1)
    t = np.clip(low * 0.6 + mid * 0.4, 0, 1)
    col = ramp(t, [(0, hexc("8c9440")), (0.4, hexc("a6a94c")), (0.7, hexc("c0b95e")), (1, hexc("d8cd78"))])
    green = smoothstep(0.55, 0.85, fft_noise(N, 2.4, seed + 2))
    col = mix(col, hexc("78a042"), green * 0.5)
    lite = strokes(N, seed + 3, 3000, 14, 0.9, angle=np.pi / 2, jitter=0.7)
    dark = strokes(N, seed + 4, 1800, 12, 0.8, angle=np.pi / 2, jitter=0.7)
    col = mix(col, hexc("e8d888"), lite * 0.5)
    col = mix(col, hexc("6e7632"), dark * 0.35)
    st = strokes(N, seed + 6, 60, 50, 3.5, angle=0.25, jitter=0.15)
    col = mix(col, hexc("b8a456"), st * 0.25)
    return col


def desert(seed):
    low = fft_noise(N, 2.6, seed)
    mid = fft_noise(N, 1.5, seed + 1)
    t = np.clip(low * 0.7 + mid * 0.3, 0, 1)
    col = ramp(t, [(0, hexc("d6b070")), (0.4, hexc("e6c886")), (0.75, hexc("f0d9a0")), (1, hexc("f8e8bc"))])
    yy, xx = np.mgrid[0:N, 0:N].astype(np.float32) / N
    warp = fft_noise(N, 2.8, seed + 2)
    ang = 0.5 + 0.9 * (seed % 3)
    ph = (xx * np.cos(ang) + yy * np.sin(ang)) * 2 * np.pi * 7 + warp * 12
    rip = 0.5 + 0.5 * np.sin(ph)
    amp = smoothstep(0.3, 0.7, fft_noise(N, 2.5, seed + 3)) * 0.12 + 0.03
    col = col * (1 - amp[..., None] * (1 - rip[..., None]))
    col = col + (amp[..., None] * rip[..., None]) * 0.25
    pebble = dots(N, seed + 4, 160, 0.9)
    col = mix(col, hexc("a07850"), np.clip(pebble * 2, 0, 1) * 0.45)
    grain = fft_noise(N, 0.2, seed + 5)
    col = col * (0.97 + 0.06 * grain[..., None])
    return col


def tundra(seed):
    low = fft_noise(N, 2.6, seed)
    mid = fft_noise(N, 1.5, seed + 1)
    t = np.clip(low * 0.65 + mid * 0.35, 0, 1)
    col = ramp(t, [(0, hexc("66785a")), (0.45, hexc("84967a")), (0.8, hexc("a2ae92")), (1, hexc("bac4a8"))])
    brown = smoothstep(0.55, 0.85, fft_noise(N, 2.3, seed + 2))
    col = mix(col, hexc("8a7a58"), brown * 0.5)
    snow = smoothstep(0.7, 0.8, fft_noise(N, 2.4, seed + 3))
    col = mix(col, hexc("e4ebee"), snow * 0.85)
    lich = dots(N, seed + 4, 260, 1.0)
    col = mix(col, hexc("b8a860"), np.clip(lich * 2, 0, 1) * 0.5)
    rock = dots(N, seed + 5, 90, 1.6)
    col = mix(col, hexc("5e6660"), np.clip(rock * 2, 0, 1) * 0.5)
    tuft = strokes(N, seed + 6, 1800, 6, 0.8, angle=np.pi / 2, jitter=0.9)
    col = mix(col, hexc("5a6e4a"), tuft * 0.35)
    return col


def snow(seed):
    low = fft_noise(N, 2.6, seed)
    mid = fft_noise(N, 1.4, seed + 1)
    t = np.clip(low * 0.6 + mid * 0.4, 0, 1)
    col = ramp(t, [(0, hexc("c4d6ec")), (0.4, hexc("dde8f6")), (0.75, hexc("f1f6fc")), (1, hexc("ffffff"))])
    yy, xx = np.mgrid[0:N, 0:N].astype(np.float32) / N
    warp = fft_noise(N, 2.6, seed + 2)
    ph = (xx * 0.8 + yy * 0.6) * 2 * np.pi * 5 + warp * 10
    rip = 0.5 + 0.5 * np.sin(ph)
    col = col * (0.965 + 0.035 * rip[..., None])
    sp = dots(N, seed + 3, 120, 0.7)
    col = mix(col, np.array([1, 1, 1]), np.clip(sp * 2, 0, 1) * 0.8)
    shade_ = smoothstep(0.6, 0.85, fft_noise(N, 2.4, seed + 4))
    col = mix(col, hexc("a9c0e0"), shade_ * 0.35)
    return col


def water(seed, deep, mid, light, caustic, waves, wavelen):
    low = fft_noise(N, 2.8, seed)
    mi = fft_noise(N, 1.6, seed + 1)
    t = np.clip(low * 0.7 + mi * 0.3, 0, 1)
    col = ramp(t, [(0, deep), (0.5, mid), (1, light)])
    r1 = ridged(N, seed + 2, 1.8)
    line = smoothstep(0.86, 0.97, r1)
    col = mix(col, light * 1.15, line * caustic)
    st = strokes(N, seed + 3, 220, wavelen, 1.6, angle=0.05, jitter=0.12)
    col = mix(col, light * 1.2, st * waves)
    st2 = strokes(N, seed + 4, 160, wavelen, 1.6, angle=0.05, jitter=0.12)
    col = mix(col, deep * 0.8, st2 * waves * 0.6)
    return col


def coast(seed):
    return water(seed, hexc("3a98b8"), hexc("4aaac6"), hexc("8ad4de"), 0.28, 0.35, 28)


def ocean(seed):
    return water(seed, hexc("1e5a9a"), hexc("2a70b0"), hexc("4a94cc"), 0.16, 0.30, 34)


def deep(seed):
    return water(seed, hexc("0c2c68"), hexc("123a7c"), hexc("2658a0"), 0.08, 0.2, 40)


def rock(seed):
    low = fft_noise(N, 2.4, seed)
    col = ramp(low, [(0, hexc("5c5148")), (0.5, hexc("7c7064")), (1, hexc("a39684"))])
    ids, d1, d2 = voronoi(N, seed + 1, 40)
    crack = smoothstep(0.0, 4.0, d2 - d1)
    col = col * (0.72 + 0.28 * crack[..., None])
    yy = np.mgrid[0:N, 0:N][0].astype(np.float32) / N
    strata = 0.5 + 0.5 * np.sin(yy * 2 * np.pi * 14 + fft_noise(N, 2.5, seed + 2) * 8)
    col = col * (0.92 + 0.12 * strata[..., None])
    return col


def sand(seed):
    low = fft_noise(N, 2.4, seed)
    col = ramp(low, [(0, hexc("d8bc82")), (0.5, hexc("e8d1a0")), (1, hexc("f4e6c0"))])
    grain = fft_noise(N, 0.2, seed + 1)
    col = col * (0.95 + 0.1 * grain[..., None])
    return col


BIOMES = [grass, plains, desert, tundra, snow, coast, ocean, deep]
BIOME_NAMES = ["GRASS", "PLAINS", "DESERT", "TUNDRA", "SNOW", "COAST", "OCEAN", "DEEP_OCEAN"]


def build_atlas():
    cols, rows = 8, 4
    atlas = np.full((rows * N, cols * N, 3), 0.5, np.float32)
    cells = []
    for b, fn in enumerate(BIOMES):
        for v in range(3):
            cells.append(fn(1000 * (b + 1) + 37 * v))
    cells.append(rock(77))
    cells.append(sand(78))
    cells.append(np.ones((N, N, 3), np.float32))
    for i, c in enumerate(cells):
        r, cc = divmod(i, cols)
        atlas[r * N:(r + 1) * N, cc * N:(cc + 1) * N] = c
    return atlas


if __name__ == "__main__":
    import sys
    save_rgb(sys.argv[1], build_atlas())
