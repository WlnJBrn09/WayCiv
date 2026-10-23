"""Tile improvements and barbarian camps. Names: imp_<id without prefix>."""
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


STONE = lambda: mat("castle", c("#b8b0a0"), 0.9, col2=c("#8a8274"), scale=10, bump=0.4)  # noqa: E731
ROOF_RED = lambda: mat("tiles", c("#b8503a"), 0.7, col2=c("#8a3a28"), scale=20, bump=0.3)  # noqa: E731
PLASTER = lambda: mat("plaster", c("#e8dcc0"), 0.9, col2=c("#c8b896"), scale=12, bump=0.2)  # noqa: E731
THATCH = lambda: mat("thatch", c("#c8a850"), 1.0, col2=c("#9a7c34"), scale=40, bump=0.6)  # noqa: E731


def hex_edge(b, m, r, h, t, z=0.0):
    pts = hex_pts(r)
    b.wall_run(m, pts + [pts[0]], h, t)


# ------------------------------------------------------------- fields

def field_plots(b, plots, crop_cols, seed=1):
    """plots: list of (cx, cy, w, h, angle, kind)."""
    rng = random.Random(seed)
    soil = mat("tilled", c("#7a5a38"), 1.0, col2=c("#5a4028"), scale=12, bump=0.2)
    for (cx, cy, w, h, ang, kind) in plots:
        ca, sa = math.cos(ang), math.sin(ang)
        col = crop_cols[kind % len(crop_cols)]
        cm = mat("crop" + col, c(col), 0.7, col2=c(col), scale=20, bump=0.3)
        # soil bed
        b.box(soil, (cx, cy, 0.005), (w, h, 0.012), yaw=ang)
        nrows = max(2, int(h / 0.075))
        for i in range(nrows):
            ly = -h / 2 + (i + 0.5) * h / nrows
            px, py = cx - ly * sa, cy + ly * ca
            # a raised ridge row of crop
            b.box(cm, (px, py, 0.03), (w * 0.94, 0.045, 0.045), yaw=ang)
            # tufts along the row
            nt = max(2, int(w / 0.07))
            for k in range(nt):
                lx = -w / 2 + (k + 0.5) * w / nt
                b.blob(cm, (px + lx * ca, py + lx * sa, 0.06), (0.03, 0.025, 0.04), seg=6)


@model("imp_farm")
def farm(b):
    plots = [(-0.38, 0.38, 0.5, 0.42, 0.0, 0), (0.28, 0.45, 0.56, 0.38, 0.0, 1), (-0.5, -0.05, 0.4, 0.55, 1.5708, 1),
             (0.05, 0.02, 0.5, 0.5, 0.0, 2), (0.52, -0.1, 0.38, 0.55, 1.5708, 0), (-0.28, -0.52, 0.55, 0.38, 0.0, 2), (0.3, -0.55, 0.5, 0.34, 0.0, 1)]
    soil = mat("tilled", c("#6a4c30"), 1.0, col2=c("#523a24"), scale=12, bump=0.2)
    b.poly(soil, hex_pts(0.97), z=0.0, h=0.008)
    field_plots(b, plots, ["#6fb040", "#d8b848", "#9ac040"])
    return {"top_down": True, "ortho": 2.0, "fit": False, "no_catcher": True, "mask": 0.93, "samples": 32}


@model("imp_plantation")
def plantation(b):
    soil = mat("tilled", c("#7a5a38"), 1.0, col2=c("#5a4028"), scale=12, bump=0.2)
    b.poly(soil, hex_pts(0.86), z=0.0, h=0.01)
    lm = mat("shrub", c("#3f8a2e"), 0.6, col2=c("#2f7022"), scale=14, bump=0.3, sss=0.2)
    lt = mat("shrubt", c("#8cc84a"), 0.6)
    for row in range(-3, 4):
        y = row * 0.22
        half = 0.78 if abs(y) < 0.43 else (1.0 - abs(y)) * 1.732 - 0.08
        if half < 0.1:
            continue
        n = int(half * 2 / 0.19)
        for k in range(n):
            x = -half + (k + 0.5) * 2 * half / n
            b.blob(lm, (x, y, 0.07), (0.1, 0.09, 0.08), seg=8)
            b.blob(lt, (x, y, 0.13), (0.06, 0.055, 0.03), seg=6)
    return {"top_down": True, "ortho": 2.0, "fit": False, "no_catcher": True, "mask": 0.9, "samples": 32}


@model("imp_pasture")
def pasture(b):
    gm = mat("pastureg", c("#8cc050"), 1.0, col2=c("#6aa038"), scale=8)
    b.poly(gm, hex_pts(0.64), z=0.0, h=0.012)
    pts = hex_pts(0.66)
    for i in range(6):
        fence_line(b, pts[i], pts[(i + 1) % 6], h=0.13, rails=2)
    # trough and haystack at the back
    wd = wood_mat()
    b.box(wd, (-0.28, 0.32, 0.04), (0.2, 0.07, 0.08))
    hm = mat("hay", c("#d8b848"), 1.0, col2=c("#b89430"), scale=40, bump=0.6)
    b.blob(hm, (0.3, 0.3, 0.07), (0.12, 0.1, 0.09), seg=10)
    return {"fit": False, "catcher": 1.0, "ystretch": True}


# -------------------------------------------------------------- mining

@model("imp_mine")
def mine(b):
    rm = rock_mat("#8a8074", "#5a5248")
    b.rock(rm, (0.0, 0.18, 0.2), (0.55, 0.36, 0.38), 1, rough=0.2)
    b.rock(rm, (-0.4, 0.0, 0.1), (0.2, 0.17, 0.16), 2)
    wd = wood_mat("#8a5a2c", "#5a3a18")
    dk = mat("darkhole", c("#0a0806"), 1.0)
    # shaft: dark opening framed by timbers
    b.box(dk, (0.0, -0.1, 0.13), (0.26, 0.08, 0.25))
    for x in (-0.15, 0.15):
        b.box(wd, (x, -0.15, 0.15), (0.05, 0.06, 0.3))
    b.box(wd, (0.0, -0.15, 0.31), (0.38, 0.07, 0.05))
    b.box(wd, (0.0, -0.16, 0.38), (0.3, 0.06, 0.04))
    # rails and cart
    ir = mat("irondk", c("#4a4a50"), 0.4, metal=0.8)
    for x in (-0.06, 0.06):
        b.box(ir, (x, -0.38, 0.012), (0.018, 0.5, 0.018))
    for i in range(6):
        b.box(wd, (0.0, -0.2 - i * 0.075, 0.006), (0.22, 0.03, 0.012))
    b.box(wd, (0.0, -0.42, 0.09), (0.19, 0.24, 0.1))
    ore = mat("oreheap", c("#6a6a70"), 0.5, col2=c("#a8a8b0"), scale=30, bump=0.8)
    b.blob(ore, (0.0, -0.42, 0.15), (0.09, 0.11, 0.05), seg=8)
    for x, y in [(-0.07, -0.5), (0.07, -0.5), (-0.07, -0.34), (0.07, -0.34)]:
        b.cyl(ir, (x - 0.1 * 0, y, 0.04), (x, y, 0.04), 0.03, 0.03, seg=8)
    # pickaxe leaning + ore pile
    b.cyl(wd, (0.4, -0.08, 0.0), (0.33, -0.08, 0.3), 0.012, 0.012, seg=5)
    b.box(ir, (0.32, -0.08, 0.31), (0.12, 0.02, 0.03), yaw=0.0)
    for i, (x, y) in enumerate([(0.38, -0.3), (0.45, -0.22), (0.3, -0.28)]):
        b.rock(ore, (x, y, 0.05), (0.07, 0.06, 0.05), 5 + i)
    return {"fit": False}


