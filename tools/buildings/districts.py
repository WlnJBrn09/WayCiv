"""District sprites: campus, holy site, encampment, commercial hub, theater (levels 1 to 3 by buildings)."""
import math

import bl
from bl import _hex_pts
from town import *  # noqa: F401,F403


def portico(x, y, g, w, d, h, ncols, mat='marble', roof='terra', rot=0.0, steps_n=2, pediment=True):
    """A temple-front: stepped base, a row of columns, an entablature and a pediment (faces -y)."""
    with at(x, y, g, rot):
        for i in range(steps_n):
            box(0, 0, i * 0.018, w + 0.04 - i * 0.02, d + 0.04 - i * 0.02, 0.018, M('stone'))
        z0 = 0.018 * steps_n
        for i in range(ncols):
            cx = -w / 2 + 0.03 + i * (w - 0.06) / max(ncols - 1, 1)
            col(cx, -d / 2 + 0.03, z0, 0.014, h, M(mat), n=10)
            col(cx, d / 2 - 0.03, z0, 0.014, h, M(mat), n=10)
        box(0, 0, z0 + h, w, d, 0.025, M(mat))
        if pediment:
            gable(0, 0, z0 + h + 0.025, w, d, d * 0.3, M(roof), overhang=0.0, mat_end=M(mat))
        return z0 + h + 0.025


def lawn(x, y, g, w, d, rot=0.0):
    box(x, y, g - 0.004, w, d, 0.006, G('grass', 1.0, 0.7, 0.95), rot)


def bench(x, y, g, rot=0.0):
    with at(x, y, g, rot):
        box(0, 0, 0.012, 0.05, 0.014, 0.006, M('woodL'))
        box(0, 0.008, 0.018, 0.05, 0.004, 0.014, M('woodL'))
        for sx in (-1, 1):
            box(sx * 0.02, 0, 0, 0.005, 0.012, 0.012, M('iron'))


def brazier(x, y, g, s=1.0):
    lathe(x, y, g, [(0.008 * s, 0), (0.014 * s, 0.045 * s), (0.026 * s, 0.065 * s), (0.02 * s, 0.07 * s)], M('bronze'), n=8)
    cone(x, y, g + 0.066 * s, 0.014 * s, 0.04 * s, M('flame', rgb='#ffb040', emit='#ff9a28', emit_strength=7.0), n=6)


