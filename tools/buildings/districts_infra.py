"""District sprites: harbours, industry, arenas, housing, waterworks."""
import math
import random

import bl
from bl import _face
from medieval import stone_house, timber_house
from ships import crane, dhow, galleon, lighthouse, rowboat
from town import *  # noqa: F401,F403


def quay(x, y, w, d, h, mat='stone'):
    box(x, y, 0, w, d, h, M(mat))
    box(x, y, h, w + 0.02, d + 0.02, 0.012, M('stone2'))
    for k in range(int(w / 0.07)):
        box(x - w / 2 + 0.035 + k * 0.07, y - d / 2 - 0.001, h * 0.3, 0.03, 0.004, h * 0.4, M('stoneD'))


def pier(x, y0, y1, w=0.16, z=0.04):
    L = y1 - y0
    box(x, (y0 + y1) / 2, z, w, L, 0.014, M('woodL'))
    for k in range(int(L / 0.05)):
        box(x, y0 + 0.025 + k * 0.05, z + 0.014, w, 0.004, 0.002, M('woodD'))
    for yy in [y0 + 0.04 + i * (L - 0.08) / 5 for i in range(6)]:
        for sx in (-1, 1):
            cyl(x + sx * (w / 2 + 0.006), yy, z - 0.07, 0.011, 0.11, M('woodD'), n=6)
    cyl(x, y0 + 0.02, z + 0.014, 0.012, 0.025, M('dark'), n=6)


def harbor(lvl):
    def b():
        quay(0, 0.45, 0.9, 0.3, 0.09)
        pier(0.12, -0.55, 0.3)
        # warehouse on the quay
        box(-0.2, 0.46, 0.1, 0.34, 0.2, 0.13, M('woodL'))
        gable(-0.2, 0.46, 0.23, 0.36, 0.22, 0.1, M('terra'), overhang=0.012, mat_end=M('woodL'))
        door(-0.2, 0.46, 0.1, 0.34, 0.2, 0.08, 0.09)
        windows(-0.2, 0.46, 0.1, 0.34, 0.2, 0.13, 2, 1, ww=0.04, wh=0.04, z0=0.6, z1=0.6, sides='f')
        for k, (cx_, cy_) in enumerate([(0.2, 0.34), (0.27, 0.36), (0.24, 0.42), (0.34, 0.38)]):
            crate(cx_, cy_, 0.1, 0.06 if k % 2 else 0.05, rot=15 * k)
        barrel(0.38, 0.5, 0.1)
        barrel(0.34, 0.52, 0.1)
        crane(0.1, 0.34, 0.1, rot=0, h=0.3)
        dhow(-0.45, -0.25, 0.0, L=0.34, rot=75)
        if lvl >= 2:
            galleon(0.42, -0.22, 0.0, L=0.52, rot=95, masts=2, hull_mat='woodD', sail_mat='cream', flag_mat='red')
            box(0.42, 0.34, 0.1, 0.2, 0.18, 0.1, M('cream'))
            hip(0.42, 0.34, 0.2, 0.24, 0.22, 0.08, M('terra'), ridge=0.5)
        if lvl >= 3:
            lighthouse(-0.62, 0.4, 0.09, h=0.46, r=0.065)
            galleon(-0.1, -0.5, 0.0, L=0.66, rot=0, masts=3, hull_mat='slate', sail_mat='white', flag_mat='blue')
            pier(-0.35, -0.2, 0.3, w=0.1)
            rowboat(0.3, -0.52, 0.0, rot=30)
    return b


def royal_navy_dockyard():
    def b():
        quay(0, 0.45, 0.9, 0.3, 0.09)
        pier(0.0, -0.6, 0.3, w=0.18)
        box(0, 0.45, 0.1, 0.62, 0.2, 0.16, M('brick'))
        windows(0, 0.45, 0.1, 0.62, 0.2, 0.16, 6, 2, ww=0.03, wh=0.045, z0=0.3, z1=0.75, sides='f')
        hip(0, 0.45, 0.26, 0.66, 0.24, 0.09, M('slate'), ridge=0.7)
        box(0, 0.45, 0.26, 0.12, 0.12, 0.14, M('brick'))
        pyr(0, 0.45, 0.4, 0.13, 0.13, 0.12, M('slate'))
        pennant(0, 0.45, 0.5, 0.14, 0.07, mat=M('red'))
        galleon(-0.34, -0.2, 0.0, L=0.66, rot=88, masts=3, hull_mat='royal', sail_mat='white', flag_mat='red')
        galleon(0.4, -0.28, 0.0, L=0.5, rot=95, masts=2, hull_mat='royal', sail_mat='white', flag_mat='red')
        crane(0.3, 0.3, 0.1, h=0.3)
        for k in range(4):
            sphere(0.14 + k * 0.03, 0.28, 0.1, 0.014, M('iron'), n=6, rings=4)
    return b


