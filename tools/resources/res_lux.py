"""Luxury resources."""
import math
import random

import lib
from lib import *  # noqa: F403

MODELS = {}


def model(name):
    def deco(fn):
        MODELS[name] = fn
        return fn
    return deco


@model("resource_citrus")
def citrus(b):
    tree(b, (-0.12, 0.08, 0), 0.75, leaf="#3f8a28", s=1.9)
    om = mat("orange", c("#f08a1e"), 0.45, col2=c("#e06a10"), scale=20)
    for (x, y, z) in [(-0.2, -0.05, 0.5), (-0.05, -0.04, 0.58), (-0.28, 0.1, 0.52), (-0.1, 0.2, 0.46), (0.02, 0.05, 0.62), (-0.2, 0.12, 0.7)]:
        b.blob(om, (x, y, z), (0.04, 0.04, 0.04), seg=8)
    for (x, y) in [(0.3, -0.2), (0.38, -0.1), (0.34, -0.3), (0.26, -0.12), (0.32, -0.2)]:
        b.blob(om, (x, y, 0.045), (0.05, 0.05, 0.05), seg=8)
    return {}


@model("resource_cocoa")
def cocoa(b):
    wd = wood_mat("#6a4a30")
    lm = mat("cleaf", c("#3f8a2e"), 0.6, col2=c("#2e6e22"), scale=8, sss=0.2)
    pm = mat("pod", c("#c0642a"), 0.5, col2=c("#8a3a18"), scale=10)
    for (x, y, h) in [(-0.15, 0.1, 0.5), (0.25, -0.05, 0.42)]:
        b.tube(wd, [(x, y, 0), (x + 0.03, y, h * 0.5), (x, y, h)], 0.045, 0.03, seg=7)
        for k in range(6):
            a = k * 1.05
            b.tube(lm, [(x, y, h), (x + math.cos(a) * 0.15, y + math.sin(a) * 0.15, h + 0.08), (x + math.cos(a) * 0.3, y + math.sin(a) * 0.3, h - 0.05)], 0.05, 0.004, seg=4)
        for (dx, dy, dz) in [(0.05, 0, 0.3), (-0.05, 0.02, 0.2), (0.0, -0.05, 0.38)]:
            b.blob(pm, (x + dx, y + dy, dz * h / 0.5), (0.03, 0.03, 0.065), seg=8, tilt=0.3)
    for i in range(3):
        b.blob(pm, (0.1 + i * 0.07, -0.38 + 0.03 * i, 0.04), (0.04, 0.04, 0.07), yaw=i, seg=8, tilt=1.4)
    return {}


