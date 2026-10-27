"""Humanoid figures: soldiers, workers, priests, spies, great people."""
import math
import inspect
from mathutils import Vector
from lib import *

NEAR = -1   # the side the camera sees
FAR = 1


def C(o, part, default):
    """Colour for a part: 'team' if the figure lists that part as team-coloured."""
    return "team" if part in o.get("team", ()) else o.get(part, default)


# ------------------------------------------------------------------ heads
def head(o, hz=1.58):
    """A figurine head facing +X: skull and jaw, nose, ears, eyes with whites,
    brows, a neck, and hair framing the face.  The skull keeps radius 0.2 about
    (0.02, 0, hz) so every helmet still fits."""
    s = o.get("skin", "skin")
    hair = o.get("hair", "hair_brown")
    style = o.get("hairstyle", "short")
    limb((0.0, 0, 1.36), (0.02, 0, hz - 0.1), 0.085, s, r1=0.08, seg=10, name="neck")
    sph((0.02, 0, hz + 0.01), 0.2, s, scale=(0.95, 0.9, 1.0), seg=16, rings=10, name="head")
    sph((0.07, 0, hz - 0.08), 0.15, s, scale=(1.0, 0.95, 0.85), seg=14, rings=8, name="jaw")
    sph((0.205, 0, hz - 0.02), 0.042, s, scale=(1.25, 0.8, 1.2), seg=8, rings=6, name="nose")
    for sy in (-1, 1):
        sph((0.0, sy * 0.178, hz - 0.01), 0.045, s, scale=(0.6, 0.45, 1.0), seg=8, rings=5, name="ear")
        sph((0.168, sy * 0.072, hz + 0.035), 0.036, "white", scale=(0.55, 1.0, 1.05), seg=8, rings=5, name="eyewhite")
        sph((0.186, sy * 0.074, hz + 0.033), 0.022, "black", scale=(0.6, 1.0, 1.1), seg=6, rings=4, name="eye")
        if hair:
            box((0.17, sy * 0.075, hz + 0.085), (0.03, 0.075, 0.02), hair, rot=(sy * 8, 0, 0), bevel=0.008, name="brow")
    if o.get("beard"):
        sph((0.12, 0, hz - 0.13), 0.12, o["beard"], scale=(0.75, 1.0, 0.85), seg=10, rings=6, name="beard")
        box((0.2, 0, hz - 0.055), (0.03, 0.12, 0.025), o["beard"], bevel=0.01, name="moustache")
    if hair and style in ("short", "long", "bun"):
        # A cap over the crown and back, cut clear of the face, with a fringe.
        sph((-0.02, 0, hz + 0.06), 0.212, hair, scale=(0.95, 0.96, 0.9), seg=14, rings=8, name="hair")
        sph((0.1, 0, hz + 0.13), 0.12, hair, scale=(0.8, 1.3, 0.55), seg=10, rings=6, name="fringe")
        for sy in (-1, 1):
            sph((0.02, sy * 0.15, hz + 0.0), 0.08, hair, scale=(0.9, 0.5, 1.3), seg=8, rings=5, name="sideburn")
    if hair and style == "long":
        box((-0.15, 0, hz - 0.17), (0.14, 0.32, 0.42), hair, bevel=0.06, name="hairback")
    elif hair and style == "bun":
        sph((-0.15, 0, hz + 0.2), 0.09, hair, name="bun")
    helmet(o, o.get("helmet"), hz)


