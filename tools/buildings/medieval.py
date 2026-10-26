"""Medieval city centres (era 1): timber-frame houses, churches, castles."""
import math
import random

import bl
from bl import _face
from town import *  # noqa: F401,F403
from town import ACC

PLASTER = ['cream', 'cream', 'sand', 'rose', 'white']
ROOFS = ['terra', 'terra', 'terraD', 'slate', 'thatch']


def timber_house(x, y, g, rnd, rot=0.0, big=False, roof=None, acc_roof=False):
    w = rnd.choice([0.17, 0.19, 0.21]) * (1.1 if big else 1.0)
    d = rnd.choice([0.14, 0.16]) * (1.1 if big else 1.0)
    h1, h2 = (0.1, 0.09) if not big else (0.12, 0.11)
    wall = rnd.choice(PLASTER)
    rf = ACC if acc_roof else (roof or rnd.choice(ROOFS))
    with at(x, y, g, rot):
        box(0, 0, 0, w + 0.004, d + 0.004, 0.02, M('stoneD'))
        box(0, 0, 0.02, w, d, h1 - 0.02, M(wall))
        box(0, -0.006, h1, w + 0.02, d + 0.02, h2, M(wall))           # jettied upper floor
        for (yy, hh, zz) in ((-d / 2 - 0.005, h1 - 0.02, 0.02), (-d / 2 - 0.016, h2, h1)):
            for k in range(4):
                box(-w / 2 + 0.01 + k * (w - 0.02) / 3, yy, zz, 0.011, 0.006, hh, M('woodD'))
            box(0, yy, zz + hh * 0.5, w, 0.006, 0.008, M('woodD'))
        strip((-w / 2, h1), (-w / 6, h1 + h2 * 0.95), 0.008, M('woodD'), y=-d / 2 - 0.0185)
        strip((w / 2, h1), (w / 6, h1 + h2 * 0.95), 0.008, M('woodD'), y=-d / 2 - 0.0185)
        windows(0, 0, 0.0, w, d - 0.012, h1, 2, 1, sides='f', ww=0.032, wh=0.04, z0=0.5, z1=0.5, frame=M('woodD'))
        windows(0, -0.012, h1, w + 0.02, d + 0.02, h2, 2, 1, sides='f', ww=0.032, wh=0.04, z0=0.5, z1=0.5, frame=M('woodD'))
        door(0, 0, 0, w, d, 0.04, 0.07)
        gable(0, -0.004, h1 + h2, w + 0.02, d + 0.03, 0.115 if not big else 0.14, M(rf), overhang=0.016, mat_end=M(wall))
        if rf != ACC and rnd.random() < 0.7:
            chimney(w * 0.28, 0.02, h1 + h2 + 0.02, 0.012, 0.09, smoke=1 if rnd.random() < 0.4 else 0, mat='stone2')


def stone_house(x, y, g, rnd, rot=0.0):
    w, d, h = 0.2, 0.15, 0.16
    with at(x, y, g, rot):
        box(0, 0, 0, w, d, h, M('stone'))
        windows(0, 0, 0, w, d, h, 2, 2, ww=0.03, wh=0.04, z0=0.3, z1=0.75)
        door(0, 0, 0, w, d, 0.04, 0.07)
        hip(0, 0, h, w + 0.03, d + 0.03, 0.09, M('slate'), ridge=0.5)
        chimney(0.05, 0.0, h + 0.04, 0.013, 0.07, smoke=0, mat='stone2')


def church(x, y, g, rot=0.0, s=1.0, acc_spire=False):
    with at(x, y, g, rot, s):
        box(0, 0.03, 0, 0.17, 0.34, 0.14, M('stone'))
        gable(0, 0.03, 0.14, 0.34, 0.17, 0.1, M('slate'), rot=90, mat_end=M('stone'))
        cyl(0, 0.24, 0, 0.065, 0.1, M('stone'), n=10)                       # apse
        cone(0, 0.24, 0.1, 0.075, 0.07, M('slate'), n=10)
        box(0, -0.17, 0, 0.1, 0.1, 0.3, M('stone'))                         # tower
        box(0, -0.17, 0.3, 0.115, 0.115, 0.015, M('stone2'))
        pyr(0, -0.17, 0.315, 0.1, 0.1, 0.2, M(ACC if acc_spire else 'slate'))
        cyl(0, -0.17, 0.5, 0.003, 0.05, M('gold'), n=4)
        box(0, -0.17 - 0.052, 0.2, 0.04, 0.004, 0.06, M('glassD'))
        for sx in (-1, 1):
            box(sx * 0.0865, 0.03, 0.04, 0.004, 0.2, 0.07, M('glassD'))


def windmill(x, y, g):
    with at(x, y, g):
        cyl(0, 0, 0, 0.065, 0.2, M('cream'), n=10, r2=0.045)
        cone(0, 0, 0.2, 0.062, 0.07, M('terraD'), n=10)
        with at(0, -0.06, 0.21):
            for k in range(4):
                a = math.radians(90 * k + 25)
                c, s_ = math.cos(a), math.sin(a)
                nx, nz = -s_ * 0.022, c * 0.022
                _face(M('white'), [(0.02 * c - nx * 0.3, -0.004, 0.02 * s_ - nz * 0.3), (0.19 * c - nx, -0.004, 0.19 * s_ - nz),
                                   (0.19 * c + nx, -0.004, 0.19 * s_ + nz), (0.02 * c + nx * 0.3, -0.004, 0.02 * s_ + nz * 0.3)])
        door(0, 0, 0, 0.13, 0.13, 0.03, 0.05)


