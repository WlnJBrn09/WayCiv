"""Bonus resources."""
import math
import random

from lib import *  # noqa: F403

MODELS = {}


def model(name):
    def deco(fn):
        MODELS[name] = fn
        return fn
    return deco


@model("resource_wheat")
def wheat(b):
    rng = random.Random(3)
    gm = mat("grain", c("#e3b94a"), 0.5, col2=c("#b88a30"), scale=30)
    stm = mat("stalk", c("#c9a24a"), 0.7)
    for i in range(70):
        a = rng.random() * 6.28
        r = 0.62 * math.sqrt(rng.random())
        x, y = r * math.cos(a) * 0.95, r * math.sin(a) * 0.8
        h = 0.34 + rng.random() * 0.14
        lx, ly = (rng.random() - 0.5) * 0.08, (rng.random() - 0.5) * 0.08
        b.cyl(stm, (x, y, 0), (x + lx, y + ly, h), 0.009, 0.006, seg=4)
        b.blob(gm, (x + lx, y + ly, h + 0.03), (0.017, 0.017, 0.06), seg=6)
        b.cyl(gm, (x + lx, y + ly, h + 0.06), (x + lx * 1.4, y + ly * 1.4, h + 0.11), 0.002, 0.001, seg=3)
    return {}


@model("resource_rice")
def rice(b):
    soil = mat("paddyw", c("#6fa0a8"), 0.1, metal=0.0, col2=c("#8dbab8"), scale=3, alpha=0.9)
    mud = mat("mud", c("#7a5a38"), 0.95, col2=c("#5a4128"), scale=8)
    b.poly(soil, hex_pts(0.6), z=0.0, h=0.03)
    b.wall_run(mud, hex_pts(0.6) + [hex_pts(0.6)[0]], 0.07, 0.07)
    lm = mat("rice", c("#7cc03a"), 0.6, col2=c("#b4d848"), scale=25)
    for row in range(-3, 4):
        for k in range(-3, 4):
            x, y = row * 0.15, k * 0.15 + (0.07 if row % 2 else 0)
            if x * x * 0.8 + y * y > 0.4:
                continue
            for j in range(3):
                a = j * 2.1 + row
                b.cyl(lm, (x, y, 0.03), (x + math.cos(a) * 0.05, y + math.sin(a) * 0.05, 0.2 + 0.03 * ((row + k) % 2)), 0.012, 0.002, seg=3)
    return {}


@model("resource_maize")
def maize(b):
    rng = random.Random(5)
    lm = mat("cornleaf", c("#5aa02e"), 0.6)
    cm = mat("cob", c("#f2c12e"), 0.5)
    hm = mat("husk", c("#8bbf4a"), 0.7)
    for i in range(14):
        a = rng.random() * 6.28
        r = 0.5 * math.sqrt(rng.random())
        x, y = r * math.cos(a), r * math.sin(a) * 0.8
        h = 0.55 + rng.random() * 0.2
        b.cyl(lm, (x, y, 0), (x, y, h), 0.016, 0.01, seg=5)
        for k in range(4):
            aa = k * 1.57 + i
            b.tube(lm, [(x, y, h * (0.35 + 0.15 * k)), (x + math.cos(aa) * 0.1, y + math.sin(aa) * 0.1, h * (0.42 + 0.15 * k)), (x + math.cos(aa) * 0.19, y + math.sin(aa) * 0.19, h * (0.36 + 0.15 * k))], 0.014, 0.003, seg=3)
        b.blob(cm, (x + 0.03, y, h * 0.62), (0.025, 0.025, 0.07), seg=8)
        b.blob(hm, (x + 0.03, y, h * 0.55), (0.03, 0.03, 0.05), seg=6)
        b.cyl(cm, (x, y, h), (x + 0.01, y, h + 0.1), 0.002, 0.002, seg=3)
    return {}