@model("imp_quarry")
def quarry(b):
    st = mat("qstone", c("#d0c8b4"), 0.9, col2=c("#a8a08c"), scale=6, bump=0.3)
    st2 = mat("qstone2", c("#b8b09a"), 0.9, col2=c("#8a826e"), scale=6, bump=0.3)
    ds = mat("qdark", c("#7a725e"), 1.0)
    # pit: terraced stepped cut, deepest at the front
    b.box(ds, (0.0, 0.0, 0.0), (1.0, 0.8, 0.03))
    for i, (w, d, x, y) in enumerate([(0.9, 0.38, 0.0, 0.3), (0.72, 0.3, -0.05, 0.08), (0.5, 0.26, -0.1, -0.12)]):
        b.box(st if i % 2 == 0 else st2, (x, y, 0.07 + i * 0.07), (w, d, 0.14 + i * 0.14))
    # cut blocks
    for (x, y, yaw, z) in [(0.38, -0.2, 0.2, 0.08), (0.32, -0.1, 0.4, 0.08), (0.36, -0.15, 0.3, 0.2), (-0.4, -0.3, -0.3, 0.07)]:
        b.box(st, (x, y, z), (0.2, 0.15, 0.14), yaw=yaw)
    # crane / derrick
    wd = wood_mat()
    b.cyl(wd, (0.0, -0.3, 0), (0.0, -0.3, 0.55), 0.025, 0.025, seg=6)
    b.cyl(wd, (0.0, -0.3, 0.52), (0.22, -0.28, 0.52), 0.02, 0.02, seg=6)
    b.cyl(wd, (0.0, -0.3, 0.1), (0.14, -0.3, 0.5), 0.014, 0.014, seg=5)
    rp = mat("rope", c("#c8b078"), 1.0)
    b.cyl(rp, (0.2, -0.28, 0.52), (0.2, -0.28, 0.3), 0.005, 0.005, seg=3)
    b.box(st, (0.2, -0.28, 0.26), (0.1, 0.08, 0.08))
    return {"fit": False}


@model("imp_camp")
def camp(b):
    ground_patch(b, "#7a8a48", "#5a6a30", 0.6, seed=2)
    hide = mat("hide", c("#c8a068"), 0.95, col2=c("#a07840"), scale=14)
    tent(b, -0.2, 0.12, 0.2, 0.34, "#c8a068", seg=10)
    tent(b, 0.2, 0.22, 0.16, 0.28, "#b89058", seg=10)
    wd = wood_mat()
    # drying rack with pelts
    for x in (0.0, 0.34):
        b.cyl(wd, (x, -0.3, 0), (x, -0.3, 0.26), 0.014, 0.014, seg=5)
    b.cyl(wd, (-0.02, -0.3, 0.25), (0.36, -0.3, 0.25), 0.012, 0.012, seg=5)
    pm = mat("pelt", c("#8a5a30"), 1.0, col2=c("#5a3a1e"), scale=18, bump=0.6)
    b.box(pm, (0.1, -0.3, 0.16), (0.1, 0.012, 0.16))
    b.box(hide, (0.24, -0.3, 0.17), (0.09, 0.012, 0.14))
    # fire pit
    sm = rock_mat("#6a6460")
    for k in range(7):
        a = k * 0.9
        b.rock(sm, (-0.28 + math.cos(a) * 0.07, -0.18 + math.sin(a) * 0.07, 0.025), (0.03, 0.03, 0.025), k, subdiv=1)
    fm = mat("fire", c("#ff8a1e"), 0.4, emit=6.0, emit_col=c("#ff7010"))
    b.cone(fm, (-0.28, -0.18, 0.02), 0.11, 0.035, seg=6)
    b.cyl(wd, (-0.33, -0.2, 0.025), (-0.23, -0.16, 0.025), 0.012, 0.012, seg=4)
    # antlers / hunted game
    am = mat("antler", c("#d8c9a8"), 0.7)
    b.tube(am, [(0.4, 0.0, 0.0), (0.42, 0.0, 0.1), (0.38, 0.0, 0.16)], 0.01, 0.005, seg=4)
    return {"fit": False}


@model("imp_lumber_mill")
def lumber_mill(b):
    ground_patch(b, "#7a6240", "#5a4630", 0.62, seed=3)
    wd = wood_mat("#9a6a38", "#6a4620")
    log = wood_mat("#7a5028", "#4a2e14")
    roof = mat("shingle", c("#6a4a30"), 0.9, col2=c("#4a3220"), scale=25, bump=0.4)
    house(b, -0.1, 0.18, 0.46, 0.3, 0.2, wd, roof, rh=0.2, door=False)
    # saw blade
    ir = mat("sawm", c("#c8ccd0"), 0.25, metal=1.0)
    b.cyl(ir, (0.18, -0.15, 0.2), (0.18, -0.145, 0.2), 0.12, 0.12, seg=18)
    b.box(wd, (0.18, -0.15, 0.08), (0.3, 0.05, 0.16))
    # logs
    for i in range(3):
        for j in range(3 - i):
            b.cyl(log, (-0.4 + j * 0.095 + i * 0.047, -0.25, 0.045 + i * 0.08), (0.1 + j * 0.095 + i * 0.047 - 0.1, -0.25, 0.045 + i * 0.08), 0.045, 0.045, seg=8)
    ce = mat("logend", c("#c8a068"), 0.8)
    b.cyl(log, (0.3, 0.05, 0.05), (0.3, -0.2, 0.05), 0.05, 0.05, seg=8)
    b.cyl(log, (0.38, 0.0, 0.05), (0.38, -0.25, 0.05), 0.05, 0.05, seg=8)
    return {"fit": False}