def helmet(o, kind, hz):
    if not kind:
        return
    if kind not in ("hood", "headband", "feather"):
        hz = hz + 0.09
    m = o.get("hat_c", "iron")
    t = "team" if "hat" in o.get("team", ()) else None
    if kind == "cap":
        sph((0, 0, hz + 0.04), 0.215, t or m, scale=(1, 1, 0.8), name="cap")
    elif kind == "bronze":
        sph((0, 0, hz + 0.05), 0.225, metal("bronze"), scale=(1, 1, 0.95), name="helm")
        box((0.15, 0, hz - 0.02), (0.1, 0.34, 0.2), metal("bronze"), name="cheek")
        # crest
        box((0, 0, hz + 0.3), (0.34, 0.07, 0.12), t or "red", name="crest")
        box((0, 0, hz + 0.24), (0.08, 0.08, 0.1), metal("bronze"), name="crestbase")
    elif kind == "conical":
        cyl((0, 0, hz + 0.18), 0.22, 0.28, metal(m) if m in ("iron", "steel", "bronze") else m, r2=0.03, seg=10, name="cone")
        cyl((0, 0, hz + 0.05), 0.225, 0.07, "dkiron", seg=10)
    elif kind == "kettle":
        cyl((0, 0, hz + 0.1), 0.22, 0.15, metal("iron"), r2=0.17, seg=10)
        cyl((0, 0, hz + 0.06), 0.27, 0.03, metal("iron"), seg=12)
    elif kind == "morion":
        sph((0, 0, hz + 0.07), 0.225, metal("steel"), scale=(1, 1, 0.8))
        box((0, 0, hz + 0.22), (0.4, 0.04, 0.08), metal("steel"), name="comb")
        box((0.0, 0, hz + 0.06), (0.62, 0.3, 0.025), metal("steel"), rot=(0, 0, 0), name="brim")
        box((0.0, 0, hz + 0.06), (0.3, 0.62, 0.025), metal("steel"), name="brim2")
    elif kind == "tricorn":
        cyl((0, 0, hz + 0.08), 0.22, 0.14, "black", r2=0.19, seg=8)
        for a in (0, 120, 240):
            with xf((0, 0, 0), (0, 0, a)):
                box((0.25, 0, hz + 0.13), (0.28, 0.2, 0.04), "black", rot=(0, -18, 0))
        sph((0.2, 0.0, hz + 0.18), 0.045, "gold")
    elif kind == "shako":
        cyl((0, 0, hz + 0.24), 0.17, 0.36, t or "black", r2=0.2, seg=10)
        box((0.17, 0, hz + 0.07), (0.1, 0.3, 0.025), "black")
        cyl((0.02, 0, hz + 0.47), 0.03, 0.12, "white", seg=6)
        sph((0.02, 0, hz + 0.54), 0.06, t or "red")
    elif kind == "bearskin":
        cyl((0, 0, hz + 0.3), 0.2, 0.5, "black", r2=0.22, seg=10, name="bearskin")
        sph((0, 0, hz + 0.55), 0.22, "black", scale=(1, 1, 0.5))
        cyl((0.1, 0, hz + 0.6), 0.025, 0.2, t or "red", seg=6)
    elif kind == "kabuto":
        sph((0, 0, hz + 0.06), 0.225, metal("dkiron"), scale=(1, 1, 0.9))
        cyl((-0.04, 0, hz - 0.1), 0.3, 0.1, t or "red", r2=0.24, seg=10)
        for sy in (-1, 1):
            limb((0.18, sy * 0.06, hz + 0.22), (0.38, sy * 0.3, hz + 0.5), 0.025, metal("gold"), r1=0.01)
    elif kind == "slouch":
        cyl((0, 0, hz + 0.09), 0.2, 0.13, m if m not in ("iron",) else "khaki", r2=0.18, seg=10)
        cyl((0, 0, hz + 0.04), 0.3, 0.03, m if m not in ("iron",) else "khaki", seg=12)
        cyl((0, 0, hz + 0.07), 0.205, 0.04, t or "brown", seg=10)
    elif kind == "modern":
        sph((0, 0, hz + 0.05), 0.225, m if m != "iron" else "olive", scale=(1.05, 1.0, 0.85))
        box((0.1, 0, hz + 0.0), (0.24, 0.38, 0.03), m if m != "iron" else "olive")
    elif kind == "turban":
        sph((0, 0, hz + 0.08), 0.235, t or "white", scale=(1, 1, 0.85))
        torus((0, 0, hz + 0.08), 0.2, 0.05, t or "white", plane="XY", seg=10)
    elif kind == "hood":
        sph((-0.03, 0, hz + 0.02), 0.255, t or m, scale=(1, 1.02, 1.0), name="hood")
        sph((0.13, 0, hz + 0.0), 0.2, o.get("skin", "skin"), scale=(0.4, 0.8, 0.9))
        sph((0.16, 0, hz + 0.02), 0.2, "black", scale=(0.25, 0.65, 0.7))
        sph((0.18, 0, hz + 0.02), 0.19, o.get("skin", "skin"), scale=(0.2, 0.7, 0.8))
    elif kind == "fur":
        cyl((0, 0, hz + 0.1), 0.24, 0.24, "fur", r2=0.22, seg=10)
        sph((0, 0, hz + 0.22), 0.22, "fur", scale=(1, 1, 0.45))
    elif kind == "horned":
        sph((0, 0, hz + 0.06), 0.225, metal("iron"), scale=(1, 1, 0.8))
        for sy in (-1, 1):
            limb((0, sy * 0.2, hz + 0.1), (0.05, sy * 0.4, hz + 0.28), 0.045, "ivory", r1=0.02)
            limb((0.05, sy * 0.4, hz + 0.28), (0.1, sy * 0.45, hz + 0.45), 0.03, "ivory", r1=0.01)
    elif kind == "feather":
        torus((0, 0, hz + 0.1), 0.19, 0.035, t or "red", plane="XY", seg=10)
        for i in range(7):
            a = -60 + i * 20
            with xf((0, 0, hz + 0.12), (0, 0, 0)):
                p = (-0.1 + 0.0, math.sin(math.radians(a)) * 0.25, 0.12 * math.cos(math.radians(a)))
                limb(p, (p[0] - 0.12, p[1] * 1.6, p[2] + 0.28), 0.035, ["green", "teal", "gold", "red"][i % 4], r1=0.01)
    elif kind == "pith":
        sph((0, 0, hz + 0.07), 0.24, "khaki", scale=(1, 1, 0.85))
        cyl((0, 0, hz + 0.03), 0.32, 0.04, "khaki", r2=0.28, seg=12)
    elif kind == "widehat":
        cyl((0, 0, hz + 0.1), 0.19, 0.18, m if m != "iron" else "brown", r2=0.17, seg=10)
        cyl((0, 0, hz + 0.04), 0.3, 0.03, m if m != "iron" else "brown", seg=12)
        cyl((0, 0, hz + 0.08), 0.2, 0.05, t or "red", seg=10)
    elif kind == "beret":
        cyl((0, 0, hz + 0.1), 0.24, 0.07, t or "red", seg=10)
    elif kind == "hardhat":
        sph((0, 0, hz + 0.06), 0.225, "yellow", scale=(1.05, 1.0, 0.85))
        box((0.15, 0, hz + 0.02), (0.14, 0.34, 0.03), "yellow")
    elif kind == "bicorne":
        sph((0, 0, hz + 0.1), 0.3, "black", scale=(0.55, 1.0, 0.42), name="bicorne")
        box((0.0, 0.0, hz + 0.2), (0.1, 0.5, 0.03), t or "gold")
        sph((0.12, 0, hz + 0.16), 0.045, "gold")
    elif kind == "mitre":
        cyl((0, 0, hz + 0.2), 0.2, 0.36, t or "white", r2=0.1, seg=4, rot=(0, 0, 45))
        box((0.1, 0, hz + 0.32), (0.04, 0.12, 0.3), "gold")
    elif kind == "headband":
        torus((0, 0, hz + 0.14), 0.16, 0.03, t or "red", plane="XY", seg=10)
    elif kind == "hat_witch":
        pass
    elif kind == "crown":
        cyl((0, 0, hz + 0.18), 0.17, 0.12, "gold", r2=0.2, seg=8)
    elif kind == "dumala":
        # tall Sikh turban
        cyl((0, 0, hz + 0.22), 0.2, 0.4, t or "blue", r2=0.17, seg=10)
        sph((0, 0, hz + 0.42), 0.18, t or "blue", scale=(1, 1, 0.6))
        torus((0.1, 0, hz + 0.28), 0.12, 0.025, "silver", plane="YZ", seg=10)
    elif kind == "wings":
        pass