def industrial(lvl, style='industrial'):
    def b():
        g = bl.plot('paveD')
        road(0, -0.85, 0, -0.1, g, 0.1, 'asphalt')
        road(-0.7, -0.4, 0.7, -0.4, g, 0.08, 'asphalt')
        wall = 'brick' if style == 'industrial' else 'woodL'
        roofm = 'slate' if style == 'industrial' else 'terraD'

        def shed(x, y, w, d, h, rot=0.0, teeth=3):
            with at(x, y, g, rot):
                box(0, 0, 0, w, d, h, M(wall if style == 'industrial' else 'cream'))
                if style == 'industrial':
                    windows(0, 0, 0, w, d, h, max(2, int(w / 0.1)), 1, ww=0.035, wh=0.05, z0=0.55, z1=0.55, sides='f')
                    for k in range(teeth):
                        x0 = -w / 2 + k * w / teeth
                        _face(M(roofm), [(x0, -d / 2 - 0.008, h), (x0 + w / teeth, -d / 2 - 0.008, h + 0.1), (x0 + w / teeth, d / 2 + 0.008, h + 0.1), (x0, d / 2 + 0.008, h)])
                        _face(M('glass'), [(x0 + w / teeth, -d / 2 - 0.008, h + 0.1), (x0 + w / teeth, -d / 2 - 0.008, h), (x0 + w / teeth, d / 2 + 0.008, h), (x0 + w / teeth, d / 2 + 0.008, h + 0.1)])
                        for yy in (-d / 2 - 0.008, d / 2 + 0.008):
                            _face(M(wall), [(x0, yy, h), (x0 + w / teeth, yy, h), (x0 + w / teeth, yy, h + 0.1)])
                else:
                    for k in range(int(w / 0.07) + 1):
                        box(-w / 2 + k * w / int(w / 0.07), -d / 2 - 0.003, 0, 0.012, 0.006, h, M('woodD'))
                    box(0, -d / 2 - 0.003, h * 0.5, w, 0.006, 0.01, M('woodD'))
                    windows(0, 0, 0, w, d, h, max(2, int(w / 0.1)), 1, ww=0.035, wh=0.05, z0=0.55, z1=0.55, sides='f', frame=M('woodD'))
                    gable(0, 0, h, w + 0.02, d + 0.02, 0.1, M(roofm), overhang=0.012, mat_end=M('cream'))
                door(0, 0, 0, w, d, 0.07, 0.08)

        shed(-0.12, 0.18, 0.5, 0.3, 0.14)
        chimney(0.22, 0.32, g, 0.035, 0.42, smoke=4, mat='brick')
        crate(0.3, -0.12, g, 0.07)
        crate(0.38, -0.08, g, 0.05, rot=30)
        box(-0.2, -0.22, g, 0.3, 0.05, 0.05, M('iron'))                                  # rail wagon
        for sx in (-1, 1):
            cyl(-0.2 + sx * 0.08, -0.22, g - 0.0, 0.02, 0.05, M('dark'), n=8)
        if lvl >= 2:
            shed(0.0, 0.58, 0.56, 0.2, 0.12, teeth=2)
            for (tx, ty) in [(-0.5, 0.0), (-0.5, -0.2)]:
                cyl(tx, ty, g, 0.09, 0.16, M('steel'), n=16)
                cyl(tx, ty, g + 0.16, 0.09, 0.012, M('iron'), n=16)
            for k in range(3):
                cyl(0.5, 0.0 + k * 0.1, g, 0.04, 0.2, M('concrete'), n=10)
                dome(0.5, 0.0 + k * 0.1, g + 0.2, 0.04, M('concreteD'), h=0.03, n=10)
            crane(0.38, -0.38, g, rot=90, h=0.3, mat='iron')
        if lvl >= 3:
            lathe(0.52, 0.42, g, [(0.1, 0), (0.07, 0.15), (0.078, 0.3)], M('concrete'), n=18, cap0=False, cap1=False)
            smoke_plume(0.52, 0.42, g + 0.3, 4, 1.4)
            lathe(0.28, 0.5, g, [(0.1, 0), (0.07, 0.15), (0.078, 0.3)], M('concrete'), n=18, cap0=False, cap1=False)
            box(0.1, -0.02, g + 0.1, 0.55, 0.02, 0.02, M('copper'))
            for k in range(5):
                box(-0.1 + k * 0.11, -0.02, g, 0.012, 0.012, 0.11, M('iron'))
            box(-0.55, 0.3, g, 0.1, 0.1, 0.34, M('steel'), taper=0.6)
            cyl(-0.55, 0.3, g + 0.34, 0.07, 0.07, M('steel'), n=12)
    return b


