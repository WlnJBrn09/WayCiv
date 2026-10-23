"""Siege engines, guns and armoured vehicles."""
import math
from mathutils import Vector
from lib import *
from figures import human, NEAR, FAR
from mounts import wheel


def _flag(p, h=0.8, c="team", L=0.5):
    limb(p, (p[0], p[1], p[2] + h), 0.025, "ltwood", seg=5)
    cloth((p[0], p[1], p[2] + h), (p[0] - L, p[1], p[2] + h - 0.03), (p[0], p[1], p[2] + h - 0.4), c, bulge=0.0, wave=0.08, nu=5, nv=3, solid=0.02)


def catapult(o=None):
    o = o or {}
    # base frame and wheels
    for sy in (-1, 1):
        beam((-0.9, sy * 0.42, 0.45), (0.95, sy * 0.42, 0.45), 0.14, 0.14, "wood")
        wheel((-0.55, sy * 0.58, 0.32), 0.32, spokes=6, w=0.07)
        wheel((0.55, sy * 0.58, 0.32), 0.32, spokes=6, w=0.07)
    limb((-0.55, -0.6, 0.32), (-0.55, 0.6, 0.32), 0.04, "dkwood", seg=5)
    limb((0.55, -0.6, 0.32), (0.55, 0.6, 0.32), 0.04, "dkwood", seg=5)
    beam((-0.9, -0.42, 0.45), (-0.9, 0.42, 0.45), 0.12, 0.12, "wood")
    beam((0.95, -0.42, 0.45), (0.95, 0.42, 0.45), 0.12, 0.12, "wood")
    # uprights + torsion skeins
    for sy in (-1, 1):
        beam((0.1, sy * 0.4, 0.5), (0.0, sy * 0.4, 1.35), 0.13, 0.13, "ltwood")
        beam((-0.45, sy * 0.4, 0.5), (-0.1, sy * 0.4, 1.0), 0.1, 0.1, "wood")
    beam((0.05, -0.4, 1.28), (0.05, 0.4, 1.28), 0.12, 0.12, "ltwood")
    cyl((0.0, 0, 0.78), 0.14, 0.62, "team", rot=(90, 0, 0), seg=10, name="skein")
    cyl((0.0, 0, 0.78), 0.17, 0.08, "dkiron", rot=(90, 0, 0), seg=8)
    # arm with bucket
    limb((0.0, 0, 0.78), (-0.95, 0, 1.45), 0.075, "ltwood", r1=0.06, seg=6)
    cyl((-1.02, 0, 1.52), 0.17, 0.14, "dkwood", r2=0.2, seg=8)
    sph((-1.02, 0, 1.62), 0.13, "stone", seg=7, rings=5)
    # windlass
    cyl((0.9, 0, 0.62), 0.07, 0.8, "dkwood", rot=(90, 0, 0), seg=6)
    _flag((0.7, 0.4, 1.3), h=0.7)
    for k in range(3):
        sph((0.6 + 0.16 * k, -0.2 + 0.12 * k, 0.6), 0.1, "stone", seg=6, rings=4)


def trebuchet(o=None):
    for sy in (-1, 1):
        beam((-0.8, sy * 0.5, 0.18), (0.8, sy * 0.5, 0.18), 0.16, 0.16, "wood")
        # A-frame legs
        beam((-0.45, sy * 0.5, 0.2), (0.0, sy * 0.15, 2.0), 0.14, 0.14, "ltwood")
        beam((0.45, sy * 0.5, 0.2), (0.0, sy * 0.15, 2.0), 0.14, 0.14, "ltwood")
    beam((-0.8, -0.5, 0.18), (-0.8, 0.5, 0.18), 0.14, 0.14, "wood")
    beam((0.8, -0.5, 0.18), (0.8, 0.5, 0.18), 0.14, 0.14, "wood")
    limb((0.0, -0.3, 2.0), (0.0, 0.3, 2.0), 0.06, "dkiron", seg=6)
    # throwing arm: short end with counterweight, long end with sling
    limb((0.5, 0, 1.2), (0.0, 0, 2.0), 0.08, "wood", seg=6)
    limb((0.0, 0, 2.0), (-1.7, 0, 0.55), 0.06, "ltwood", r1=0.04, seg=6)
    with xf((0.5, 0, 1.25)):
        box((0, 0, -0.18), (0.45, 0.45, 0.45), "team", bevel=0.04, name="counterweight")
        box((0, 0, 0.04), (0.5, 0.5, 0.06), "dkiron")
    limb((-1.7, 0, 0.55), (-1.75, 0, 0.12), 0.012, "rope", seg=4)
    sph((-1.78, 0, 0.1), 0.1, "stone", seg=7, rings=5)
    for k in range(3):
        sph((-1.3 - 0.15 * k, 0.5, 0.15), 0.1, "stone", seg=6, rings=4)
    _flag((0.0, 0.2, 2.0), h=0.45, L=0.4)