# ------------------------------------------------------------------ items
def _hand(side, p):
    return Vector((p[0], side * p[1], p[2]))


def item_spear(o, side=FAR, length=2.35, head="steel", tip=0.2):
    hx = (0.18, side * 0.34, 1.0)
    base = (0.0, side * 0.36, -0.05)
    top = (0.35, side * 0.36, length - 0.05)
    limb(base, top, 0.035, "ltwood", seg=6)
    p = Vector(top)
    d_ = (Vector(top) - Vector(base)).normalized()
    limb(p - d_ * 0.02, p + d_ * tip, 0.065, metal(head), r1=0.0, seg=6)
    return Vector(hx)


def item_pike(o, side=FAR):
    return item_spear(o, side, length=2.9, tip=0.14)


def item_sword(o, side=FAR, length=0.78, raised=True):
    h = (0.3, side * 0.36, 1.22)
    if raised:
        tip = (0.62, side * 0.38, 1.9)
    else:
        tip = (0.9, side * 0.36, 1.1)
    hv, tv = Vector(h), Vector(tip)
    dv = (tv - hv).normalized()
    start = hv - dv * 0.06
    limb(start, hv + dv * 0.1, 0.03, "leather", seg=6)
    limb(hv + dv * 0.1, hv + dv * 0.13, 0.08, metal("gold"), seg=6)
    beam(hv + dv * 0.12, hv + dv * (0.12 + length), 0.075, 0.025, metal("steel"), up=(0, 1, 0))
    return hv


def item_club(o, side=FAR):
    h = (0.28, side * 0.34, 1.18)
    hv = Vector(h)
    tip = Vector((0.5, side * 0.36, 1.85))
    dv = (tip - hv).normalized()
    limb(hv - dv * 0.08, tip, 0.04, "ltwood", r1=0.11, seg=7)
    sph(tip, 0.11, "ltwood", seg=7, rings=4)
    return hv


def item_axe(o, side=FAR, big=False):
    hv = Vector((0.3, side * 0.36, 1.15))
    tip = Vector((0.55, side * 0.38, 1.85 if not big else 2.05))
    dv = (tip - hv).normalized()
    limb(hv - dv * 0.25, tip, 0.035, "ltwood", seg=6)
    with xf(tip - dv * 0.1):
        box((0.0, 0.0, 0.0), (0.06, 0.34, 0.28), metal("steel"), name="axehead")
    return hv


def item_pick(o, side=FAR):
    hv = Vector((0.3, side * 0.34, 1.25))
    tip = Vector((0.5, side * 0.38, 1.9))
    dv = (tip - hv).normalized()
    limb(hv - dv * 0.35, tip, 0.04, "ltwood", seg=6)
    with xf(tip):
        limb((0, -0.3, 0.0), (0, 0.3, 0.0), 0.045, metal("iron"), r1=0.015, seg=6)
        sph((0, 0, 0), 0.06, metal("iron"))
    return hv


def item_hammer(o, side=FAR):
    hv = Vector((0.3, side * 0.34, 1.0))
    tip = Vector((0.5, side * 0.38, 1.55))
    dv = (tip - hv).normalized()
    limb(hv - dv * 0.2, tip, 0.04, "ltwood", seg=6)
    with xf(tip + dv * 0.02):
        box((0, 0, 0), (0.12, 0.34, 0.14), metal("iron"), name="hammerhead")
    return hv


def item_shield(o, side=NEAR, kind="round", r=0.34, c="team", rim="bronze", boss=True):
    """Shield held on the arm, tilted to face the camera."""
    cen = Vector((0.2, side * 0.36, 1.0))
    ang = -50 * (-side)  # yaw of the normal towards the near side
    with xf(cen, (0, 0, side * 50)):
        # local X is the shield normal
        if kind == "round":
            cyl((0, 0, 0), r, 0.07, c, rot=(0, 90, 0), seg=24, name="shield")
            torus((0.0, 0, 0), r, 0.035, metal(rim), plane="YZ", seg=24, tseg=6)
            torus((0.037, 0, 0), r * 0.62, 0.016, "white" if c == "team" else "team", plane="YZ", seg=20, tseg=4)
            for k in range(10):
                a = 2 * math.pi * k / 10
                sph((0.04, r * 0.84 * math.cos(a), r * 0.84 * math.sin(a)), 0.018, metal(rim), seg=6, rings=4, name="rivet")
            if boss:
                sph((0.06, 0, 0), 0.1, metal(rim), scale=(0.6, 1, 1))
        elif kind == "rect":      # Roman scutum
            box((0, 0, 0), (0.07, r * 1.35, r * 2.1), c, bevel=0.03, name="scutum")
            box((0.03, 0, 0), (0.06, r * 1.4, 0.07), metal(rim))
            box((0.03, 0, 0), (0.06, 0.07, r * 2.12), metal(rim))
            sph((0.06, 0, 0), 0.09, metal(rim), scale=(0.5, 1, 1))
        elif kind == "oval":
            sph((0, 0, 0), 1.0, c, scale=(0.05, r * 0.75, r * 1.5), seg=12, rings=8, name="oval")
            limb((0.04, 0, -r * 1.45), (0.04, 0, r * 1.45), 0.03, "dkwood", seg=5)
        elif kind == "kite":
            pts = [(-r * 0.8, r * 0.9), (r * 0.8, r * 0.9), (r * 0.8, 0.0), (0, -r * 1.6), (-r * 0.8, 0.0)]
            poly(pts, c, plane="YZ", thick=0.07, name="kite")
            limb((0.05, 0, r * 0.88), (0.05, 0, -r * 1.5), 0.025, metal(rim), seg=5)
            limb((0.05, -r * 0.78, r * 0.6), (0.05, r * 0.78, r * 0.6), 0.025, metal(rim), seg=5)
        elif kind == "wicker":    # Persian spara
            box((0, 0, 0), (0.07, r * 1.5, r * 1.7), c, bevel=0.04, name="wicker")
            box((0.03, 0, 0), (0.06, r * 1.55, 0.05), "ltwood")
            box((0.03, 0, 0.18), (0.06, r * 1.55, 0.05), "ltwood")
            box((0.03, 0, -0.18), (0.06, r * 1.55, 0.05), "ltwood")
        elif kind == "pavise":
            box((0, 0, 0), (0.08, r * 1.5, r * 2.4), "ltwood", bevel=0.03)
            box((0.04, 0, 0), (0.06, 0.12, r * 2.4), c)
    return cen + Vector((0, side * 0.0, 0.0))