@model("imp_fishing_boats")
def fishing_boats(b):
    water_rings(b, 0.7, 2)
    hull = mat("hull", c("#8a5a30"), 0.7, col2=c("#5a3818"), scale=6, wave=True)
    trim = mat("hulltrim", c("#d8c8a0"), 0.8)
    sail = mat("sail2", c("#f0e8d4"), 0.95)
    sail_r = mat("sailr", c("#c0402e"), 0.9)
    for (x, y, yaw, s, sl) in [(-0.12, 0.12, 0.3, 1.2, sail), (0.22, -0.2, 2.4, 0.95, sail_r)]:
        P = lib._P(x, y, yaw, s)
        b.blob(hull, P(0, 0, 0.04), (0.22 * s, 0.08 * s, 0.05 * s), yaw, seg=12)
        b.blob(trim, P(0, 0, 0.075), (0.2 * s, 0.07 * s, 0.01 * s), yaw, seg=12)
        b.cyl(wood_mat(), P(0.02, 0, 0.06), P(0.02, 0, 0.5), 0.012 * s, 0.01 * s, seg=5)
        # triangular sails
        lib.tri(b, sl, P(0.03, 0.0, 0.14), P(0.2, 0.0, 0.12), P(0.03, 0.0, 0.5))
        lib.tri(b, sl, P(0.0, 0.0, 0.14), P(-0.16, 0.0, 0.12), P(0.0, 0.0, 0.46))
        # catch
        fm = mat("fishc", c("#9ab8d0"), 0.3, metal=0.4)
        b.blob(fm, P(-0.08, 0.02, 0.1), (0.04 * s, 0.015 * s, 0.015 * s), yaw, seg=6)
    # floating net buoys
    bm_ = mat("buoy", c("#e04a2a"), 0.5)
    for (x, y) in [(0.35, 0.2), (0.28, 0.3), (0.4, 0.1), (-0.4, -0.2)]:
        b.blob(bm_, (x, y, 0.03), (0.03, 0.03, 0.03), seg=6)
    return {"fit": False}


@model("imp_oil_well")
def oil_well(b):
    ground_patch(b, "#4a3e32", "#2a2420", 0.6, seed=4)
    om = mat("oilp", c("#101014"), 0.05, coat=1.0)
    b.blob(om, (0.15, -0.3, 0.0), (0.22, 0.12, 0.02), seg=14)
    ir = mat("derrick", c("#8a8e96"), 0.4, metal=0.9)
    rd = mat("derrickr", c("#c0402e"), 0.5, metal=0.4)
    # lattice derrick tower
    top = (0.0, 0.1, 0.8)
    for (x, y) in [(-0.17, -0.07), (0.17, -0.07), (-0.17, 0.27), (0.17, 0.27)]:
        b.cyl(ir, (x, y, 0), (top[0] + (x - 0) * 0.15, top[1] + (y - 0.1) * 0.15, top[2]), 0.015, 0.012, seg=5)
    for z in (0.15, 0.32, 0.5, 0.66):
        s = 0.17 * (1 - z / 0.95)
        for (a, bb) in [((-s, -s), (s, -s)), ((s, -s), (s, s)), ((s, s), (-s, s)), ((-s, s), (-s, -s))]:
            b.cyl(ir, (a[0], a[1] + 0.1, z), (bb[0], bb[1] + 0.1, z), 0.008, 0.008, seg=4)
    b.cyl(rd, (0.0, 0.1, 0.0), (0.0, 0.1, 0.85), 0.012, 0.012, seg=5)
    b.box(rd, (0.0, 0.1, 0.84), (0.1, 0.1, 0.05))
    # engine shed + tanks
    b.box(rd, (-0.34, -0.12, 0.07), (0.2, 0.16, 0.14))
    b.cyl(ir, (0.33, 0.28, 0.0), (0.33, 0.28, 0.2), 0.1, 0.1, seg=14)
    b.cyl(rd, (0.33, 0.28, 0.2), (0.33, 0.28, 0.23), 0.1, 0.1, seg=14)
    b.cyl(ir, (0.12, -0.12, 0.03), (0.33, 0.2, 0.1), 0.012, 0.012, seg=5)
    return {"fit": False}


@model("imp_offshore_oil_rig")
def offshore_rig(b):
    water_rings(b, 0.78, 2)
    ir = mat("rigm", c("#8a8e96"), 0.4, metal=0.9)
    rd = mat("rigr", c("#d8502e"), 0.5, metal=0.3)
    yl = mat("rigy", c("#e8c030"), 0.5)
    for (x, y) in [(-0.3, -0.2), (0.3, -0.2), (-0.3, 0.25), (0.3, 0.25)]:
        b.cyl(ir, (x, y, -0.05), (x, y, 0.3), 0.04, 0.04, seg=8)
    b.box(yl, (0.0, 0.02, 0.34), (0.85, 0.65, 0.06))
    b.box(rd, (-0.2, 0.0, 0.46), (0.28, 0.24, 0.18))
    b.box(ir, (0.2, -0.1, 0.44), (0.24, 0.2, 0.14))
    top = (0.08, 0.12, 1.0)
    for (x, y) in [(-0.05, 0.05), (0.2, 0.05), (-0.05, 0.3), (0.2, 0.3)]:
        b.cyl(ir, (x, y, 0.37), (top[0] + (x - 0.08) * 0.2, top[1] + (y - 0.17) * 0.2, 0.95), 0.012, 0.01, seg=5)
    for z in (0.5, 0.65, 0.8):
        s = 0.125 * (1 - (z - 0.37) / 0.7)
        b.box(ir, (0.08 + 0.0, 0.17, z), (s * 2, s * 2, 0.01))
    b.cyl(rd, (0.08, 0.17, 0.37), (0.08, 0.17, 1.0), 0.01, 0.01, seg=5)
    # flare stack and helipad
    b.cyl(ir, (-0.35, 0.3, 0.37), (-0.35, 0.3, 0.8), 0.012, 0.012, seg=5)
    fm = mat("flare", c("#ff8a1e"), 0.4, emit=6.0, emit_col=c("#ff7010"))
    b.cone(fm, (-0.35, 0.3, 0.8), 0.1, 0.03, seg=6)
    b.cyl(yl, (0.28, 0.32, 0.37), (0.28, 0.32, 0.4), 0.14, 0.14, seg=16)
    hm = mat("hmark", c("#f4f4f4"), 0.5)
    b.box(hm, (0.28, 0.32, 0.405), (0.12, 0.03, 0.005))
    b.box(hm, (0.28, 0.32, 0.405), (0.03, 0.12, 0.005))
    return {"fit": False, "target": 1.2, "elev": 52.0, "ortho": 1.9}


@model("imp_fort")
def fort(b):
    ground_patch(b, "#8a7a5a", "#6a5c40", 0.7, seed=5)
    st = STONE()
    b.wall_run(st, [(-0.42, -0.34), (0.42, -0.34), (0.42, 0.34), (-0.42, 0.34), (-0.42, -0.34)], 0.2, 0.09)
    b.box(mat("ftop", c("#9a9282"), 0.9), (0, 0, 0.012), (0.84, 0.68, 0.024))
    for (x, y) in [(-0.42, -0.34), (0.42, -0.34), (0.42, 0.34), (-0.42, 0.34)]:
        tower(b, x, y, 0.2, 0.34, st)
    # gate
    b.box(mat("gate", c("#3a2616"), 0.8), (0, -0.385, 0.08), (0.16, 0.02, 0.16))
    b.box(wood_mat(), (0.0, 0.0, 0.1), (0.3, 0.26, 0.2))
    gable(b, ROOF_RED(), 0.0, 0.0, 0.2, 0.3, 0.26, 0.12, 0)
    flag(b, (0.42, 0.34, 0.34), 0.28, "#2a4aa8")
    return {"fit": False}