def campus(lvl):
    def b():
        g = bl.plot('grass')
        road(0, -0.8, 0, -0.1, g, 0.07, 'paveL')
        road(-0.6, -0.45, 0.6, -0.45, g, 0.06, 'paveL')
        # main hall
        hx, hy = (-0.12, 0.2) if lvl >= 2 else (0.0, 0.18)
        box(hx, hy, g, 0.5, 0.26, 0.22, M('cream'))
        windows(hx, hy, g, 0.5, 0.26, 0.22, 6, 2, ww=0.03, wh=0.05, z0=0.3, z1=0.72, sides='f')
        hip(hx, hy, g + 0.22, 0.54, 0.3, 0.09, M('slate'), ridge=0.7)
        portico(hx, hy - 0.2, g, 0.26, 0.1, 0.2, 4, mat='marble', roof='slate')
        box(hx, hy + 0.0, g + 0.22, 0.13, 0.13, 0.14, M('cream'))                       # clock tower
        box(hx, hy - 0.066, g + 0.3, 0.06, 0.004, 0.05, M('white'))
        pyr(hx, hy, g + 0.36, 0.14, 0.14, 0.15, M('verdi'))
        cyl(hx, hy, g + 0.51, 0.003, 0.05, M('gold'), n=4)
        # quad, trees and students
        lawn(hx, hy - 0.42, g, 0.5, 0.28)
        for tx in (-0.58, 0.58):
            tree(tx, -0.1, 1.0, 'round', g)
        for tx in (-0.4, 0.4):
            tree(tx, -0.5, 0.8, 'round', g)
        bench(hx - 0.14, hy - 0.5, g)
        bench(hx + 0.14, hy - 0.5, g)
        crowd(g, [(hx - 0.06, hy - 0.36), (hx + 0.08, hy - 0.3), (hx + 0.16, hy - 0.4)])
        if lvl >= 2:
            # library rotunda + observatory
            cx, cy = 0.4, 0.2
            box(cx, cy, g, 0.26, 0.26, 0.15, M('brick'))
            windows(cx, cy, g, 0.26, 0.26, 0.15, 3, 1, ww=0.03, wh=0.06, z0=0.5, z1=0.5, sides='f')
            cyl(cx, cy, g + 0.15, 0.11, 0.07, M('cream'), n=16)
            dome(cx, cy, g + 0.22, 0.115, M('verdi'), h=0.12, n=16)
            cyl(cx, cy, g + 0.34, 0.006, 0.05, M('gold'), n=4)
            ox, oy = -0.5, 0.3
            cyl(ox, oy, g, 0.075, 0.2, M('stone'), n=14)
            dome(ox, oy, g + 0.2, 0.08, M('white', rough=0.3), h=0.09, n=14)
            box(ox, oy - 0.04, g + 0.22, 0.02, 0.05, 0.1, M('dark'), rot=0)
            cyl(ox + 0.02, oy - 0.03, g + 0.27, 0.007, 0.07, M('steel'), n=6)
        if lvl >= 3:
            lx, ly = 0.45, -0.18
            box(lx, ly, g, 0.3, 0.2, 0.2, M('glass', rough=0.06, metal=0.45))
            for i in range(5):
                box(lx, ly, g + 0.02 + i * 0.04, 0.304, 0.204, 0.005, M('white', rgb='#dfe6ec', style='plain'))
            box(lx, ly, g + 0.2, 0.32, 0.22, 0.015, M('steel'))
            dish(lx - 0.07, ly + 0.02, g + 0.215, 0.06, 0)
            solar_panel(lx + 0.07, ly + 0.02, g + 0.215, 0.1, 0.07, 22)
            sphere(-0.44, -0.18, g + 0.0, 0.06, M('steel', rough=0.15), n=14, rings=9)
            cyl(-0.44, -0.18, g, 0.075, 0.012, M('stone2'), n=14)
    return b


def holy_site(lvl):
    def b():
        g = bl.plot('paveL')
        gold = M('gold')
        road(0, -0.85, 0, -0.1, g, 0.12, 'paveL')
        for (tx, ty) in [(-0.62, -0.42), (0.62, -0.42), (-0.66, 0.2), (0.66, 0.2)]:
            tree(tx, ty, 0.95, 'round', g)
        if lvl == 1:
            z = portico(0, 0.12, g, 0.62, 0.38, 0.19, 6, roof='terra')
            box(0, 0.12, g + 0.036, 0.46, 0.28, 0.19, M('marble'))
            for sx in (-1, 1):
                cyl(sx * 0.2, -0.28, g, 0.012, 0.06, M('bronze'), n=6)
                brazier(sx * 0.2, -0.28, g + 0.06)
            sphere(0, 0.12, z + 0.1, 0.03, gold, n=10, rings=6)
        else:
            portico(0, -0.18, g, 0.6, 0.3, 0.17, 6, roof='terra')
            box(0, -0.18, g + 0.036, 0.44, 0.22, 0.17, M('marble'))
            # domed sanctuary behind the temple
            cyl(0, 0.3, g, 0.21, 0.04, M('stone'), n=24)
            cyl(0, 0.3, g + 0.04, 0.17, 0.2, M('cream'), n=24)
            for k in range(10):
                a = 2 * math.pi * k / 10
                col(0.19 * math.cos(a), 0.3 + 0.19 * math.sin(a), g + 0.04, 0.012, 0.2, M('marble'), n=8)
            cyl(0, 0.3, g + 0.24, 0.2, 0.02, M('marble'), n=24)
            dome(0, 0.3, g + 0.26, 0.18, gold, h=0.2, n=24)
            cyl(0, 0.3, g + 0.46, 0.006, 0.08, gold, n=4)
        if lvl >= 3:
            for sx in (-1, 1):
                x = sx * 0.52
                cyl(x, 0.28, g, 0.04, 0.05, M('stone'), n=10)
                cyl(x, 0.28, g + 0.05, 0.026, 0.34, M('marble'), n=12, r2=0.02)
                cyl(x, 0.28, g + 0.39, 0.04, 0.012, gold, n=12)
                cone(x, 0.28, g + 0.402, 0.03, 0.1, gold, n=12)
                brazier(sx * 0.4, -0.45, g, 1.2)
            crowd(g, [(-0.1, -0.5), (0.08, -0.56), (0.02, -0.38), (-0.2, -0.3), (0.2, -0.32)], ('white', 'cream', 'purple', 'blue'))
    return b