def item_bow(o, side=FAR, long=False):
    """Bow held in front by the `side` hand, the other hand draws the string."""
    cx, cy, cz = 0.5, side * 0.12, 1.22
    h = 0.62 if not long else 0.8
    pts = []
    n = 6
    for i in range(n + 1):
        t = -1 + 2 * i / n
        pts.append((cx + 0.12 * (1 - t * t) * 0 - 0.0 + 0.1 * t * t - 0.05, cy, cz + h * t))
    for a, b in zip(pts[:-1], pts[1:]):
        limb(a, b, 0.032, "ltwood", seg=5)
    top, bot = pts[-1], pts[0]
    pull = Vector((0.25, -side * 0.0 + side * 0.02, cz + 0.02))
    pull.y = cy - side * 0.0
    limb(top, pull, 0.01, "white", seg=4)
    limb(bot, pull, 0.01, "white", seg=4)
    # arrow
    limb((pull.x, cy, cz + 0.02), (cx + 0.55, cy, cz + 0.02), 0.014, "ltwood", seg=4)
    limb((cx + 0.55, cy, cz + 0.02), (cx + 0.63, cy, cz + 0.02), 0.03, metal("steel"), r1=0.0, seg=4)
    return {side: Vector((cx - 0.05, cy, cz)), -side: pull}


def item_crossbow(o, side=FAR):
    cx, cz = 0.42, 1.2
    beam((0.12, 0, cz - 0.02), (0.75, 0, cz + 0.04), 0.06, 0.08, "dkwood")
    # prod (the bow part), across Y
    limb((0.7, -0.4, cz + 0.04), (0.7, 0.4, cz + 0.04), 0.03, "ltwood", seg=5)
    limb((0.7, -0.4, cz + 0.04), (0.35, 0.0, cz + 0.05), 0.008, "white", seg=4)
    limb((0.7, 0.4, cz + 0.04), (0.35, 0.0, cz + 0.05), 0.008, "white", seg=4)
    limb((0.3, 0, cz + 0.08), (0.86, 0, cz + 0.08), 0.014, "ltwood", seg=4)
    limb((0.86, 0, cz + 0.08), (0.94, 0, cz + 0.08), 0.035, metal("steel"), r1=0.0, seg=4)
    return {FAR: Vector((0.32, 0.1, cz - 0.04)), NEAR: Vector((0.56, -0.1, cz))}


def item_sling(o, side=FAR):
    h = Vector((0.12, side * 0.34, 1.82))
    r0 = h + Vector((0.12, side * 0.28, 0.25))
    limb(h, r0, 0.012, "rope", seg=4)
    limb(h, h + Vector((-0.3, side * 0.2, 0.3)), 0.012, "rope", seg=4)
    sph(r0, 0.055, "stone")
    return h


def item_musket(o, side=FAR, rifle=False, bayonet=True):
    a = Vector((0.18, -0.08, 1.05))
    b = Vector((0.95, 0.06, 1.55))
    limb(a - (b - a).normalized() * 0.3, a + (b - a) * 0.15, 0.06, "dkwood", seg=6)
    limb(a, b, 0.032 if not rifle else 0.028, metal("dkiron") if not rifle else "dkgrey", seg=6)
    if bayonet:
        dv = (b - a).normalized()
        limb(b, b + dv * 0.5, 0.025, metal("steel"), r1=0.0, seg=4)
    return {NEAR: a + (b - a) * 0.12, FAR: a + (b - a) * 0.5 + Vector((0, 0.1, 0))}


def item_staff(o, side=FAR, topper="none", c="ltwood"):
    h = Vector((0.2, side * 0.34, 1.0))
    base = Vector((0.08, side * 0.38, -0.05))
    top = Vector((0.3, side * 0.38, 2.05))
    limb(base, top, 0.035, c, seg=6)
    if topper == "cross":
        limb(top, top + Vector((0, 0, 0.3)), 0.03, "gold", seg=5)
        limb(top + Vector((0, -0.12, 0.2)), top + Vector((0, 0.12, 0.2)), 0.03, "gold", seg=5)
    elif topper == "orb":
        sph(top + Vector((0, 0, 0.08)), 0.1, "gold")
    elif topper == "crook":
        torus(top + Vector((0, 0, 0.1)), 0.14, 0.03, "gold", plane="XZ", seg=10)
    return h


