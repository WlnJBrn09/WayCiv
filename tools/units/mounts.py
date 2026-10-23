"""Animals and animal-drawn things: horses, camels, elephants, oxen, chariots, wagons."""
import math
from mathutils import Vector
from lib import *
from figures import human, NEAR, FAR


def wheel(loc, R, m="dkwood", spokes=8, w=0.09, hub="wood", rim="dkiron", plane="XZ"):
    torus(loc, R, w * 0.5, m, plane=plane, seg=16, tseg=5)
    for k in range(spokes // 2):
        a = math.pi * k / (spokes // 2)
        if plane == "XZ":
            p0 = (loc[0] - R * math.cos(a), loc[1], loc[2] - R * math.sin(a))
            p1 = (loc[0] + R * math.cos(a), loc[1], loc[2] + R * math.sin(a))
        else:
            p0 = (loc[0] - R * math.cos(a), loc[1] - R * math.sin(a), loc[2])
            p1 = (loc[0] + R * math.cos(a), loc[1] + R * math.sin(a), loc[2])
        limb(p0, p1, w * 0.3, m, seg=5)
    if plane == "XZ":
        cyl(loc, R * 0.2, w * 1.4, hub, rot=(90, 0, 0), seg=8)
    else:
        cyl(loc, R * 0.2, w * 1.4, hub, seg=8)


def _leg(a, b, c, r, col, hoof="black"):
    limb(a, b, r, col, r1=r * 0.75)
    limb(b, c, r * 0.72, col, r1=r * 0.6)
    box((c[0] + 0.03, c[1], c[2] + 0.02), (0.14, 0.13, 0.08), hoof, bevel=0.02)


def horse(o=None):
    """Horse standing at the origin facing +X.  Options: coat, mane, blanket (team),
    barding ('cloth' | 'plate'), rider (human dict), rider_scale, saddle_z, horn_ears."""
    o = dict(o or {})
    coat = o.get("coat", "#8a5a38")
    mane = o.get("mane", "#2b1d14")
    leg = o.get("legs", coat)
    sph((0, 0, 1.1), 1.0, coat, scale=(0.78, 0.34, 0.37), seg=12, rings=7, name="barrel")
    sph((0.5, 0, 1.15), 1.0, coat, scale=(0.36, 0.33, 0.4), name="chest")
    sph((-0.52, 0, 1.13), 1.0, coat, scale=(0.36, 0.33, 0.38), name="rump")
    # legs
    for sy in (-1, 1):
        _leg((0.52, sy * 0.17, 1.02), (0.58, sy * 0.17, 0.52), (0.55, sy * 0.17, 0.06), 0.1, leg)
        _leg((-0.56, sy * 0.17, 1.02), (-0.4, sy * 0.17, 0.55), (-0.52, sy * 0.17, 0.06), 0.11, leg)
    # neck + head
    limb((0.55, 0, 1.2), (1.0, 0, 1.78), 0.2, coat, r1=0.13, name="neck")
    limb((0.98, 0, 1.8), (1.34, 0, 1.52), 0.14, coat, r1=0.085, name="head")
    sph((1.36, 0, 1.5), 0.095, o.get("muzzle", "#5a3a28"), scale=(0.9, 0.9, 0.9))
    for sy in (-1, 1):
        cyl((0.95, sy * 0.07, 1.95), 0.04, 0.14, coat, r2=0.015, seg=5)
        sph((1.17, sy * 0.1, 1.72), 0.025, "black", seg=5, rings=3)
    # mane + tail
    for i in range(6):
        t = i / 5
        box((0.62 + 0.36 * t - 0.07, 0, 1.3 + 0.52 * t + 0.1), (0.07, 0.07, 0.2), mane, rot=(0, -28, 0), bevel=0.02)
    limb((-0.8, 0, 1.3), (-1.12, 0, 0.68), 0.07, mane, r1=0.02)
    sph((-1.12, 0, 0.64), 0.06, mane, scale=(1, 1, 1.5))
    # saddle gear
    sz = 1.43
    if "blanket" in o:
        box((0, 0, 1.4), (0.62, 0.78, 0.07), o["blanket"], bevel=0.03, name="blanket")
        # hanging cloth at the flanks
        for sy in (-1, 1):
            box((0, sy * 0.36, 1.2), (0.6, 0.05, 0.4), o["blanket"], bevel=0.02, rot=(sy * 8, 0, 0))
    box((-0.02, 0, 1.46), (0.42, 0.34, 0.07), "leather", bevel=0.03, name="saddle")
    box((-0.22, 0, 1.52), (0.06, 0.34, 0.14), "leather", bevel=0.03)
    if o.get("barding") == "cloth":
        c = o.get("barding_c", "team")
        sph((0.0, 0, 1.1), 1.0, c, scale=(0.5, 0.375, 0.4), seg=12, rings=6, name="barding")
        box((0.55, 0, 1.0), (0.4, 0.62, 0.3), c, bevel=0.03)
        for sy in (-1, 1):
            box((0.0, sy * 0.38, 0.85), (1.0, 0.04, 0.55), c, bevel=0.02, rot=(sy * 6, 0, 0))
        # chamfron
        box((1.18, 0, 1.68), (0.22, 0.14, 0.28), metal("steel"), rot=(0, 40, 0))
    elif o.get("barding") == "plate":
        sph((0.45, 0, 1.2), 1.0, metal("steel"), scale=(0.3, 0.34, 0.4))
        box((1.2, 0, 1.66), (0.3, 0.16, 0.26), metal("steel"), rot=(0, 40, 0))
        for sy in (-1, 1):
            box((0.0, sy * 0.36, 0.95), (0.9, 0.04, 0.4), o.get("barding_c", "team"), bevel=0.02)
    if o.get("plume"):
        cyl((1.0, 0, 2.02), 0.03, 0.25, o["plume"], seg=5)
    r = o.get("rider")
    if r is not None:
        rs = o.get("rider_scale", 0.9)
        with xf((-0.05, 0, 1.5 - 0.8 * rs + 0.04), (0, 0, 0), rs):
            human(dict(r, seated=True))
    return o


def ox(o=None):
    """Draft ox (same frame as the horse but chunkier, with horns and a hump)."""
    o = dict(o or {})
    coat = o.get("coat", "#6b4630")
    sph((0, 0, 0.9), 1.0, coat, scale=(0.72, 0.38, 0.38), seg=12, rings=7)
    sph((0.3, 0, 1.18), 1.0, coat, scale=(0.3, 0.3, 0.26), name="hump")
    for sy in (-1, 1):
        _leg((0.45, sy * 0.2, 0.85), (0.5, sy * 0.2, 0.45), (0.47, sy * 0.2, 0.05), 0.12, coat)
        _leg((-0.45, sy * 0.2, 0.85), (-0.4, sy * 0.2, 0.45), (-0.45, sy * 0.2, 0.05), 0.12, coat)
    limb((0.6, 0, 1.0), (0.95, 0, 1.15), 0.24, coat, r1=0.2)
    sph((1.05, 0, 1.08), 0.23, coat, scale=(1.0, 0.85, 0.95))
    sph((1.25, 0, 0.98), 0.14, "#d8b8a0", scale=(0.8, 1, 0.8))
    for sy in (-1, 1):
        limb((1.0, sy * 0.15, 1.25), (1.0, sy * 0.4, 1.35), 0.04, "ivory", r1=0.03)
        limb((1.0, sy * 0.4, 1.35), (1.05, sy * 0.46, 1.55), 0.03, "ivory", r1=0.01)
        sph((0.98, sy * 0.25, 1.12), 0.07, coat, scale=(0.5, 1, 0.5))
    limb((-0.7, 0, 1.0), (-0.85, 0, 0.5), 0.05, coat, r1=0.03)
    # yoke
    limb((0.7, -0.4, 1.3), (0.7, 0.4, 1.3), 0.045, "dkwood", seg=6)
    return o


def camel(o=None):
    """Dromedary with team-coloured pack blankets."""
    o = dict(o or {})
    coat = o.get("coat", "#c49a62")
    sph((0, 0, 1.25), 1.0, coat, scale=(0.78, 0.38, 0.38), seg=12, rings=7)
    sph((0.4, 0, 1.3), 1.0, coat, scale=(0.3, 0.33, 0.36))
    sph((-0.45, 0, 1.28), 1.0, coat, scale=(0.32, 0.33, 0.36))
    for sy in (-1, 1):
        _leg((0.5, sy * 0.19, 1.2), (0.55, sy * 0.19, 0.62), (0.52, sy * 0.19, 0.06), 0.095, coat, hoof="#5a4630")
        _leg((-0.5, sy * 0.19, 1.2), (-0.38, sy * 0.19, 0.65), (-0.5, sy * 0.19, 0.06), 0.1, coat, hoof="#5a4630")
    # S-curved neck
    limb((0.55, 0, 1.3), (0.85, 0, 1.55), 0.17, coat, r1=0.12)
    limb((0.85, 0, 1.55), (0.95, 0, 2.0), 0.12, coat, r1=0.1)
    limb((0.95, 0, 2.0), (1.28, 0, 2.15), 0.11, coat, r1=0.08)
    sph((1.3, 0, 2.13), 0.09, "#8a6a42", scale=(1.1, 0.8, 0.8))
    for sy in (-1, 1):
        sph((1.05, sy * 0.08, 2.1), 0.02, "black", seg=4, rings=3)
    limb((-0.75, 0, 1.4), (-0.85, 0, 0.9), 0.05, coat, r1=0.02)
    if o.get("humps", True):
        for hx in (0.2, -0.28):
            sph((hx, 0, 1.68), 0.3, coat, scale=(0.8, 0.8, 1.05))
    # packs
    c = o.get("pack", "team")
    box((0.0, 0, 1.72), (1.0, 0.95, 0.07), c, bevel=0.03, name="rug")
    for sy in (-1, 1):
        box((0.0, sy * 0.52, 1.35), (0.9, 0.05, 0.6), c, bevel=0.02, rot=(sy * 6, 0, 0))
        box((0.15, sy * 0.62, 1.15), (0.55, 0.3, 0.45), "rope", bevel=0.05)
        sph((-0.3, sy * 0.58, 1.05), 0.2, "brown", scale=(1, 0.9, 1.1))
    box((0.0, 0, 2.0), (0.5, 0.5, 0.35), "canvas", bevel=0.06)
    box((0.0, 0, 2.2), (0.3, 0.3, 0.1), "brown", bevel=0.03)
    return o


def elephant(o=None):
    """War elephant with a howdah.  o['crew'] = list of (human dict, x, y) in the
    howdah; o['mahout'] = human dict astride the neck."""
    o = dict(o or {})
    skin = o.get("skin", "#8b8b88")
    sph((0, 0, 1.55), 1.0, skin, scale=(1.05, 0.7, 0.72), seg=12, rings=7, name="body")
    for sx in (-0.65, 0.65):
        for sy in (-1, 1):
            cyl((sx, sy * 0.42, 0.58), 0.27, 1.15, skin, r2=0.23, seg=9, name="leg")
            box((sx + 0.04, sy * 0.42, 0.03), (0.4, 0.34, 0.1), "#6a655c")
    # head, trunk, tusks, ears
    sph((1.12, 0, 1.8), 0.5, skin, scale=(1.0, 0.85, 1.0), name="head")
    pts = [(1.45, 0, 1.55), (1.68, 0, 1.2), (1.74, 0, 0.8), (1.62, 0, 0.45), (1.46, 0, 0.35)]
    rr = [0.2, 0.17, 0.14, 0.11, 0.09]
    for i in range(len(pts) - 1):
        limb(pts[i], pts[i + 1], rr[i], skin, r1=rr[i + 1], seg=8)
    for sy in (-1, 1):
        limb((1.45, sy * 0.2, 1.45), (1.95, sy * 0.33, 1.15), 0.07, "ivory", r1=0.035, seg=6)
        limb((1.95, sy * 0.33, 1.15), (2.15, sy * 0.33, 1.4), 0.035, "ivory", r1=0.015, seg=6)
        sph((0.98, sy * 0.58, 1.85), 0.38, skin, scale=(0.22, 1.0, 1.2), rot=(0, 0, sy * 20), name="ear")
        sph((1.34, sy * 0.28, 2.0), 0.05, "black", seg=5, rings=3)
    limb((-1.0, 0, 1.6), (-1.1, 0, 0.8), 0.06, skin, r1=0.03)
    # blanket + howdah
    c = o.get("blanket", "team")
    box((0.0, 0, 2.2), (1.25, 1.1, 0.08), c, bevel=0.04, name="blanket")
    for sy in (-1, 1):
        box((0.0, sy * 0.58, 1.9), (1.2, 0.05, 0.62), c, bevel=0.02, rot=(sy * 10, 0, 0))
        box((0.0, sy * 0.58, 1.55), (1.2, 0.07, 0.07), "gold")
    if o.get("howdah", True):
        box((-0.1, 0, 2.3), (1.0, 0.85, 0.1), "dkwood", bevel=0.03)
        for (hx, hy, hl, hw) in ((-0.1, -0.42, 1.0, 0.05), (-0.1, 0.42, 1.0, 0.05), (-0.6, 0, 0.05, 0.85), (0.4, 0, 0.05, 0.85)):
            box((hx, hy, 2.5), (hl, hw, 0.32), c if hx == -0.1 and hy == -0.42 else "ltwood", bevel=0.02)
        for sx in (-0.6, 0.4):
            for sy in (-0.42, 0.42):
                limb((sx, sy, 2.34), (sx, sy, 3.0), 0.03, "gold", seg=5)
        box((-0.1, 0, 3.02), (1.1, 0.95, 0.08), c, bevel=0.04, name="canopy")
        cyl((-0.1, 0, 3.12), 0.2, 0.12, c, r2=0.05, seg=8)
    for (r, cx, cy, sc) in o.get("crew", []):
        with xf((cx, cy, 2.35), (0, 0, 0), sc):
            human(dict(r, seated=False, stride=0.0))
    if o.get("mahout"):
        with xf((0.95, 0, 2.3 - 0.8 * 0.55), (0, 0, 0), 0.55):
            human(dict(o["mahout"], seated=True))
    return o


def chariot(o=None):
    """Two-wheeled chariot drawn by horses.  o: horses (1/2), team body, crew list."""
    o = dict(o or {})
    body = o.get("body_c", "team")
    off = -0.7
    # floor, rails
    box((off, 0, 0.55), (0.9, 0.78, 0.07), "dkwood", bevel=0.03, name="floor")
    for sy in (-1, 1):
        box((off, sy * 0.4, 0.8), (0.9, 0.05, 0.45), body, bevel=0.025, name="side")
    box((off - 0.43, 0, 0.8), (0.05, 0.8, 0.45), body, bevel=0.025)
    for sy in (-1, 1):
        wheel((off, sy * 0.52, 0.42), 0.42, spokes=8, w=0.08)
    limb((off + 0.4, 0, 0.55), (0.45, 0, 1.2), 0.045, "ltwood", seg=6)  # pole
    # horses
    n = o.get("horses", 2)
    ys = [0.0] if n == 1 else [-0.3, 0.3]
    for y in ys:
        with xf((0.55, y, 0), (0, 0, 0), 0.82):
            horse(dict(o.get("horse", {})))
    limb((0.5, ys[0] * 1.0 - 0.0, 1.3), (0.5, ys[-1], 1.3), 0.04, "ltwood", seg=5)
    for (r, cx, cy, sc) in o.get("crew", [(dict(body="tunic"), off, 0, 0.8)]):
        with xf((cx, cy, 0.58), (0, 0, 0), sc):
            human(dict(r, stride=0.0))
    return o


def wagon(o=None):
    """Covered wagon (team canvas) drawn by an ox; driver on the box."""
    o = dict(o or {})
    cov = o.get("cover", "team")
    off = -0.45
    box((off, 0, 0.62), (1.7, 0.95, 0.12), "wood", bevel=0.03, name="bed")
    for sy in (-1, 1):
        box((off, sy * 0.5, 0.82), (1.7, 0.07, 0.32), "ltwood", bevel=0.02)
    box((off - 0.85, 0, 0.82), (0.07, 0.95, 0.32), "ltwood", bevel=0.02)
    # canvas bow
    cyl((off - 0.1, 0, 0.98), 0.58, 1.55, cov, rot=(0, 90, 0), seg=14, name="canvas")
    for k in (-0.75, 0.55):
        torus((off + k, 0, 0.98), 0.6, 0.025, "ltwood", plane="YZ", seg=14, tseg=4)
    # wheels
    for wx in (off - 0.5, off + 0.55):
        for sy in (-1, 1):
            wheel((wx, sy * 0.58, 0.42), 0.42 if wx > off else 0.36, spokes=8, w=0.08)
    limb((off + 0.85, 0, 0.6), (0.65, 0, 1.0), 0.05, "ltwood", seg=6)
    with xf((0.95, 0, 0), (0, 0, 0), 0.95):
        ox()
    # bundles at the back
    sph((off - 0.95, -0.2, 0.85), 0.22, "undyed", scale=(1, 1, 1.1))
    sph((off - 0.92, 0.25, 0.8), 0.18, "brown")
    # driver
    with xf((off + 0.8, 0.0, 0.7), (0, 0, 0), 0.6):
        human(dict(body="tunic", top="green", helmet="widehat", hat_c="ltwood", seated=True,
                   items=[("staff", {"side": FAR})]))
    return o