def encampment(lvl):
    def b():
        g = bl.plot('dirt')
        R = 0.74
        pts = _hex_pts(R)
        wall_mat = 'woodL' if lvl < 3 else 'stone'
        for i in range(6):
            a, c2 = pts[i], pts[(i + 1) % 6]
            if i == 4:                                   # the gate faces the viewer (south-west edge)
                continue
            if lvl >= 3:
                wall_seg(a[0], a[1], c2[0], c2[1], g, h=0.12, t=0.05, wall='stone2')
            else:
                palisade(a[0], a[1], c2[0], c2[1], g, h=0.12 if lvl == 1 else 0.14, mat='woodL')
        ga, gb = pts[4], pts[5]
        for p in (ga, gb):
            tower(p[0], p[1], g, 0.05, 0.24, wall=wall_mat, roof='terraD', rh=0.11, crenel=(lvl >= 3))
        # tents round a fire
        brazier(0.0, 0.0, g, 0.9)
        cone(0.0, 0.0, g, 0.03, 0.035, M('dark'), n=8)
        for k, (tx, ty, rr) in enumerate([(-0.3, 0.22, 15), (0.3, 0.22, -15), (0.0, 0.42, 0), (-0.38, -0.12, 30), (0.38, -0.12, -30)][:3 + lvl]):
            tent(tx, ty, g, 0.22, 'cream' if k % 2 else 'sand', rot=rr, h=0.16)
            banner(tx + 0.0, ty + 0.1, g + 0.0, 0.2, 0.05, mat=M('red'))
        # drill ground: dummies and weapon racks
        for sx in (-1, 1):
            cyl(sx * 0.2, -0.3, g, 0.008, 0.07, M('woodD'), n=5)
            sphere(sx * 0.2, -0.3, g + 0.07, 0.022, M('thatch'), n=6, rings=4)
            box(sx * 0.2, -0.3, g + 0.045, 0.06, 0.008, 0.008, M('woodD'))
        for k in range(3):
            cyl(-0.05 + k * 0.05, -0.12, g, 0.003, 0.09, M('iron'), n=4)
        if lvl >= 2:
            box(0.0, 0.58, g, 0.4, 0.14, 0.1, M('woodL'))                      # barracks
            gable(0.0, 0.58, g + 0.1, 0.42, 0.16, 0.08, M('terraD'), overhang=0.01, mat_end=M('woodL'))
            windows(0, 0.58, g, 0.4, 0.14, 0.1, 4, 1, ww=0.03, wh=0.035, z0=0.5, z1=0.5, sides='f')
            for k in range(3):
                cyl(-0.5 + k * 0.05, 0.1, g, 0.01, 0.0, M('wood'), n=3)
            fence(0.42, 0.0, 0.42, -0.3, g, 0.06)
            for k, c in enumerate(['woodD', 'brickD']):
                with at(0.5, -0.1 - k * 0.12, g, 90):
                    box(0, 0, 0.03, 0.12, 0.06, 0.035, M('woodD'))
                    box(0, 0, 0.065, 0.1, 0.05, 0.02, M('thatch'))
        if lvl >= 3:
            box(0.0, 0.58, g + 0.18, 0.1, 0.1, 0.12, M('stone'))
            box(0.0, 0.58, g + 0.1, 0.2, 0.12, 0.08, M('stone'))
            tower(-0.45, 0.42, g, 0.06, 0.34, 'stone', crenel=True)
            banner(-0.45, 0.42, g + 0.34, 0.2, 0.06, mat=M('red'))
            # catapult
            with at(-0.46, -0.28, g, 25):
                box(0, 0, 0.012, 0.14, 0.08, 0.02, M('woodD'))
                box(-0.02, 0, 0.032, 0.012, 0.012, 0.09, M('wood'))
                box(0.01, 0, 0.06, 0.1, 0.01, 0.01, M('wood'), rot=0)
                for sy in (-1, 1):
                    cyl(0.0, sy * 0.045, 0.0, 0.022, 0.012, M('woodD'), n=10)
    return b


