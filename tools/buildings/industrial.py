"""Industrial city centres (era 2): brick terraces, chimneys, rail, town halls."""
import math
import random

import bl
from bl import _face
from town import *  # noqa: F401,F403
from town import ACC


def terrace(x, y, g, rnd, rot=0.0, n=2, tall=False):
    """A row of brick houses under one slate roof, one chimney per house."""
    w1, d = 0.1, 0.15
    h = 0.17 if not tall else 0.26
    brick = rnd.choice(['brick', 'brick', 'brickD', 'sandD'])
    W = w1 * n
    with at(x, y, g, rot):
        box(0, 0, 0, W, d, h, M(brick))
        box(0, 0, 0, W + 0.006, d + 0.006, 0.02, M('stoneD'))
        for i in range(n):
            cx = -W / 2 + w1 * (i + 0.5)
            windows(cx, 0, 0, w1, d, h, 1, 3 if tall else 2, ww=0.028, wh=0.04, z0=0.5 if not tall else 0.3, z1=0.78, sides='f')
            door(cx, 0, 0, w1, d, 0.032, 0.065)
        gable(0, 0, h, W + 0.01, d + 0.02, 0.085, M(rnd.choice(['slate', 'slate', 'slateL'])), overhang=0.008, mat_end=M(brick))
        for i in range(n):
            chimney(-W / 2 + w1 * (i + 0.5), 0.0, h + 0.03, 0.011, 0.08 + 0.02 * (i % 2), smoke=0)


def factory(x, y, g, rot=0.0, big=False):
    w, d = (0.4, 0.2) if big else (0.3, 0.17)
    with at(x, y, g, rot):
        box(0, 0, 0, w, d, 0.12, M('brickD'))
        windows(0, 0, 0, w, d, 0.12, 5 if big else 4, 1, ww=0.035, wh=0.05, z0=0.55, z1=0.55, sides='f')
        for k in range(3):
            sx0 = -w / 2 + k * w / 3
            _face(M('slate'), [(sx0, -d / 2 - 0.01, 0.12), (sx0 + w / 3, -d / 2 - 0.01, 0.2), (sx0 + w / 3, d / 2 + 0.01, 0.2), (sx0, d / 2 + 0.01, 0.12)])
            _face(M('glass'), [(sx0 + w / 3, -d / 2 - 0.01, 0.2), (sx0 + w / 3, -d / 2 - 0.01, 0.12), (sx0 + w / 3, d / 2 + 0.01, 0.12), (sx0 + w / 3, d / 2 + 0.01, 0.2)])
            for yy in (-d / 2 - 0.01, d / 2 + 0.01):
                _face(M('brickD'), [(sx0, yy, 0.12), (sx0 + w / 3, yy, 0.12), (sx0 + w / 3, yy, 0.2)])
        chimney(w * 0.38, d * 0.3, 0, 0.026, 0.4, smoke=4, mat='brick')
        door(0, 0, 0, w, d, 0.07, 0.08)


def gasworks(x, y, g):
    cyl(x, y, g, 0.075, 0.1, M('iron'), n=14)
    for k in range(8):
        a = 2 * math.pi * k / 8
        cyl(x + 0.08 * math.cos(a), y + 0.08 * math.sin(a), g, 0.005, 0.15, M('dark'), n=4)
    dome(x, y, g + 0.1, 0.075, M('steel'), h=0.03, n=14)


def station(x, y, g, rot=0.0):
    """A train shed with a locomotive on the track in front."""
    with at(x, y, g, rot):
        box(0, 0.02, 0, 0.42, 0.18, 0.11, M('brick'))
        windows(0, 0.02, 0, 0.42, 0.18, 0.11, 6, 1, ww=0.03, wh=0.05, z0=0.5, z1=0.5, sides='f')
        gable(0, 0.02, 0.11, 0.44, 0.2, 0.09, M('slate'), mat_end=M('brick'), overhang=0.01)
        box(0, -0.14, 0, 0.5, 0.05, 0.012, M('concreteD'))              # platform
        for yy in (-0.22, -0.28):
            box(0, yy, 0, 0.6, 0.012, 0.012, M('iron'))
        for k in range(10):
            box(-0.27 + k * 0.06, -0.25, 0, 0.015, 0.08, 0.007, M('woodD'))
        # locomotive
        cyl(-0.1, -0.25, 0.024, 0.026, 0.12, M('dark'), n=10)
        box(0.02, -0.25, 0.015, 0.1, 0.045, 0.05, M('dark'))
        box(0.12, -0.25, 0.015, 0.08, 0.046, 0.04, M(ACC))
        cyl(-0.17, -0.25, 0.05, 0.007, 0.05, M('dark'), n=6, r2=0.012)
        smoke_plume(-0.17, -0.25, 0.12, 3, 0.8)
        for k in range(3):
            cyl(-0.12 + k * 0.025, -0.25, 0.0, 0.014, 0.01, M('iron'), n=8)


