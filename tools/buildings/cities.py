"""City-center sprites.

    city_<era>_<tier>     era 0 ancient, 1 medieval, 2 industrial, 3 modern;
                          tier 1..4 = population 1-3, 4-8, 9-15, 16+
    capital_<era>_<tier>  the same with a palace
    walls_<style>_<back|front>   style 0 palisade, 1 stone, 2 modern;
                          drawn before / after the city when it has walls

Owner-coloured parts use the 'acc' material (flags, roofs, the plot ring);
the _tint.png layer is painted over the sprite in the owner's colour.
"""
import math
import random

import bl
from bl import _hex_pts
from mathutils import Matrix
from parts import *  # noqa: F401,F403

SLOTS = [(0.0, 0.05), (-0.4, -0.2), (0.4, -0.18), (0.0, -0.42), (-0.42, 0.3), (0.42, 0.32), (0.0, 0.46),
         (-0.62, 0.02), (0.62, 0.05), (-0.25, -0.52), (0.28, -0.5), (-0.6, -0.3), (0.6, -0.32), (-0.22, 0.62), (0.22, 0.62)]
COUNT = {1: 3, 2: 6, 3: 10, 4: 15}
ACC = 'acc'


SCALE = {1: 1.4, 2: 1.22, 3: 1.1, 4: 1.0}


def ground(era, tier=4):
    """Lay the plot, then scale everything built afterwards (small towns get bigger buildings)."""
    if era == 0:
        g = plot('earth', top='dirt', rim=ACC)
    elif era == 1:
        g = plot('pave', rim=ACC)
    elif era == 2:
        g = plot('paveD', rim=ACC)
    else:
        g = plot('concreteD', top='concrete', rim=ACC)
    s = SCALE[tier]
    bl._stack.append(bl._stack[-1] @ Matrix.Diagonal((s, s, s, 1.0)))  # reset() clears the stack per render
    return g


# --- ancient ---------------------------------------------------------------

def ancient_house(g, x, y, i, rnd):
    if i % 3 == 0:
        hut(x, y, g, 0.085 + rnd.random() * 0.02, 0.08, wall='adobe', roof='thatch')
    elif i % 3 == 1:
        house(x, y, g, 0.17, 0.14, 0.09, wall='adobe', roof='thatch', rot=rnd.choice([0, 90, 20, -20]), rh=0.1, floors=1, win=False)
    else:
        house(x, y, g, 0.15, 0.15, 0.08, wall='sand', roof='thatch', style='pyr', rh=0.09, win=False)


def ancient_hall(g, tier, capital):
    if capital:
        stepped(0, 0.05, g, 0.56, 3, 0.07, 0.07, 'sand', alt='adobe')
        box(0, 0.05, g + 0.21, 0.26, 0.2, 0.12, M('stone'))
        gable(0, 0.05, g + 0.33, 0.3, 0.24, 0.1, M('terra'), overhang=0.02)
        for i in range(4):
            col(-0.1 + i * 0.067, -0.08, g + 0.21, 0.014, 0.12, M('marble'), n=8)
        flag(0.0, 0.05, g + 0.43, 0.2, M(ACC), w=0.11)
        stair_up(0.0, -0.26, g, 0.12, 0.2, 0.21, 3)
    else:
        box(0, 0.05, g, 0.34, 0.22, 0.12, M('adobe'))
        gable(0, 0.05, g + 0.12, 0.38, 0.26, 0.16, M('thatch'), overhang=0.02, mat_end=M('adobe'))
        windows(0, 0.05, g, 0.34, 0.22, 0.12, 3, 1)
        door(0, 0.05, g, 0.34, 0.22, 0.07, 0.09)
        flag(0.12, 0.05, g + 0.26, 0.14, M(ACC), w=0.09)


def stair_up(x, y, z, w, length, rise, n):
    for i in range(n):
        box(x, y + length * (i + 0.5) / n, z, w, length / n, rise * (i + 1) / n, M('stone2'))


def ancient(tier, capital):
    def b():
        g = ground(0, tier)
        rnd = random.Random(3 + tier)
        if tier < 3:
            tree(-0.62 / SCALE[tier], 0.3, 0.9, 'palm', z=g)
        for i in range(1, COUNT[tier]):
            x, y = SLOTS[i]
            ancient_house(g, x, y, i, rnd)
        ancient_hall(g, tier, capital)
        if tier >= 3:
            box(0.0, 0.46, g, 0.2, 0.2, 0.1, M('stone'))
            box(0.0, 0.46, g + 0.1, 0.14, 0.14, 0.1, M('stone2'))
            cyl(-0.62, -0.05, g, 0.012, 0.28, M('woodD'), n=5)
            box(-0.62, -0.05, g + 0.25, 0.1, 0.1, 0.07, M('woodL'))
            cone(-0.62, -0.05, g + 0.32, 0.08, 0.08, M('thatch'), n=6)
        if tier >= 4:
            cyl(0.0, -0.3, g, 0.03, 0.02, M('stone2'), n=8)
    return b