def entertainment(lvl):
    def b():
        g = bl.plot('paveL')
        R = 0.44 if lvl == 1 else 0.5
        n = 16
        cyl(0, 0.04, g, R * 0.85, 0.02, M('sandD'), n=28)
        tiers = 2 if lvl >= 2 else 1
        for tier in range(tiers):
            for k in range(n):
                a = 2 * math.pi * k / n
                w = 2 * R * math.sin(math.pi / n) * 1.03
                arch_wall(R * math.cos(a), 0.04 + R * math.sin(a), g + 0.16 * tier, w, 0.16, 0.05,
                          M('cream' if tier == 0 else 'stone'), rot=math.degrees(a) - 90 + 180, iw=w * 0.55, ih=0.1)
        top = g + 0.16 * tiers
        def ring(rr, z):
            return [(rr * math.cos(2 * math.pi * k / 28), 0.04 + rr * math.sin(2 * math.pi * k / 28), z) for k in range(28)]
        loft([ring(R * 0.82, g + 0.012), ring(R * 0.9, g + 0.07), ring(R * 0.97, top - 0.01)], M('stone2'), cap0=False, cap1=False)
        loft([ring(R * 0.97, top - 0.01), ring(R * 1.03, top + 0.012)], M('stone'), cap0=False, cap1=False)
        for k in range(28):
            a = 2 * math.pi * k / 28
            if k % 2 == 0:
                box(R * 0.93 * math.cos(a), 0.04 + R * 0.93 * math.sin(a), g + 0.05, 0.012, 0.012, 0.012, M(['red', 'blue', 'cream'][k % 3]), rot=math.degrees(a))
        for k in range(6):
            a = math.radians(60 * k + 30)
            banner(R * 0.95 * math.cos(a), 0.04 + R * 0.95 * math.sin(a), top + 0.02, 0.16, 0.05, mat=M(['red', 'blue', 'yellow'][k % 3]))
        for sx in (-1, 1):
            tree(sx * 0.68, -0.42, 0.9, 'round', g)
        if lvl >= 2:
            stall(-0.52, -0.42, g, w=0.12, awning='red', rot=0)
            stall(0.52, -0.42, g, w=0.12, awning='blue', rot=0)
            crowd(g, [(-0.12, -0.48), (0.1, -0.5), (0.0, -0.58)], ('red', 'cream', 'blue'))
        if lvl >= 3:
            for sx in (-1, 1):
                for sy in (-1, 1):
                    cyl(sx * 0.62, 0.04 + sy * 0.34, g, 0.012, 0.42, M('steel'), n=6)
                    box(sx * 0.62, 0.04 + sy * 0.34, g + 0.42, 0.09, 0.02, 0.05, M('glow_fl', rgb='#fff6c8', emit='#fff2a0', emit_strength=5.0))
    return b