@model("imp_airstrip")
def airstrip(b):
    asph = mat("asph", c("#3a3c42"), 0.9, col2=c("#4a4c52"), scale=20, bump=0.1)
    ln = mat("rwline", c("#f0f0e8"), 0.7)
    gr = mat("apron", c("#6a6c70"), 0.9)
    ground_patch(b, "#6a9a44", "#527a34", 1.1, seed=22)
    b.box(asph, (0.0, 0.0, 0.015), (1.3, 0.24, 0.02), yaw=0.52)
    for i in range(-5, 6):
        b.box(ln, (math.cos(0.52) * i * 0.11, math.sin(0.52) * i * 0.11, 0.028), (0.06, 0.02, 0.004), yaw=0.52)
    b.box(gr, (-0.18, 0.28, 0.015), (0.4, 0.26, 0.02))
    # hangar
    hm = mat("hangar", c("#8a9aa8"), 0.5, metal=0.6)
    b.box(hm, (-0.2, 0.32, 0.09), (0.28, 0.22, 0.18))
    b.cyl(hm, (-0.34, 0.32, 0.18), (-0.06, 0.32, 0.18), 0.11, 0.11, seg=12)
    # plane
    pm = mat("planew", c("#e8e8e8"), 0.35, metal=0.5)
    pr = mat("planer", c("#2a4aa8"), 0.4)
    P = lib._P(0.25, -0.2, 0.52, 1.0)
    b.blob(pm, P(0, 0, 0.07), (0.17, 0.035, 0.035), 0.52, seg=10)
    b.box(pr, P(0.0, 0.0, 0.075), (0.06, 0.34, 0.012), yaw=0.52)
    b.box(pr, P(-0.14, 0.0, 0.1), (0.04, 0.12, 0.01), yaw=0.52)
    b.box(pr, P(-0.14, 0.0, 0.09), (0.04, 0.01, 0.07), yaw=0.52)
    # windsock
    b.cyl(wood_mat(), (0.4, 0.3, 0), (0.4, 0.3, 0.25), 0.01, 0.01, seg=4)
    b.cyl(mat("sock", c("#e05a2a"), 0.8), (0.4, 0.3, 0.23), (0.52, 0.32, 0.2), 0.03, 0.015, seg=6)
    return {"fit": False, "ystretch": True}


@model("imp_missile_silo")
def missile_silo(b):
    cn = mat("concrete", c("#9a9a96"), 0.9, col2=c("#7a7a76"), scale=14, bump=0.2)
    dk = mat("silodark", c("#101010"), 1.0)
    b.cyl(cn, (0, 0, 0), (0, 0, 0.05), 0.55, 0.55, seg=24)
    b.cyl(dk, (0, 0, 0.05), (0, 0, 0.052), 0.26, 0.26, seg=20)
    # sliding door halves
    b.box(cn, (-0.2, -0.02, 0.08), (0.26, 0.5, 0.06), yaw=0.0)
    b.box(cn, (0.32, 0.0, 0.08), (0.26, 0.5, 0.06), yaw=0.0)
    ms = mat("missile", c("#e8e8e4"), 0.35, metal=0.3)
    rd = mat("missiler", c("#c0302a"), 0.4)
    b.cyl(ms, (0.05, 0.02, 0.03), (0.05, 0.02, 0.5), 0.1, 0.1, seg=12)
    b.cyl(rd, (0.05, 0.02, 0.5), (0.05, 0.02, 0.72), 0.1, 0.0, seg=12)
    b.cyl(rd, (0.05, 0.02, 0.12), (0.05, 0.02, 0.2), 0.105, 0.105, seg=12)
    ya = mat("hazard", c("#e8c030"), 0.6, stripes=24, col2=c("#202020"), scale=24)
    b.cyl(ya, (0, 0, 0.05), (0, 0, 0.056), 0.52, 0.52, seg=24)
    b.cyl(cn, (0, 0, 0.05), (0, 0, 0.056), 0.45, 0.45, seg=24)
    b.cyl(dk, (0.05, 0.02, 0.05), (0.05, 0.02, 0.055), 0.2, 0.2, seg=16)
    return {"fit": False}


@model("imp_beach_resort")
def beach_resort(b):
    sand = mat("rsand", c("#e8d49a"), 1.0, col2=c("#d0b878"), scale=14)
    b.blob(sand, (0, 0, -0.02), (0.75, 0.6, 0.05))
    wl = mat("hotelw", c("#f4f0e8"), 0.7)
    b.box(wl, (-0.1, 0.25, 0.2), (0.6, 0.22, 0.4))
    rm = mat("hotelr", c("#2a9ab8"), 0.5)
    b.box(rm, (-0.1, 0.25, 0.41), (0.64, 0.26, 0.04))
    wm = mat("hotelwin", c("#2a6a90"), 0.2, coat=1.0)
    for i in range(5):
        b.box(wm, (-0.34 + i * 0.12, 0.138, 0.28), (0.07, 0.01, 0.09))
        b.box(wm, (-0.34 + i * 0.12, 0.138, 0.13), (0.07, 0.01, 0.09))
    # umbrellas + loungers
    for (x, y, col) in [(-0.3, -0.12, "#e04a4a"), (0.05, -0.2, "#f0c030"), (0.35, -0.05, "#3a8ad8")]:
        b.cyl(wood_mat(), (x, y, 0), (x, y, 0.3), 0.01, 0.01, seg=5)
        b.cyl(mat("umb" + col, c(col), 0.8, col2=c("#f4f4f4"), scale=1, stripes=6), (x, y, 0.27), (x, y, 0.38), 0.16, 0.0, seg=12)
        b.box(mat("lounge", c("#f4f4f0"), 0.9), (x + 0.04, y - 0.09, 0.03), (0.1, 0.2, 0.04), yaw=0.2)
    pm = mat("palm", c("#4a8a30"), 0.6)
    b.tube(wood_mat(), [(0.42, 0.28, 0), (0.44, 0.28, 0.25), (0.4, 0.28, 0.42)], 0.025, 0.015, seg=6)
    for k in range(6):
        a = k * 1.05
        b.tube(pm, [(0.4, 0.28, 0.42), (0.4 + math.cos(a) * 0.13, 0.28 + math.sin(a) * 0.13, 0.47), (0.4 + math.cos(a) * 0.24, 0.28 + math.sin(a) * 0.24, 0.38)], 0.035, 0.004, seg=4)
    return {"fit": False}


# ------------------------------------------------- unique improvements

@model("imp_goody_hut")
def goody_hut(b):
    ground_patch(b, "#8a6a44", "#6a4e30", 0.62, seed=7)
    hut_round(b, -0.22, 0.1, 0.17, 0.14)
    hut_round(b, 0.2, 0.18, 0.15, 0.12)
    hut_round(b, 0.05, -0.18, 0.16, 0.13)
    flag(b, (0.34, -0.1, 0.0), 0.38, "#c8402e", 0.12, 0.08)
    fm = mat("fire", c("#ff8a1e"), 0.4, emit=6.0, emit_col=c("#ff7010"))
    b.cone(fm, (-0.2, -0.28, 0.02), 0.08, 0.03, seg=6)
    return {"fit": False}