def townhall(g, capital):
    if capital:
        # neoclassical capitol: wings, portico and a tall dome in the owner's colour
        box(0, 0.12, g, 0.56, 0.28, 0.2, M('cream'))
        windows(0, 0.12, g, 0.56, 0.28, 0.2, 7, 2, ww=0.03, wh=0.05, z0=0.3, z1=0.72, sides='f')
        box(0, 0.12, g + 0.2, 0.6, 0.32, 0.025, M('marble'))
        for i in range(6):
            col(-0.2 + i * 0.08, -0.07, g, 0.016, 0.2, M('marble'))
        box(0, -0.07, g + 0.2, 0.52, 0.07, 0.022, M('marble'))
        gable(0, -0.07, g + 0.222, 0.52, 0.075, 0.06, M('marble'), mat_end=M('marble'))
        stair_ramp(0, -0.22, g, 0.44, 0.14, 0.04, 4, 'marble')
        cyl(0, 0.14, g + 0.225, 0.14, 0.08, M('cream'), n=20)
        for k in range(12):
            a = 2 * math.pi * k / 12
            col(0.12 * math.cos(a), 0.14 + 0.12 * math.sin(a), g + 0.225, 0.008, 0.08, M('marble'), n=6, base=False, cap=False)
        cyl(0, 0.14, g + 0.305, 0.145, 0.014, M('marble'), n=20)
        dome(0, 0.14, g + 0.32, 0.135, M(ACC), h=0.17, n=20)
        cyl(0, 0.14, g + 0.49, 0.025, 0.04, M('marble'), n=10)
        cone(0, 0.14, g + 0.53, 0.02, 0.07, M('gold'), n=8)
        pennant(0, 0.14, g + 0.6, 0.12, 0.06)
        for sx in (-1, 1):
            pennant(sx * 0.4, -0.18, g, 0.34, 0.08)
    else:
        box(0, 0.08, g, 0.38, 0.22, 0.2, M('brick'))
        windows(0, 0.08, g, 0.38, 0.22, 0.2, 5, 2, ww=0.03, wh=0.05, z0=0.3, z1=0.72, sides='f')
        door(0, 0.08, g, 0.38, 0.22, 0.06, 0.08)
        hip(0, 0.08, g + 0.2, 0.4, 0.24, 0.07, M('slate'), ridge=0.6)
        box(0, 0.0, g + 0.2, 0.12, 0.12, 0.2, M('brick'))              # clock tower
        box(0, -0.0, g + 0.4, 0.13, 0.13, 0.014, M('stone'))
        box(0, -0.0 - 0.066, g + 0.3, 0.07, 0.004, 0.07, M('white'))
        pyr(0, 0.0, g + 0.414, 0.12, 0.12, 0.16, M(ACC))


def build(tier, capital):
    def b():
        g = bl.plot('paveD')
        rnd = random.Random(90 + tier + (9 if capital else 0))
        s = {1: 1.6, 2: 1.42, 3: 1.3, 4: 1.2}[tier]
        with at(0, 0, 0, 0, s):
            n = {1: 3, 2: 6, 3: 10, 4: 14}[tier]
            keepout = [(0, 0.14, 0.42, 0.3)] if capital else [(0, 0.1, 0.26, 0.24)]
            road(0, -0.9, 0, -0.1, g, 0.1, 'asphalt')
            if tier >= 2:
                road(-0.85, -0.3, 0.85, -0.3, g, 0.08, 'asphalt')
            if tier >= 4:
                keepout += [(0, -0.58, 0.34, 0.16)]
            pos = layout(rnd, n, keepout + [(0, -0.5, 0.07, 0.3)], extent=0.8 / s, bias=0.05)
            for k, (x, y) in enumerate(pos):
                r = facing(x, y)
                if tier >= 3 and k == 4:
                    factory(x, y, g, rot=0, big=(tier >= 4))
                elif tier >= 4 and k == 8:
                    gasworks(x, y, g)
                else:
                    terrace(x, y, g, rnd, r, n=2 + (k % 2), tall=(k % 4 == 0 and tier >= 2))
            townhall(g, capital)
            if tier >= 2:
                for k, (x, y) in enumerate([(-0.64, 0.3), (0.66, 0.34)]):
                    tree(x, y, 0.9, 'round', g)
            if tier >= 3:
                lamp(-0.1, -0.2, g)
                lamp(0.1, -0.2, g)
                crowd(g, [(-0.15, -0.24), (0.14, -0.28), (0.04, -0.12)])
            if tier >= 4:
                station(0.0, -0.58, g)
    return b
