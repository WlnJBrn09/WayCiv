"""Water textures.  Both are seamless and mapped in world space by the game
(one repeat per few hexes), so water has no per-tile seams:
  water.png   neutral light detail (wave trains, swell, caustic web) that
              the game tints with a per-corner depth colour
  glint.png   mostly black sun glints on wave crests, drawn additively and
              scrolled over the water"""
import numpy as np
from scipy import ndimage as ndi
from texlib import *

N = 512

# integer wave vectors keep the texture seamless
WAVES = [(3, 1, 1.0), (-2, 4, 0.8), (5, -2, 0.6), (1, 6, 0.55), (7, 3, 0.45), (-6, 5, 0.4), (3, 11, 0.35), (-9, 8, 0.3), (14, -3, 0.25), (2, -16, 0.2)]


def wave_height(seed, warp_amp=0.9):
    yy, xx = np.mgrid[0:N, 0:N].astype(np.float32) / N
    w1 = blur(fft_noise(N, 3.0, seed), 10) - 0.5
    w2 = blur(fft_noise(N, 3.0, seed + 1), 10) - 0.5
    h = np.zeros((N, N), np.float32)
    r = np.random.default_rng(seed)
    for kx, ky, a in WAVES:
        ph = r.uniform(0, 2 * np.pi)
        arg = 2 * np.pi * (kx * (xx + w1 * 0.05 * warp_amp) + ky * (yy + w2 * 0.05 * warp_amp)) + ph
        h += a * np.sin(arg + 2.4 * w1)
    return h / sum(a for _, _, a in WAVES) * 1.6


def water_base(seed=4100):
    h = wave_height(seed)
    # light from the upper left lights the slopes facing it
    gy, gx = np.gradient(h)
    slope = -(gx + gy) * N * 0.012
    swell = fft_noise(N, 3.4, seed + 7)
    web = smoothstep(0.82, 0.98, blur(ridged(N, seed + 4, 3.0), 1.5)) * 0.55 + smoothstep(0.84, 0.98, blur(ridged(N, seed + 5, 3.0), 1.5)) * 0.45
    v = 0.80 + 0.3 * np.clip(slope, -1, 1) + 0.05 * (swell - 0.5) * 2 + 0.03 * web
    drift = fft_noise(N, 3.4, seed + 6) - 0.5
    rgb = np.stack([v * (1.0 - 0.04 * drift * 2), v, v * (1.0 + 0.04 * drift * 2)], -1)
    return np.clip(rgb, 0, 1)


def glint(seed=4200):
    """Additive sparkle: short streaks along x, clustered on the swell."""
    r = np.random.default_rng(seed)
    w = r.standard_normal((N, N)).astype(np.float32)
    streak = ndi.gaussian_filter(w, (0.9, 7.0), mode="wrap")
    streak = (streak - streak.mean()) / streak.std()
    g = smoothstep(1.9, 3.0, streak)
    swell = fft_noise(N, 3.0, seed + 1)
    g = g * (0.15 + 0.85 * smoothstep(0.35, 0.7, swell))
    return np.repeat(np.clip(g, 0, 1)[..., None], 3, axis=-1)


def build_sheet():
    return water_base()


def build_glint():
    return glint()
