"""Modern city centres (era 3): glass towers, offices, parks, parliament."""
import random

import bl
from town import *  # noqa: F401,F403
from town import ACC


def glass_tower(x, y, g, w, d, h, rot=0.0, setback=False, crown=ACC, body='glass'):
    """Curtain-wall tower: dark glass with pale floor bands, optional setback and a crown."""
    with at(x, y, g, rot):
        box(0, 0, 0, w + 0.01, d + 0.01, 0.02, M('concreteD'))
        h1 = h * (0.62 if setback else 1.0)
        box(0, 0, 0.02, w, d, h1, M(body, rough=0.06, metal=0.45))
        n = max(3, int(h1 / 0.035))
        for i in range(n):
            box(0, 0, 0.02 + (i + 0.5) * h1 / n, w * 1.006, d * 1.006, 0.007, M('white', rgb='#dfe6ec', style='plain'))
        for sx in (-1, 1):
            box(sx * w / 2, 0, 0.02, 0.008, d * 1.006, h1, M('steel'))
        top = 0.02 + h1
        if setback:
            h2 = h - h1
            box(0, 0, top, w * 0.68, d * 0.68, h2, M(body, rough=0.06, metal=0.45))
            n2 = max(2, int(h2 / 0.035))
            for i in range(n2):
                box(0, 0, top + (i + 0.5) * h2 / n2, w * 0.69, d * 0.69, 0.007, M('white', rgb='#dfe6ec', style='plain'))
            top += h2
            w, d = w * 0.68, d * 0.68
        box(0, 0, top, w * 1.02, d * 1.02, 0.018, M(crown))
        if h > 0.5:
            cyl(0, 0, top + 0.018, 0.004, 0.12, M('steel'), n=4)


def office(x, y, g, rnd, rot=0.0, h=0.2):
    w, d = 0.2, 0.17
    with at(x, y, g, rot):
        box(0, 0, 0, w, d, h, M(rnd.choice(['concrete', 'white', 'sand'])))
        n = max(2, int(h / 0.06))
        for i in range(n):
            box(0, -d / 2 - 0.002, 0.03 + i * (h - 0.04) / n, w * 0.92, 0.004, 0.032, M('glassD', rough=0.05))
        box(0, 0, h, w + 0.012, d + 0.012, 0.014, M('concreteD'))
        if rnd.random() < 0.5:
            box(0.04, 0.02, h + 0.014, 0.06, 0.06, 0.03, M('steel'))
            solar_panel(-0.05, 0.0, h + 0.014, 0.08, 0.06, 20)


def apartments(x, y, g, rnd, rot=0.0, h=0.2):
    w, d = 0.2, 0.15
    col_ = rnd.choice(['white', 'cream', 'rose', 'orange'])
    with at(x, y, g, rot):
        box(0, 0, 0, w, d, h, M(col_))
        rows = max(2, int(h / 0.055))
        for i in range(rows):
            for k in range(3):
                cx = -w / 2 + w * (k + 0.5) / 3
                zz = 0.04 + i * (h - 0.06) / rows
                box(cx, -d / 2 - 0.002, zz, 0.04, 0.004, 0.032, M('glassD', rough=0.05))
                box(cx, -d / 2 - 0.012, zz - 0.004, 0.056, 0.022, 0.006, M('concreteD'))   # balcony
        box(0, 0, h, w + 0.01, d + 0.01, 0.012, M('concreteD'))


def park(x, y, g, w=0.24, d=0.18, rnd=None):
    lawn_patch(x, y, g, w, d)
    tree(x - w * 0.25, y + d * 0.1, 0.8, 'round', g)
    tree(x + w * 0.2, y - d * 0.05, 0.7, 'round', g)
    tree(x + w * 0.05, y + d * 0.3, 0.6, 'bush', g)


def car(x, y, g, rot=0.0, colr='red'):
    with at(x, y, g, rot):
        box(0, 0, 0.003, 0.05, 0.026, 0.014, M(colr, rough=0.3))
        box(0.002, 0, 0.017, 0.026, 0.022, 0.011, M('glassD'))


