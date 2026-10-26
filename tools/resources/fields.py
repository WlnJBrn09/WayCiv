"""Farm and plantation field decals and the farmhouse sprites that sit on them.

Names: imp_farm_<era>_<variant> / imp_plantation_<variant> (top-down decals,
the hex-shaped edge is softened later by post.py), imp_farmhouse_<era>
(3/4 view). <era> 0..3 = ancient, medieval, industrial, modern. Every decal
keeps a clear yard at YARD where the game draws the farmhouse.
"""
import math
import random

import lib
from lib import *  # noqa: F403

MODELS = {}

# Where the farmhouse stands in decal space (x right, y up), and its clearing.
YARD = (-0.42, 0.34)
YARD_R = 0.27
APOTHEM = 0.866

SOIL = ("#7a5a38", "#5e4228")
CROPS = {
    # name: (leaf colours, row spacing, tuft spacing, tuft half size)
    "wheat": (["#e0bc52", "#d4aa3e", "#e8c860"], 0.056, 0.04, (0.022, 0.02, 0.04)),
    "barley": (["#b8c45a", "#a8b84c", "#c4cc66"], 0.056, 0.04, (0.022, 0.02, 0.036)),
    "greens": (["#5aa83e", "#4c9a36", "#68b448"], 0.07, 0.052, (0.03, 0.028, 0.03)),
    "cabbage": (["#8cc850", "#7cb844", "#9ad45c"], 0.085, 0.07, (0.04, 0.038, 0.03)),
    "lavender": (["#8a6ab8", "#7a5aa8", "#9a7ac8"], 0.07, 0.05, (0.026, 0.024, 0.032)),
    "rapeseed": (["#ecd030", "#e0c420", "#f4dc48"], 0.056, 0.04, (0.026, 0.024, 0.03)),
    "plough": (["#6a4a2c", "#7c5a38"], 0.05, 0.0, None),
    "fallow": (["#7aae4a", "#6c9e40", "#86b856"], 0.08, 0.06, (0.034, 0.03, 0.02)),
}


def hexd(x, y):
    """Distance from the hex centre toward its nearest flat edge (apothem units 1)."""
    return max(abs(x) , abs(x * 0.5 + y * 0.866), abs(x * 0.5 - y * 0.866)) / APOTHEM


def yard_clear(cx, cy, hw, hh, margin=0.0):
    nx = min(max(YARD[0], cx - hw), cx + hw)
    ny = min(max(YARD[1], cy - hh), cy + hh)
    return math.hypot(nx - YARD[0], ny - YARD[1]) > YARD_R + margin


def soil_mat():
    return mat("tilled", c(SOIL[0]), 1.0, col2=c(SOIL[1]), scale=14, bump=0.25)


def plot_rows(b, rng, cx, cy, w, h, ang, crop):
    """One field plot: soil bed, ridge rows, tufts. w along the rows."""
    cols, spacing, step, tuft = CROPS[crop]
    ca, sa = math.cos(ang), math.sin(ang)
    b.box(soil_mat(), (cx, cy, 0.006), (w, h, 0.014), yaw=ang)
    nrows = max(1, int(h / spacing))
    ridge = mat("ridge" + crop, c("#6a4a2c" if crop != "plough" else cols[0]), 1.0, col2=c(SOIL[1]), scale=30, bump=0.3)
    for i in range(nrows):
        ly = -h / 2 + (i + 0.5) * h / nrows
        px, py = cx - ly * sa, cy + ly * ca
        if crop == "plough":
            m = mat("plough" + str(i % 2), c(cols[i % 2]), 1.0, col2=c(SOIL[1]), scale=40, bump=0.4)
            b.cyl(m, (px - w / 2 * ca + 0.0, py - w / 2 * sa, 0.022), (px + w / 2 * ca, py + w / 2 * sa, 0.022),
                  spacing * 0.42, spacing * 0.42, seg=5, smooth=True)
            continue
        b.cyl(ridge, (px - w / 2 * ca, py - w / 2 * sa, 0.018), (px + w / 2 * ca, py + w / 2 * sa, 0.018),
              spacing * 0.3, spacing * 0.3, seg=5)
        cm = mat("crop" + cols[i % 3], c(cols[i % 3]), 0.7, col2=c(cols[(i + 1) % 3]), scale=25, bump=0.3, sss=0.15)
        nt = max(2, int(w / step))
        for k in range(nt):
            lx = -w / 2 + (k + 0.5) * w / nt + (rng.random() - 0.5) * 0.012
            sz = 0.85 + rng.random() * 0.4
            b.blob(cm, (px + lx * ca, py + lx * sa, 0.035), (tuft[0] * sz, tuft[1] * sz, tuft[2] * sz), seg=5)