@model("resource_bananas")
def bananas(b):
    wd = wood_mat("#7a6a3a", "#5a4a24")
    lm = mat("bleaf", c("#5fae35"), 0.55, col2=c("#3f8a28"), scale=10, sss=0.2)
    bm = mat("banana", c("#f0d02a"), 0.4, col2=c("#d8b020"), scale=20)
    gm = mat("bgreen", c("#8fbf3a"), 0.5)
    for (x, y, h) in [(-0.2, 0.15, 0.55), (0.25, 0.08, 0.5), (0.0, -0.25, 0.45)]:
        b.tube(wd, [(x, y, 0), (x + 0.02, y, h * 0.5), (x, y + 0.01, h)], 0.045, 0.03, seg=8)
        for k in range(7):
            a = k * 0.9 + x * 5
            L = 0.38
            b.tube(lm, [(x, y, h), (x + math.cos(a) * L * 0.5, y + math.sin(a) * L * 0.5, h + 0.12), (x + math.cos(a) * L, y + math.sin(a) * L, h - 0.03)], 0.045, 0.004, seg=4)
        for k in range(4):
            a = k * 1.4
            cx, cy = x + math.cos(a) * 0.06, y + math.sin(a) * 0.06
            for j in range(5):
                b.tube(bm if j % 2 == 0 else gm, [(cx + (j - 2) * 0.012, cy, h - 0.03), (cx + (j - 2) * 0.02, cy + 0.03, h - 0.1), (cx + (j - 2) * 0.012, cy + 0.04, h - 0.16)], 0.015, 0.007, seg=5)
    # loose bunch on the ground
    for j in range(5):
        b.tube(bm, [(0.28 + j * 0.025, -0.3, 0.03), (0.3 + j * 0.025, -0.38, 0.05), (0.28 + j * 0.025, -0.45, 0.03)], 0.02, 0.01, seg=5)
    return {}


@model("resource_cattle")
def cattle(b):
    quad(b, -0.15, 0.1, 0.5, 1.15, "cow", "#7a4a2a", "#f2ebdc")
    quad(b, 0.22, -0.2, 2.9, 1.0, "cow", "#2a1f1a", "#f2ebdc")
    gm = grass_mat()
    for p in [(-0.5, -0.2), (0.45, 0.3), (0.0, 0.5), (-0.3, -0.5)]:
        b.tuft(gm, (p[0], p[1], 0), 0.15, 6)
    return {}


@model("resource_sheep")
def sheep(b):
    quad(b, -0.2, 0.12, 0.4, 1.3, "sheep")
    quad(b, 0.25, -0.12, 3.5, 1.1, "sheep")
    quad(b, 0.1, 0.38, 0.0, 0.7, "sheep")
    gm = grass_mat()
    for p in [(-0.5, -0.3), (0.5, 0.25), (0.0, -0.45)]:
        b.tuft(gm, (p[0], p[1], 0), 0.13, 6)
    return {}


@model("resource_deer")
def deer(b):
    quad(b, -0.12, 0.05, 0.6, 1.3, "deer", "#a8703c", "#e8d4b0")
    quad(b, 0.3, -0.25, 3.0, 0.95, "deer", "#9a6a3a", "#e8d4b0")
    gm = grass_mat("#7a8a4a", "#5a6a38")
    for p in [(-0.5, -0.2), (0.45, 0.3), (0.0, 0.5)]:
        b.tuft(gm, (p[0], p[1], 0), 0.14, 6)
    return {}


@model("resource_fish")
def fish_res(b):
    water_rings(b, 0.55, 2)
    fish(b, -0.12, 0.12, 0.05, 0.4, 1.5)
    fish(b, 0.2, -0.1, 0.05, 0.5 + 0.8, 1.3, "#9ac6a8")
    fish(b, -0.1, -0.25, 0.04, 0.2, 1.1, "#7fb4d8")
    return {"fit": False}