@model("imp_great_wall")
def great_wall(b):
    st = mat("gwall", c("#c8b894"), 0.9, col2=c("#a89874"), scale=10, bump=0.4)
    b.wall_run(st, [(-0.75, -0.25), (-0.25, -0.1), (0.25, 0.1), (0.75, 0.25)], 0.2, 0.2, st, 0.03)
    for i in range(7):
        t = i / 6
        x = -0.7 + 1.4 * t
        y = -0.23 + 0.46 * t
        b.box(st, (x, y + 0.085, 0.25), (0.06, 0.04, 0.07))
        b.box(st, (x, y - 0.085, 0.25), (0.06, 0.04, 0.07))
    tower(b, 0.0, 0.0, 0.3, 0.42, st)
    b.box(mat("gwroof", c("#6a8a58"), 0.6), (0.0, 0.0, 0.45), (0.34, 0.34, 0.04))
    b.box(mat("gwdoor", c("#3a2616"), 0.8), (0.0, -0.152, 0.1), (0.1, 0.01, 0.16))
    return {"fit": False, "ortho": 1.9}


@model("imp_kurgan")
def kurgan(b):
    gm = grass_mat("#7aa048", "#5a8034")
    b.blob(gm, (0, 0.05, 0.0), (0.55, 0.45, 0.28), seg=20)
    st = rock_mat("#8a847a")
    for k in range(6):
        a = k * 1.05
        x, y = math.cos(a) * 0.5, math.sin(a) * 0.4 + 0.05
        b.box(st, (x, y, 0.1), (0.07, 0.05, 0.2), yaw=a)
    wd = wood_mat()
    b.cyl(wd, (0.0, 0.05, 0.26), (0.0, 0.05, 0.62), 0.015, 0.012, seg=5)
    b.box(wd, (0.0, 0.05, 0.52), (0.14, 0.015, 0.015))
    fm = mat("kflag", c("#c8402e"), 0.7)
    b.box(fm, (0.07, 0.05, 0.57), (0.12, 0.006, 0.08))
    return {"fit": False}


@model("imp_mission")
def mission(b):
    ground_patch(b, "#8a7a5a", "#6a5c40", 0.6, seed=8)
    pl = PLASTER()
    house(b, -0.05, 0.1, 0.4, 0.28, 0.2, pl, ROOF_RED(), rh=0.14, door=True)
    tower(b, 0.2, 0.0, 0.14, 0.4, pl, ROOF_RED())
    b.box(wood_mat(), (0.2, -0.075, 0.3), (0.05, 0.01, 0.08))
    cm = mat("cross", c("#d8c070"), 0.4, metal=0.8)
    b.box(cm, (0.2, 0.0, 0.62), (0.015, 0.015, 0.1))
    b.box(cm, (0.2, 0.0, 0.64), (0.06, 0.015, 0.015))
    return {"fit": False}


@model("imp_roman_fort")
def roman_fort(b):
    ground_patch(b, "#8a7a5a", "#6a5c40", 0.72, seed=9)
    pts = [(-0.4, -0.33), (0.4, -0.33), (0.4, 0.33), (-0.4, 0.33)]
    for i in range(4):
        palisade(b, pts[i], pts[(i + 1) % 4], 0.2)
    wd = wood_mat("#8a5a2c")
    for (x, y) in pts:
        b.box(wd, (x, y, 0.15), (0.12, 0.12, 0.3))
        b.box(wd, (x, y, 0.33), (0.17, 0.17, 0.03))
        b.cone(mat("rtent", c("#c0392a"), 0.9), (x, y, 0.34), 0.14, 0.1, seg=4)
    tm = mat("rtent2", c("#d8c8a0"), 0.95)
    for (x, y) in [(-0.12, 0.05), (0.14, 0.1), (0.0, -0.15)]:
        house(b, x, y, 0.2, 0.14, 0.07, tm, mat("rtent3", c("#c0392a"), 0.9), rh=0.08, door=False)
    flag(b, (0.0, 0.2, 0.0), 0.4, "#c0392a", 0.14, 0.1)
    return {"fit": False}


@model("imp_sphinx")
def sphinx(b):
    sn = mat("sphinx", c("#d8b878"), 0.95, col2=c("#b89858"), scale=8, bump=0.4)
    sand = mat("ssand", c("#e0c88a"), 1.0, col2=c("#c8b070"), scale=10)
    b.blob(sand, (0, 0, -0.02), (0.75, 0.6, 0.05))
    P = lib._P(0, 0, 0.0, 1.0)
    b.blob(sn, (0, 0.0, 0.13), (0.38, 0.14, 0.12), 0.0)
    b.box(sn, (0.0, 0.0, 0.05), (0.7, 0.26, 0.1))
    for y in (-0.1, 0.1):
        b.box(sn, (0.42, y, 0.05), (0.26, 0.09, 0.09))
    b.tube(sn, [(0.3, 0, 0.14), (0.36, 0, 0.3)], 0.08, 0.07, seg=8)
    b.blob(sn, (0.37, 0, 0.38), (0.09, 0.085, 0.11), seg=10)
    hm = mat("nemes", c("#3a5aa8"), 0.5, stripes=10, col2=c("#e8c048"), scale=1)
    b.blob(hm, (0.35, 0, 0.42), (0.12, 0.13, 0.1), seg=10)
    b.box(hm, (0.3, 0.0, 0.3), (0.05, 0.2, 0.12))
    b.box(sn, (-0.4, 0.0, 0.1), (0.1, 0.1, 0.1))
    return {"fit": False}


@model("imp_stepwell")
def stepwell(b):
    st = mat("swell", c("#d8b890"), 0.9, col2=c("#b89870"), scale=8, bump=0.3)
    dk = mat("swdark", c("#1a1410"), 1.0)
    wt = mat("swwater", c("#3a7a9a"), 0.1, coat=1.0)
    b.box(st, (0, 0, 0.04), (1.0, 0.8, 0.08))
    n = 5
    for i in range(n):
        s = 0.64 - i * 0.11
        z = 0.08 - 0.0
        b.box(st if i % 2 == 0 else mat("swell2", c("#c8a880"), 0.9), (0, 0, 0.09 + 0.001 * i), (s + 0.01 * 0, s * 0.8, 0.012))
    b.box(wt, (0, 0, 0.09), (0.13, 0.1, 0.015))
    # raised rim walls with pillars
    for (x, y, w, d) in [(0, 0.32, 0.9, 0.08), (0.45, 0.0, 0.08, 0.6), (-0.45, 0.0, 0.08, 0.6), (0, -0.34, 0.9, 0.06)]:
        b.box(st, (x, y, 0.15), (w, d, 0.15))
    for x in (-0.4, 0.4):
        b.box(mat("swpil", c("#e8c8a0"), 0.85), (x, 0.3, 0.3), (0.07, 0.07, 0.3))
        b.box(mat("swpil", c("#e8c8a0"), 0.85), (x, -0.32, 0.25), (0.07, 0.07, 0.2))
    return {"fit": False}