def hedge(b, rng, p0, p1, h=0.07, wall=False):
    """A hedgerow (or low stone wall) along one plot edge."""
    L = math.hypot(p1[0] - p0[0], p1[1] - p0[1])
    n = max(2, int(L / 0.07))
    if wall:
        st = mat("fwall", c("#b0a890"), 0.95, col2=c("#8a826e"), scale=18, bump=0.5)
        b.cyl(st, (p0[0], p0[1], 0.03), (p1[0], p1[1], 0.03), 0.028, 0.028, seg=5)
        return
    lm = mat("hedge" + str(rng.randint(0, 2)), c(["#3f7a2c", "#4a8832", "#376e28"][rng.randint(0, 2)]), 0.8, col2=c("#5a9a3a"), scale=30, bump=0.3)
    for i in range(n + 1):
        t = i / n
        x, y = p0[0] + (p1[0] - p0[0]) * t, p0[1] + (p1[1] - p0[1]) * t
        s = 0.8 + rng.random() * 0.5
        b.blob(lm, (x, y, h * 0.5), (0.05 * s, 0.05 * s, h * 0.6 * s), seg=6)


def plot_border(b, rng, cx, cy, w, h, ang, kind):
    ca, sa = math.cos(ang), math.sin(ang)
    corners = [(-w / 2, -h / 2), (w / 2, -h / 2), (w / 2, h / 2), (-w / 2, h / 2)]
    pts = [(cx + x * ca - y * sa, cy + x * sa + y * ca) for x, y in corners]
    for i in range(4):
        if rng.random() < 0.55:
            hedge(b, rng, pts[i], pts[(i + 1) % 4], wall=(kind == "wall"))


def round_tree(b, x, y, s=1.0, col="#4a8a30"):
    lm = leaf_mat(col)
    b.cyl(wood_mat(), (x, y, 0), (x, y, 0.1 * s), 0.012 * s, 0.01 * s, seg=5)
    b.blob(lm, (x, y, 0.16 * s), (0.1 * s, 0.1 * s, 0.09 * s), seg=8)
    b.blob(lm, (x + 0.05 * s, y - 0.02 * s, 0.13 * s), (0.07 * s, 0.07 * s, 0.07 * s), seg=8)


def bsp(rng, x0, y0, x1, y1, min_side, out, depth=0, stop=0.25):
    w, h = x1 - x0, y1 - y0
    if (w < 2 * min_side and h < 2 * min_side) or (depth > 1 and rng.random() < stop):
        out.append((x0, y0, x1, y1))
        return
    if w >= h and w >= 2 * min_side or h < 2 * min_side:
        s = x0 + w * (0.38 + rng.random() * 0.24)
        bsp(rng, x0, y0, s, y1, min_side, out, depth + 1, stop)
        bsp(rng, s, y0, x1, y1, min_side, out, depth + 1, stop)
    else:
        s = y0 + h * (0.38 + rng.random() * 0.24)
        bsp(rng, x0, y0, x1, s, min_side, out, depth + 1, stop)
        bsp(rng, x0, s, x1, y1, min_side, out, depth + 1, stop)


