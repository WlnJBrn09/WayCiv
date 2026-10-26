"""Strategic resources and artifact sites."""
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


@model("resource_horses")
def horses(b):
    quad(b, -0.12, 0.1, 0.5, 1.25, "horse", "#7a4a26", "#2a1c14", "#1c1410")
    quad(b, 0.28, -0.22, 3.0, 1.1, "horse", "#e8e0d0", "#c8bfae", "#8a8272")
    gm = grass_mat()
    for p in [(-0.5, -0.2), (0.45, 0.3), (0.0, 0.5)]:
        b.tuft(gm, (p[0], p[1], 0), 0.15, 6)
    return {"target": 1.25}


@model("resource_iron")
def iron(b):
    rm = rock_mat("#5c5a58", "#383634")
    rust = mat("rustore", c("#a8502a"), 0.8, col2=c("#6a3018"), scale=8, bump=0.5)
    fe = mat("iron", c("#8a9098"), 0.3, metal=1.0, col2=c("#5a5e66"), scale=10)
    b.rock(rm, (-0.1, 0.08, 0.14), (0.34, 0.28, 0.26), 1)
    b.rock(rm, (0.28, -0.1, 0.1), (0.22, 0.2, 0.18), 2)
    b.rock(rm, (-0.3, -0.25, 0.08), (0.16, 0.14, 0.12), 3)
    for i, (x, y, z, r) in enumerate([(-0.0, -0.1, 0.22, 0.1), (-0.22, 0.14, 0.3, 0.09), (0.34, -0.04, 0.2, 0.07)]):
        b.rock(rust, (x, y, z), (r, r * 0.9, r * 0.8), 10 + i, rough=0.35)
    for i, (x, y, z) in enumerate([(0.1, 0.2, 0.16), (-0.12, -0.1, 0.3), (0.32, -0.24, 0.1)]):
        b.rock(fe, (x, y, z), (0.07, 0.06, 0.05), 20 + i, rough=0.3)
    b.box(fe, (0.1, 0.36, 0.03), (0.2, 0.08, 0.05), yaw=0.2)
    b.box(fe, (0.14, 0.36, 0.08), (0.2, 0.08, 0.05), yaw=0.25)
    return {}


@model("resource_coal")
def coal(b):
    cm = mat("coal", c("#1c1c1e"), 0.4, col2=c("#3a3a3e"), scale=10, bump=0.5, coat=0.3)
    sp = mat("coalsp", c("#5a5a62"), 0.15, metal=0.6)
    for i, (x, y, z, r) in enumerate([(-0.1, 0.05, 0.12, 0.28), (0.2, -0.1, 0.1, 0.22), (-0.28, -0.2, 0.08, 0.16), (0.12, 0.28, 0.08, 0.16), (-0.3, 0.25, 0.07, 0.13), (0.38, 0.18, 0.06, 0.1), (-0.0, -0.3, 0.06, 0.1)]):
        b.rock(cm, (x, y, z), (r, r * 0.9, r * 0.8), i, subdiv=1, rough=0.3)
    for i in range(6):
        b.rock(sp, (-0.2 + 0.08 * i, 0.0 + 0.05 * (i % 3), 0.24), (0.03, 0.03, 0.02), 40 + i, subdiv=1)
    return {}


@model("resource_niter")
def niter(b):
    rm = rock_mat("#8a8478", "#5a564e")
    nm = mat("niter", c("#f4f0d8"), 0.4, col2=c("#d8d4b8"), scale=40, bump=0.7)
    ym = mat("niteryl", c("#e8dc90"), 0.35)
    b.rock(rm, (0.0, 0.05, 0.1), (0.4, 0.3, 0.18), 1)
    b.rock(rm, (0.3, -0.2, 0.07), (0.17, 0.15, 0.11), 2)
    rng = random.Random(4)
    for i in range(22):
        a = rng.random() * 6.28
        r = 0.35 * math.sqrt(rng.random())
        x, y = r * math.cos(a), r * math.sin(a) * 0.8
        h = 0.1 + rng.random() * 0.14
        lx, ly = (rng.random() - 0.5) * 0.3, (rng.random() - 0.5) * 0.3
        b.cyl(nm if i % 3 else ym, (x, y, 0.14), (x + lx * h, y + ly * h, 0.14 + h), 0.016, 0.004, seg=4)
    return {}


@model("resource_aluminum")
def aluminum(b):
    bx = mat("bauxite", c("#a8502e"), 0.85, col2=c("#c8844a"), scale=7, bump=0.7)
    bx2 = mat("bauxite2", c("#8a4226"), 0.85, col2=c("#a8502e"), scale=9, bump=0.7)
    al = mat("alum", c("#dce2e8"), 0.18, metal=1.0, col2=c("#b4bcc4"), scale=8)
    b.rock(bx, (-0.12, 0.08, 0.12), (0.32, 0.26, 0.22), 1)
    b.rock(bx2, (0.22, -0.05, 0.1), (0.2, 0.18, 0.16), 2)
    b.rock(bx2, (-0.3, -0.22, 0.07), (0.15, 0.13, 0.1), 3)
    for i in range(3):
        b.box(al, (0.15 + 0.02 * i, -0.34 + 0.0, 0.035 + 0.06 * i), (0.26 - 0.03 * i, 0.12, 0.06), yaw=0.15 * (i - 1))
    b.rock(al, (-0.1, 0.0, 0.3), (0.06, 0.05, 0.04), 7)
    return {}