@model("imp_ziggurat")
def ziggurat(b):
    st = mat("zig", c("#c8a068"), 0.95, col2=c("#a88048"), scale=8, bump=0.4)
    st2 = mat("zig2", c("#d8b078"), 0.95, col2=c("#b89058"), scale=8, bump=0.4)
    stepped(b, st, 0, 0.05, 0.0, 0.9, tiers=4, th=0.12, shrink=0.72, m2=st2)
    b.box(mat("zigtemple", c("#3a6ab8"), 0.5, coat=0.4), (0, 0.05, 0.54), (0.2, 0.2, 0.12))
    b.box(mat("zigramp", c("#b89058"), 0.9), (0, -0.4, 0.12), (0.18, 0.28, 0.24), rot=lib.Matrix.Rotation(0.0, 4, "X"))
    for i in range(4):
        b.box(st2, (0, -0.33 - 0.08 * (3 - i) + 0.0, 0.06 + i * 0.1), (0.2, 0.08, 0.12))
    return {"fit": False}


@model("imp_outback_station")
def outback_station(b):
    ground_patch(b, "#b87a44", "#9a6234", 0.7, seed=10)
    house(b, -0.15, 0.18, 0.32, 0.22, 0.12, mat("owall", c("#d8b888"), 0.9), mat("otin", c("#8a98a0"), 0.4, metal=0.8), rh=0.1)
    # windmill
    ir = mat("wmill", c("#a8aeb4"), 0.4, metal=0.8)
    for (x, y) in [(-0.04, -0.04), (0.04, -0.04), (-0.04, 0.04), (0.04, 0.04)]:
        b.cyl(ir, (0.3 + x * 2, -0.05 + y * 2, 0), (0.3 + x * 0.5, -0.05 + y * 0.5, 0.5), 0.01, 0.008, seg=4)
    for k in range(8):
        a = k * math.pi / 4
        b.box(ir, (0.3 + math.cos(a) * 0.08, -0.07, 0.52 + math.sin(a) * 0.08), (0.02, 0.01, 0.1), rot=lib.Matrix.Rotation(a + 1.57, 4, "Y"))
    b.blob(ir, (0.3, -0.05, 0.52), (0.03, 0.03, 0.03), seg=6)
    pts = hex_pts(0.72)
    for i in range(0, 6):
        fence_line(b, pts[i], pts[(i + 1) % 6], h=0.1, rails=2) if i in (2, 3, 4) else None
    return {"fit": False}


@model("imp_pairidaeza")
def pairidaeza(b):
    gm = mat("pgrass", c("#7ab04a"), 1.0, col2=c("#5a9438"), scale=8)
    b.poly(gm, hex_pts(0.7), z=0.0, h=0.012)
    wl = mat("pwall", c("#d8c8a0"), 0.9)
    pts = hex_pts(0.72)
    b.wall_run(wl, pts + [pts[0]], 0.1, 0.05)
    b.disc(mat("ppond", c("#3a8aa8"), 0.05, coat=1.0), (0, 0, 0.012), 0.2, 0.01, seg=20)
    tree(b, (-0.35, 0.2, 0), 0.5, "#4a8a30", s=1.1)
    tree(b, (0.35, 0.25, 0), 0.55, "#5a9a38", s=1.2)
    tree(b, (0.0, 0.42, 0), 0.5, "#3a7a2a", "#6a4423", "pine", 1.0)
    tree(b, (-0.3, -0.3, 0), 0.45, "#6a9a30", s=1.0)
    fl = [("#e04a7a", -0.12, -0.3), ("#f0c030", 0.2, -0.3), ("#a04ac8", 0.38, -0.12)]
    for (col, x, y) in fl:
        for k in range(5):
            b.blob(mat("fl" + col, c(col), 0.6), (x + math.cos(k * 1.3) * 0.04, y + math.sin(k * 1.3) * 0.04, 0.03), (0.025, 0.025, 0.02), seg=6)
    return {"fit": False, "ystretch": True}


@model("imp_pyramid")
def pyramid_nubian(b):
    sand = mat("psand", c("#e0c88a"), 1.0, col2=c("#c8b070"), scale=10)
    b.blob(sand, (0, 0, -0.02), (0.7, 0.55, 0.05))
    st = mat("npyr", c("#c8a468"), 0.95, col2=c("#a88448"), scale=8, bump=0.4)
    for (x, y, w, h) in [(-0.2, 0.05, 0.4, 0.5), (0.22, -0.1, 0.32, 0.4), (0.15, 0.3, 0.22, 0.28)]:
        b.cyl(st, (x, y, 0), (x, y, h), w * 0.7, 0.0, seg=4, smooth=False)
    return {"fit": False}


@model("imp_kampung")
def kampung(b):
    water_rings(b, 0.7, 1)
    wd = wood_mat("#8a6a3a")
    th = THATCH()
    pl = mat("kfloor", c("#a88858"), 0.9)
    for (x, y) in [(-0.2, 0.1), (0.22, 0.0), (0.0, -0.26)]:
        for dx, dy in [(-0.09, -0.07), (0.09, -0.07), (-0.09, 0.07), (0.09, 0.07)]:
            b.cyl(wd, (x + dx, y + dy, -0.04), (x + dx, y + dy, 0.18), 0.012, 0.012, seg=5)
        b.box(pl, (x, y, 0.19), (0.26, 0.2, 0.02))
        house(b, x, y, 0.2, 0.15, 0.1, mat("kwall", c("#c8a470"), 1.0), th, z=0.2, rh=0.12)
    b.box(pl, (0.0, 0.0, 0.18), (0.5, 0.05, 0.015), yaw=0.8)
    return {"fit": False}


@model("imp_alcazar")
def alcazar(b):
    ground_patch(b, "#a88858", "#7a6040", 0.7, seed=11)
    st = mat("alc", c("#e0c898"), 0.9, col2=c("#c0a878"), scale=10, bump=0.3)
    b.box(st, (0, 0.0, 0.14), (0.7, 0.5, 0.28))
    battlements(b, st, 0, 0, 0.7, 0.5, 0.28, 7, 0.05, 0.05)
    for (x, y) in [(-0.35, -0.25), (0.35, -0.25), (0.35, 0.25), (-0.35, 0.25)]:
        tower(b, x, y, 0.18, 0.42, st, mat("alcr", c("#2a6a8a"), 0.5, coat=0.3), round_=True)
    tower(b, 0.0, 0.0, 0.26, 0.5, st, mat("alcr", c("#2a6a8a"), 0.5, coat=0.3), round_=True)
    b.box(mat("alcg", c("#3a2616"), 0.8), (0.0, -0.255, 0.08), (0.14, 0.01, 0.16))
    return {"fit": False}