def layout(era, variant):
    """Plot list [(cx, cy, w, h, ang, crop)] and extras for an era."""
    rng = random.Random(f"farm{era}{variant}")
    plots = []
    crops_pool = {
        0: ["wheat", "barley", "greens", "plough", "fallow", "wheat", "lavender"],
        1: ["wheat", "barley", "greens", "plough", "rapeseed", "wheat", "cabbage", "lavender"],
        2: ["wheat", "rapeseed", "barley", "greens", "plough", "wheat", "cabbage"],
        3: ["wheat", "rapeseed", "greens", "barley", "plough", "lavender"],
    }[era]
    leaves = []
    if era == 0:
        bsp(rng, -1.0, -1.0, 1.0, 1.0, 0.34, leaves, stop=0.2)
        jitter, gap = 0.12, 0.045
    elif era == 1:
        bsp(rng, -1.0, -1.0, 1.0, 1.0, 0.7, leaves, stop=0.0)
        jitter, gap = 0.03, 0.03
    elif era == 2:
        bsp(rng, -1.0, -1.0, 1.0, 1.0, 0.5, leaves, stop=0.1)
        jitter, gap = 0.0, 0.06
    else:
        bsp(rng, -1.0, -1.0, 1.0, 1.0, 0.55, leaves, stop=0.1)
        jitter, gap = 0.0, 0.07
    glob = rng.uniform(-0.2, 0.2) if era >= 2 else 0.0
    for (x0, y0, x1, y1) in leaves:
        cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
        w, h = x1 - x0 - gap * 2, y1 - y0 - gap * 2
        if hexd(cx, cy) > 0.98 or w < 0.12 or h < 0.12:
            continue
        if not yard_clear(cx, cy, w / 2, h / 2, 0.02):
            cut = [(x0, y0, YARD[0] - YARD_R - 0.04, y1), (YARD[0] + YARD_R + 0.04, y0, x1, y1),
                   (x0, y0, x1, YARD[1] - YARD_R - 0.04), (x0, YARD[1] + YARD_R + 0.04, x1, y1)]
            x0, y0, x1, y1 = max(cut, key=lambda r_: max(0, r_[2] - r_[0]) * max(0, r_[3] - r_[1]))
            cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
            w, h = x1 - x0 - gap * 2, y1 - y0 - gap * 2
            if w < 0.12 or h < 0.12 or hexd(cx, cy) > 0.98:
                continue
        ang = glob + rng.uniform(-jitter, jitter)
        crop = crops_pool[rng.randrange(len(crops_pool))]
        if era == 1:
            # open-field strips: thin parallel strips inside the block
            n = max(2, int(min(w, h) / 0.17))
            vertical = h > w
            for i in range(n):
                if vertical:
                    sw = w / n
                    plots.append((x0 + gap + sw * (i + 0.5), cy, h, sw - 0.012, 1.5708 + ang, crop if i % 3 else crops_pool[rng.randrange(len(crops_pool))]))
                else:
                    sh = h / n
                    plots.append((cx, y0 + gap + sh * (i + 0.5), w, sh - 0.012, ang, crop if i % 3 else crops_pool[rng.randrange(len(crops_pool))]))
            continue
        plots.append((cx, cy, w if w >= h else h, h if w >= h else w, ang + (0 if w >= h else 1.5708), crop))
    return rng, plots


def circle_field(b, rng, cx, cy, r):
    """A centre-pivot irrigated circle: concentric rings of crop."""
    soil = soil_mat()
    b.cyl(soil, (cx, cy, 0.0), (cx, cy, 0.014), r, r, seg=36)
    cols = rng.choice([["#5aa83e", "#7cc050"], ["#e0bc52", "#c8a040"], ["#8cc850", "#6aae44"]])
    n = int(r / 0.045)
    for i in range(n):
        rr = r * (i + 0.5) / n
        m = mat("pivot" + cols[i % 2], c(cols[i % 2]), 0.7, col2=c(cols[(i + 1) % 2]), scale=24, bump=0.3)
        b.ring(m, (cx, cy, 0.03), rr, r / n * 0.78, seg=48)
    ir = mat("pivarm", c("#d8dce0"), 0.3, metal=0.9)
    a = rng.uniform(0, 6.28)
    b.cyl(ir, (cx, cy, 0.07), (cx + math.cos(a) * r * 0.97, cy + math.sin(a) * r * 0.97, 0.07), 0.008, 0.008, seg=4)