def commercial(lvl):
    def b():
        g = bl.plot('paveL')
        cols = ['red', 'blue', 'yellow', 'green', 'purple', 'orange']
        road(0, -0.85, 0, 0.1, g, 0.1, 'paveD')
        fountain(0, -0.1, g, 0.9)
        # market stalls with striped awnings
        spots = [(-0.36, -0.34, 12), (0.36, -0.34, -12), (-0.46, 0.04, 80), (0.46, 0.04, -80)]
        for k, (x, y, r) in enumerate(spots[: 2 + lvl]):
            with at(x, y, g, r):
                box(0, 0, 0, 0.14, 0.09, 0.04, M('woodL'))
                for ci in range(4):
                    box(-0.045 + ci * 0.03, 0.0, 0.04, 0.03, 0.08, 0.02, M(['orange', 'red', 'yellow', 'green'][(k + ci) % 4]))
                for sx in (-1, 1):
                    cyl(sx * 0.065, -0.04, 0, 0.005, 0.1, M('woodD'), n=4)
                    cyl(sx * 0.065, 0.04, 0, 0.005, 0.1, M('woodD'), n=4)
                for ci in range(6):
                    box(-0.067 + ci * 0.0268, 0.0, 0.1 + 0.0, 0.0268, 0.1, 0.01, M(cols[k % 6] if ci % 2 == 0 else 'white'), taper=1.0)
        crowd(g, [(-0.18, -0.4), (0.18, -0.2), (0.08, 0.06), (-0.12, 0.02), (0.3, -0.5)], ('red', 'blue', 'green', 'orange', 'purple'))
        for (cx_, cy_) in [(0.6, -0.3), (-0.62, -0.28)]:
            tree(cx_, cy_, 0.8, 'round', g)
        crate(0.22, -0.52, g, 0.05)
        crate(0.28, -0.5, g, 0.04, rot=25)
        barrel(0.15, -0.54, g)
        barrel(0.19, -0.5, g)
        cart(-0.22, -0.55, g, rot=-6, load='orange')
        # merchants' shops along both sides of the square, awnings out front
        walls = ['cream', 'sand', 'plaster', 'cream']
        for sx in (-1, 1):
            for k, y in enumerate((-0.12, 0.12, 0.34)[: 1 + lvl]):
                x = sx * 0.66
                with at(x, y, g, -sx * 90):
                    box(0, 0, 0, 0.18, 0.14, 0.13 + 0.02 * (k % 2), M(walls[(k + (sx > 0)) % 4]))
                    gable(0, 0, 0.13 + 0.02 * (k % 2), 0.2, 0.16, 0.07, M('terra'), overhang=0.01, mat_end=M('cream'))
                    windows(0, 0, 0, 0.18, 0.14, 0.13, 2, 1, ww=0.03, wh=0.035, z0=0.65, z1=0.65, sides='f')
                    door(0, 0, 0, 0.18, 0.14, 0.035, 0.06)
                    for ci in range(5):
                        box(-0.072 + ci * 0.036, -0.1, 0.07, 0.036, 0.06, 0.008,
                            M(cols[(k + ci + (sx > 0)) % 6] if ci % 2 == 0 else 'white'))
        # hall at the back
        if lvl == 1:
            box(0, 0.4, g, 0.34, 0.16, 0.12, M('cream'))
            gable(0, 0.4, g + 0.12, 0.36, 0.18, 0.1, M('terra'), overhang=0.012, mat_end=M('cream'))
            windows(0, 0.4, g, 0.34, 0.16, 0.12, 3, 1, ww=0.04, wh=0.05, z0=0.5, z1=0.5, sides='f')
            door(0, 0.4, g, 0.34, 0.16, 0.06, 0.08)
        else:
            box(0, 0.42, g, 0.6, 0.22, 0.2, M('cream'))
            box(0, 0.42, g + 0.1, 0.62, 0.23, 0.012, M('woodD'))
            windows(0, 0.42, g, 0.6, 0.22, 0.2, 6, 2, ww=0.032, wh=0.05, z0=0.3, z1=0.72, sides='f', frame=M('woodD'))
            gable(0, 0.42, g + 0.2, 0.64, 0.24, 0.13, M('terra'), overhang=0.014, mat_end=M('cream'))
            for k in range(7):
                col(-0.27 + k * 0.09, 0.28, g, 0.01, 0.1, M('woodD'), n=6, base=False, cap=False)
            box(0, 0.285, g + 0.1, 0.6, 0.02, 0.02, M('red'))
            banner(0.0, 0.3, g + 0.2, 0.13, 0.07, mat=M('red'))
        if lvl >= 3:
            box(0.55, 0.38, g, 0.22, 0.22, 0.26, M('marble'))
            for k in range(4):
                col(0.46 + k * 0.06, 0.26, g, 0.01, 0.2, M('marble'), n=8)
            dome(0.55, 0.38, g + 0.26, 0.1, M('gold'), h=0.1, n=14)
            glass_x, glass_y = -0.55, 0.36
            box(glass_x, glass_y, g, 0.15, 0.15, 0.48, M('glass', rough=0.06, metal=0.45))
            for i in range(10):
                box(glass_x, glass_y, g + 0.02 + i * 0.045, 0.154, 0.154, 0.006, M('white', rgb='#dfe6ec', style='plain'))
            box(glass_x, glass_y, g + 0.48, 0.16, 0.16, 0.014, M('steel'))
    return b