@model("imp_monastery")
def monastery(b):
    ground_patch(b, "#7a8a4a", "#5a6a30", 0.68, seed=12)
    pl = PLASTER()
    rf = ROOF_RED()
    house(b, -0.18, 0.15, 0.5, 0.2, 0.16, pl, rf, rh=0.12)
    house(b, 0.25, -0.05, 0.2, 0.4, 0.16, pl, rf, yaw=1.5708, rh=0.12)
    tower(b, -0.35, -0.12, 0.14, 0.42, pl, rf)
    b.box(mat("mcross", c("#d8c070"), 0.4, metal=0.8), (-0.35, -0.12, 0.65), (0.014, 0.014, 0.08))
    b.box(mat("mcross", c("#d8c070"), 0.4, metal=0.8), (-0.35, -0.12, 0.66), (0.05, 0.014, 0.014))
    # garden
    g = grass_mat()
    b.poly(mat("mgarden", c("#5a9a38"), 1.0), [(-0.1, -0.35), (0.15, -0.35), (0.15, -0.1), (-0.1, -0.1)], z=0.0, h=0.015)
    tree(b, (0.0, -0.22, 0), 0.4, "#3a7a2a", s=0.8)
    return {"fit": False}


@model("imp_rock_hewn_church")
def rock_hewn(b):
    rm = mat("rhrock", c("#a87858"), 0.95, col2=c("#8a5a3e"), scale=5, bump=0.6)
    dk = mat("rhdark", c("#1a0e08"), 1.0)
    b.rock(rm, (0, 0.05, 0.0), (0.62, 0.5, 0.2), 1, rough=0.1)
    b.box(rm, (0, 0.0, 0.1), (0.85, 0.68, 0.2))
    # cross-shaped sunken court
    b.box(dk, (0, 0, 0.202), (0.12, 0.5, 0.01))
    b.box(dk, (0, 0.05, 0.202), (0.46, 0.12, 0.01))
    b.box(mat("rhin", c("#c89870"), 0.9), (0, 0, 0.215), (0.08, 0.46, 0.01))
    b.box(mat("rhin", c("#c89870"), 0.9), (0, 0.05, 0.215), (0.42, 0.08, 0.01))
    b.box(dk, (0.0, -0.34, 0.07), (0.16, 0.02, 0.14))
    b.box(mat("rhcross", c("#e8d8b0"), 0.6), (0.0, 0.28, 0.24), (0.03, 0.03, 0.14))
    b.box(mat("rhcross", c("#e8d8b0"), 0.6), (0.0, 0.28, 0.27), (0.1, 0.03, 0.03))
    return {"fit": False}


@model("imp_mahavihara")
def mahavihara(b):
    ground_patch(b, "#a89868", "#8a7a4c", 0.7, seed=13)
    wm = mat("stupa", c("#f4f0e4"), 0.7)
    gd = mat("stupag", c("#d8a838"), 0.3, metal=1.0)
    b.cyl(wm, (-0.1, 0.05, 0), (-0.1, 0.05, 0.06), 0.3, 0.28, seg=16)
    b.blob(wm, (-0.1, 0.05, 0.06), (0.25, 0.25, 0.2), seg=18)
    b.box(wm, (-0.1, 0.05, 0.3), (0.1, 0.1, 0.06))
    b.cyl(gd, (-0.1, 0.05, 0.33), (-0.1, 0.05, 0.62), 0.04, 0.0, seg=8)
    for i in range(3):
        b.disc(gd, (-0.1, 0.05, 0.36 + i * 0.07), 0.05 - i * 0.012, 0.012, seg=10)
    house(b, 0.3, -0.12, 0.26, 0.4, 0.14, mat("mvw", c("#e8a838"), 0.8), mat("mvr", c("#b8402a"), 0.7), yaw=1.5708, rh=0.1)
    return {"fit": False}


@model("imp_trading_dome")
def trading_dome(b):
    ground_patch(b, "#b89a68", "#98804c", 0.7, seed=14)
    st = mat("tdw", c("#e8dcc0"), 0.8)
    gl = mat("tddome", c("#2a8aa8"), 0.2, coat=0.8)
    b.cyl(st, (0, 0.05, 0), (0, 0.05, 0.14), 0.36, 0.34, seg=18)
    b.blob(gl, (0, 0.05, 0.14), (0.34, 0.34, 0.3), seg=20)
    gd = mat("tdgold", c("#d8a838"), 0.3, metal=1.0)
    b.cyl(gd, (0, 0.05, 0.42), (0, 0.05, 0.56), 0.02, 0.0, seg=6)
    for k in range(8):
        a = k * math.pi / 4
        b.box(st, (math.cos(a) * 0.36, 0.05 + math.sin(a) * 0.36, 0.08), (0.07, 0.07, 0.16), yaw=a)
    for (x, y) in [(-0.45, -0.25), (0.45, -0.25)]:
        b.box(mat("awn", c("#c8402e"), 0.9, stripes=6, col2=c("#f4f0e4"), scale=1), (x, y, 0.16), (0.24, 0.18, 0.04))
        b.cyl(wood_mat(), (x - 0.1, y - 0.07, 0), (x - 0.1, y - 0.07, 0.15), 0.01, 0.01, seg=4)
        b.cyl(wood_mat(), (x + 0.1, y - 0.07, 0), (x + 0.1, y - 0.07, 0.15), 0.01, 0.01, seg=4)
    return {"fit": False}


@model("imp_batey")
def batey(b):
    ground_patch(b, "#8a6a44", "#6a4e30", 0.66, seed=15)
    th = THATCH()
    hut_round(b, -0.2, 0.1, 0.15, 0.13)
    hut_round(b, 0.18, 0.15, 0.14, 0.12)
    b.poly(mat("bplaza", c("#a88858"), 1.0), [(-0.18, -0.35), (0.3, -0.35), (0.3, -0.1), (-0.18, -0.1)], z=0.0, h=0.012)
    for i in range(3):
        b.cyl(mat("bcane", c("#7ab040"), 0.5), (-0.1 + 0.04 * i, -0.25 - 0.02 * i, 0.03), (0.2 + 0.04 * i, -0.2 - 0.02 * i, 0.03), 0.025, 0.025, seg=6)
    b.cyl(wood_mat(), (0.38, -0.05, 0), (0.38, -0.05, 0.3), 0.025, 0.025, seg=6)
    b.cyl(wood_mat(), (0.38, -0.05, 0.28), (0.1, -0.2, 0.28), 0.015, 0.015, seg=5)
    return {"fit": False}


@model("imp_hacienda")
def hacienda(b):
    ground_patch(b, "#b88a54", "#986c3c", 0.7, seed=16)
    wl = mat("hacw", c("#f0e0c0"), 0.9)
    rf = ROOF_RED()
    house(b, 0.0, 0.2, 0.62, 0.22, 0.18, wl, rf, rh=0.1)
    house(b, -0.3, -0.05, 0.22, 0.38, 0.18, wl, rf, yaw=1.5708, rh=0.1, door=False)
    house(b, 0.3, -0.05, 0.22, 0.38, 0.18, wl, rf, yaw=1.5708, rh=0.1, door=False)
    b.disc(mat("hacfn", c("#5aa8c8"), 0.1, coat=1.0), (0.0, -0.1, 0.0), 0.08, 0.04, seg=14)
    for x in (-0.18, 0.18):
        b.cyl(wl, (x, -0.14, 0), (x, -0.14, 0.2), 0.03, 0.03, seg=8)
        b.blob(mat("hacpot", c("#3a8a30"), 0.7), (x, -0.14, 0.24), (0.06, 0.06, 0.05), seg=8)
    return {"fit": False}