# --- medieval --------------------------------------------------------------

def timber_house(g, x, y, i, rnd, big=False):
    w, d, h = (0.19, 0.15, 0.14) if not big else (0.21, 0.17, 0.2)
    rot = rnd.choice([0, 0, 90, 15, -15])
    box(x, y, g, w, d, h, M('cream'), rot)
    with at(x, y, g, rot):
        for k in range(4):
            box(-w / 2 + (k + 0.5) * w / 4 + 0.0, -d / 2 - 0.001, 0.0, 0.01, 0.004, h, M('woodD'))
        box(0, -d / 2 - 0.001, h * 0.55, w, 0.004, 0.01, M('woodD'))
    gable(x, y, g + h, w, d, 0.11, M('terra' if i % 3 else ACC), rot, overhang=0.015, mat_end=M('cream'))
    windows(x, y, g, w, d, h, 2, 2 if big else 1, rot=rot)
    door(x, y, g, w, d, rot=rot)


def medieval_hall(g, tier, capital):
    if capital:
        box(0, 0.08, g, 0.5, 0.3, 0.26, M('stone'))
        windows(0, 0.08, g, 0.5, 0.3, 0.26, 4, 2)
        for sx in (-1, 1):
            tower(sx * 0.27, 0.08, g, 0.07, 0.36, 'stone', roof=ACC, rh=0.18)
        box(0, 0.08, g + 0.26, 0.52, 0.32, 0.02, M('stone2'))
        dome(0, 0.1, g + 0.28, 0.1, M('gold'), h=0.12, n=14)
        flag(0.0, 0.1, g + 0.4, 0.14, M(ACC), w=0.1)
        box(0.0, -0.1, g, 0.12, 0.05, 0.12, M('stone2'))
    else:
        box(0, 0.06, g, 0.32, 0.26, 0.28, M('stone'))
        windows(0, 0.06, g, 0.32, 0.26, 0.28, 3, 2)
        for (sx, sy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)]:
            tower(sx * 0.16, 0.06 + sy * 0.13, g, 0.045, 0.34, 'stone', crenel=True)
        box(0, 0.06, g + 0.28, 0.3, 0.24, 0.02, M('stone2'))
        flag(0.0, 0.06, g + 0.3, 0.16, M(ACC), w=0.1)


def church(g, x, y):
    box(x, y, g, 0.34, 0.16, 0.13, M('stone'))
    gable(x, y, g + 0.13, 0.34, 0.16, 0.1, M('slate'), mat_end=M('stone'))
    box(x + 0.15, y, g, 0.1, 0.1, 0.3, M('stone'))
    pyr(x + 0.15, y, g + 0.3, 0.11, 0.11, 0.2, M('slate'))


def windmill(g, x, y):
    cyl(x, y, g, 0.07, 0.2, M('cream'), n=8, r2=0.05)
    cone(x, y, g + 0.2, 0.07, 0.07, M('terraD'), n=8)
    with at(x, y - 0.06, g + 0.21):
        for k in range(4):
            a = 90 * k + 20
            ax = math.radians(a)
            _f = [(-0.008 * math.sin(ax), 0, 0.008 * math.cos(ax)),
                  (0.008 * math.sin(ax), 0, -0.008 * math.cos(ax)),
                  (0.008 * math.sin(ax) + 0.2 * math.cos(ax), 0, -0.008 * math.cos(ax) + 0.2 * math.sin(ax)),
                  (-0.008 * math.sin(ax) + 0.2 * math.cos(ax), 0, 0.008 * math.cos(ax) + 0.2 * math.sin(ax))]
            bl._face(M('white'), _f)


def medieval(tier, capital):
    def b():
        g = ground(1, tier)
        rnd = random.Random(7 + tier)
        for i in range(1, COUNT[tier]):
            x, y = SLOTS[i]
            if i == 3 and tier >= 3:
                church(g, x, y)
            elif i == 9 and tier >= 4:
                windmill(g, x, y)
            else:
                timber_house(g, x, y, i, rnd, big=(i % 4 == 0 and tier >= 3))
        medieval_hall(g, tier, capital)
        if tier >= 2:
            tree(-0.62 / SCALE[tier], 0.05, 0.8, 'round', z=g)
    return b


# --- industrial ------------------------------------------------------------