def keep(x, y, g, w=0.34, d=0.28, h=0.28, rot=0.0):
    """A crenellated keep with four corner turrets."""
    with at(x, y, g, rot):
        box(0, 0, 0, w, d, h, M('stone'))
        windows(0, 0, 0, w, d, h, 3, 2, ww=0.028, wh=0.045, z0=0.38, z1=0.78)
        door(0, 0, 0, w, d, 0.06, 0.09)
        box(0, 0, h, w + 0.03, d + 0.03, 0.025, M('stone2'))
        for sx, sy in ((-1, -1), (1, -1), (-1, 1), (1, 1)):
            tower(sx * w / 2, sy * d / 2, 0, 0.05, h + 0.06, 'stone', crenel=True)
        for i in range(7):
            for sy in (-1, 1):
                box(-w / 2 + 0.02 + i * (w - 0.04) / 6, sy * (d / 2 + 0.012), h + 0.025, 0.028, 0.012, 0.03, M('stone'))
        banner(0, 0, h + 0.025, 0.2, 0.08)


def castle(g, capital):
    """Walled castle: gatehouse, curtain walls, conical-roofed towers and a great hall."""
    w, d = 0.6, 0.4
    cy = 0.06
    pts = [(-w / 2, -d / 2), (w / 2, -d / 2), (w / 2, d / 2), (-w / 2, d / 2)]
    for i in range(4):
        a, b = pts[i], pts[(i + 1) % 4]
        if i == 0:
            wall_seg(a[0], a[1] + cy, -0.08, a[1] + cy, g, h=0.15, t=0.045, wall='stone')
            wall_seg(0.08, a[1] + cy, b[0], b[1] + cy, g, h=0.15, t=0.045, wall='stone')
        else:
            wall_seg(a[0], a[1] + cy, b[0], b[1] + cy, g, h=0.15, t=0.045, wall='stone')
    for (px, py) in pts:
        tower(px, py + cy, g, 0.06, 0.27, 'stone', roof=ACC, rh=0.17)
    gy = pts[0][1] + cy
    box(0, gy, g, 0.18, 0.08, 0.19, M('stone'))
    box(0, gy - 0.0405, g, 0.07, 0.003, 0.11, M('dark'))
    box(0, gy, g + 0.19, 0.2, 0.1, 0.02, M('stone2'))
    for sx in (-1, 1):
        tower(sx * 0.1, gy, g, 0.035, 0.24, 'stone', crenel=True)
    box(-0.08, 0.17, g, 0.34, 0.17, 0.15, M('cream'))
    windows(-0.08, 0.17, g, 0.34, 0.17, 0.15, 4, 1, sides='f', ww=0.03, wh=0.06, z0=0.55, z1=0.55, frame=M('stone'))
    gable(-0.08, 0.17, g + 0.15, 0.36, 0.19, 0.12, M(ACC), overhang=0.012, mat_end=M('cream'))
    box(0.2, 0.2, g, 0.12, 0.12, 0.3, M('stone'))
    cone(0.2, 0.2, g + 0.3, 0.085, 0.16, M('slate'), n=4)
    keep(-0.2, 0.26, g, 0.18, 0.15, 0.34)
    banner(0.0, -0.03, g, 0.3, 0.09)
    if capital:
        for sx in (-1, 1):
            banner(sx * 0.2, -0.04, g, 0.22, 0.07)


def build(tier, capital):
    def b():
        g = bl.plot('grass' if tier <= 2 else 'paveL')
        rnd = random.Random(70 + tier + (6 if capital else 0))
        s = {1: 1.6, 2: 1.42, 3: 1.3, 4: 1.2}[tier]
        with at(0, 0, 0, 0, s):
            n = {1: 3, 2: 7, 3: 12, 4: 17}[tier]
            keepout = [(0, 0.2, 0.42, 0.3), (0, -0.2, 0.1, 0.18)] if capital else [(0, 0.1, 0.3, 0.26)]
            road(0, -0.9, 0, 0.0, g, 0.09, 'dirt' if tier <= 2 else 'paveD')
            if tier >= 2:
                road(-0.8, -0.14, 0.8, -0.14, g, 0.08, 'dirt' if tier <= 2 else 'paveD')
            pos = layout(rnd, n, keepout + [(0, -0.45, 0.07, 0.35)], extent=0.82 / s)
            for k, (x, y) in enumerate(pos):
                r = facing(x, y)
                if k == 2 and tier >= 3:
                    church(x, y, g, rot=0, s=0.9)
                elif k == 7 and tier >= 4:
                    windmill(x, y, g)
                elif k % 5 == 3 and tier >= 2:
                    stone_house(x, y, g, rnd, r)
                else:
                    timber_house(x, y, g, rnd, r, big=(k % 4 == 0 and tier >= 3), acc_roof=(k % 4 == 1 and tier >= 2))
            if capital:
                castle(g, True)
            else:
                keep(0, 0.1, g)
            for (x, y, sc) in [(-0.64, 0.34, 1.0), (0.62, 0.42, 0.9), (-0.66, -0.35, 0.85)][:1 + tier // 2]:
                tree(x, y, sc, 'round', g)
            if tier >= 2:
                well(0.2, -0.3, g)
            if tier >= 3:
                stall(-0.22, -0.34, g, 0.14, 0.1, awning=ACC, goods='red')
                stall(0.0, -0.58, g, 0.14, 0.1, awning='blue', goods='orange')
                crowd(g, [(-0.1, -0.42), (0.1, -0.5), (0.06, -0.3), (-0.3, -0.52)])
                cart(0.34, -0.5, g, rot=-8)
            if tier >= 4:
                hay(-0.6, -0.1, g)
                fountain(0.0, -0.36, g, 0.7)
    return b