@model("imp_feitoria")
def feitoria(b):
    water = mat("fpier", c("#8a6a3a"), 0.9)
    ground_patch(b, "#b89a68", "#98804c", 0.65, seed=17)
    st = mat("fw", c("#f0e8d4"), 0.9)
    house(b, -0.1, 0.12, 0.42, 0.28, 0.2, st, mat("fr", c("#2a6a8a"), 0.6), rh=0.14)
    tower(b, 0.28, 0.2, 0.16, 0.42, st, mat("fr", c("#2a6a8a"), 0.6))
    flag(b, (0.28, 0.2, 0.62), 0.22, "#2a8a4a", 0.12, 0.08)
    for i in range(4):
        b.box(water, (-0.3 + i * 0.06, -0.35, 0.02), (0.05, 0.5, 0.02))
    crates = [(0.2, -0.2), (0.3, -0.12), (0.24, -0.3)]
    for (x, y) in crates:
        crate(b, (x, y, 0), 1.0)
    barrel(b, (-0.05, -0.28, 0), 1.0)
    return {"fit": False}


@model("imp_chateau")
def chateau(b):
    ground_patch(b, "#7a9a4a", "#5a7a34", 0.72, seed=18)
    st = mat("chw", c("#f0e8d4"), 0.85, col2=c("#d8ccb0"), scale=12, bump=0.2)
    rf = mat("chr", c("#3a5a8a"), 0.45, coat=0.3)
    house(b, 0.0, 0.1, 0.56, 0.24, 0.22, st, rf, rh=0.2)
    house(b, -0.3, -0.12, 0.2, 0.32, 0.22, st, rf, yaw=1.5708, rh=0.18, door=False)
    house(b, 0.3, -0.12, 0.2, 0.32, 0.22, st, rf, yaw=1.5708, rh=0.18, door=False)
    for (x, y) in [(-0.3, 0.12), (0.3, 0.12), (0.0, -0.3)]:
        tower(b, x, y, 0.15, 0.3, st, rf, round_=True)
    gm = mat("chgarden", c("#4a8a38"), 1.0)
    b.poly(gm, [(-0.15, -0.38), (0.15, -0.38), (0.15, -0.05), (-0.15, -0.05)], z=0.0, h=0.012)
    return {"fit": False}


@model("imp_colossal_head")
def colossal_head(b):
    st = mat("olmec", c("#7a7a74"), 0.9, col2=c("#5a5a54"), scale=6, bump=0.6)
    sand = grass_mat("#7aa048", "#5a8034")
    b.blob(sand, (0, 0, 0.0), (0.5, 0.42, 0.05))
    b.cyl(st, (0, 0.0, 0.0), (0, 0.0, 0.12), 0.22, 0.22, seg=12)
    b.blob(st, (0, 0.0, 0.34), (0.26, 0.24, 0.26), seg=14)
    b.blob(st, (0, -0.16, 0.34), (0.2, 0.1, 0.12), seg=10)
    dk = mat("olmecd", c("#2a2a28"), 1.0)
    for x in (-0.09, 0.09):
        b.blob(dk, (x, -0.22, 0.4), (0.045, 0.02, 0.025), seg=8)
    b.blob(st, (0, -0.28, 0.3), (0.07, 0.05, 0.05), seg=8)
    b.box(dk, (0, -0.26, 0.2), (0.14, 0.02, 0.025))
    b.blob(st, (0, 0.0, 0.56), (0.26, 0.23, 0.08), seg=12)
    for x in (-0.25, 0.25):
        b.blob(st, (x, 0.0, 0.34), (0.04, 0.08, 0.1), seg=8)
    return {"fit": False}


# ------------------------------------------------------------- camps

@model("barbarian_camp")
def barbarian_camp(b):
    ground_patch(b, "#5a4a3a", "#3a2e24", 0.68, seed=20)
    dk = "#5a2a2a"
    tent(b, -0.28, 0.1, 0.22, 0.36, "#6a3030", seg=10, door_col="#120808")
    tent(b, 0.22, 0.2, 0.18, 0.3, "#4a3a34", seg=10, door_col="#120808")
    tent(b, 0.3, -0.2, 0.15, 0.24, "#5a2a2a", seg=10, door_col="#120808")
    # fire
    sm = rock_mat("#4a4440")
    for k in range(7):
        a = k * 0.9
        b.rock(sm, (-0.02 + math.cos(a) * 0.08, -0.22 + math.sin(a) * 0.08, 0.025), (0.035, 0.035, 0.03), k, subdiv=1)
    wd = wood_mat("#4a3020", "#2a1a10")
    for a in (0.3, 1.9, 3.5):
        b.cyl(wd, (-0.02 + math.cos(a) * 0.07, -0.22 + math.sin(a) * 0.07, 0.03), (-0.02, -0.22, 0.09), 0.012, 0.012, seg=4)
    fm = mat("fire", c("#ff8a1e"), 0.4, emit=8.0, emit_col=c("#ff7010"))
    b.cone(fm, (-0.02, -0.22, 0.03), 0.15, 0.045, seg=6)
    fm2 = mat("fire2", c("#ffd040"), 0.4, emit=10.0, emit_col=c("#ffd040"))
    b.cone(fm2, (-0.02, -0.22, 0.03), 0.09, 0.025, seg=6)
    # banner pole with dark flag and skull
    b.cyl(wd, (0.05, 0.1, 0), (0.05, 0.1, 0.66), 0.014, 0.012, seg=5)
    bm = mat("bflag", c("#1c1418"), 0.8)
    b.box(bm, (0.15, 0.1, 0.55), (0.2, 0.01, 0.2), yaw=0.0)
    sk = mat("skull", c("#e8e0d0"), 0.7)
    b.blob(sk, (0.15, 0.092, 0.57), (0.04, 0.01, 0.045), seg=8)
    b.box(sk, (0.15, 0.092, 0.51), (0.03, 0.008, 0.02))
    b.blob(mat("bred", c("#c0302a"), 0.5), (0.05, 0.1, 0.68), (0.025, 0.025, 0.025), seg=6)
    # spikes
    for (x, y) in [(-0.5, -0.1), (-0.45, -0.3), (0.5, 0.0), (0.0, 0.42), (-0.2, 0.4)]:
        b.cone(wd, (x, y, 0), 0.22, 0.02, seg=4, tip_r=0.0)
    return {"fit": False}


@model("imp_generic")
def generic(b):
    ground_patch(b, "#8a7a5a", "#6a5c40", 0.62, seed=21)
    house(b, -0.1, 0.05, 0.36, 0.26, 0.18, PLASTER(), ROOF_RED(), rh=0.14)
    house(b, 0.22, -0.12, 0.2, 0.18, 0.12, PLASTER(), ROOF_RED(), rh=0.1, door=False)
    flag(b, (0.1, 0.25, 0.0), 0.4, "#2a4aa8", 0.14, 0.1)
    return {"fit": False}