def item_book(o, side=NEAR, c="team"):
    p = Vector((0.3, side * 0.2, 1.0))
    box(p, (0.24, 0.3, 0.06), c, rot=(0, -25, 0), name="book")
    box(p + Vector((0, 0, 0.0)), (0.2, 0.26, 0.07), "ivory", rot=(0, -25, 0))
    return p


def item_torch(o, side=FAR):
    hv = Vector((0.3, side * 0.34, 1.3))
    tip = Vector((0.45, side * 0.36, 1.85))
    limb(hv - Vector((0, 0, 0.1)), tip, 0.03, "dkwood", seg=5)
    sph(tip + Vector((0, 0, 0.12)), 0.13, "#ffb030", scale=(1, 1, 1.5), seg=6, rings=4)
    sph(tip + Vector((0, 0, 0.1)), 0.08, "#fff0a0", scale=(1, 1, 1.5), seg=6, rings=4)
    return hv


def item_bag(o, side=NEAR, c="brown"):
    # satchel at the hip
    sph((0.0, side * 0.36, 0.8), 0.17, c, scale=(0.7, 0.5, 0.9), name="bag")
    limb((0.0, 0.0, 1.3), (0.0, side * 0.36, 0.85), 0.02, "leather", seg=4)
    return None


def item_flag(o, side=FAR, c="team"):
    hv = Vector((0.2, side * 0.34, 1.0))
    base = Vector((0.0, side * 0.38, -0.05))
    top = Vector((0.1, side * 0.38, 2.5))
    limb(base, top, 0.03, "ltwood", seg=6)
    cloth((0.1, side * 0.38, 2.45), (0.1, side * 0.38 - 0.0, 2.45), (0.1, side * 0.38, 1.9), c)
    # pennant
    cloth((0.08, side * 0.38, 2.45), (-0.62, side * 0.38, 2.4), (0.08, side * 0.38, 1.9), c, bulge=0.0, axis=(0, 1, 0), wave=0.12, nu=6, nv=3)
    sph(top + Vector((0, 0, 0.05)), 0.06, "gold")
    return hv


def item_binoculars(o, side=NEAR):
    p = Vector((0.2, 0.0, 1.55))
    for sy in (-1, 1):
        cyl((p.x + 0.12, sy * 0.06, p.z), 0.045, 0.2, "black", rot=(0, 90, 0), seg=8)
    return {NEAR: Vector((0.2, -0.1, 1.45)), FAR: Vector((0.2, 0.1, 1.45))}


def item_shovel(o, side=NEAR):
    hv = Vector((0.3, side * 0.34, 1.1))
    base = Vector((0.1, side * 0.38, -0.05))
    top = Vector((0.45, side * 0.38, 1.7))
    limb(base, top, 0.035, "ltwood", seg=6)
    with xf(base):
        box((0.0, 0.0, 0.1), (0.04, 0.2, 0.32), metal("iron"), rot=(0, 10, 0))
    return hv


def item_rifle_slung(o, side=NEAR, rifle=True):
    limb((-0.2, side * 0.2, 0.55), (0.1, side * 0.28, 1.8), 0.03, "dkgrey", seg=6)
    limb((-0.2, side * 0.2, 0.55), (-0.25, side * 0.2, 0.3), 0.05, "dkwood", seg=6)
    return None


def item_missile_launcher(o, side=FAR):
    # shoulder-fired AT launcher
    a = Vector((-0.45, -0.18, 1.5))
    b = Vector((0.85, -0.12, 1.55))
    limb(a, b, 0.09, "olive", seg=8)
    limb(b, b + Vector((0.1, 0, 0.0)), 0.12, "dkgrey", seg=8)
    cyl((0.25, -0.12, 1.65), 0.07, 0.2, "dkgrey", rot=(0, 90, 0), seg=6)
    return {NEAR: Vector((0.1, -0.12, 1.4)), FAR: Vector((0.5, 0.05, 1.45))}


def item_scroll(o, side=NEAR):
    p = Vector((0.25, side * 0.25, 1.05))
    cyl(p, 0.05, 0.3, "ivory", rot=(90, 0, 0), seg=8)
    cyl(p + Vector((0, 0, 0)), 0.055, 0.04, "red", rot=(90, 0, 0), seg=8)
    return p


def item_lute(o, side=NEAR):
    p = Vector((0.3, 0.0, 1.05))
    sph(p, 0.17, "ltwood", scale=(0.5, 1.0, 1.0), rot=(0, 20, 0))
    limb(p, p + Vector((0.2, 0.0, 0.5)), 0.03, "dkwood", seg=5)
    return {NEAR: Vector((0.25, -0.12, 1.2)), FAR: Vector((0.32, 0.12, 1.0))}


def item_palette(o, side=NEAR):
    p = Vector((0.3, side * 0.3, 1.1))
    cyl(p, 0.17, 0.03, "ltwood", rot=(0, 70, 0), seg=10)
    for i, c in enumerate(("red", "blue", "yellow")):
        sph(p + Vector((0.03, (i - 1) * 0.07, 0.05 * (i % 2))), 0.03, c, seg=5, rings=3)
    return p


def item_quill(o, side=FAR):
    hv = Vector((0.3, side * 0.3, 1.25))
    limb(hv, hv + Vector((0.12, 0, 0.35)), 0.012, "ivory", seg=4)
    return hv


def item_flask(o, side=FAR):
    p = Vector((0.32, side * 0.28, 1.25))
    sph(p, 0.09, "glass", seg=8, rings=5)
    cyl(p + Vector((0, 0, 0.12)), 0.03, 0.12, "glass", seg=6)
    sph(p - Vector((0, 0, 0.02)), 0.06, "#7de07d", seg=6, rings=4)
    return p