def battering_ram(o=None):
    for sx in (-0.6, 0.6):
        for sy in (-1, 1):
            wheel((sx, sy * 0.55, 0.3), 0.3, spokes=6, w=0.07)
    limb((-0.6, -0.6, 0.3), (-0.6, 0.6, 0.3), 0.04, "dkwood", seg=5)
    limb((0.6, -0.6, 0.3), (0.6, 0.6, 0.3), 0.04, "dkwood", seg=5)
    # frame posts and roof
    for sx in (-0.95, 0.95):
        for sy in (-0.5, 0.5):
            limb((sx, sy, 0.4), (sx, sy, 1.35), 0.07, "wood", seg=6)
    box((0, 0, 0.42), (2.1, 1.2, 0.1), "dkwood", bevel=0.03)
    # gabled hide roof
    for sy in (-1, 1):
        beam((-1.05, sy * 0.62, 1.2), (1.05, sy * 0.62, 1.2), 0.1, 0.1, "wood")
        box((0, sy * 0.35, 1.52), (2.2, 0.8, 0.08), "team", rot=(sy * -28, 0, 0), bevel=0.03, name="roof")
    beam((-1.1, 0, 1.55), (1.1, 0, 1.55), 0.1, 0.1, "dkwood")
    # the ram
    limb((-1.15, 0, 0.85), (1.3, 0, 0.85), 0.13, "dkwood", seg=8)
    limb((1.3, 0, 0.85), (1.52, 0, 0.85), 0.17, metal("iron"), r1=0.12, seg=8)
    limb((-0.4, 0, 1.45), (-0.4, 0, 0.9), 0.015, "rope", seg=4)
    limb((0.6, 0, 1.45), (0.6, 0, 0.9), 0.015, "rope", seg=4)


def siege_tower(o=None):
    for sx in (-0.5, 0.5):
        for sy in (-1, 1):
            wheel((sx, sy * 0.68, 0.3), 0.3, spokes=6, w=0.07)
    box((0, 0, 0.45), (1.6, 1.3, 0.15), "dkwood", bevel=0.03)
    # tower body
    box((0, 0, 1.55), (1.3, 1.1, 2.0), "ltwood", bevel=0.03, name="tower")
    for sy in (-1, 1):
        box((0.0, sy * 0.57, 1.3), (1.32, 0.05, 1.4), "team", bevel=0.02, name="hide")
    box((0.67, 0, 1.3), (0.05, 1.1, 1.4), "team", bevel=0.02)
    for z in (0.6, 1.1, 1.6, 2.1):
        box((0, 0, z), (1.36, 1.16, 0.06), "dkwood", bevel=0.02)
    for sx in (-0.66, 0.66):
        for sy in (-0.56, 0.56):
            limb((sx, sy, 0.5), (sx, sy, 2.6), 0.05, "wood", seg=6)
    # battlements
    for i in range(5):
        box((-0.55 + 0.275 * i, -0.56, 2.75), (0.15, 0.08, 0.2), "ltwood")
        box((-0.55 + 0.275 * i, 0.56, 2.75), (0.15, 0.08, 0.2), "ltwood")
    # drawbridge ramp on the front
    box((1.1, 0, 2.1), (1.0, 0.9, 0.06), "wood", rot=(0, -50, 0), bevel=0.02)
    _flag((-0.4, 0.3, 2.6), h=0.8)


def _cannon_barrel(p0, p1, r0, r1, m="dkiron", band="gold"):
    limb(p0, p1, r0, metal(m), r1=r1, seg=10)
    sph(p0, r0 * 1.15, metal(m), seg=8, rings=5)
    dv = (Vector(p1) - Vector(p0)).normalized()
    limb(Vector(p1) - dv * 0.12, Vector(p1) + dv * 0.05, r1 * 1.25, metal(m), seg=10)