def theater(lvl):
    def b():
        g = bl.plot('paveL')
        marble, gold = M('marble'), M('gold')
        # semicircular seating bowl facing the stage (stage to the south)
        for t in range(5):
            ri, ro = 0.16 + 0.07 * t, 0.16 + 0.07 * (t + 1)
            angs = [math.radians(10 + 160 * i / 18) for i in range(19)]
            pts = [(ro * math.cos(a), 0.05 + ro * math.sin(a) * 0.95) for a in angs]
            pts += [(ri * math.cos(a), 0.05 + ri * math.sin(a) * 0.95) for a in reversed(angs)]
            ext(pts, g, 0.025 * (t + 1) + 0.01, M('stone' if t % 2 else 'cream'))
        cyl(0, 0.05, g, 0.16, 0.012, M('sandD'), n=24)
        box(0, -0.13, g, 0.42, 0.1, 0.035, M('woodL'))                                     # stage
        box(0, -0.2, g, 0.5, 0.05, 0.12, M('cream'))                                        # skene
        for k in range(5):
            col(-0.2 + k * 0.1, -0.235, g, 0.011, 0.12, marble, n=8)
        box(0, -0.2, g + 0.12, 0.52, 0.06, 0.015, marble)
        gable(0, -0.2, g + 0.135, 0.52, 0.065, 0.045, M('terra'), mat_end=marble)
        for sx in (-1, 1):
            banner(sx * 0.32, -0.3, g, 0.2, 0.06, mat=M('purple'))
            tree(sx * 0.62, -0.38, 0.9, 'round', g)
        if lvl >= 2:
            # colonnaded promenade behind the seating
            portico(0, 0.58, g, 0.8, 0.14, 0.14, 9, roof='terra', steps_n=1, pediment=False)
            box(0, 0.58, g + 0.18, 0.84, 0.16, 0.02, M('terra'))
            crowd(g, [(-0.2, -0.36), (0.2, -0.36), (0.0, -0.4)], ('purple', 'cream', 'red'))
        if lvl >= 3:
            for sx in (-1, 1):
                box(sx * 0.55, 0.15, g, 0.14, 0.26, 0.18, M('cream'))
                cyl(sx * 0.55, 0.15, g + 0.18, 0.06, 0.04, M('cream'), n=12)
                dome(sx * 0.55, 0.15, g + 0.22, 0.07, gold, h=0.08, n=12)
            for sx in (-1, 1):
                brazier(sx * 0.4, -0.45, g, 1.1)
    return b


SPRITES = {}
for _lv in (1, 2, 3):
    SPRITES[f'district_campus_{_lv}'] = (campus(_lv), False)
    SPRITES[f'district_holy_site_{_lv}'] = (holy_site(_lv), False)
    SPRITES[f'district_encampment_{_lv}'] = (encampment(_lv), False)
    SPRITES[f'district_commercial_hub_{_lv}'] = (commercial(_lv), False)
    SPRITES[f'district_theater_{_lv}'] = (theater(_lv), False)
