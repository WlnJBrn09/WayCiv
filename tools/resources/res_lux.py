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
    """Dye vats of red, indigo and saffron, and freshly dyed cloth drying on a line."""
    wd = wood_mat("#7a5232", "#4e331d")
    for (x, y, col) in [(-0.3, -0.12, "#c8283a"), (0.02, -0.26, "#2a46b0"), (0.32, -0.1, "#f0b81c")]:
        b.cyl(wd, (x, y, 0), (x, y, 0.16), 0.13, 0.14, seg=16)
        b.ring(wd, (x, y, 0.16), 0.14, 0.02, 0.02, seg=24)
        b.blob(mat("dye" + col, c(col), 0.15, coat=0.6), (x, y, 0.15), (0.125, 0.125, 0.012), seg=20)
    for x in (-0.42, 0.42):
        b.cyl(wd, (x, 0.22, 0), (x, 0.22, 0.5), 0.018, 0.016, seg=6)
    b.cyl(mat("line", c("#d8c8a0"), 0.9), (-0.42, 0.22, 0.47), (0.42, 0.22, 0.47), 0.006, 0.006, seg=4)
    for i, col in enumerate(["#c8283a", "#2a46b0", "#f0b81c", "#7a2a8a", "#e06a20"]):
        x = -0.32 + i * 0.16
        b.box(mat("cloth" + col, c(col), 0.9, col2=c(col), scale=30, bump=0.2), (x, 0.22, 0.34), (0.13, 0.01, 0.26),
              yaw=0.08 * (i % 2 - 0.5))
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
    """Red cinnabar ore, a glass flask and a bright pool of quicksilver."""
    cm = mat("cinnabar", c("#c0261e"), 0.45, col2=c("#701410"), scale=6, bump=0.5)
    rm = rock_mat("#8a7a6a", "#5a4c40")
    b.rock(rm, (-0.15, 0.2, 0.12), (0.34, 0.26, 0.2), 1)
    b.rock(cm, (-0.28, 0.12, 0.16), (0.2, 0.17, 0.18), 4)
    b.rock(cm, (0.05, 0.28, 0.13), (0.16, 0.13, 0.15), 5)
    b.rock(cm, (0.35, 0.12, 0.08), (0.13, 0.11, 0.1), 2)
    qm = mat("quick", c("#e4e8ec"), 0.03, metal=1.0)
    b.blob(qm, (0.05, -0.2, 0.0), (0.26, 0.17, 0.025), seg=20)
    for (x, y, r) in [(0.1, -0.17, 0.045), (-0.04, -0.24, 0.035), (0.28, -0.26, 0.03), (-0.18, -0.12, 0.025)]:
        b.blob(qm, (x, y, r * 0.7), (r, r, r * 0.8), seg=10)
    gl = mat("flask", c("#d8ecf4"), 0.04, coat=1.0, transmission=0.85, ior=1.45)
    b.blob(gl, (0.36, -0.12, 0.09), (0.09, 0.09, 0.09), seg=14)
    b.cyl(gl, (0.36, -0.12, 0.16), (0.36, -0.12, 0.27), 0.03, 0.03, seg=10)
    b.blob(qm, (0.36, -0.12, 0.05), (0.07, 0.07, 0.04), seg=12)
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
    """A mulberry tree with cocoons, and bolts of shining silk in three colours."""
    wd = wood_mat("#6a4a30")
    lm = mat("mulb", c("#7ab040"), 0.55, col2=c("#5a9a30"), scale=10, sss=0.3)
    cm = mat("cocoon", c("#fbf8ee"), 0.9, col2=c("#e8e2d0"), scale=40, bump=0.5)
    b.tube(wd, [(-0.2, 0.18, 0), (-0.18, 0.18, 0.25), (-0.2, 0.18, 0.45)], 0.04, 0.025, seg=7)
    b.blob(lm, (-0.2, 0.18, 0.6), (0.28, 0.24, 0.18), seg=10)
    b.blob(lm, (-0.05, 0.18, 0.5), (0.16, 0.14, 0.1), seg=8)
    b.blob(lm, (-0.35, 0.16, 0.5), (0.14, 0.12, 0.1), seg=8)
    for (x, y, z) in [(-0.1, 0.06, 0.44), (-0.3, 0.06, 0.42), (-0.2, 0.02, 0.4)]:
        b.blob(cm, (x, y, z), (0.04, 0.04, 0.06), seg=8, tilt=0.3)
    for i, (col, col2) in enumerate([("#c8283a", "#e85a6a"), ("#e0b030", "#fff0a0"), ("#2a8a8a", "#6ad0c8")]):
        sm = mat("silk" + col, c(col), 0.2, col2=c(col2), scale=3, coat=0.8, wave=True)
        y = -0.12 - 0.11 * i
        b.cyl(sm, (0.0 + 0.04 * i, y, 0.075), (0.42 + 0.04 * i, y - 0.04, 0.075), 0.07, 0.07, seg=16)
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
    """Tea bushes clipped into rounded hedges along a terraced slope, a picker's basket."""
    soft_ground(b, "#6a5232", "#4e3a22", 0.6, 0.8, z=-0.01, scale=8)
    earth = mat("teaterr", c("#7a5a38"), 0.95, col2=c("#5a4128"), scale=8, bump=0.3)
    gm = mat("teag", c("#2f7a2a"), 0.55, col2=c("#58a03a"), scale=26, bump=0.6, sss=0.2)
    tip = mat("teatip", c("#a8d858"), 0.5, col2=c("#8ac040"), scale=30)
    for k, (y, z) in enumerate([(0.3, 0.12), (0.05, 0.06), (-0.2, 0.0)]):
        b.box(earth, (0.0, y, z / 2), (0.95 - 0.1 * k, 0.24, z + 0.02))
        for i in range(2):
            yy = y - 0.05 + i * 0.1
            b.blob(gm, (0.0, yy, z + 0.09), (0.42 - 0.05 * k, 0.06, 0.08), seg=16)
            for j in range(7):
                b.blob(tip, (-0.33 + j * 0.11 + 0.03 * i, yy, z + 0.16), (0.04, 0.03, 0.015), seg=6)
    wk = mat("basket", c("#b88a4a"), 0.8, col2=c("#8a6030"), scale=40, bump=0.4)
    b.cyl(wk, (0.32, -0.38, 0), (0.32, -0.38, 0.12), 0.08, 0.1, seg=14)
    b.blob(tip, (0.32, -0.38, 0.12), (0.085, 0.085, 0.03), seg=10)
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
    """An oak with a pig snuffling at its roots and a basket of black truffles."""
    tree(b, (-0.15, 0.15, 0), 0.8, leaf="#4f7a28", trunk="#4a3220", s=2.0)
    quad(b, 0.2, -0.12, yaw=2.6, s=0.9, kind="cow", body="#e8a8a0", accent="#e8a8a0", dark="#b07070")
    tm = mat("truffle", c("#2a2018"), 0.8, col2=c("#4a3a2a"), scale=40, bump=1.0)
    wk = mat("basket", c("#b88a4a"), 0.8, col2=c("#8a6030"), scale=40, bump=0.4)
    b.cyl(wk, (-0.25, -0.3, 0), (-0.25, -0.3, 0.09), 0.1, 0.12, seg=14)
    for i, (x, y) in enumerate([(-0.27, -0.29), (-0.21, -0.32), (-0.26, -0.35), (-0.22, -0.26)]):
        b.rock(tm, (x, y, 0.1), (0.035, 0.035, 0.03), 30 + i, rough=0.25)
    for i, (x, y, r) in enumerate([(0.0, -0.35, 0.04), (0.08, -0.4, 0.035)]):
        b.rock(tm, (x, y, r * 0.5), (r, r * 0.9, r * 0.8), 40 + i, rough=0.2)
    gm = grass_mat("#5a7a2a", "#3d5a20")
    for p in [(0.38, 0.12), (-0.42, -0.1), (0.05, 0.3)]:
        b.tuft(gm, (p[0], p[1], 0), 0.12, 5)
    return {}