def bombard(o=None):
    for sy in (-1, 1):
        wheel((0.0, sy * 0.58, 0.4), 0.4, spokes=8, w=0.09)
    limb((0.0, -0.62, 0.4), (0.0, 0.62, 0.4), 0.05, "dkwood", seg=6)
    beam((-0.9, 0, 0.62), (0.9, 0, 0.62), 0.5, 0.2, "wood", bevel=0.05, name="bed")
    box((-0.9, 0, 0.5), (0.2, 0.5, 0.22), "dkwood")
    _cannon_barrel((-0.55, 0, 0.98), (1.4, 0, 1.18), 0.26, 0.2, m="dkiron")
    for k in range(3):
        torus((-0.2 + 0.55 * k, 0, 1.06 + 0.1 * k * 0.4), 0.24 - 0.02 * k, 0.03, metal("dkiron"), plane="YZ", seg=10, tseg=4)
    for k in range(4):
        sph((-0.5 + 0.18 * k, 0.55, 0.12 + (0.1 if k == 3 else 0)), 0.09, "dkgrey", seg=6, rings=4)
    _flag((-0.8, 0.0, 0.7), h=0.9, L=0.5)


def field_cannon(o=None):
    for sy in (-1, 1):
        wheel((0.2, sy * 0.52, 0.45), 0.45, spokes=12, w=0.08, m="ltwood")
    limb((0.2, -0.55, 0.45), (0.2, 0.55, 0.45), 0.05, "dkwood", seg=6)
    for sy in (-1, 1):
        beam((-0.2, sy * 0.2, 0.62), (-1.35, sy * 0.15, 0.18), 0.1, 0.1, "wood")
    beam((-0.15, 0, 0.62), (1.1, 0, 0.78), 0.45, 0.14, "ltwood", name="carriage")
    _cannon_barrel((-0.4, 0, 0.86), (1.6, 0, 1.0), 0.19, 0.12, m="#6b5a3a")
    box((-0.6, 0.0, 0.95), (0.06, 0.1, 0.08), metal("dkiron"))
    for k in range(3):
        sph((-0.2 + 0.18 * k, -0.48, 0.14), 0.09, "dkgrey", seg=6, rings=4)
    with xf((-1.2, 0.2, 0), (0, 0, 20), 0.85):
        human(dict(body="coat", top="team", team=("top",), legs="white", helmet="shako", hat_c="black", boots="black",
                   items=[("staff", {"side": FAR})]))


def track(x0, x1, y, r, w, col="dkgrey", z=None):
    z = r if z is None else z
    cyl((x0, y, z), r, w, col, rot=(90, 0, 0), seg=12)
    cyl((x1, y, z), r, w, col, rot=(90, 0, 0), seg=12)
    box(((x0 + x1) / 2, y, z), (x1 - x0, w, 2 * r), col, bevel=0.03)
    # road wheels
    n = max(3, int((x1 - x0) / (r * 1.5)))
    for i in range(n + 1):
        x = x0 + (x1 - x0) * i / n
        sgn = 1 if y > 0 else -1
        cyl((x, y + sgn * w * 0.5, z - r * 0.15), r * 0.62, 0.03, "#585d60", rot=(90, 0, 0), seg=10)
        cyl((x, y + sgn * w * 0.5, z - r * 0.15), r * 0.22, 0.05, "dkiron", rot=(90, 0, 0), seg=6)


def tank(o=None):
    hull = "#6a6e48"
    for sy in (-1, 1):
        track(-1.1, 1.1, sy * 0.75, 0.3, 0.3)
    poly([(-1.35, 0.3), (1.1, 0.3), (1.55, 0.6), (1.1, 0.85), (-1.35, 0.85)], hull, plane="XZ", thick=1.25, bevel=0.03, name="hull")
    for sy in (-1, 1):
        box((0.0, sy * 0.78, 0.68), (2.4, 0.06, 0.1), hull)
    box((-1.1, 0, 0.9), (0.5, 1.0, 0.1), "dkgrey")
    # turret
    with xf((-0.05, 0, 0.85)):
        cyl((0, 0, 0.2), 0.58, 0.4, hull, r2=0.5, seg=12, name="turret")
        cyl((0, 0, 0.05), 0.6, 0.12, "team", seg=12, name="turretband")
        box((0.5, 0, 0.22), (0.3, 0.55, 0.32), hull, bevel=0.05, name="mantlet")
        limb((0.5, 0, 0.25), (2.05, 0, 0.3), 0.075, metal("dkiron"), seg=8)
        limb((1.7, 0, 0.3), (1.95, 0, 0.3), 0.1, metal("dkiron"), seg=8)
        cyl((-0.15, 0.2, 0.46), 0.15, 0.1, "dkgrey", seg=8)
        limb((-0.55, -0.2, 0.4), (-0.55, -0.2, 1.1), 0.012, "dkgrey", seg=4)
        _flag((-0.55, -0.2, 1.0), h=0.0001, L=0.0001)
    cloth((-0.6, -0.2, 1.95), (-1.0, -0.2, 1.9), (-0.6, -0.2, 1.65), "team", bulge=0, wave=0.05, nu=4, nv=3)