def item_coins(o, side=FAR):
    p = Vector((0.3, side * 0.3, 0.9))
    sph(p, 0.14, "brown", scale=(1, 1, 1.1), seg=8, rings=5)
    cyl(p + Vector((0, 0, 0.14)), 0.05, 0.04, "rope", seg=6)
    sph(p + Vector((0.12, 0, 0.12)), 0.05, "gold", seg=6, rings=4)
    return p


def item_gear(o, side=FAR):
    p = Vector((0.3, side * 0.3, 1.25))
    cyl(p, 0.14, 0.05, metal("iron"), rot=(0, 90, 0), seg=8)
    cyl(p, 0.06, 0.07, "dkiron", rot=(0, 90, 0), seg=6)
    return p


def item_anchorish(o, side=FAR):
    return None


def item_spyglass(o, side=FAR):
    hv = Vector((0.25, side * 0.3, 1.55))
    limb(hv, hv + Vector((0.45, side * 0.1, 0.12)), 0.04, metal("gold"), r1=0.06, seg=7)
    return hv


def item_dagger(o, side=FAR):
    hv = Vector((0.3, side * 0.3, 1.05))
    limb(hv, hv + Vector((0.3, 0, 0.08)), 0.02, metal("steel"), r1=0.0, seg=4)
    limb(hv - Vector((0.05, 0, 0)), hv, 0.03, "black", seg=4)
    return hv


def item_cross_pendant(o):
    box((0.18, 0, 1.15), (0.03, 0.04, 0.16), "gold")
    box((0.18, 0, 1.18), (0.03, 0.12, 0.04), "gold")


def item_medic_bag(o, side=NEAR):
    p = Vector((0.0, side * 0.4, 0.85))
    box(p, (0.28, 0.14, 0.24), "white", bevel=0.04)
    box(p + Vector((0, side * 0.075, 0)), (0.2, 0.02, 0.06), "red")
    box(p + Vector((0, side * 0.075, 0)), (0.06, 0.02, 0.2), "red")
    return None


def item_mg(o):
    # tripod machine gun (gunner crouches behind)
    for ang in (0, 120, 240):
        a = math.radians(ang)
        limb((0.55, 0, 0.55), (0.55 + 0.38 * math.cos(a), 0.38 * math.sin(a), 0.0), 0.025, "dkgrey", seg=5)
    cyl((0.55, 0, 0.58), 0.05, 0.16, "dkgrey", seg=6)
    beam((0.15, 0, 0.74), (1.15, 0, 0.78), 0.12, 0.14, "dkgrey")
    limb((0.7, 0, 0.77), (1.45, 0, 0.8), 0.04, metal("dkiron"), seg=7)
    cyl((0.8, 0, 0.77), 0.065, 0.3, metal("steel"), rot=(0, 90, 0), seg=8)
    box((0.35, 0, 0.88), (0.2, 0.05, 0.12), "olive")


def item_lance(o, side=NEAR, length=2.9, pennant="team"):
    hv = Vector((0.32, side * 0.38, 1.0))
    base = Vector((-0.6, side * 0.42, 0.95))
    top = Vector((length - 0.6, side * 0.4, 1.7))
    limb(base, top, 0.04, "ltwood", seg=6)
    dv = (top - base).normalized()
    limb(top, top + dv * 0.3, 0.06, metal("steel"), r1=0.0, seg=5)
    if pennant:
        p = top - dv * 0.1
        cloth(p, p - dv * 0.5 + Vector((0, 0, -0.1)), p + Vector((0, 0, -0.3)) - dv * 0.1, pennant, bulge=0.0, wave=0.1, nu=4, nv=2)
    return hv


def item_saber(o, side=FAR):
    h = Vector((0.3, side * 0.36, 1.25))
    pts = [h + Vector((0.05 + 0.05 * t, side * 0.02 * t, 0.12 + 0.2 * t)) for t in (0, 1, 2, 3, 4)]
    tip_pts = []
    for i in range(5):
        t = i / 4
        tip_pts.append(h + Vector((0.1 + 0.55 * t * t + 0.0, side * 0.04 * t, 0.1 + 0.8 * t)))
    for a, b in zip(tip_pts[:-1], tip_pts[1:]):
        limb(a, b, 0.03, metal("steel"), r1=0.025, seg=4)
    limb(h - Vector((0, 0, 0.1)), h + Vector((0, 0, 0.12)), 0.03, "leather", seg=5)
    box(h + Vector((0, 0, 0.12)), (0.06, 0.2, 0.04), metal("gold"))
    return h


ITEMS = dict(
    lance=item_lance, saber=item_saber,
    spear=item_spear, pike=item_pike, sword=item_sword, club=item_club, axe=item_axe,
    pick=item_pick, hammer=item_hammer, shield=item_shield, bow=item_bow, crossbow=item_crossbow,
    sling=item_sling, musket=item_musket, staff=item_staff, book=item_book, torch=item_torch,
    bag=item_bag, flag=item_flag, binoculars=item_binoculars, shovel=item_shovel,
    rifle_slung=item_rifle_slung, launcher=item_missile_launcher, scroll=item_scroll,
    lute=item_lute, palette=item_palette, quill=item_quill, flask=item_flask, coins=item_coins,
    gear=item_gear, spyglass=item_spyglass, dagger=item_dagger, medic_bag=item_medic_bag,
)