def neighborhood(lvl, style='neighborhood'):
    def b():
        g = bl.plot('paveL')
        rnd = random.Random(31 + lvl)
        if style == 'mbanza':
            for k, (x, y) in enumerate([(-0.35, 0.3), (0.35, 0.3), (-0.45, -0.1), (0.45, -0.1), (0.0, 0.5), (-0.2, -0.4), (0.2, -0.4)]):
                adobe_house(x, y, g, rnd, variant=2)
            fountain(0, 0, g, 0.7, 'stone')
            return
        road(0, -0.85, 0, 0.7, g, 0.09, 'paveD')
        road(-0.7, 0.0, 0.7, 0.0, g, 0.07, 'paveD')
        spots = [(-0.34, 0.36, 0), (0.34, 0.36, 0), (-0.34, -0.34, 0), (0.34, -0.34, 0)]
        if lvl >= 2:
            spots += [(-0.6, 0.02, 90), (0.6, 0.02, -90)]
        for k, (x, y, r) in enumerate(spots):
            if lvl == 1:
                timber_house(x, y, g, rnd, r, big=False)
            elif lvl == 2:
                timber_house(x, y, g, rnd, r, big=True)
            else:
                stone_house(x, y, g, rnd, r) if k % 3 == 1 else timber_house(x, y, g, rnd, r, big=True)
        if lvl >= 3:
            for (x, y) in [(-0.2, 0.62), (0.2, 0.62)]:
                box(x, y, g, 0.18, 0.15, 0.36, M('brick'))
                windows(x, y, g, 0.18, 0.15, 0.36, 3, 4, ww=0.03, wh=0.04, sides='f')
                hip(x, y, g + 0.36, 0.2, 0.17, 0.07, M('slate'), ridge=0.5)
        for (x, y) in [(-0.12, -0.12), (0.12, 0.12), (0.12, -0.12), (-0.12, 0.12)]:
            tree(x, y, 0.55, 'round', g)
        lamp(-0.07, -0.55, g)
        lamp(0.07, 0.2, g)
        crowd(g, [(0.03, -0.3), (-0.04, 0.12)], ('orange', 'cream'))
    return b


def aqueduct(style='aqueduct'):
    def b():
        g = bl.plot('grass' if style == 'aqueduct' else 'paveL')
        ang = 28
        n = 6
        L = 1.4
        for i in range(n):
            t = (i + 0.5) / n - 0.5
            arch_wall(t * L * math.cos(math.radians(ang)), t * L * math.sin(math.radians(ang)), g, L / n * 1.02, 0.36, 0.11,
                      M('stone'), rot=ang, iw=L / n * 0.64, ih=0.25)
        with at(0, 0, g + 0.36, ang):
            box(0, 0, 0, L * 1.02, 0.16, 0.035, M('stone2'))
            box(0, 0, 0.035, L * 0.99, 0.1, 0.008, M('water', rough=0.04))
            for sy in (-1, 1):
                box(0, sy * 0.075, 0.035, L * 1.02, 0.018, 0.035, M('stone'))
        if style == 'bath':
            pool(0.0, -0.42, g, 0.54, 0.28, rim='marble')
            for i in range(5):
                col(-0.22 + 0.11 * i, -0.6, g, 0.015, 0.16, M('marble'))
            box(0, -0.6, g + 0.16, 0.56, 0.05, 0.02, M('marble'))
            gable(0, -0.6, g + 0.18, 0.56, 0.06, 0.05, M('terra'), mat_end=M('marble'))
        else:
            box(-0.4, -0.38, g, 0.26, 0.2, 0.1, M('stone'))
            cyl(-0.4, -0.38, g + 0.1, 0.08, 0.04, M('stone2'), n=12)
            for (x, y, s_) in [(0.45, 0.45, 1.0), (-0.5, 0.4, 0.9), (0.42, -0.45, 0.9)]:
                tree(x, y, s_, 'round', g)
    return b


SPRITES = {}
for _lv in (1, 2, 3):
    SPRITES[f'district_harbor_{_lv}'] = (harbor(_lv), False)
    SPRITES[f'district_industrial_zone_{_lv}'] = (industrial(_lv), False)
    SPRITES[f'district_entertainment_complex_{_lv}'] = (entertainment(_lv), False)
    SPRITES[f'district_neighborhood_{_lv}'] = (neighborhood(_lv), False)
SPRITES['district_aqueduct_1'] = (aqueduct(), False)
SPRITES['district_bath_1'] = (aqueduct('bath'), False)
SPRITES['district_hansa_1'] = (industrial(2, 'hansa'), False)
SPRITES['district_mbanza_1'] = (neighborhood(1, 'mbanza'), False)
SPRITES['district_royal_navy_dockyard_1'] = (royal_navy_dockyard(), False)