def row_house(g, x, y, i, rnd, tall=False):
    w, d = 0.2, 0.15
    h = 0.17 if not tall else 0.26
    rot = rnd.choice([0, 0, 90])
    box(x, y, g, w, d, h, M('brick' if i % 2 else 'brickD'), rot)
    windows(x, y, g, w, d, h, 3, 3 if tall else 2, rot=rot)
    door(x, y, g, w, d, rot=rot)
    gable(x, y, g + h, w, d, 0.09, M('slate'), rot, overhang=0.012)
    if i % 2:
        chimney(x + 0.05, y + 0.03, g + h, 0.012, 0.1, smoke=1 if i % 4 == 1 else 0)


def industrial_hall(g, tier, capital):
    if capital:
        box(0, 0.08, g, 0.56, 0.32, 0.26, M('cream'))
        windows(0, 0.08, g, 0.56, 0.32, 0.26, 5, 2)
        for i in range(6):
            col(-0.25 + i * 0.1, -0.1, g, 0.017, 0.24, M('marble'))
        box(0, -0.1, g + 0.24, 0.6, 0.08, 0.025, M('marble'))
        gable(0, -0.1, g + 0.265, 0.6, 0.09, 0.07, M('marble'), mat_end=M('marble'))
        cyl(0, 0.1, g + 0.26, 0.13, 0.1, M('cream'), n=16)
        dome(0, 0.1, g + 0.36, 0.13, M(ACC), h=0.15, n=16)
        flag(0.0, 0.1, g + 0.5, 0.14, M('white'), w=0.09)
    else:
        box(0, 0.06, g, 0.4, 0.24, 0.22, M('brick'))
        windows(0, 0.06, g, 0.4, 0.24, 0.22, 4, 2)
        gable(0, 0.06, g + 0.22, 0.4, 0.24, 0.1, M('slate'), mat_end=M('brick'))
        box(0.0, 0.06, g + 0.2, 0.1, 0.1, 0.3, M('brick'))
        box(0.0, 0.06 - 0.052, g + 0.4, 0.07, 0.004, 0.07, M('white'))
        pyr(0.0, 0.06, g + 0.5, 0.12, 0.12, 0.12, M(ACC))


def industrial(tier, capital):
    def b():
        g = ground(2, tier)
        rnd = random.Random(11 + tier)
        for i in range(1, COUNT[tier]):
            x, y = SLOTS[i]
            if i == 6 and tier >= 2:
                box(x, y, g, 0.34, 0.16, 0.12, M('concreteD'))
                sawtooth(x, y, g + 0.12, 0.34, 0.16)
                chimney(x + 0.1, y + 0.05, g, 0.025, 0.34, smoke=3)
            elif i == 10 and tier >= 3:
                box(x, y, g, 0.22, 0.14, 0.1, M('iron'))
                cyl(x - 0.06, y, g + 0.1, 0.04, 0.0, M('iron'))
                road_strip(x - 0.2, y - 0.1, x + 0.2, y - 0.1, 0.03, g, M('iron'))
            else:
                row_house(g, x, y, i, rnd, tall=(i % 3 == 0 and tier >= 3))
        industrial_hall(g, tier, capital)
        if tier >= 2:
            road_strip(-0.7 / SCALE[tier], -0.05, 0.7 / SCALE[tier], -0.05, 0.05, g, M('asphalt'))
    return b


# --- modern ----------------------------------------------------------------

def modern_hall(g, tier, capital):
    if capital:
        box(0, 0.06, g, 0.6, 0.3, 0.2, M('white'))
        windows(0, 0.06, g, 0.6, 0.3, 0.2, 6, 2, mat=M('glassD'))
        for i in range(7):
            col(-0.27 + i * 0.09, -0.12, g, 0.015, 0.2, M('marble'), n=8)
        box(0, -0.12, g + 0.2, 0.62, 0.07, 0.02, M('marble'))
        cyl(0, 0.08, g + 0.2, 0.14, 0.1, M('white'), n=18)
        dome(0, 0.08, g + 0.3, 0.14, M('white', rough=0.3), h=0.15, n=18)
        cone(0, 0.08, g + 0.45, 0.012, 0.1, M(ACC), n=6)
        for sx in (-1, 1):
            box(sx * 0.4, 0.05, g, 0.15, 0.2, 0.14, M('white'))
        for k in range(5):
            flag(-0.24 + k * 0.12, -0.3, g, 0.2, M(['red', 'white', 'blue', ACC, 'white'][k]), w=0.08)
    else:
        skyscraper(0.0, 0.06, g, 0.2, 0.2, 0.62 if tier >= 2 else 0.38, body='glass')
        box(0.0, 0.06, g + (0.62 if tier >= 2 else 0.38), 0.14, 0.14, 0.05, M(ACC))
        cyl(0.0, 0.06, g + (0.67 if tier >= 2 else 0.43), 0.004, 0.12, M('steel'), n=4)