# ------------------------------------------------------------------ human
def boot_(foot, boot):
    """A boot: rounded toe cap, shaft, and a darker sole with a heel."""
    x, y, z = foot
    sph((x + 0.1, y, z - 0.01), 0.075, boot, scale=(1.35, 0.85, 0.7), seg=12, rings=7, name="toe")
    limb((x - 0.01, y, z - 0.03), (x - 0.01, y, z + 0.1), 0.075, boot, r1=0.07, seg=10, name="bootshaft")
    box((x + 0.06, y, z - 0.07), (0.28, 0.13, 0.03), "dkwood", bevel=0.012, name="sole")
    box((x - 0.04, y, z - 0.055), (0.08, 0.12, 0.04), "dkwood", bevel=0.01, name="heel")


def fist(c, skin, side):
    """A closed hand: palm, a row of knuckles, and the thumb."""
    sph(c, 0.07, skin, scale=(1.0, 0.85, 1.0), seg=10, rings=7, name="hand")
    for k in range(4):
        sph(c + Vector((0.045, side * (-0.035 + 0.023 * k), 0.015 - 0.008 * k)), 0.025, skin, seg=6, rings=4,
            name="knuckle")
    sph(c + Vector((0.025, -side * 0.055, 0.03)), 0.026, skin, scale=(1.2, 1, 1), seg=6, rings=4, name="thumb")


def hem(z, rx, ry, m, n=14, depth=0.022):
    """A loose hem: a skirt ring with soft vertical folds."""
    for k in range(n):
        a = 2 * math.pi * k / n
        limb((rx * 0.97 * math.cos(a), ry * 0.97 * math.sin(a), z + 0.01), (rx * 0.9 * math.cos(a), ry * 0.9 * math.sin(a), z + 0.22),
             depth, m, r1=depth * 0.4, seg=6, name="fold")


def _torso(o, kind):
    top = C(o, "top", "undyed")
    legs = C(o, "legs", "brown")
    rings = []
    n = 8

    def rg(z, rx, ry):
        return [(rx * math.cos(2 * math.pi * i / n), ry * math.sin(2 * math.pi * i / n), z) for i in range(n)]

    if kind in ("robe",):
        rings = [rg(0.02, 0.34, 0.36), rg(0.4, 0.27, 0.29), rg(0.75, 0.18, 0.22), rg(1.0, 0.2, 0.26), rg(1.3, 0.17, 0.3), rg(1.42, 0.1, 0.17)]
        ring_loft(rings, top, smooth=True, name="robe")
        return
    if kind in ("coat",):
        rings = [rg(0.42, 0.25, 0.28), rg(0.72, 0.19, 0.24), rg(1.0, 0.2, 0.26), rg(1.3, 0.17, 0.3), rg(1.42, 0.1, 0.17)]
        ring_loft(rings, top, smooth=True, name="coat")
        limb((0.0, -0.1, 0.78), (0.0, -0.1, 0.1), 0.1, legs)
        limb((0.0, 0.1, 0.78), (0.0, 0.1, 0.1), 0.1, legs)
        return
    rings = [rg(0.68, 0.15, 0.2), rg(1.0, 0.2, 0.26), rg(1.3, 0.17, 0.3), rg(1.42, 0.1, 0.17)]
    ring_loft(rings, top, smooth=True, name="torso")