@model("resource_whales")
def whales(b):
    """A humpback breaking the surface: a long dark back, a raised fluke, a spout."""
    water_rings(b, 0.7, 2)
    wm = mat("whale", c("#34495e"), 0.35, col2=c("#5a7088"), scale=6, coat=0.3)
    wb = mat("whalebelly", c("#d8e0e8"), 0.4)
    # the long back breaking the surface, the head a little higher
    b.blob(wm, (0.05, 0.0, 0.03), (0.42, 0.15, 0.1), -0.15, seg=20)
    b.blob(wm, (0.32, 0.04, 0.05), (0.16, 0.13, 0.09), -0.15, seg=16)
    b.blob(wb, (0.2, -0.06, 0.02), (0.22, 0.06, 0.03), -0.15, seg=12)
    b.blob(wm, (-0.12, 0.02, 0.12), (0.05, 0.015, 0.05), -0.15, seg=8)  # the little dorsal fin
    # the fluke, lifted clear behind
    b.tube(wm, [(-0.33, -0.08, 0.04), (-0.42, -0.12, 0.14), (-0.46, -0.14, 0.26)], 0.05, 0.03, seg=8)
    for sgn in (-1, 1):
        b.blob(wm, (-0.47, -0.14 + sgn * 0.1, 0.3), (0.06, 0.11, 0.02), sgn * 0.5, seg=10, tilt=sgn * 0.3)
        b.blob(wb, (-0.465, -0.14 + sgn * 0.1, 0.295), (0.05, 0.095, 0.015), sgn * 0.5, seg=10, tilt=sgn * 0.3)
    foam = mat("splash", c("#f4faff"), 0.5, alpha=0.85)
    for (x, y, r) in [(0.42, 0.05, 0.08), (-0.33, -0.08, 0.07), (0.1, 0.0, 0.06)]:
        b.blob(foam, (x, y, 0.0), (r * 1.6, r, 0.02), seg=12)
    smoke(b, (0.45, 0.08, 0.2), 4, 0.8, "#e8f4ff", 0.6)
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
    """A folded stack of denim jeans on a crate and a bolt of indigo cloth."""
    dn = mat("denim", c("#2a4a86"), 0.95, col2=c("#1e3a6e"), scale=60, bump=0.4)
    dn2 = mat("denim2", c("#4a6eb0"), 0.95, col2=c("#3a5a96"), scale=60, bump=0.4)
    st = mat("stitch", c("#e0a838"), 0.6)
    crate(b, (-0.12, 0.12, 0), 1.6, 0.15)
    for i in range(4):
        z = 0.27 + 0.055 * i
        b.box(dn if i % 2 else dn2, (-0.12, 0.12, z), (0.42, 0.28, 0.05), yaw=0.15 + 0.06 * (i % 2 - 0.5))
        b.box(st, (-0.12 - 0.2, 0.12 + 0.03, z + 0.002), (0.012, 0.22, 0.046), yaw=0.15)
    # one pair laid out flat: waistband, two legs, back pockets
    lay = -0.5
    ca, sa = math.cos(lay), math.sin(lay)
    def pt(lx, ly):
        return (0.22 + lx * ca - ly * sa, -0.22 + lx * sa + ly * ca)
    x, y = pt(0.0, 0.0)
    b.box(dn2, (x, y, 0.015), (0.26, 0.2, 0.03), yaw=lay)
    for sgn in (-1, 1):
        x, y = pt(sgn * 0.065, -0.25)
        b.box(dn2, (x, y, 0.012), (0.11, 0.34, 0.024), yaw=lay + sgn * 0.06)
        x, y = pt(sgn * 0.06, 0.02)
        b.box(dn, (x, y, 0.032), (0.08, 0.08, 0.006), yaw=lay)
    x, y = pt(0.0, 0.1)
    b.box(st, (x, y, 0.032), (0.26, 0.012, 0.006), yaw=lay)
    b.cyl(dn, (-0.42, -0.25, 0.07), (-0.1, -0.38, 0.07), 0.07, 0.07, seg=14)
    return {}