def hay_rolls(b, rng, cx, cy, ang, n=3):
    hm = mat("hayroll", c("#dcbc50"), 0.9, col2=c("#b89430"), scale=30, bump=0.4)
    for i in range(n):
        x, y = cx + math.cos(ang) * 0.13 * i, cy + math.sin(ang) * 0.13 * i
        b.cyl(hm, (x - 0.04, y, 0.05), (x + 0.04, y, 0.05), 0.05, 0.05, seg=10)


def farm_scene(b, era, variant):
    rng, plots = layout(era, variant)
    for (cx, cy, w, h, ang, crop) in plots:
        if era == 3 and w > 0.7 and h > 0.5 and rng.random() < 0.55:
            circle_field(b, rng, cx, cy, min(w, h) / 2 * 0.95)
            continue
        plot_rows(b, rng, cx, cy, w, h, ang, crop)
        if era <= 2:
            plot_border(b, rng, cx, cy, w + 0.02, h + 0.02, ang, "wall" if era == 0 and rng.random() < 0.3 else "hedge")
    # a few trees and hay on the edges and around the yard
    spots = [(-0.82, 0.42), (-0.62, 0.7), (-0.18, 0.64), (-0.7, 0.05), (0.78, -0.45), (0.1, -0.78), (0.82, 0.3)]
    rng.shuffle(spots)
    for (x, y) in spots[:4]:
        round_tree(b, x, y, rng.uniform(0.9, 1.4), rng.choice(["#4a8a30", "#5a9a38", "#3f7a2c"]))
    if era >= 2:
        hay_rolls(b, rng, 0.3, -0.7, 0.2, 3)


def reg_farm(era, variant):
    def fn(b):
        farm_scene(b, era, variant)
        return {"top_down": True, "ortho": 2.0, "fit": False, "res": 448, "samples": 40, "catcher": 1.4}
    MODELS[f"imp_farm_{era}_{variant}"] = fn


for _e in range(4):
    for _v in range(2):
        reg_farm(_e, _v)


# ---------------------------------------------------------- plantation