def human(o):
    """Build one humanoid at the origin facing +X from a parameter dict."""
    o = dict(o)
    kind = o.get("body", "tunic")
    s = o.get("scale", 1.0)
    seated = o.get("seated")
    with xf((0, 0, 0), (0, 0, 0), s):
        stride = o.get("stride", 0.08)
        legc = C(o, "legs", "brown")
        boot = o.get("boots", "leather")
        if kind not in ("robe",):
            for sy in (-1, 1):
                if seated:
                    knee = (0.26, sy * 0.3, 0.7)
                    foot = (0.14, sy * 0.34, 0.25)
                else:
                    off = stride * (1 if sy == 1 else -1)
                    knee = (off * 0.5, sy * 0.12, 0.45)
                    foot = (off, sy * 0.12, 0.08)
                if o.get("barelegs"):
                    lc = o.get("skin", "skin")
                else:
                    lc = legc
                limb((0, sy * 0.11, 0.8), knee, 0.105, lc, r1=0.085)
                sph(knee, 0.088, lc, seg=10, rings=7, name="knee")
                limb(knee, foot, 0.085, lc, r1=0.07)
                boot_(foot, boot)
        if kind == "bare":
            _torso(dict(o, top=o.get("skin", "skin")), "tunic")
            # kilt
            cyl((0, 0, 0.62), 0.22, 0.34, C(o, "legs", "undyed"), r2=0.17, seg=10)
        elif kind == "tunic":
            _torso(o, "tunic")
            cyl((0, 0, 0.68), 0.25, 0.3, C(o, "top", "undyed"), r2=0.18, seg=14)
            hem(0.62, 0.25, 0.25, C(o, "top", "undyed"))
        elif kind == "robe":
            _torso(o, "robe")
            hem(0.0, 0.34, 0.36, C(o, "top", "undyed"), n=18, depth=0.024)
            for sy in (-1, 1):
                box((0.1, sy * 0.1, 0.04), (0.28, 0.13, 0.08), boot)
        elif kind == "coat":
            _torso(o, "coat")
        elif kind == "armor":
            _torso(dict(o, top=o.get("under", "undyed")), "tunic")
            cyl((0, 0, 0.66), 0.25, 0.32, o.get("under", "undyed"), r2=0.17, seg=10)
            ring_loft_armor(o)
        elif kind == "plate":
            _torso(dict(o, top=metal(o.get("metal", "steel"))), "tunic")
            cyl((0, 0, 0.66), 0.25, 0.3, metal(o.get("metal", "steel")), r2=0.17, seg=10)
            for sy in (-1, 1):
                sph((0, sy * 0.3, 1.3), 0.12, metal(o.get("metal", "steel")))
        elif kind == "uniform":
            _torso(o, "tunic")
        # belt
        if kind in ("tunic", "armor", "uniform", "coat", "plate", "bare"):
            cyl((0, 0, 0.74), 0.205, 0.07, o.get("belt", "leather"), r2=0.2, seg=14)
            box((0.205, 0, 0.775), (0.02, 0.07, 0.055), metal("bronze"), bevel=0.008, name="buckle")
        # team pieces
        tm = o.get("team", ())
        if "tabard" in tm:
            box((0.19, 0, 0.95), (0.04, 0.34, 0.68), "team", bevel=0.015, name="tabard")
        if "sash" in tm:
            beam((0.1, -0.2, 1.38), (0.1, 0.2, 0.75), 0.07, 0.34, "team", up=(1, 0, 0))
        if "cape" in tm:
            cloth((-0.15, -0.3, 1.38), (-0.15, 0.3, 1.38), (-0.4, -0.34, 0.45), "team", bulge=0.0, axis=(-1, 0, 0), nu=5, nv=4, solid=0.03, wave=0.0)
            cloth((-0.15, -0.3, 1.38), (-0.15, 0.3, 1.38), (-0.4, -0.3, 0.45), "team", bulge=0.1, axis=(-1, 0, 0), nu=5, nv=4)
        if "scarf" in tm:
            torus((0.0, 0, 1.4), 0.17, 0.05, "team", plane="XY", seg=10)
            box((0.1, -0.12, 1.15), (0.06, 0.1, 0.4), "team")
        if "backpack" in tm or o.get("backpack"):
            box((-0.22, 0, 1.05), (0.2, 0.38, 0.45), "team" if "backpack" in tm else o.get("backpack"), bevel=0.04)
            cyl((-0.22, 0, 1.33), 0.14, 0.38, "brown", rot=(90, 0, 0), seg=8)
        if o.get("wings"):
            wings_hussar(o)
        if o.get("quiver"):
            cyl((-0.22, -0.12, 1.15), 0.07, 0.5, "leather", rot=(0, -20, 0), seg=7)
            for k in range(3):
                limb((-0.3, -0.12 + 0.04 * (k - 1), 1.4), (-0.36, -0.12 + 0.04 * (k - 1), 1.62), 0.012, "white", seg=4)
        if o.get("aura"):
            torus((0, 0, 0.03), 0.55, 0.035, "gold", plane="XY", seg=20, tseg=5)
        if o.get("halo"):
            torus((-0.12, 0, 1.58 + 0.42), 0.2, 0.025, "gold", plane="XY", seg=16, tseg=5)
        # items (collect hand targets)
        hands = {NEAR: None, FAR: None}
        for it in o.get("items", ()):
            name, kw = (it, {}) if isinstance(it, str) else (it[0], dict(it[1]))
            fn = ITEMS[name]
            kw.setdefault("side", inspect.signature(fn).parameters["side"].default)
            r = fn(o, **kw)
            if isinstance(r, dict):
                hands.update(r)
            elif r is not None:
                hands[kw["side"]] = r
        hands.update(o.get("hands", {}))
        # arms
        sc = o.get("sleeve", C(o, "top", "undyed"))
        for sy in (-1, 1):
            sh = Vector((0.0, sy * 0.3, 1.27))
            tgt = hands[sy]
            if tgt is None:
                tgt = Vector((0.08, sy * 0.36, 0.78))
            arm(sh, tgt, sy, sc, o.get("skin", "skin"), o.get("barearms", False))
        head(o, 1.58)
    return o


def ring_loft_armor(o):
    n = 8
    rings = []
    for z, rx, ry in ((0.82, 0.18, 0.23), (1.0, 0.23, 0.29), (1.3, 0.2, 0.33), (1.42, 0.12, 0.2)):
        rings.append([(rx * math.cos(2 * math.pi * i / n), ry * math.sin(2 * math.pi * i / n), z) for i in range(n)])
    ring_loft(rings, metal(o.get("metal", "iron")), smooth=True, name="cuirass")
    for sy in (-1, 1):
        sph((0, sy * 0.31, 1.3), 0.12, metal(o.get("metal", "iron")))


def wings_hussar(o):
    # Polish winged hussar: feathered wing frame on the back
    for sy in (-1, 1):
        limb((-0.2, sy * 0.08, 1.2), (-0.35, sy * 0.1, 2.35), 0.025, "ltwood", seg=5)
        for i in range(8):
            t = i / 7
            base = Vector((-0.28, sy * (0.1 + 0.05 * t), 1.25 + 1.0 * t))
            tip = base + Vector((-0.2, sy * 0.55, 0.45 + 0.3 * t))
            limb(base, tip, 0.04, "white" if i % 2 == 0 else "ivory", r1=0.015, seg=5)


def arm(sh, hand, side, sleeve, skin, bare):
    L1 = L2 = 0.34
    dv = hand - sh
    dist = min(dv.length, (L1 + L2) * 0.985)
    dn = dv.normalized()
    hand_c = sh + dn * dist
    pole = Vector((-0.25, side * 1.0, -0.6))
    pole = pole - dn * pole.dot(dn)
    if pole.length < 1e-4:
        pole = Vector((0, side, 0))
    pole.normalize()
    hgt = math.sqrt(max(L1 * L1 - (dist / 2) ** 2, 0.0))
    elbow = sh + dn * (dist / 2) + pole * hgt
    sph(sh, 0.115, sleeve, name="shoulder")
    limb(sh, elbow, 0.092, sleeve if not bare else skin, r1=0.08)
    limb(elbow, hand_c, 0.08, sleeve if not bare else skin, r1=0.07)
    if not bare:
        limb(elbow + (hand_c - elbow) * 0.78, hand_c, 0.075, skin, r1=0.065)
    fist(hand_c, skin, side)