def modern_armor(o=None):
    hull = "#7a7a52"
    for sy in (-1, 1):
        track(-1.2, 1.2, sy * 0.78, 0.27, 0.32, z=0.27)
        box((0.0, sy * 0.78, 0.5), (2.9, 0.1, 0.25), "#68683f", bevel=0.04, name="skirt")
    poly([(-1.5, 0.3), (1.2, 0.3), (1.7, 0.55), (1.2, 0.78), (-1.5, 0.78)], hull, plane="XZ", thick=1.3, bevel=0.03, name="hull")
    with xf((-0.1, 0, 0.78)):
        poly([(-0.7, 0.0), (0.5, 0.0), (0.85, 0.15), (0.85, 0.3), (0.5, 0.45), (-0.7, 0.45), (-0.8, 0.2)], hull, plane="XZ", thick=0.95, bevel=0.04, name="turret")
        for sy in (-1, 1):
            box((0.1, sy * 0.5, 0.25), (0.7, 0.05, 0.28), "team", bevel=0.02, name="armorpanel")
        limb((0.8, 0, 0.3), (2.7, 0, 0.32), 0.075, metal("dkiron"), seg=8)
        limb((1.5, 0, 0.31), (1.9, 0, 0.31), 0.1, metal("dkiron"), seg=8)
        cyl((-0.35, -0.25, 0.52), 0.14, 0.12, "dkgrey", seg=8)
        box((-0.2, 0.25, 0.5), (0.3, 0.2, 0.1), "dkgrey")
        limb((-0.75, 0.3, 0.3), (-0.75, 0.3, 1.2), 0.012, "dkgrey", seg=4)


def ifv(o=None):
    hull = "#6f7a52"
    for sy in (-1, 1):
        for sx in (-0.9, -0.3, 0.3, 0.9):
            cyl((sx, sy * 0.8, 0.34), 0.34, 0.26, "dkgrey", rot=(90, 0, 0), seg=10)
            cyl((sx, sy * 0.8 + sy * 0.13, 0.34), 0.16, 0.05, "dkiron", rot=(90, 0, 0), seg=8)
    poly([(-1.5, 0.35), (1.3, 0.35), (1.55, 0.6), (1.3, 0.95), (-1.5, 0.95)], hull, plane="XZ", thick=1.35, bevel=0.05, name="hull")
    box((-0.3, 0, 1.15), (1.5, 1.1, 0.4), hull, bevel=0.06, name="cabin")
    box((-0.3, 0, 1.2), (1.52, 1.12, 0.1), "team", bevel=0.03, name="stripe")
    with xf((0.6, 0, 1.1)):
        cyl((0, 0, 0.12), 0.38, 0.28, hull, r2=0.32, seg=10)
        limb((0.1, 0, 0.14), (1.1, 0, 0.14), 0.05, metal("dkiron"), seg=7)
        box((0.35, 0.3, 0.18), (0.25, 0.1, 0.15), "dkgrey")
    for sy in (-1, 1):
        box((-0.3, sy * 0.58, 0.9), (1.2, 0.04, 0.2), "dkgrey")