def modern(tier, capital):
    def b():
        g = ground(3, tier)
        rnd = random.Random(17 + tier)
        k = SCALE[tier]
        road_strip(-0.72 / k, -0.05, 0.72 / k, -0.05, 0.1, g, M('asphalt'))
        road_strip(0.0, -0.74 / k, 0.0, 0.74 / k, 0.1, g, M('asphalt'))
        for i in range(1, COUNT[tier]):
            x, y = SLOTS[i]
            if abs(x) < 0.05 and abs(y + 0.05) < 0.4 and i != 6:
                continue
            kind = i % 4
            if kind == 0 and tier >= 2:
                h = 0.3 + 0.08 * (tier + rnd.random() * 2)
                skyscraper(x, y, g, 0.17, 0.17, h, body='glass' if i % 8 else 'steel')
                box(x, y, g + h, 0.1, 0.1, 0.03, M(ACC if i % 2 else 'steel'))
            elif kind == 1:
                h = 0.16 + 0.04 * tier
                box(x, y, g, 0.2, 0.18, h, M('concrete'))
                windows(x, y, g, 0.2, 0.18, h, 3, max(2, int(h / 0.07)), mat=M('glassD'))
                box(x, y, g + h, 0.21, 0.19, 0.012, M('slate'))
            elif kind == 2:
                lawn_patch(x, y, g, 0.22, 0.18)
                tree(x - 0.04, y, 0.7, 'round', z=g)
                tree(x + 0.05, y + 0.02, 0.6, 'round', z=g)
            else:
                h = 0.12 + 0.03 * tier
                box(x, y, g, 0.2, 0.15, h, M('white'))
                windows(x, y, g, 0.2, 0.15, h, 3, 2, mat=M('glassD'))
                box(x, y, g + h, 0.21, 0.16, 0.012, M(ACC))
        modern_hall(g, tier, capital)
    return b


ERAS = [ancient, medieval, industrial, modern]


# --- walls -----------------------------------------------------------------

def walls(style, part):
    def b():
        bl.USE_CATCHER = True
        r = 0.8
        pts = _hex_pts(r)
        mats = {0: 'woodL', 1: 'stone', 2: 'concrete'}

        def seg(a, c2):
            # split side edges at y = 0 so back/front parts line up
            pieces = [(a, c2)]
            if (a[1] > 0 > c2[1]) or (a[1] < 0 < c2[1]):
                t = a[1] / (a[1] - c2[1])
                m = (a[0] + (c2[0] - a[0]) * t, 0.0)
                pieces = [(a, m), (m, c2)]
            for (p, q) in pieces:
                my = (p[1] + q[1]) / 2
                if (my > 0.0) == (part == 'back'):
                    if style == 0:
                        palisade(p[0], p[1], q[0], q[1], 0.0, h=0.17, mat='woodL')
                    elif style == 1:
                        wall_seg(p[0], p[1], q[0], q[1], 0.0, h=0.17, t=0.06, wall='stone')
                    else:
                        wall_seg(p[0], p[1], q[0], q[1], 0.0, h=0.12, t=0.07, wall='concrete', crenel=False)
                        wall_seg(p[0], p[1], q[0], q[1], 0.12, h=0.02, t=0.05, wall='concreteD', crenel=False)

        for i in range(6):
            seg(pts[i], pts[(i + 1) % 6])
        for p in pts:
            if (p[1] > 0.05) == (part == 'back'):
                if style == 0:
                    cyl(p[0], p[1], 0.0, 0.03, 0.26, M('woodD'), n=6)
                    box(p[0], p[1], 0.26, 0.1, 0.1, 0.05, M('woodL'))
                    cone(p[0], p[1], 0.31, 0.08, 0.08, M('thatch'), n=6)
                elif style == 1:
                    tower(p[0], p[1], 0.0, 0.065, 0.3, 'stone', roof='terra', rh=0.17)
                else:
                    box(p[0], p[1], 0.0, 0.12, 0.12, 0.16, M('concreteD'), rot=45)
                    cyl(p[0], p[1], 0.16, 0.025, 0.1, M('steel'), n=6)
                    box(p[0], p[1], 0.26, 0.05, 0.05, 0.015, M('red'))
    return b


SPRITES = {}
for _e, _fn in enumerate(ERAS):
    for _t in (1, 2, 3, 4):
        SPRITES[f'city_{_e}_{_t}'] = (_fn(_t, False), True)
        SPRITES[f'capital_{_e}_{_t}'] = (_fn(_t, True), True)
for _s in (0, 1, 2):
    for _p in ('back', 'front'):
        SPRITES[f'walls_{_s}_{_p}'] = (walls(_s, _p), False)