def reg_plantation(variant):
    """Terraced rows of bushes (tea / cotton / orchard) with hedges between bands."""
    def fn(b):
        rng = random.Random(f"plant{variant}")
        soil = soil_mat()
        kinds = [("#3f8a2e", "#8cc84a", "#4a9a34"), ("#46902f", "#c8e060", "#58a83a")] if variant == 0 else [("#356e28", "#f0ece0", "#448a34"), ("#2f7a3a", "#e05a48", "#3f8f44")]
        y = -0.95
        band = 0
        while y < 0.95:
            nrows = rng.choice([3, 4])
            lm, tm, lm2 = kinds[band % 2]
            m1 = mat("shrub" + lm, c(lm), 0.6, col2=c(lm2), scale=14, bump=0.3, sss=0.25)
            m2 = mat("shrubt" + tm, c(tm), 0.5, sss=0.2)
            m3 = mat("berry" + tm, c("#f4f0e4" if variant == 1 else tm), 0.5)
            for r_ in range(nrows):
                yy = y + r_ * 0.15
                n = int(2.0 / 0.115)
                xs = []
                for k in range(n):
                    x = -1.0 + (k + 0.5) * 2.0 / n
                    if hexd(x, yy) > 0.9 or math.hypot(x - YARD[0], yy - YARD[1]) < YARD_R + 0.05:
                        continue
                    xs.append(x)
                    s_ = 0.9 + rng.random() * 0.3
                    b.blob(m1, (x, yy, 0.055 * s_), (0.07 * s_, 0.065 * s_, 0.06 * s_), seg=7)
                    b.blob(m2, (x + 0.008, yy + 0.008, 0.1 * s_), (0.045 * s_, 0.042 * s_, 0.028 * s_), seg=6)
                    if variant == 1 and rng.random() < 0.6:
                        b.blob(m3, (x + 0.03, yy - 0.03, 0.11 * s_), (0.016, 0.016, 0.016), seg=5)
                if xs:
                    b.box(soil, ((xs[0] + xs[-1]) / 2, yy, 0.004), (xs[-1] - xs[0] + 0.12, 0.14, 0.01))
            hy = y + nrows * 0.15 + 0.02
            for x in [-0.9 + 0.07 * i for i in range(27)]:
                if hexd(x, hy) < 0.88 and math.hypot(x - YARD[0], hy - YARD[1]) > YARD_R + 0.05:
                    lh = mat("hedgeb", c("#3a7428"), 0.8, col2=c("#5a9a3a"), scale=30, bump=0.3)
                    b.blob(lh, (x, hy, 0.035), (0.045, 0.04, 0.04), seg=6)
            y += nrows * 0.15 + 0.07
            band += 1
        for (x, y_) in [(-0.78, 0.55), (0.8, -0.4), (0.15, -0.8)]:
            round_tree(b, x, y_, 1.3, "#3f7a2c")
        return {"top_down": True, "ortho": 2.0, "fit": False, "res": 448, "samples": 40, "catcher": 1.4}
    MODELS[f"imp_plantation_{variant}"] = fn


for _v in range(2):
    reg_plantation(_v)


# ----------------------------------------------------------- farmhouses