def truck_base(L=2.9, W=1.3, cab="#6a7050"):
    for sx in (-1.0, -0.2, 1.0):
        for sy in (-1, 1):
            cyl((sx, sy * 0.72, 0.36), 0.36, 0.25, "#222222", rot=(90, 0, 0), seg=10)
            cyl((sx, sy * 0.72 + sy * 0.13, 0.36), 0.18, 0.05, "dkiron", rot=(90, 0, 0), seg=8)
    box((0.0, 0, 0.66), (L, W, 0.22), "dkgrey", bevel=0.04, name="chassis")
    box((1.15, 0, 1.12), (0.7, 1.2, 0.7), cab, bevel=0.06, name="cab")
    box((1.5, 0, 1.2), (0.06, 0.95, 0.35), "glass", bevel=0.02)
    box((1.5, 0, 0.82), (0.2, 1.2, 0.3), cab, bevel=0.05)
    box((1.17, 0, 1.52), (0.7, 1.2, 0.06), "team", bevel=0.02, name="roofstripe")


def rocket_artillery(o=None):
    truck_base()
    with xf((-0.55, 0, 0.8), (0, 0, 0)):
        box((0.0, 0, 0.1), (1.4, 1.1, 0.12), "dkgrey")
        # launcher box tilted up
        with xf((0, 0, 0.2), (0, -28, 0)):
            box((0.1, 0, 0.35), (1.7, 1.0, 0.7), "olive", bevel=0.05, name="pod")
            for iy in range(3):
                for iz in range(2):
                    cyl((0.97, -0.28 + 0.28 * iy, 0.2 + 0.3 * iz), 0.1, 0.06, "black", rot=(0, 90, 0), seg=8)
            box((0.1, 0, 0.75), (1.6, 0.96, 0.08), "team", bevel=0.02, name="pod_stripe")


def mobile_sam(o=None):
    truck_base(cab="#6a7050")
    with xf((-0.5, 0, 0.8)):
        box((0.0, 0, 0.1), (1.4, 1.1, 0.12), "dkgrey")
        with xf((0, 0, 0.25), (0, -35, 0)):
            for iy in (-1, 1):
                cyl((0.2, iy * 0.32, 0.38), 0.17, 1.9, "#d8d8cc", rot=(0, 90, 0), seg=10, name="canister")
                cyl((1.17, iy * 0.32, 0.38), 0.14, 0.05, "team", rot=(0, 90, 0), seg=10)
                cyl((-0.7, iy * 0.32, 0.38), 0.17, 0.05, "dkgrey", rot=(0, 90, 0), seg=10)
            box((0.2, 0, 0.22), (1.5, 0.9, 0.12), "olive")
        cyl((-0.1, 0.0, 0.2), 0.2, 0.1, "dkgrey", seg=8)


def antiair_gun(o=None):
    for a in (0, 90, 180, 270):
        r = math.radians(a + 45)
        beam((0, 0, 0.35), (0.95 * math.cos(r), 0.95 * math.sin(r), 0.12), 0.13, 0.1, "dkgrey")
        cyl((1.0 * math.cos(r), 1.0 * math.sin(r), 0.04), 0.14, 0.08, "dkiron", seg=8)
    cyl((0, 0, 0.55), 0.3, 0.45, "olive", seg=10, name="pedestal")
    with xf((0, 0, 0.9), (0, 0, 0)):
        box((0, 0, 0.1), (0.8, 0.7, 0.3), "olive", bevel=0.05, name="mount")
        box((-0.1, 0, 0.0), (0.6, 0.15, 0.1), "team")
        for sy in (-0.2, 0.2):
            for sz in (0.1, 0.35):
                limb((0.1, sy, sz + 0.1), (1.5, sy, sz + 0.55), 0.045, metal("dkiron"), seg=7)
                limb((1.2, sy, sz + 0.44), (1.5, sy, sz + 0.55), 0.06, metal("dkiron"), seg=7)
        box((-0.5, 0.0, 0.3), (0.3, 0.5, 0.35), "olive", bevel=0.04)
        box((-0.6, 0, 0.1), (0.4, 0.5, 0.06), "dkgrey")