@model("resource_oil")
def oil(b):
    om = mat("oilpool", c("#101014"), 0.04, coat=1.0, metal=0.3)
    rm = rock_mat("#7a6a58", "#4a3e32")
    b.blob(om, (0, 0, 0.0), (0.48, 0.38, 0.04), 0.4, seg=20)
    b.blob(om, (0.38, 0.1, 0.0), (0.18, 0.14, 0.035), 0.0, seg=14)
    b.blob(om, (-0.22, -0.28, 0.0), (0.15, 0.12, 0.03), 0.0, seg=12)
    for (x, y, r) in [(-0.1, 0.05, 0.05), (0.15, -0.08, 0.04), (0.05, 0.15, 0.035)]:
        b.blob(om, (x, y, r * 0.5), (r, r, r), seg=10)
    b.rock(rm, (-0.42, 0.18, 0.08), (0.17, 0.15, 0.11), 1)
    b.rock(rm, (0.12, 0.38, 0.06), (0.14, 0.12, 0.09), 2)
    b.rock(rm, (0.3, -0.35, 0.06), (0.12, 0.1, 0.08), 3)
    return {}


@model("resource_uranium")
def uranium(b):
    rm = rock_mat("#58605a", "#363c38")
    gm = mat("uran", c("#7aff3a"), 0.15, emit=3.0, emit_col=c("#5aff20"), coat=1.0)
    gl = mat("uglow", c("#8aff5a"), 0.2, emit=0.25, alpha=0.12)
    b.rock(rm, (-0.05, 0.05, 0.12), (0.38, 0.3, 0.22), 1)
    b.rock(rm, (0.3, -0.18, 0.08), (0.2, 0.17, 0.13), 2)
    for (x, y, h, tx, ty) in [(-0.12, 0.05, 0.38, 0.2, 0.1), (0.08, -0.03, 0.3, -0.2, 0.15), (0.3, -0.18, 0.24, 0.1, -0.2), (-0.3, -0.05, 0.22, 0.35, -0.1)]:
        b.crystal(gm, (x, y, 0.1), h, 0.06, (tx, ty), sides=5, tip=0.4)
    b.blob(gl, (0.0, 0.0, 0.22), (0.5, 0.4, 0.3), seg=14)
    return {}


@model("resource_antiquity_site")
def antiquity(b):
    st = mat("ruin", c("#d4c4a0"), 0.9, col2=c("#a89874"), scale=6, bump=0.5)
    st2 = mat("ruin2", c("#bfae88"), 0.9, col2=c("#8a7c5a"), scale=6, bump=0.5)
    soft_ground(b, "#d8c08a", "#c0a870", 0.62, 0.81)
    # broken columns
    b.cyl(st, (-0.28, 0.12, 0.0), (-0.28, 0.12, 0.5), 0.09, 0.08, seg=10)
    b.box(st2, (-0.28, 0.12, 0.54), (0.24, 0.24, 0.07))
    b.cyl(st, (0.0, 0.2, 0.0), (0.0, 0.2, 0.28), 0.09, 0.085, seg=10)
    b.cyl(st2, (0.3, -0.05, 0.0), (0.3, -0.05, 0.14), 0.09, 0.09, seg=10)
    b.cyl(st, (0.2, -0.32, 0.07), (0.5, -0.36, 0.07), 0.08, 0.08, seg=10)  # fallen
    b.box(st2, (-0.08, -0.15, 0.04), (0.3, 0.22, 0.08), yaw=0.3)
    b.rock(st, (-0.4, -0.2, 0.06), (0.1, 0.09, 0.07), 1)
    b.rock(st2, (0.12, 0.38, 0.05), (0.1, 0.08, 0.06), 2)
    cm = mat("amph", c("#b8603a"), 0.7, col2=c("#8a4020"), scale=8)
    b.blob(cm, (0.05, -0.3, 0.07), (0.06, 0.06, 0.08), seg=10, tilt=1.2)
    b.cyl(cm, (0.12, -0.3, 0.07), (0.17, -0.3, 0.07), 0.035, 0.045, seg=10)
    return {}


@model("resource_shipwreck")
def shipwreck(b):
    water_rings(b, 0.62, 2)
    wd = wood_mat("#6a4a2a", "#3a2814")
    dk = wood_mat("#8a6a3a", "#5a4020")
    # hull ribs and planks, listing
    for i in range(7):
        x = -0.4 + i * 0.13
        s = 1.0 - abs(i - 3) * 0.12
        b.tube(wd, [(x, -0.22 * s, 0.0), (x, -0.27 * s, 0.12), (x, -0.2 * s, 0.24)], 0.022, 0.016, seg=5)
        b.tube(wd, [(x, 0.22 * s, 0.0), (x, 0.27 * s, 0.1), (x, 0.2 * s, 0.2)], 0.022, 0.016, seg=5)
    b.cyl(wd, (-0.42, 0.0, 0.03), (0.45, 0.0, 0.03), 0.03, 0.03, seg=6)
    for y in (-0.2, -0.1, 0.1, 0.2):
        b.box(dk, (-0.05, y, 0.03), (0.8, 0.07, 0.02))
    # mast
    b.cyl(wd, (0.1, 0.0, 0.03), (0.28, 0.02, 0.62), 0.035, 0.022, seg=7)
    b.cyl(wd, (0.22, 0.0, 0.5), (0.18, 0.4, 0.52), 0.018, 0.018, seg=6)
    cl = mat("sail", c("#d8d0b8"), 0.95)
    b.box(cl, (0.2, 0.2, 0.36), (0.02, 0.34, 0.22), yaw=0.05)
    b.blob(wd, (-0.4, 0.0, 0.2), (0.03, 0.03, 0.2), seg=6, tilt=0.4)
    return {"fit": False, "target": 1.0}