@model("resource_crabs")
def crabs(b):
    water_rings(b, 0.62, 2)
    sand = mat("sand", c("#d8c08a"), 0.95, col2=c("#c0a870"), scale=15)
    b.blob(sand, (0, 0, -0.01), (0.55, 0.45, 0.05))
    cm = mat("crab", c("#e0522a"), 0.4, col2=c("#b83a1e"), scale=12, coat=0.3)
    em = mat("ceye", c("#101010"), 0.2)
    for (x, y, yaw, s) in [(-0.12, 0.05, 0.4, 1.3), (0.2, -0.12, 2.6, 1.0)]:
        P = lib_P(x, y, yaw, s)
        b.blob(cm, P(0, 0, 0.07), (0.1 * s, 0.14 * s, 0.045 * s), yaw)
        for side in (-1, 1):
            b.tube(cm, [P(0.08 * side, 0.1, 0.07), P(0.17 * side, 0.2, 0.09), P(0.14 * side, 0.27, 0.07)], 0.018 * s, 0.014 * s, seg=5)
            b.blob(cm, P(0.14 * side, 0.3, 0.07), (0.045 * s, 0.05 * s, 0.025 * s), yaw)
            for k in range(3):
                b.tube(cm, [P(0.08 * side, -0.07 + k * 0.05, 0.05), P(0.17 * side, -0.07 + k * 0.05, 0.08), P(0.2 * side, -0.07 + k * 0.05, 0.0)], 0.01 * s, 0.006 * s, seg=4)
            b.cyl(cm, P(0.04 * side, 0.12, 0.1), P(0.04 * side, 0.14, 0.15), 0.006 * s, 0.006 * s, seg=4)
            b.blob(em, P(0.04 * side, 0.14, 0.16), (0.014 * s, 0.014 * s, 0.014 * s), yaw, seg=6)
    return {"fit": False}


def lib_P(x, y, yaw, s):
    import lib
    return lib._P(x, y, yaw, s)


@model("resource_stone")
def stone(b):
    sm = mat("stone", c("#c9c3b4"), 0.85, col2=c("#a39d90"), scale=6, bump=0.3)
    sm2 = mat("stone2", c("#b7b1a2"), 0.85, col2=c("#8c8678"), scale=6, bump=0.3)
    b.box(sm, (-0.18, 0.05, 0.08), (0.42, 0.26, 0.16), yaw=0.12)
    b.box(sm2, (-0.1, 0.02, 0.23), (0.34, 0.24, 0.14), yaw=-0.1)
    b.box(sm, (0.25, -0.15, 0.08), (0.3, 0.22, 0.16), yaw=0.5)
    b.box(sm2, (0.1, 0.3, 0.07), (0.24, 0.2, 0.14), yaw=-0.3)
    b.rock(sm2, (0.42, 0.18, 0.08), (0.12, 0.1, 0.1), 1)
    b.rock(sm, (-0.42, -0.2, 0.07), (0.14, 0.12, 0.1), 2)
    b.rock(sm, (0.0, -0.32, 0.06), (0.1, 0.09, 0.08), 3)
    return {}


@model("resource_copper")
def copper(b):
    rm = rock_mat("#8a7a68", "#5a4c40")
    cu = mat("copper", c("#c8683a"), 0.35, metal=0.9, col2=c("#a04a28"), scale=12)
    pat = mat("patina", c("#4aa890"), 0.6, col2=c("#2e8070"), scale=15)
    b.rock(rm, (-0.1, 0.05, 0.12), (0.34, 0.28, 0.24), 1)
    b.rock(rm, (0.3, -0.12, 0.1), (0.22, 0.2, 0.18), 2)
    b.rock(rm, (-0.3, -0.25, 0.08), (0.18, 0.16, 0.14), 3)
    for i, (x, y, z, r) in enumerate([(-0.02, -0.12, 0.2, 0.09), (-0.2, 0.12, 0.28, 0.08), (0.34, -0.05, 0.2, 0.07), (0.1, 0.18, 0.2, 0.07), (-0.34, -0.2, 0.15, 0.06), (0.3, -0.28, 0.1, 0.06)]):
        b.rock(cu, (x, y, z), (r, r * 0.9, r * 0.8), 10 + i, rough=0.35)
    b.rock(pat, (0.12, 0.0, 0.3), (0.07, 0.06, 0.05), 20)
    b.rock(pat, (-0.22, -0.02, 0.35), (0.05, 0.05, 0.04), 21)
    return {}