def parliament(g):
    box(0, 0.1, g, 0.62, 0.3, 0.2, M('white'))
    windows(0, 0.1, g, 0.62, 0.3, 0.2, 8, 2, ww=0.03, wh=0.05, z0=0.3, z1=0.72, sides='f', mat=M('glassD'))
    box(0, 0.1, g + 0.2, 0.64, 0.32, 0.02, M('marble'))
    for i in range(8):
        col(-0.24 + i * 0.069, -0.09, g, 0.014, 0.2, M('marble'), n=8)
    box(0, -0.09, g + 0.2, 0.6, 0.07, 0.02, M('marble'))
    gable(0, -0.09, g + 0.22, 0.6, 0.075, 0.055, M('marble'), mat_end=M('marble'))
    stair_ramp(0, -0.2, g, 0.52, 0.1, 0.03, 3, 'marble')
    cyl(0, 0.12, g + 0.22, 0.14, 0.09, M('white'), n=22)
    cyl(0, 0.12, g + 0.31, 0.15, 0.012, M('marble'), n=22)
    dome(0, 0.12, g + 0.322, 0.14, M('white', rough=0.3), h=0.16, n=22)
    cyl(0, 0.12, g + 0.48, 0.02, 0.05, M(ACC), n=8)
    cone(0, 0.12, g + 0.53, 0.018, 0.06, M('gold'), n=8)
    for sx in (-1, 1):
        pennant(sx * 0.2, -0.24, g, 0.34, 0.08)
    fountain(0, -0.36, g, 0.75)


def build(tier, capital):
    def b():
        g = bl.plot('concrete')
        rnd = random.Random(120 + tier + (9 if capital else 0))
        s = {1: 1.6, 2: 1.42, 3: 1.3, 4: 1.2}[tier]
        with at(0, 0, 0, 0, s):
            n = {1: 3, 2: 6, 3: 10, 4: 14}[tier]
            road(0, -0.9, 0, 0.55, g, 0.1, 'asphalt')
            road(-0.85, -0.28, 0.85, -0.28, g, 0.1, 'asphalt')
            for xx in (-0.5, -0.3, 0.3, 0.5):
                pass
            keepout = [(0, 0.1, 0.36, 0.34)] if capital else [(0, 0.1, 0.16, 0.2)]
            keepout += [(0, -0.4, 0.09, 0.45), (0, -0.28, 0.9, 0.07)]
            pos = layout(rnd, n, keepout, extent=0.8 / s)
            for k, (x, y) in enumerate(pos):
                kind = k % 5
                if kind in (0, 3) and tier >= 2:
                    h = min(0.3 + 0.07 * tier + rnd.random() * 0.12, 0.62 - 0.5 * max(y, 0.0))
                    glass_tower(x, y, g, 0.15, 0.15, h, rot=0, setback=(k % 2 == 0 and h > 0.45),
                                crown=ACC if k % 3 == 0 else 'steel', body='glass' if k % 4 else 'glassD')
                elif kind == 1 or kind == 4:
                    office(x, y, g, rnd, 0, h=0.14 + 0.04 * tier + rnd.random() * 0.06)
                elif kind == 2:
                    park(x, y, g)
                else:
                    apartments(x, y, g, rnd, 0, h=0.14 + 0.03 * tier)
            if capital:
                parliament(g)
            else:
                glass_tower(0, 0.1, g, 0.2, 0.2, 0.55 if tier >= 2 else 0.36, setback=tier >= 2, crown=ACC)
            for (cx_, cy_, c_) in [(-0.1, -0.28, 'red'), (0.18, -0.28, 'blue'), (0.04, -0.5, 'white'), (-0.04, -0.7, 'orange'), (0.04, 0.4, 'steel')][:tier + 1]:
                car(cx_, cy_, g, rot=0 if cy_ == -0.28 else 90, colr=c_)
            if tier >= 3:
                crowd(g, [(-0.16, -0.38), (0.15, -0.4)])
    return b