def at_gun(o=None):
    for sy in (-1, 1):
        wheel((0.0, sy * 0.52, 0.34), 0.34, spokes=6, w=0.14, m="#2a2a2a", hub="dkiron")
    for sy in (-1, 1):
        beam((-0.1, sy * 0.15, 0.55), (-1.4, sy * 0.35, 0.1), 0.1, 0.1, "olive")
    box((0.0, 0, 0.55), (0.6, 0.5, 0.2), "olive", bevel=0.04)
    limb((-0.3, 0, 0.7), (1.7, 0, 0.82), 0.08, metal("dkiron"), r1=0.06, seg=8)
    box((0.7, 0, 0.82), (0.06, 0.95, 0.6), "olive", bevel=0.03, name="shield")
    box((0.74, 0, 0.82), (0.04, 0.7, 0.3), "team", bevel=0.02, name="shieldpaint")
    for k in range(2):
        cyl((-0.5 + 0.15 * k, 0.75, 0.1), 0.06, 0.3, "#b8a060", rot=(0, 80, 0), seg=6)
    with xf((-0.9, 0.35, 0), (0, 0, 15), 0.85):
        human(dict(body="uniform", top="#6a6a40", legs="#5a5a3a", helmet="modern", hat_c="olive", team=("scarf",), boots="black",
                   items=[("rifle_slung", {})]))


def artillery(o=None):
    for sy in (-1, 1):
        wheel((0.2, sy * 0.62, 0.38), 0.38, spokes=6, w=0.15, m="#2a2a2a", hub="dkiron")
        beam((-0.2, sy * 0.2, 0.62), (-1.7, sy * 0.55, 0.08), 0.1, 0.12, "olive")
        cyl((-1.75, sy * 0.57, 0.04), 0.1, 0.06, "dkiron", seg=8)
    box((0.1, 0, 0.7), (0.7, 0.9, 0.22), "olive", bevel=0.05, name="cradle")
    limb((-0.5, 0, 0.92), (2.0, 0, 1.55), 0.1, metal("dkiron"), r1=0.075, seg=9)
    cyl((-0.25, 0, 0.82), 0.2, 0.38, "olive", rot=(0, 90 - 20, 0), seg=9)
    limb((1.8, 0, 1.5), (2.1, 0, 1.6), 0.13, metal("dkiron"), seg=9)
    box((0.7, 0, 1.0), (0.06, 1.0, 0.55), "olive", rot=(0, -12, 0), bevel=0.03, name="shield")
    box((0.71, 0, 1.05), (0.05, 0.75, 0.25), "team", rot=(0, -12, 0), bevel=0.02, name="shieldpaint")
    with xf((-1.0, 0.55, 0), (0, 0, 20), 0.85):
        human(dict(body="uniform", top="#6a6a40", legs="#5a5a3a", helmet="modern", hat_c="olive", team=("scarf",), boots="black", items=[]))


def helicopter(o=None):
    with xf((0, 0, 0.9)):
        sph((0.1, 0, 0.5), 0.62, "#5a6050", scale=(1.25, 0.62, 0.7), seg=12, rings=7, name="body")
        sph((0.62, 0, 0.55), 0.4, "glass", scale=(1.0, 0.9, 0.85), seg=10, rings=6, name="canopy")
        limb((-0.5, 0, 0.58), (-2.1, 0, 0.8), 0.22, "#5a6050", r1=0.07, seg=8, name="boom")
        poly([(-1.8, 0.78), (-2.2, 0.78), (-2.3, 1.3), (-2.0, 1.3)], "team", plane="XZ", thick=0.05, name="fin")
        box((-2.15, 0.0, 0.72), (0.4, 0.8, 0.05), "#5a6050", bevel=0.01)
        cyl((0.0, 0, 1.0), 0.07, 0.2, "dkgrey", seg=6)
        for k in range(4):
            a = math.radians(k * 90 + 25)
            beam((0, 0, 1.12), (1.9 * math.cos(a), 1.9 * math.sin(a), 1.12), 0.12, 0.03, "dkgrey")
        for sy in (-1, 1):
            limb((0.5, sy * 0.4, 0.0), (-0.45, sy * 0.4, 0.0), 0.04, "dkgrey", seg=6)
            limb((0.3, sy * 0.4, 0.0), (0.25, sy * 0.3, 0.35), 0.03, "dkgrey", seg=5)
            limb((-0.3, sy * 0.4, 0.0), (-0.25, sy * 0.3, 0.35), 0.03, "dkgrey", seg=5)
        for sy in (-1, 1):
            cyl((0.15, sy * 0.45, 0.4), 0.09, 0.8, "dkgrey", rot=(0, 90, 0), seg=7)
        cyl((-0.2, 0, 0.5), 0.38, 0.05, "team", seg=12, rot=(90, 0, 0))
        # tail rotor
        cyl((-2.2, 0.1, 1.05), 0.2, 0.03, "dkgrey", rot=(90, 0, 0), seg=10)