@model("resource_coffee")
def coffee(b):
    lm = mat("coffeel", c("#2f6a2a"), 0.4, col2=c("#3f8a34"), scale=12, coat=0.5)
    rm = mat("cherry", c("#c8202a"), 0.35, coat=0.6)
    for (x, y, r) in [(-0.2, 0.05, 0.22), (0.2, 0.0, 0.2), (0.0, -0.25, 0.18)]:
        b.blob(lm, (x, y, r * 0.8), (r, r, r * 0.85), seg=10)
        b.blob(lm, (x + r * 0.5, y + r * 0.3, r * 0.5), (r * 0.7, r * 0.7, r * 0.6), seg=8)
        for k in range(6):
            a = k * 1.1
            b.blob(rm, (x + math.cos(a) * r * 0.9, y + math.sin(a) * r * 0.9, r * 0.7 + 0.03 * (k % 3)), (0.025, 0.025, 0.025), seg=6)
    sack(b, (0.28, 0.28, 0), 1.0, "#c9b078", "#3a2214")
    bm = mat("bean", c("#4a2a16"), 0.5)
    for i in range(6):
        b.blob(bm, (-0.15 + 0.04 * (i % 3), 0.38 + 0.03 * (i // 3), 0.02), (0.025, 0.017, 0.015), yaw=i, seg=6)
    return {}


@model("resource_cotton")
def cotton(b):
    lm = mat("cotleaf", c("#4a8a30"), 0.6)
    wm = mat("cotton", c("#fbfaf4"), 1.0, col2=c("#e8e6dc"), scale=25, bump=0.6)
    rng = random.Random(1)
    for (x, y) in [(-0.25, 0.1), (0.1, 0.15), (0.3, -0.1), (-0.05, -0.2), (-0.3, -0.25)]:
        b.cyl(lm, (x, y, 0), (x, y, 0.28), 0.014, 0.01, seg=5)
        for k in range(4):
            a = k * 1.7 + x
            b.tube(lm, [(x, y, 0.12 + 0.05 * k), (x + math.cos(a) * 0.1, y + math.sin(a) * 0.1, 0.18 + 0.05 * k)], 0.02, 0.008, seg=4)
        for k in range(4):
            a = k * 1.6 + y * 3
            r = 0.04
            b.blob(wm, (x + math.cos(a) * 0.05, y + math.sin(a) * 0.05, 0.25 + 0.03 * (k % 2)), (r, r, r), seg=8)
    b.blob(wm, (0.35, 0.3, 0.07), (0.1, 0.08, 0.07), seg=10)
    b.blob(wm, (0.45, 0.25, 0.06), (0.06, 0.06, 0.05), seg=8)
    return {}


@model("resource_diamonds")
def diamonds(b):
    rm = rock_mat("#6a6460", "#403c3a")
    dm = mat("diamond", c("#d8f4ff"), 0.05, col2=c("#a8dcf0"), scale=4, emit=0.25, emit_col=c("#bfe8ff"), coat=1.0)
    b.rock(rm, (-0.05, 0.05, 0.12), (0.4, 0.32, 0.22), 1)
    b.rock(rm, (0.3, -0.15, 0.08), (0.2, 0.18, 0.13), 2)
    for (x, y, h, tx, ty) in [(-0.12, 0.05, 0.42, 0.2, 0.1), (0.1, -0.02, 0.34, -0.2, 0.15), (0.3, -0.15, 0.26, 0.1, -0.2), (-0.3, -0.05, 0.28, 0.35, -0.1), (0.0, 0.2, 0.22, 0, 0.35)]:
        b.crystal(dm, (x, y, 0.1), h, 0.07, (tx, ty), sides=5, tip=0.4)
    return {}


@model("resource_dyes")
def dyes(b):
    cloth = mat("dyecloth", c("#d8c8a0"), 1.0)
    b.box(cloth, (0, 0, 0.005), (1.0, 0.7, 0.01), yaw=0.1)
    for (x, y, col) in [(-0.3, 0.1, "#c8283a"), (0.0, -0.05, "#2a4aa8"), (0.3, 0.1, "#e8b820"), (0.0, 0.25, "#7a2a8a")]:
        pot(b, (x, y, 0), 1.1, "#b0583a", col)
        heap(b, (x + 0.02, y - 0.17, 0.0), 0.08, 0.1, col)
    pl = mat("indigo", c("#3a7a3a"), 0.6)
    b.tube(pl, [(0.4, -0.25, 0), (0.42, -0.25, 0.2), (0.38, -0.22, 0.3)], 0.012, 0.004, seg=4)
    return {}


@model("resource_furs")
def furs(b):
    fm = mat("pelt", c("#8a5a30"), 1.0, col2=c("#5a3a1e"), scale=18, bump=0.8)
    fw = mat("pelt2", c("#f0ece4"), 1.0, col2=c("#c8c0b0"), scale=18, bump=0.8)
    fg = mat("pelt3", c("#6a6a70"), 1.0, col2=c("#44444a"), scale=18, bump=0.8)
    rm = rock_mat("#8a847a")
    b.rock(rm, (-0.25, 0.15, 0.1), (0.2, 0.18, 0.16), 1)
    for (x, y, z, m, yaw) in [(0.05, -0.05, 0.03, fm, 0.2), (0.05, 0.02, 0.07, fw, -0.5), (-0.05, -0.2, 0.05, fg, 0.9)]:
        b.blob(m, (x, y, z), (0.33, 0.22, 0.03), yaw, seg=14)
        for k in range(4):
            a = yaw + k * 1.57 + 0.7
            b.blob(m, (x + math.cos(a) * 0.28, y + math.sin(a) * 0.18, z), (0.1, 0.05, 0.025), a, seg=8)
    b.blob(fm, (0.35, 0.25, 0.06), (0.1, 0.09, 0.06), seg=8)
    return {}


@model("resource_gypsum")
def gypsum(b):
    rm = mat("gyprock", c("#e8e4d8"), 0.7, col2=c("#c8c4b4"), scale=8, bump=0.4)
    cr = mat("gypcr", c("#f4f2ec"), 0.15, col2=c("#d8dcd8"), scale=5, coat=0.6)
    b.rock(rm, (0, 0.05, 0.1), (0.4, 0.3, 0.2), 1)
    b.rock(rm, (0.3, -0.22, 0.07), (0.17, 0.15, 0.12), 2)
    for (x, y, h, tx, ty) in [(-0.15, 0.1, 0.4, 0.15, 0.1), (0.05, 0.12, 0.34, -0.1, 0.25), (0.15, -0.05, 0.3, -0.2, -0.1), (-0.3, -0.05, 0.25, 0.3, 0.0), (0.0, -0.2, 0.2, 0.0, -0.4)]:
        b.crystal(cr, (x, y, 0.1), h, 0.05, (tx, ty), sides=4, tip=0.25)
    return {}


@model("resource_incense")
def incense(b):
    wd = wood_mat("#7a5a3a")
    lm = mat("incl", c("#7a9a40"), 0.7)
    am = mat("resin", c("#e0a030"), 0.2, coat=0.8, emit=0.15)
    b.tube(wd, [(-0.18, 0.05, 0), (-0.12, 0.0, 0.25), (-0.2, 0.03, 0.45), (-0.15, 0.08, 0.6)], 0.05, 0.02, seg=7)
    b.tube(wd, [(-0.14, 0.02, 0.3), (-0.02, 0.05, 0.45)], 0.025, 0.012, seg=5)
    for (x, y, z) in [(-0.15, 0.08, 0.62), (-0.02, 0.05, 0.48), (-0.28, 0.0, 0.5)]:
        b.blob(lm, (x, y, z), (0.12, 0.1, 0.05), seg=8)
    for (x, y) in [(-0.2, -0.1), (-0.12, -0.12), (-0.25, -0.15)]:
        b.blob(am, (x, y, 0.03), (0.035, 0.03, 0.03), seg=6)
    # censer
    bm = mat("bronze", c("#b88a3a"), 0.35, metal=0.9)
    b.blob(bm, (0.28, -0.08, 0.1), (0.11, 0.11, 0.07), seg=12)
    b.cyl(bm, (0.28, -0.08, 0.0), (0.28, -0.08, 0.04), 0.05, 0.03, seg=8)
    b.cyl(bm, (0.28, -0.08, 0.15), (0.28, -0.08, 0.17), 0.11, 0.11, seg=12)
    smoke(b, (0.28, -0.08, 0.2), 5, 1.0, "#f0f0f0", 0.55)
    return {}


@model("resource_ivory")
def ivory(b):
    soft_ground(b, "#d6bd88", "#c0a870", 0.55, 0.76)
    iv = mat("ivory", c("#f2e8cc"), 0.35, col2=c("#d8c9a4"), scale=14, bump=0.1)
    b.tube(iv, [(-0.3, 0.0, 0.0), (-0.36, -0.02, 0.2), (-0.3, -0.04, 0.42), (-0.12, -0.05, 0.62), (0.05, -0.05, 0.66)], 0.075, 0.012, seg=10)
    b.tube(iv, [(0.0, 0.1, 0.0), (0.06, 0.12, 0.2), (0.0, 0.14, 0.42), (-0.18, 0.15, 0.6), (-0.34, 0.15, 0.62)], 0.07, 0.012, seg=10)
    b.blob(iv, (-0.3, 0.0, 0.0), (0.1, 0.09, 0.05), seg=8)
    b.blob(iv, (0.0, 0.1, 0.0), (0.1, 0.09, 0.05), seg=8)
    rm = rock_mat("#9a8a70")
    b.rock(rm, (0.0, -0.3, 0.05), (0.14, 0.12, 0.09), 4)
    b.rock(rm, (-0.4, 0.2, 0.05), (0.1, 0.09, 0.07), 5)
    return {}


@model("resource_jade")
def jade(b):
    jm = mat("jade", c("#3fb078"), 0.12, col2=c("#1e7a50"), scale=4, coat=0.8, sss=0.5)
    jm2 = mat("jade2", c("#6fd09a"), 0.1, col2=c("#3fa070"), scale=4, coat=0.8, sss=0.5)
    rm = rock_mat("#7a756e")
    b.rock(rm, (0.05, 0.1, 0.07), (0.34, 0.26, 0.14), 1)
    b.rock(jm, (-0.12, 0.0, 0.22), (0.26, 0.22, 0.2), 2, rough=0.18)
    b.rock(jm2, (0.22, -0.08, 0.14), (0.17, 0.15, 0.14), 3, rough=0.18)
    b.rock(jm, (-0.28, -0.25, 0.08), (0.12, 0.1, 0.09), 4, rough=0.18)
    b.rock(jm2, (0.12, -0.3, 0.07), (0.1, 0.08, 0.08), 5, rough=0.18)
    return {}


@model("resource_marble")
def marble(b):
    mm = mat("marble", c("#f2f0ec"), 0.25, col2=c("#a8a8b0"), scale=3, coat=0.4, stripes=6)
    mm2 = mat("marble2", c("#e8e6e2"), 0.3, col2=c("#8a8a94"), scale=3, coat=0.3, stripes=4)
    b.box(mm, (-0.12, 0.05, 0.1), (0.44, 0.3, 0.2), yaw=0.15)
    b.box(mm2, (-0.05, 0.03, 0.28), (0.34, 0.26, 0.16), yaw=-0.1)
    b.box(mm2, (0.3, -0.18, 0.09), (0.28, 0.24, 0.18), yaw=0.5)
    rm = mat("mrough", c("#d0cec8"), 0.8, col2=c("#a8a6a0"), scale=5, bump=0.4)
    b.rock(rm, (0.12, 0.3, 0.09), (0.14, 0.12, 0.1), 2)
    b.rock(rm, (-0.4, -0.15, 0.07), (0.12, 0.1, 0.09), 3)
    return {}


@model("resource_mercury")
def mercury(b):
    qm = mat("quick", c("#d8dce0"), 0.04, metal=1.0)
    b.blob(qm, (0, 0, 0.0), (0.45, 0.35, 0.04), seg=18)
    for (x, y, r) in [(0.12, 0.05, 0.07), (-0.15, 0.1, 0.05), (0.3, -0.25, 0.05), (-0.3, -0.2, 0.04), (0.25, 0.28, 0.04)]:
        b.blob(qm, (x, y, r * 0.8), (r, r, r), seg=10)
    cm = mat("cinnabar", c("#b0201c"), 0.4, col2=c("#701410"), scale=6, bump=0.4)
    b.rock(cm, (-0.3, 0.12, 0.1), (0.17, 0.15, 0.14), 1)
    b.rock(cm, (0.35, 0.18, 0.08), (0.12, 0.1, 0.1), 2)
    b.rock(cm, (-0.1, -0.35, 0.07), (0.11, 0.1, 0.08), 3)
    return {}


@model("resource_pearls")
def pearls(b):
    water_rings(b, 0.58, 2)
    sh = mat("shell", c("#e8c8c0"), 0.4, col2=c("#c09090"), scale=6, stripes=14)
    inner = mat("nacre", c("#f6ecf0"), 0.15, col2=c("#d0e0f0"), scale=4, coat=1.0)
    pe = mat("pearl", c("#fdfaf6"), 0.08, coat=1.0, col2=c("#e8f0f8"), scale=3)
    for (x, y, yaw, s, open_) in [(-0.1, 0.08, 0.3, 1.3, True), (0.28, -0.15, 2.0, 0.9, True), (-0.3, -0.25, 4.0, 0.7, False)]:
        b.blob(sh, (x, y, 0.03 * s), (0.14 * s, 0.12 * s, 0.035 * s), yaw, seg=12)
        if open_:
            b.blob(inner, (x, y, 0.06 * s), (0.11 * s, 0.1 * s, 0.015 * s), yaw, seg=12)
            b.blob(sh, (x - 0.1 * s * math.cos(yaw), y - 0.1 * s * math.sin(yaw), 0.13 * s), (0.14 * s, 0.12 * s, 0.03 * s), yaw, seg=12, tilt=-1.1)
            b.blob(pe, (x, y, 0.1 * s), (0.05 * s, 0.05 * s, 0.05 * s), seg=12)
        else:
            b.blob(sh, (x, y, 0.07 * s), (0.14 * s, 0.12 * s, 0.035 * s), yaw, seg=12)
    return {"fit": False}


@model("resource_salt")
def salt(b):
    sm = mat("salt", c("#f6f6f4"), 0.35, col2=c("#d8dce4"), scale=30, bump=0.5)
    pk = mat("pinksalt", c("#e8b8b0"), 0.3, col2=c("#c88c88"), scale=8, coat=0.4)
    for (x, y, r, h) in [(-0.1, 0.05, 0.3, 0.3), (0.28, -0.08, 0.2, 0.2), (-0.3, -0.22, 0.17, 0.16), (0.1, 0.3, 0.16, 0.14)]:
        b.cyl(sm, (x, y, 0), (x, y, h), r, r * 0.12, seg=7, smooth=False)
    for (x, y, tx, ty) in [(0.25, 0.22, 0.2, 0.1), (-0.35, 0.15, 0.1, 0.2), (0.0, -0.3, 0.3, 0.0)]:
        b.box(pk, (x, y, 0.06), (0.12, 0.1, 0.1), yaw=x * 7)
        b.box(pk, (x + 0.05, y - 0.03, 0.15), (0.08, 0.08, 0.08), yaw=y * 5)
    return {}


@model("resource_silk")
def silk(b):
    wd = wood_mat("#6a4a30")
    lm = mat("mulb", c("#7ab040"), 0.55, col2=c("#5a9a30"), scale=10, sss=0.3)
    cm = mat("cocoon", c("#fbf8ee"), 0.9, col2=c("#e8e2d0"), scale=40, bump=0.5)
    b.tube(wd, [(-0.15, 0.1, 0), (-0.12, 0.1, 0.25), (-0.15, 0.1, 0.45)], 0.04, 0.025, seg=7)
    b.blob(lm, (-0.15, 0.1, 0.6), (0.28, 0.24, 0.18), seg=10)
    b.blob(lm, (0.0, 0.1, 0.5), (0.16, 0.14, 0.1), seg=8)
    b.blob(lm, (-0.3, 0.08, 0.5), (0.14, 0.12, 0.1), seg=8)
    for (x, y, z) in [(-0.05, 0.0, 0.44), (-0.25, 0.0, 0.42), (-0.15, -0.05, 0.4)]:
        b.blob(cm, (x, y, z), (0.04, 0.04, 0.06), seg=8, tilt=0.3)
    for (x, y, yaw) in [(0.2, -0.2, 0.3), (0.3, -0.12, 1.2), (0.12, -0.3, 2.0), (0.35, -0.28, 0.8)]:
        b.blob(cm, (x, y, 0.045), (0.055, 0.045, 0.045), yaw, seg=8)
    rd = mat("silkred", c("#b02838"), 0.3, col2=c("#8a1c2a"), scale=40, coat=0.4)
    b.cyl(rd, (-0.35, -0.25, 0.08), (-0.1, -0.3, 0.08), 0.08, 0.08, seg=12)
    return {}


@model("resource_silver")
def silver(b):
    rm = rock_mat("#7a756e", "#4a4540")
    sv = mat("silver", c("#d4d8de"), 0.22, metal=1.0, col2=c("#a0a4aa"), scale=10)
    b.rock(rm, (-0.12, 0.08, 0.12), (0.34, 0.28, 0.22), 1)
    b.rock(rm, (0.25, -0.12, 0.09), (0.2, 0.18, 0.15), 2)
    for i, (x, y, z, r) in enumerate([(-0.02, -0.1, 0.2, 0.09), (-0.22, 0.14, 0.27, 0.08), (0.3, -0.04, 0.2, 0.07), (0.12, 0.2, 0.15, 0.07)]):
        b.rock(sv, (x, y, z), (r, r * 0.9, r * 0.8), 10 + i, rough=0.35)
    for (x, y, yaw, z) in [(0.1, -0.35, 0.3, 0.03), (0.2, -0.33, 0.35, 0.03), (0.15, -0.34, 0.33, 0.08)]:
        b.box(sv, (x, y, z), (0.16, 0.07, 0.05), yaw=yaw)
    return {}


@model("resource_spices")
def spices(b):
    sack(b, (-0.25, 0.1, 0), 1.2, "#c9b078", "#d8801a")
    sack(b, (0.12, 0.2, 0), 1.0, "#c0a068", "#a82020")
    sack(b, (0.3, -0.1, 0), 1.1, "#c9b078", "#e8c030")
    heap(b, (-0.1, -0.2, 0), 0.1, 0.12, "#c84a1a")
    heap(b, (0.12, -0.3, 0), 0.08, 0.1, "#e8a820")
    cm = mat("chili", c("#d02018"), 0.35, coat=0.5)
    for i in range(4):
        b.tube(cm, [(-0.4 + 0.05 * i, -0.15 - 0.03 * i, 0.02), (-0.36 + 0.05 * i, -0.18 - 0.03 * i, 0.04), (-0.32 + 0.05 * i, -0.2 - 0.03 * i, 0.02)], 0.018, 0.004, seg=5)
    return {}


@model("resource_sugar")
def sugar(b):
    rng = random.Random(2)
    cm = mat("cane", c("#7ab040"), 0.45, col2=c("#a8c860"), scale=20, stripes=18)
    lm = mat("canel", c("#5a9a30"), 0.5)
    for i in range(22):
        a = rng.random() * 6.28
        r = 0.42 * math.sqrt(rng.random())
        x, y = r * math.cos(a), r * math.sin(a) * 0.9
        h = 0.5 + rng.random() * 0.25
        lx, ly = (rng.random() - 0.5) * 0.08, (rng.random() - 0.5) * 0.08
        b.cyl(cm, (x, y, 0), (x + lx, y + ly, h), 0.02, 0.016, seg=6)
        for k in range(3):
            aa = k * 2.1 + i
            b.tube(lm, [(x + lx, y + ly, h), (x + lx + math.cos(aa) * 0.09, y + ly + math.sin(aa) * 0.09, h + 0.06), (x + lx + math.cos(aa) * 0.17, y + ly + math.sin(aa) * 0.17, h - 0.04)], 0.016, 0.003, seg=3)
    for i in range(4):
        b.cyl(cm, (0.25 + 0.03 * i, -0.4, 0.03 + 0.03 * i), (-0.05 + 0.03 * i, -0.38, 0.03 + 0.03 * i), 0.022, 0.022, seg=6)
    return {}


@model("resource_tea")
def tea(b):
    gm = mat("teag", c("#3a7a2e"), 0.55, col2=c("#5a9a38"), scale=22, bump=0.5)
    tip = mat("teatip", c("#9ac850"), 0.5)
    for k, y in enumerate([-0.3, -0.05, 0.2]):
        for i in range(4):
            x = -0.3 + i * 0.2
            b.blob(gm, (x, y + 0.04 * k - 0.04, 0.09), (0.12, 0.1, 0.09), seg=8)
            b.blob(tip, (x, y + 0.04 * k - 0.04, 0.16), (0.07, 0.06, 0.03), seg=6)
    cm = mat("teacup", c("#f2eee4"), 0.2, coat=0.8)
    b.cyl(cm, (0.35, 0.38, 0.0), (0.35, 0.38, 0.08), 0.04, 0.07, seg=12)
    b.disc(mat("teaf", c("#a85a20"), 0.2), (0.35, 0.38, 0.065), 0.062, 0.01, seg=12)
    return {}


@model("resource_tobacco")
def tobacco(b):
    lm = mat("tobl", c("#6a9a30"), 0.6, col2=c("#8aaa40"), scale=8, sss=0.3)
    dm = mat("tobd", c("#9a6a28"), 0.8, col2=c("#7a4a18"), scale=12)
    for (x, y) in [(-0.2, 0.1), (0.12, 0.18), (0.22, -0.1), (-0.12, -0.2)]:
        b.cyl(lm, (x, y, 0), (x, y, 0.4), 0.02, 0.012, seg=5)
        for k in range(6):
            a = k * 1.05 + x * 4
            z = 0.08 + 0.05 * k
            b.tube(lm, [(x, y, z), (x + math.cos(a) * 0.12, y + math.sin(a) * 0.12, z + 0.05), (x + math.cos(a) * 0.2, y + math.sin(a) * 0.2, z - 0.04)], 0.05, 0.006, seg=4)
    for i in range(3):
        b.tube(dm, [(0.3, 0.3 + 0.06 * i, 0.1), (0.3, 0.3 + 0.06 * i, 0.05), (0.31, 0.3 + 0.06 * i, 0.0)], 0.03, 0.02, seg=6)
    b.cyl(wood_mat(), (0.2, 0.25, 0.14), (0.4, 0.25, 0.14), 0.01, 0.01, seg=5)
    return {}


@model("resource_truffles")
def truffles(b):
    tree(b, (-0.12, 0.12, 0), 0.8, leaf="#6a8a2a", trunk="#4a3220", s=2.0)
    tm = mat("truffle", c("#2a2018"), 0.8, col2=c("#4a3a2a"), scale=40, bump=1.0)
    for i, (x, y, r) in enumerate([(0.2, -0.2, 0.08), (0.34, -0.1, 0.06), (0.12, -0.32, 0.05), (0.3, -0.3, 0.07), (-0.05, -0.3, 0.05)]):
        b.rock(tm, (x, y, r * 0.5), (r, r * 0.9, r * 0.8), 30 + i, rough=0.2)
    gm = grass_mat("#5a7a2a", "#3d5a20")
    for p in [(0.0, -0.1), (0.35, 0.1), (-0.3, -0.2)]:
        b.tuft(gm, (p[0], p[1], 0), 0.12, 5)
    return {}


@model("resource_whales")
def whales(b):
    water_rings(b, 0.7, 2)
    wm = mat("whale", c("#4a6a8a"), 0.35, col2=c("#8aa6bc"), scale=6, coat=0.3)
    wb = mat("whalebelly", c("#d8e0e8"), 0.4)
    b.blob(wm, (0.0, 0.0, 0.05), (0.55, 0.22, 0.18), 0.3, seg=18)
    b.blob(wb, (0.05, 0.02, 0.0), (0.45, 0.2, 0.1), 0.3, seg=14)
    b.tube(wm, [(-0.45, -0.14, 0.07), (-0.6, -0.2, 0.18), (-0.66, -0.2, 0.3)], 0.1, 0.03, seg=8)
    b.blob(wm, (-0.66, -0.2, 0.32), (0.07, 0.15, 0.03), 0.4 + 0.3, seg=10)
    b.blob(wm, (-0.66, -0.2, 0.32), (0.15, 0.07, 0.03), 0.4 + 0.3, seg=10)
    b.blob(wm, (0.2, 0.18, 0.05), (0.14, 0.05, 0.03), 0.3 + 0.5, seg=8)
    em = mat("weye", c("#101018"), 0.2)
    b.blob(em, (0.36, 0.1, 0.14), (0.02, 0.02, 0.02), seg=6)
    smoke(b, (0.25, 0.0, 0.22), 4, 0.8, "#e8f4ff", 0.6)
    return {"fit": False}


@model("resource_wine")
def wine(b):
    wd = wood_mat("#6a4a30")
    lm = mat("vine", c("#5a9a30"), 0.55, col2=c("#3f7a28"), scale=12, sss=0.3)
    gp = mat("grape", c("#5a2a7a"), 0.25, coat=0.8, col2=c("#3a1a58"), scale=30)
    for x in (-0.35, -0.05, 0.25):
        b.cyl(wd, (x, 0.15, 0), (x, 0.15, 0.5), 0.02, 0.02, seg=5)
    b.cyl(wd, (-0.4, 0.15, 0.45), (0.3, 0.15, 0.45), 0.012, 0.012, seg=5)
    b.cyl(wd, (-0.4, 0.15, 0.3), (0.3, 0.15, 0.3), 0.012, 0.012, seg=5)
    for i in range(9):
        x = -0.38 + i * 0.09
        b.blob(lm, (x, 0.15, 0.38 + 0.06 * (i % 2)), (0.07, 0.04, 0.09), seg=6)
    for x in (-0.28, -0.1, 0.1, 0.2):
        for k in range(8):
            a = k * 0.8
            b.blob(gp, (x + math.cos(a) * 0.025, 0.1 + math.sin(a) * 0.02, 0.3 - 0.015 * k), (0.026, 0.026, 0.026), seg=6)
    barrel(b, (0.12, -0.28, 0), 1.2)
    barrel(b, (0.38, -0.2, 0), 0.9)
    return {}


@model("resource_jeans")
def jeans(b):
    dn = mat("denim", c("#2a4a86"), 0.95, col2=c("#1e3a6e"), scale=60, bump=0.4)
    dn2 = mat("denim2", c("#3a5ea0"), 0.95, col2=c("#2a4a86"), scale=60, bump=0.4)
    for i in range(4):
        b.box(dn if i % 2 else dn2, (0, 0, 0.04 + 0.075 * i), (0.5, 0.34, 0.07), yaw=0.1 * (i % 2 - 0.5))
    st = mat("stitch", c("#e0a838"), 0.6)
    b.box(st, (0.0, 0.0, 0.31), (0.12, 0.05, 0.01))
    b.cyl(dn, (-0.3, -0.35, 0.09), (0.3, -0.35, 0.09), 0.09, 0.09, seg=12)
    return {}


@model("resource_perfume")
def perfume(b):
    gl = mat("glass", c("#f0b0c8"), 0.05, coat=1.0, transmission=0.6, ior=1.45)
    gl2 = mat("glass2", c("#a8d8f0"), 0.05, coat=1.0, transmission=0.6, ior=1.45)
    gd = mat("goldcap", c("#d8a838"), 0.3, metal=1.0)
    for (x, y, g, s) in [(-0.12, 0.05, gl, 1.3), (0.2, -0.1, gl2, 1.0)]:
        b.box(g, (x, y, 0.14 * s), (0.2 * s, 0.14 * s, 0.24 * s))
        b.cyl(gd, (x, y, 0.26 * s), (x, y, 0.34 * s), 0.05 * s, 0.05 * s, seg=10)
        b.blob(gd, (x, y, 0.36 * s), (0.05 * s, 0.05 * s, 0.03 * s), seg=8)
    fm = mat("petal", c("#e04a7a"), 0.5)
    for k in range(6):
        a = k * 1.05
        b.blob(fm, (0.3 + math.cos(a) * 0.05, 0.28 + math.sin(a) * 0.05, 0.03), (0.04, 0.025, 0.012), a, seg=6)
    return {}


@model("resource_cosmetics")
def cosmetics(b):
    pk = mat("cpink", c("#e86a98"), 0.3, coat=0.6)
    gd = mat("cgold", c("#d8a838"), 0.3, metal=1.0)
    wh = mat("cwhite", c("#f4f0f0"), 0.4, coat=0.5)
    b.cyl(pk, (-0.2, 0.05, 0), (-0.2, 0.05, 0.1), 0.15, 0.15, seg=18)
    b.cyl(wh, (-0.2, 0.05, 0.1), (-0.2, 0.05, 0.14), 0.15, 0.15, seg=18)
    b.disc(gd, (-0.2, 0.05, 0.14), 0.08, 0.006, seg=16)
    b.cyl(gd, (0.2, -0.05, 0), (0.2, -0.05, 0.14), 0.045, 0.045, seg=10)
    b.cyl(pk, (0.2, -0.05, 0.14), (0.2, -0.05, 0.24), 0.04, 0.032, seg=10)
    b.blob(pk, (0.2, -0.05, 0.25), (0.032, 0.032, 0.03), seg=8)
    b.blob(pk, (0.1, 0.3, 0.05), (0.1, 0.1, 0.05), seg=10)
    b.cyl(gd, (0.28, 0.25, 0.0), (0.28, 0.25, 0.1), 0.06, 0.06, seg=10)
    return {}


@model("resource_toys")
def toys(b):
    cols = ["#d83a3a", "#3a7ad8", "#e8c030", "#3aa850"]
    for i, (x, y, z, yaw) in enumerate([(-0.2, 0.0, 0.07, 0.2), (0.02, 0.05, 0.07, -0.2), (-0.08, 0.02, 0.21, 0.5), (0.2, -0.15, 0.07, 0.7), (-0.25, -0.25, 0.07, 0.1)]):
        m = mat("toy" + cols[i % 4], c(cols[i % 4]), 0.4, coat=0.3)
        b.box(m, (x, y, z), (0.14, 0.14, 0.14), yaw=yaw)
    tb = mat("teddy", c("#b07a3c"), 1.0, col2=c("#8a5a28"), scale=40, bump=0.5)
    b.blob(tb, (0.25, 0.25, 0.1), (0.1, 0.09, 0.11), seg=10)
    b.blob(tb, (0.25, 0.25, 0.25), (0.075, 0.07, 0.07), seg=10)
    b.blob(tb, (0.18, 0.25, 0.3), (0.025, 0.02, 0.025), seg=6)
    b.blob(tb, (0.32, 0.25, 0.3), (0.025, 0.02, 0.025), seg=6)
    for sx in (-1, 1):
        b.blob(tb, (0.25 + sx * 0.1, 0.22, 0.14), (0.035, 0.035, 0.06), seg=6)
    return {}


@model("resource_cinnamon")
def cinnamon(b):
    cm = mat("cinn", c("#a8602c"), 0.8, col2=c("#7a4018"), scale=35, bump=0.8)
    tm = mat("twine", c("#c8b078"), 1.0)
    for (x, y, yaw) in [(-0.15, 0.05, 0.3), (0.15, -0.1, -0.5)]:
        for i in range(9):
            a = i * 0.7
            dx, dy = math.cos(a) * 0.05, math.sin(a) * 0.05
            b.cyl(cm, (x + dx, y + dy, 0.0), (x + dx * 0.8 + 0.02, y + dy * 0.8, 0.34), 0.018, 0.018, seg=6)
        b.cyl(tm, (x, y, 0.12), (x, y, 0.15), 0.075, 0.075, seg=10)
    for i in range(5):
        b.cyl(cm, (-0.3 + 0.025 * i, -0.25 + 0.01 * i, 0.03), (0.1 + 0.025 * i, -0.3, 0.03), 0.02, 0.02, seg=6)
    lm = mat("cinnleaf", c("#4a8a30"), 0.5, coat=0.4)
    for k in range(5):
        a = k * 1.3
        b.blob(lm, (0.35 + math.cos(a) * 0.08, 0.28 + math.sin(a) * 0.08, 0.06), (0.09, 0.04, 0.012), a, seg=8)
    return {}


@model("resource_cloves")
def cloves(b):
    lm = mat("clvl", c("#3a7a30"), 0.5, coat=0.4, col2=c("#2a5a24"), scale=10)
    bd = mat("clvb", c("#e87a8a"), 0.5)
    cm = mat("clove", c("#5a2e18"), 0.7)
    for (x, y, r) in [(-0.15, 0.12, 0.25), (0.22, 0.05, 0.2)]:
        b.blob(lm, (x, y, r * 0.9), (r, r, r), seg=10)
        for k in range(7):
            a = k * 0.9
            b.blob(bd, (x + math.cos(a) * r * 0.9, y + math.sin(a) * r * 0.9, r * 0.9 + 0.04 * (k % 3)), (0.022, 0.022, 0.03), seg=6)
    for i in range(14):
        a = i * 2.4
        rr = 0.12 * math.sqrt(i / 14)
        b.cone(cm, (0.0 + math.cos(a) * rr, -0.3 + math.sin(a) * rr, 0.01), 0.07, 0.014, seg=5, tip_r=0.008)
        b.blob(cm, (math.cos(a) * rr, -0.3 + math.sin(a) * rr, 0.08), (0.013, 0.013, 0.013), seg=5)
    return {}


@model("resource_honey")
def honey(b):
    sk = mat("skep", c("#d8a840"), 0.9, col2=c("#a87820"), scale=30, bump=0.6)
    for i, (z, r) in enumerate([(0.0, 0.2), (0.07, 0.18), (0.14, 0.15), (0.21, 0.11), (0.27, 0.07)]):
        b.cyl(sk, (-0.1, 0.1, z), (-0.1, 0.1, z + 0.08), r, r * 0.92, seg=14)
    b.blob(sk, (-0.1, 0.1, 0.36), (0.06, 0.06, 0.04), seg=8)
    dk = mat("hole", c("#3a2810"), 0.9)
    b.blob(dk, (-0.1, -0.08, 0.06), (0.05, 0.02, 0.05), seg=8)
    hm = mat("honey", c("#f0a818"), 0.1, coat=1.0, emit=0.2, emit_col=c("#f0a818"), transmission=0.2)
    gl = mat("hjar", c("#e8e8e0"), 0.1, alpha=0.5, coat=1.0)
    b.cyl(hm, (0.25, -0.1, 0.0), (0.25, -0.1, 0.17), 0.09, 0.09, seg=14)
    b.cyl(gl, (0.25, -0.1, 0.17), (0.25, -0.1, 0.2), 0.092, 0.092, seg=14)
    b.cyl(wood_mat(), (0.25, -0.1, 0.2), (0.25, -0.1, 0.24), 0.095, 0.095, seg=14)
    hc = mat("comb", c("#f0b830"), 0.4, coat=0.5)
    for k in range(7):
        a = k * 1.05
        r = 0.045 if k else 0
        b.prism(hc, (0.0 + math.cos(a) * r * 1.9 - 0.05, -0.32 + math.sin(a) * r * 1.9, 0.0), 0.05, 0.04, sides=6)
    bee = mat("bee", c("#f0c020"), 0.5, stripes=60, col2=c("#202020"), scale=60)
    for (x, y, z) in [(0.0, 0.2, 0.4), (0.15, 0.3, 0.3), (-0.3, 0.3, 0.25)]:
        b.blob(bee, (x, y, z), (0.03, 0.02, 0.02), seg=6)
        b.blob(mat("wing", c("#e8f0f8"), 0.2, alpha=0.5), (x, y + 0.01, z + 0.025), (0.02, 0.035, 0.004), seg=6)
    return {}