def farmhouse_scene(b, era):
    """The farmhouse cluster, 3/4 view, on its own soft yard."""
    ground_patch(b, "#8a7048", "#a88a58", 0.55, seed=30 + era, grass=True)
    plaster = mat("fhplaster", c("#eadcb8"), 0.9, col2=c("#d0bc92"), scale=14, bump=0.2)
    wood = wood_mat("#8a5a2c", "#5a3a18")
    fence = wood_mat("#8a6a3a", "#6a4a24")
    if era == 0:
        hut_round(b, -0.12, 0.08, 0.2, 0.15)
        hut_round(b, 0.22, -0.02, 0.13, 0.1)
        hm = mat("hay", c("#dcbc50"), 1.0, col2=c("#b89430"), scale=40, bump=0.6)
        b.blob(hm, (0.3, 0.28, 0.1), (0.13, 0.11, 0.12), seg=10)
        b.cyl(wood, (0.3, 0.28, 0.0), (0.3, 0.28, 0.26), 0.012, 0.008, seg=4)
        sack(b, (-0.3, -0.18, 0), 0.8, content="#e0bc52")
        pts = [(-0.38, -0.3), (0.38, -0.3), (0.38, -0.12)]
        for i in range(2):
            fence_line(b, pts[i], pts[i + 1], h=0.1, rails=2)
        for k in range(3):
            b.blob(mat("chicken", c("#f4ecdc"), 0.9), (0.0 + k * 0.08, -0.22, 0.03), (0.03, 0.022, 0.025), seg=6)
    elif era == 1:
        tim = mat("timber", c("#7a4e2c"), 0.8, col2=c("#5a3a1e"), scale=10, bump=0.2)
        house(b, -0.1, 0.1, 0.46, 0.3, 0.17, plaster, mat("tilesr", c("#b8503a"), 0.7, col2=c("#8a3a28"), scale=20, bump=0.3), rh=0.17)
        for x in (-0.3, -0.1, 0.1):
            b.box(tim, (x, -0.054, 0.09), (0.02, 0.012, 0.17))
        b.box(tim, (-0.1, -0.054, 0.12), (0.46, 0.012, 0.02))
        b.box(mat("chim", c("#9a8878"), 0.9, bump=0.2), (0.06, 0.12, 0.34), (0.06, 0.06, 0.14))
        house(b, 0.3, 0.3, 0.3, 0.24, 0.14, mat("barnw", c("#8a5a34"), 0.9, col2=c("#6a4424"), scale=12, bump=0.2), mat("thatchb", c("#c8a850"), 1.0, col2=c("#9a7c34"), scale=40, bump=0.6), rh=0.14, door=False)
        b.blob(mat("hay", c("#dcbc50"), 1.0), (-0.32, -0.12, 0.08), (0.1, 0.09, 0.09), seg=10)
        fence_line(b, (-0.4, -0.3), (0.2, -0.3), h=0.1, rails=2)
    elif era == 2:
        red = mat("barnred", c("#a8372a"), 0.8, col2=c("#8a2a20"), scale=16, bump=0.2)
        roof = mat("tin", c("#8a9098"), 0.4, metal=0.7, col2=c("#a8aeb4"), scale=10)
        house(b, -0.18, 0.02, 0.5, 0.34, 0.22, red, roof, rh=0.2, door=False)
        wm = mat("whiteline", c("#f4f0e8"), 0.7)
        b.box(wm, (-0.18, -0.172, 0.11), (0.14, 0.012, 0.2))
        b.box(wm, (-0.18, -0.172, 0.11), (0.04, 0.014, 0.2), yaw=0)
        # silo
        sm = mat("silo", c("#c8ccd0"), 0.35, metal=0.8, col2=c("#a8aeb4"), scale=10)
        b.cyl(sm, (0.22, 0.12, 0), (0.22, 0.12, 0.4), 0.08, 0.08, seg=14)
        b.blob(sm, (0.22, 0.12, 0.4), (0.08, 0.08, 0.05), seg=12)
        house(b, 0.14, -0.22, 0.26, 0.2, 0.12, mat("fhw", c("#f0e8d4"), 0.9), mat("fhr", c("#4a5a7a"), 0.6), rh=0.1)
    else:
        sheet = mat("shed", c("#d8dcdc"), 0.4, metal=0.5, col2=c("#b8c0c4"), scale=10)
        roof = mat("shedroof", c("#4a6a8a"), 0.4, metal=0.6)
        house(b, -0.15, 0.1, 0.55, 0.34, 0.2, sheet, roof, rh=0.14, door=False)
        sm = mat("silo", c("#e0e4e6"), 0.3, metal=0.9)
        for k in range(2):
            b.cyl(sm, (0.22, 0.2 - k * 0.2, 0), (0.22, 0.2 - k * 0.2, 0.36), 0.085, 0.085, seg=14)
            b.cyl(mat("silotop", c("#8a9098"), 0.4, metal=0.8), (0.22, 0.2 - k * 0.2, 0.36), (0.22, 0.2 - k * 0.2, 0.46), 0.085, 0.0, seg=14)
        # tractor
        tr = mat("tractor", c("#3a8a3a"), 0.5)
        dk = mat("tyre", c("#18181a"), 0.9)
        b.box(tr, (-0.3, -0.22, 0.09), (0.22, 0.1, 0.07))
        b.box(tr, (-0.34, -0.22, 0.17), (0.09, 0.09, 0.07))
        b.cyl(dk, (-0.37, -0.28, 0.07), (-0.37, -0.16, 0.07), 0.07, 0.07, seg=12)
        b.cyl(dk, (-0.2, -0.28, 0.045), (-0.2, -0.16, 0.045), 0.045, 0.045, seg=10)
        # solar panels
        pv = mat("pv", c("#1a2a5a"), 0.15, coat=1.0)
        for i in range(3):
            b.box(pv, (0.1 + i * 0.13, -0.28, 0.07), (0.11, 0.17, 0.012), rot=lib.Matrix.Rotation(-0.5, 4, "X"))
    return {"fit": False, "target": 1.0}


def reg_house(era):
    MODELS[f"imp_farmhouse_{era}"] = lambda b: farmhouse_scene(b, era)


for _e in range(4):
    reg_house(_e)