def balloon(o=None):
    # basket
    box((0, 0, 0.35), (0.6, 0.6, 0.4), "ltwood", bevel=0.05, name="basket")
    for sx in (-0.3, 0.3):
        for sy in (-0.3, 0.3):
            limb((sx, sy, 0.5), (sx * 1.7, sy * 1.7, 2.3), 0.012, "rope", seg=4)
    # envelope: lofted gores in alternating colours
    n = 12
    prof = [(0.0, 2.1), (0.5, 2.3), (0.85, 2.9), (0.95, 3.5), (0.8, 4.1), (0.45, 4.6), (0.0, 4.75)]
    rings = []
    for (r, z) in prof:
        rings.append([(r * math.cos(2 * math.pi * i / n), r * math.sin(2 * math.pi * i / n), z) for i in range(n)])
    # alternating bands via two lofts (cheap): the full envelope team-coloured, white stripes on top
    ring_loft(rings, "team", smooth=True, name="envelope")
    for k in range(0, n, 2):
        a0, a1 = 2 * math.pi * k / n, 2 * math.pi * (k + 1) / n
        rr = []
        for (r, z) in prof[1:-1]:
            rr.append([(r * 1.01 * math.cos(a0), r * 1.01 * math.sin(a0), z), (r * 1.01 * math.cos(a1), r * 1.01 * math.sin(a1), z)])
        # thin white stripe from consecutive pairs
        for i in range(len(rr) - 1):
            pts = [rr[i][0], rr[i][1], rr[i + 1][1], rr[i + 1][0]]
            _quad(pts, "white")
    # tether rope & burner
    limb((0, 0, 0.2), (-0.6, 0.3, 0.0), 0.02, "rope", seg=4)
    cyl((0, 0, 0.7), 0.06, 0.2, "dkiron", seg=6)
    sph((0, 0, 0.88), 0.07, "#ffb030", scale=(1, 1, 1.6), seg=6, rings=4)


def _quad(pts, m):
    from lib import _mk
    verts = [W(p) for p in pts]
    _mk("quad", verts, [(0, 1, 2, 3)], mat(m) if isinstance(m, str) else m, recalc=False)


def strike_drone_single(scale=1.0):
    """Sleek flying-wing loitering munition: dark body, swept wings, team tail stripe."""
    with xf((0, 0, 0), (0, 0, 0), scale):
        tube([(-0.9, 0.02, 0.02, 0), (-0.5, 0.1, 0.09, 0), (0.2, 0.16, 0.12, 0), (0.7, 0.1, 0.09, 0), (1.1, 0.03, 0.03, -0.01)], "#2c3036", n=10, name="fuselage")
        poly([(0.35, 0.1), (-0.6, 0.12), (-0.95, 1.2), (-0.7, 1.28), (0.15, 0.5)], "#3a3f46", plane="XY", thick=0.04, name="wingL", bevel=0.01)
        poly([(0.35, -0.1), (-0.6, -0.12), (-0.95, -1.2), (-0.7, -1.28), (0.15, -0.5)], "#3a3f46", plane="XY", thick=0.04, name="wingR", bevel=0.01)
        for sy in (-1, 1):
            poly([(-0.55, 0.13 * sy), (-0.85, 0.5 * sy), (-0.8, 0.52 * sy), (-0.5, 0.15 * sy)], "team", plane="XY", offset=0.025, thick=0.02, name="stripe")
            poly([(-0.78, 1.2 * sy), (-0.95, 1.2 * sy), (-0.98, 1.3 * sy), (-0.8, 1.28 * sy)], "#4a4f56", plane="XY", offset=0.02, thick=0.1)
        poly([(-0.6, 0.0), (-1.0, 0.0), (-1.05, 0.02), (-0.62, 0.02)], "#2c3036", plane="XZ", thick=0.05)
        sph((0.95, 0, 0.0), 0.06, "#8cc8ff", scale=(1.2, 1, 1), seg=6, rings=4)
        cyl((-0.92, 0, 0.0), 0.06, 0.12, "#101214", rot=(0, 90, 0), seg=8)


def strike_drones(o=None):
    for (x, y, z) in ((0.5, 0.0, 0.0), (-0.45, -0.95, 0.25), (-0.45, 0.95, 0.25)):
        with xf((x, y, 0.9 + z)):
            strike_drone_single(0.55)