@model("resource_perfume")
def perfume(b):
    """Cut-glass perfume bottles on a little tray beside a rose bush."""
    gl = mat("glass", c("#f0a8c8"), 0.03, coat=1.0, transmission=0.7, ior=1.45)
    gl2 = mat("glass2", c("#a8d8f0"), 0.03, coat=1.0, transmission=0.7, ior=1.45)
    gl3 = mat("glass3", c("#f4d890"), 0.03, coat=1.0, transmission=0.7, ior=1.45)
    gd = mat("goldcap", c("#d8a838"), 0.25, metal=1.0)
    wd = wood_mat("#6a3a22", "#4a2614")
    b.cyl(wd, (0.05, -0.05, 0), (0.05, -0.05, 0.04), 0.34, 0.34, seg=24)
    b.ring(gd, (0.05, -0.05, 0.04), 0.34, 0.02, 0.01, seg=32)
    for (x, y, g, s) in [(-0.08, 0.02, gl, 1.25), (0.16, -0.08, gl2, 1.0), (0.0, -0.22, gl3, 0.85)]:
        b.blob(g, (x, y, 0.04 + 0.1 * s), (0.1 * s, 0.1 * s, 0.1 * s), seg=16)
        b.cyl(g, (x, y, 0.04 + 0.18 * s), (x, y, 0.04 + 0.24 * s), 0.025 * s, 0.025 * s, seg=10)
        b.blob(gd, (x, y, 0.04 + 0.27 * s), (0.045 * s, 0.045 * s, 0.045 * s), seg=10)
    bush(b, (-0.36, 0.25, 0), 0.16, "#3f7a2a", "#2f6020")
    fm = mat("petal", c("#e03a6a"), 0.5, sss=0.2)
    for (x, y, z) in [(-0.4, 0.2, 0.2), (-0.3, 0.3, 0.22), (-0.42, 0.33, 0.16), (-0.28, 0.18, 0.15)]:
        b.blob(fm, (x, y, z), (0.04, 0.04, 0.03), seg=8)
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
