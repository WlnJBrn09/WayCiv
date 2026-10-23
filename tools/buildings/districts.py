"""District sprites: district_<name>_<level> (level 1 to 3 = how many of the
district's buildings the city has: none, one, two or more)."""
import math

import bl
from bl import _face, _hex_pts
from parts import *  # noqa: F401,F403


def arc_steps(cx, cy, z, r0, dr, tiers, a0, a1, th, mat, seg=16):
    """Rising ring-sector steps (amphitheatre seating)."""
    for t in range(tiers):
        ri, ro = r0 + dr * t, r0 + dr * (t + 1)
        angs = [math.radians(a0 + (a1 - a0) * i / seg) for i in range(seg + 1)]
        pts = [(cx + ro * math.cos(a), cy + ro * math.sin(a)) for a in angs]
        pts += [(cx + ri * math.cos(a), cy + ri * math.sin(a)) for a in reversed(angs)]
        ext(pts, z, th * (t + 1), mat)


def trees_around(spots, kind='round', s=1.0, z=0.0):
    for (x, y) in spots:
        tree(x, y, s, kind, z)


def sawtooth(x, y, z, w, d, rot=0.0, n=3, mat='slate'):
    th = 0.1
    with at(x, y, z, rot):
        for i in range(n):
            sx0 = -w / 2 + i * w / n
            sx1 = sx0 + w / n
            _face(M(mat), [(sx0, -d / 2, 0), (sx1, -d / 2, th), (sx1, d / 2, th), (sx0, d / 2, 0)])
            _face(M('glass'), [(sx1, -d / 2, th), (sx1, -d / 2, 0), (sx1, d / 2, 0), (sx1, d / 2, th)])
            for yy in (-d / 2, d / 2):
                _face(M('brick'), [(sx0, yy, 0), (sx1, yy, 0), (sx1, yy, th)])


# ---------------------------------------------------------------------------

def campus(lvl):
    def b():
        g = plot('paveL')
        lawn_patch(0.0, -0.4, g, 0.95, 0.22)
        path(0, -0.55, 0, 0.05, g)
        trees_around([(-0.58, -0.38), (0.58, -0.38), (-0.5, -0.5), (0.5, -0.5)], s=1.0, z=g)
        lx, ly = (-0.3, 0.2) if lvl >= 2 else (0.0, 0.12)
        house(lx, ly, g, 0.5, 0.32, 0.22, wall='cream', roof='slate', rh=0.14, floors=2)
        for i in range(4):
            col(lx - 0.18 + i * 0.12, ly - 0.2, g, 0.017, 0.2, M('marble'))
        box(lx, ly - 0.2, g + 0.2, 0.5, 0.06, 0.02, M('marble'))
        gable(lx, ly - 0.2, g + 0.22, 0.5, 0.07, 0.07, M('slate'), mat_end=M('marble'))
        if lvl >= 2:
            ux, uy = 0.33, 0.2
            box(ux, uy, g, 0.4, 0.34, 0.32, M('brick'))
            windows(ux, uy, g, 0.4, 0.34, 0.32, 4, 3)
            box(ux, uy, g + 0.32, 0.44, 0.38, 0.02, M('stone'))
            cyl(ux, uy, g + 0.34, 0.12, 0.1, M('stone'), n=12)
            dome(ux, uy, g + 0.44, 0.13, M('verdi'), h=0.14, n=14)
            cone(ux, uy, g + 0.58, 0.012, 0.07, M('gold'), n=6)
            box(ux, uy - 0.2, g, 0.16, 0.06, 0.2, M('marble'))
        if lvl >= 3:
            box(-0.28, -0.2, g, 0.34, 0.2, 0.18, M('glass', rough=0.08, metal=0.2))
            box(-0.28, -0.2, g + 0.18, 0.36, 0.22, 0.02, M('steel'))
            dish(-0.2, -0.2, g + 0.2, 0.06, 0)
            cyl(0.4, -0.3, g, 0.1, 0.16, M('white'), n=14)
            dome(0.4, -0.3, g + 0.16, 0.1, M('white', rough=0.3), h=0.09, n=14)
            box(0.4, -0.37, g + 0.2, 0.025, 0.05, 0.1, M('dark'))
    return b


def holy_site(lvl, style='temple'):
    def b():
        g = plot('paveL')
        trees_around([(-0.6, -0.4), (0.6, -0.4), (-0.62, 0.28), (0.62, 0.28)], s=0.9, z=g)
        gold = M('gold')
        if lvl == 1:
            temple(0, -0.02, g, 0.58, 0.42, ncols=4, ch=0.17, roof='terra')
            sphere(0.0, 0.0, g + 0.46, 0.04, gold, n=10, rings=6)
            lawn_patch(0, -0.45, g, 0.5, 0.16)
        else:
            temple(0, -0.2, g, 0.62, 0.38, ncols=4, ch=0.15, roof='terra')
            cyl(0, 0.26, g, 0.24, 0.05, M('stone'), n=20)
            cyl(0, 0.26, g + 0.05, 0.19, 0.2, M('marble'), n=20)
            for k in range(8):
                a = 2 * math.pi * k / 8
                col(0.2 * math.cos(a), 0.26 + 0.2 * math.sin(a), g + 0.05, 0.016, 0.2, M('marble'), n=8)
            dome(0, 0.26, g + 0.25, 0.2, gold, h=0.2, n=20)
            cone(0, 0.26, g + 0.45, 0.012, 0.1, gold, n=6)
        if lvl >= 3:
            for sx in (-1, 1):
                x = sx * 0.52
                cyl(x, 0.12, g, 0.05, 0.4, M('marble'), n=10, r2=0.035)
                cyl(x, 0.12, g + 0.4, 0.052, 0.02, gold, n=10)
                cone(x, 0.12, g + 0.42, 0.045, 0.14, gold, n=10)
                glow(x, 0.12 - 0.03, g + 0.29, 0.02, 0.01, 0.04, '#ffdf9a', strength=4)
    return b


def lavra():
    """Russian: onion domes."""
    def b():
        g = plot('paveL')
        trees_around([(-0.6, -0.4), (0.6, -0.4), (-0.6, 0.3)], kind='pine', z=g)
        box(0, 0.05, g, 0.42, 0.36, 0.3, M('white'))
        windows(0, 0.05, g, 0.42, 0.36, 0.3, 3, 2, z0=0.4, z1=0.7)
        for (x, y, r, h) in [(0, 0.05, 0.1, 0.3), (-0.2, 0.2, 0.06, 0.2), (0.2, 0.2, 0.06, 0.2),
                             (-0.2, -0.1, 0.06, 0.2), (0.2, -0.1, 0.06, 0.2)]:
            if x != 0:
                box(x, y, g, 0.14, 0.14, 0.2, M('white'))
            top = g + h
            cyl(x, y, top, r * 0.85, 0.07, M('white'), n=12)
            lathe(x, y, top + 0.07, [(r * 0.85, 0), (r * 1.35, r * 0.9), (r * 1.1, r * 1.8), (r * 0.5, r * 2.5), (0.0, r * 3.1)],
                  M('verdi' if x else 'gold'), n=14)
            cyl(x, y, top + 0.07 + r * 3.0, 0.004, 0.06, M('gold'), n=4)
    return b


def encampment(lvl):
    def b():
        g = plot('earth', top='dirt', rim='stone2')
        pts = _hex_pts(0.72)
        for i in range(6):
            a, c2 = pts[i], pts[(i + 1) % 6]
            if i == 2:
                continue  # gate
            if lvl >= 3:
                wall_seg(a[0], a[1], c2[0], c2[1], g, h=0.1, t=0.045, wall='stone2')
            else:
                palisade(a[0], a[1], c2[0], c2[1], g, h=0.1 if lvl == 1 else 0.12)
        for i in (2, 3):
            tower(pts[i][0], pts[i][1], g, 0.05, 0.2, wall='woodL' if lvl < 3 else 'stone', roof='terraD', rh=0.1)
        cyl(0.0, -0.05, g, 0.035, 0.012, M('dark'), n=8)
        for k in range(5):
            a = 2 * math.pi * k / 5
            cone(0.02 * math.cos(a), -0.05 + 0.02 * math.sin(a), g + 0.012, 0.012, 0.05,
                 M('orange', emit='#ff8a30', emit_strength=6), n=5)
        tent(-0.3, 0.2, g, 0.2, 'cream', rot=10)
        tent(0.3, 0.2, g, 0.2, 'cream', rot=-10)
        tent(-0.02, 0.38, g, 0.22, 'sandD', rot=0)
        flag(0.0, 0.52, g, 0.3, M('red'), w=0.12)
        for sx in (-1, 1):
            cyl(sx * 0.3, -0.25, g, 0.01, 0.1, M('woodD'), n=5)
            sphere(sx * 0.3, -0.25, g + 0.08, 0.025, M('thatch'), n=6, rings=4)
        if lvl >= 2:
            house(0.42, -0.08, g, 0.3, 0.2, 0.12, wall='woodL', roof='terraD', rot=-20, rh=0.1, floors=1)
            house(-0.42, -0.08, g, 0.3, 0.2, 0.12, wall='woodL', roof='terraD', rot=20, rh=0.1, floors=1)
        if lvl >= 3:
            box(0.0, -0.4, g, 0.34, 0.14, 0.14, M('stone'))
            gable(0.0, -0.4, g + 0.14, 0.34, 0.14, 0.09, M('terraD'))
            tower(0.0, 0.6, g, 0.06, 0.3, 'stone', crenel=True)
            flag(0.0, 0.6, g + 0.3, 0.14, M('red'), w=0.08)
    return b


def thanh():
    def b():
        g = plot('grassD', top='grass', rim='dirt')
        pts = _hex_pts(0.72)
        for i in range(6):
            a, c2 = pts[i], pts[(i + 1) % 6]
            wall_seg(a[0], a[1], c2[0], c2[1], g, h=0.07, t=0.1, wall='earth', crenel=False)
        tree(-0.5, 0.2, 0.8, 'palm', z=g)
        tree(0.5, 0.2, 0.8, 'palm', z=g)
        hut(-0.2, 0.1, g, 0.1, 0.1)
        hut(0.22, 0.12, g, 0.1, 0.1)
        house(0.0, -0.18, g, 0.3, 0.2, 0.1, wall='adobe', roof='thatch', rh=0.14, floors=1)
        flag(0.0, 0.36, g, 0.3, M('red'), w=0.12)
        cyl(0.0, 0.36, g, 0.04, 0.1, M('stone'), n=8)
    return b


def oppidum():
    def b():
        g = plot('dirt', top='dirt', rim='earth')
        pts = _hex_pts(0.74)
        for i in range(6):
            if i == 2:
                continue
            a, c2 = pts[i], pts[(i + 1) % 6]
            palisade(a[0], a[1], c2[0], c2[1], g, h=0.14, mat='wood')
        for (x, y) in [(-0.3, 0.2), (0.25, 0.25), (0.0, -0.1), (-0.35, -0.2), (0.38, -0.12)]:
            hut(x, y, g, 0.1, 0.08, wall='woodL', roof='thatch')
        flag(0.0, 0.5, g, 0.3, M('blue'), w=0.12)
    return b


def commercial(lvl):
    def b():
        g = plot('paveL')
        cols = ['red', 'blue', 'yellow', 'green', 'purple', 'orange']
        trees_around([(-0.62, 0.3), (0.62, 0.3)], s=0.8, z=g)
        cyl(0, -0.02, g, 0.08, 0.05, M('stone'), n=14)
        cyl(0, -0.02, g + 0.05, 0.06, 0.004, M('water', rough=0.05), n=14)
        stall(-0.38, -0.28, g, awning=cols[0], goods='orange', rot=10)
        stall(0.38, -0.28, g, awning=cols[1], goods='yellow', rot=-10)
        if lvl == 1:
            stall(-0.36, 0.22, g, awning=cols[2], goods='red', rot=-5)
            stall(0.36, 0.22, g, awning=cols[3], goods='orange', rot=5)
            crate(0.02, 0.35, g)
            crate(0.08, 0.38, g, 0.04)
            barrel(-0.07, 0.36, g)
        else:
            house(0, 0.4, g, 0.62, 0.28, 0.24, wall='cream', roof='terra', rh=0.15, floors=2)
            for i in range(5):
                box(-0.24 + i * 0.12, 0.23, g, 0.05, 0.03, 0.17, M('woodL'))
            box(0, 0.23, g + 0.17, 0.62, 0.05, 0.02, M(cols[0]))
            gable(0, 0.23, g + 0.19, 0.62, 0.07, 0.04, M(cols[0]))
            stall(-0.46, 0.0, g, awning=cols[2], goods='red', rot=90)
            stall(0.46, 0.0, g, awning=cols[3], goods='orange', rot=90)
        if lvl >= 3:
            skyscraper(-0.52, 0.36, g, 0.14, 0.14, 0.5, body='glass')
            box(0.55, 0.34, g, 0.22, 0.22, 0.26, M('marble'))
            dome(0.55, 0.34, g + 0.26, 0.09, M('gold'), h=0.1, n=14)
    return b


def _pier():
    box(0, -0.3, 0.03, 0.2, 0.8, 0.03, M('woodL'))
    box(0, 0.05, 0.034, 0.75, 0.16, 0.03, M('woodL'))
    for (px, py) in [(-0.09, -0.65), (0.09, -0.65), (-0.09, -0.35), (0.09, -0.35), (-0.35, 0.0), (0.35, 0.0),
                     (-0.35, 0.1), (0.35, 0.1)]:
        cyl(px, py, -0.05, 0.016, 0.1, M('woodD'), n=6)


def harbor(lvl):
    def b():
        bl.USE_CATCHER = True
        g = 0.0
        _pier()
        box(0, 0.4, g, 0.5, 0.25, 0.1, M('stone'))
        crate(-0.2, 0.0, g + 0.06, 0.06)
        crate(-0.12, 0.06, g + 0.06, 0.05, rot=20)
        barrel(0.25, 0.06, g + 0.06)
        barrel(0.3, 0.02, g + 0.06)
        ship(-0.5, -0.38, g + 0.01, rot=70, s=1.0, sail=True)
        if lvl >= 2:
            house(0.0, 0.42, g + 0.1, 0.42, 0.2, 0.14, wall='cream', roof='terra', rh=0.12, floors=1)
            cyl(0.4, 0.15, g + 0.06, 0.02, 0.22, M('iron'), n=6)
            box(0.4, 0.03, g + 0.26, 0.02, 0.28, 0.02, M('iron'))
            ship(0.52, -0.35, g + 0.01, rot=100, s=0.9, sail=True, hull='brickD')
        if lvl >= 3:
            cyl(-0.38, 0.45, g + 0.1, 0.07, 0.3, M('white'), n=12, r2=0.045)
            cyl(-0.38, 0.45, g + 0.25, 0.062, 0.05, M('red'), n=12, r2=0.055)
            cyl(-0.38, 0.45, g + 0.4, 0.05, 0.05, M('dark'), n=12)
            cone(-0.38, 0.45, g + 0.45, 0.06, 0.07, M('red'), n=12)
            glow(-0.38, 0.45, g + 0.33, 0.03, 0.03, 0.04, '#fff2b0', strength=8)
            ship(0.0, -0.62, g + 0.01, rot=90, s=1.3, sail=True, hull='slate', sail_mat='cream')
    return b


def royal_navy_dockyard():
    def b():
        bl.USE_CATCHER = True
        _pier()
        box(0, 0.4, 0.0, 0.62, 0.3, 0.12, M('stone'))
        house(0, 0.42, 0.12, 0.5, 0.2, 0.16, wall='brick', roof='slate', rh=0.12)
        ship(-0.12, -0.45, 0.01, L=0.55, W=0.14, rot=90, s=1.4, hull='royal', sail=True)
        ship(0.4, -0.25, 0.01, rot=110, s=1.0, hull='royal')
        flag(0.3, 0.42, 0.28, 0.2, M('red'), w=0.1)
    return b


def theater(lvl):
    def b():
        g = plot('paveL')
        marble, gold = M('marble'), M('gold')
        trees_around([(-0.6, -0.4), (0.6, -0.4)], s=0.9, z=g)
        arc_steps(0, 0.0, g, 0.18, 0.07, 4, 20, 160, 0.03, M('stone'))
        arc_steps(0, 0.0, g, 0.0, 0.18, 1, 180, 360, 0.012, M('sandD'))
        box(0, -0.32, g, 0.42, 0.1, 0.03, M('woodL'))
        if lvl >= 2:
            for i in range(5):
                col(-0.3 + i * 0.15, 0.46, g, 0.016, 0.22, marble)
            box(0, 0.46, g + 0.22, 0.7, 0.07, 0.025, marble)
            gable(0, 0.46, g + 0.245, 0.7, 0.08, 0.06, M('terra'), mat_end=marble)
        if lvl >= 3:
            for sx in (-1, 1):
                box(sx * 0.5, 0.0, g, 0.16, 0.22, 0.2, M('cream'))
                dome(sx * 0.5, 0.0, g + 0.2, 0.09, gold, h=0.1, n=12)
        flag(-0.3, -0.35, g, 0.2, M('purple'), w=0.1)
        flag(0.3, -0.35, g, 0.2, M('purple'), w=0.1)
    return b


def acropolis():
    def b():
        g = plot('grassD', top='grass')
        ext(_hex_pts(0.72), g, 0.14, M('rock'))
        ext(_hex_pts(0.66), g + 0.14, 0.02, M('paveL'))
        z = g + 0.16
        temple(0, 0.05, z, 0.72, 0.46, ncols=6, ch=0.2, roof='terra')
        for k in range(3):
            tree(-0.55 + 0.55 * k, -0.45, 0.8, 'round', z=g)
        flag(0.0, -0.3, z, 0.18, M('purple'), w=0.1)
    return b


def industrial(lvl, style='industrial'):
    def b():
        g = plot('concreteD', top='concrete', rim='stone2')
        if style == 'hansa':
            def fac(x, y, w, d, h, rot=0.0):
                box(x, y, g, w, d, h, M('white'), rot)
                for i in range(5):
                    box(x - w / 2 + (i + 0.5) * w / 5, y, g, 0.012, d * 1.01, h * 1.005, M('woodD'), rot)
                gable(x, y, g + h, w, d, d * 0.4, M('terraD'), rot, overhang=0.02)
                windows(x, y, g, w, d, h, 3, 1, rot=rot)
        else:
            def fac(x, y, w, d, h, rot=0.0):
                box(x, y, g, w, d, h, M('brick'), rot)
                windows(x, y, g, w, d, h, 4, 1, rot=rot, z0=0.6)
                sawtooth(x, y, g + h, w, d, rot)
        if lvl == 1:
            fac(-0.1, 0.1, 0.5, 0.32, 0.16)
            chimney(0.25, 0.25, g, 0.04, 0.36, smoke=2)
            crate(0.3, -0.2, g, 0.07)
            crate(0.38, -0.12, g, 0.05, rot=30)
            box(-0.15, -0.35, g, 0.3, 0.08, 0.05, M('iron'))
        else:
            fac(-0.12, 0.2, 0.62, 0.36, 0.2)
            chimney(0.3, 0.38, g, 0.04, 0.46, smoke=3)
            chimney(0.4, 0.28, g, 0.035, 0.38, smoke=1)
            cyl(-0.42, -0.2, g, 0.1, 0.22, M('steel'), n=14)
            cyl(-0.42, -0.2, g + 0.22, 0.1, 0.02, M('iron'), n=14)
            cyl(0.0, -0.3, g, 0.07, 0.17, M('steel'), n=14)
            box(0.35, -0.12, g, 0.2, 0.14, 0.1, M('orange'))
            box(0.2, -0.38, g, 0.22, 0.08, 0.03, M('iron'))
        if lvl >= 3:
            lathe(0.5, 0.0, g, [(0.13, 0), (0.095, 0.18), (0.11, 0.34)], M('concrete'), n=16, cap0=False, cap1=False)
            smokestack_puff(0.5, 0.0, g + 0.34, 1.2)
            box(0.1, -0.05, g + 0.08, 0.5, 0.025, 0.025, M('copper'))
            cyl(-0.5, 0.2, g, 0.012, 0.3, M('iron'), n=6)
            box(-0.4, 0.2, g + 0.3, 0.25, 0.012, 0.02, M('iron'))
    return b


def entertainment(lvl):
    def b():
        g = plot('paveL')
        R = 0.4 if lvl == 1 else 0.5
        n = 14
        cyl(0, 0.04, g, R * 0.85, 0.03, M('sandD'), n=24)
        for tier in range(2 if lvl >= 2 else 1):
            for k in range(n):
                a = 2 * math.pi * k / n
                w = 2 * R * math.sin(math.pi / n) * 1.02
                arch_wall(R * math.cos(a), 0.04 + R * math.sin(a), g + 0.17 * tier, w, 0.17, 0.05,
                          M('stone' if tier else 'cream'), rot=math.degrees(a) - 90 + 180, iw=w * 0.5, ih=0.1)
        top = g + (0.34 if lvl >= 2 else 0.17)
        cyl(0, 0.04, top, R * 1.0, 0.025, M('stone2'), n=24, r2=R * 0.85, cap1=False)
        for k in range(4):
            a = math.radians(90 * k + 45)
            flag(R * 0.95 * math.cos(a), 0.04 + R * 0.95 * math.sin(a), top, 0.14, M(['red', 'blue', 'yellow', 'green'][k]), w=0.08)
        if lvl >= 3:
            for sx in (-1, 1):
                for sy in (-1, 1):
                    cyl(sx * 0.62, sy * 0.4 + 0.04, g, 0.012, 0.4, M('steel'), n=6)
                    box(sx * 0.62, sy * 0.4 + 0.04, g + 0.4, 0.08, 0.02, 0.04,
                        M('yellow', emit='#fff2a0', emit_strength=3))
        if lvl >= 2:
            stall(-0.58, -0.5, g, w=0.12, awning='red', rot=0)
            stall(0.58, -0.5, g, w=0.12, awning='blue', rot=0)
    return b


def street_carnival():
    def b():
        g = plot('paveL')
        colors = ['red', 'yellow', 'green', 'blue', 'purple', 'orange']
        for k in range(6):
            a = math.radians(30 + 60 * k)
            tent(0.5 * math.cos(a), 0.45 * math.sin(a) - 0.02, g, 0.18, colors[k], rot=math.degrees(a) + 90, h=0.14)
        cyl(0, 0, g, 0.18, 0.04, M('woodL'), n=16)
        cyl(0, 0, g + 0.04, 0.02, 0.28, M('woodD'), n=6)
        for k in range(8):
            a = 2 * math.pi * k / 8
            box(0.16 * math.cos(a), 0.16 * math.sin(a), g + 0.04, 0.03, 0.03, 0.12, M(colors[k % 6]))
        sphere(0, 0, g + 0.32, 0.05, M('gold'), n=10, rings=6)
        flag(0.0, -0.3, g, 0.3, M('green'), w=0.14)
    return b


def aqueduct(style='aqueduct'):
    def b():
        g = plot('grass' if style == 'aqueduct' else 'paveL', top='grass' if style == 'aqueduct' else 'paveL')
        ang = 25
        n = 6
        L = 1.35
        for i in range(n):
            t = (i + 0.5) / n - 0.5
            arch_wall(t * L * math.cos(math.radians(ang)), t * L * math.sin(math.radians(ang)), g, L / n * 1.02, 0.34, 0.1,
                      M('stone'), rot=ang, iw=L / n * 0.62, ih=0.22)
        with at(0, 0, g + 0.34, ang):
            box(0, 0, 0, L * 1.02, 0.15, 0.04, M('stone2'))
            box(0, 0, 0.04, L * 0.99, 0.1, 0.01, M('water', rough=0.05))
            box(0, 0.075, 0.04, L * 1.02, 0.015, 0.03, M('stone'))
            box(0, -0.075, 0.04, L * 1.02, 0.015, 0.03, M('stone'))
        if style == 'bath':
            pool(0.0, -0.4, g, 0.5, 0.26, rim='marble')
            for i in range(4):
                col(-0.2 + 0.133 * i, -0.56, g, 0.016, 0.16, M('marble'))
            box(0, -0.56, g + 0.16, 0.5, 0.05, 0.02, M('marble'))
        else:
            box(-0.35, -0.35, g, 0.22, 0.18, 0.1, M('stone'))
            cyl(-0.35, -0.35, g + 0.1, 0.07, 0.04, M('stone2'), n=12)
            tree(0.45, 0.45, 1.0, 'round', z=g)
            tree(-0.5, 0.4, 0.9, 'round', z=g)
            tree(0.4, -0.45, 0.9, 'round', z=g)
    return b


def neighborhood(lvl, style='neighborhood'):
    def b():
        g = plot('pave', top='paveD', rim='stone2')
        road_strip(-0.72, 0, 0.72, 0, 0.1, g, M('asphalt'))
        road_strip(0, -0.78, 0, 0.78, 0.1, g, M('asphalt'))
        walls = ['cream', 'rose', 'sand', 'adobe', 'stone', 'cream']
        roofs = ['terra', 'slate', 'terraD', 'brick']
        spots1 = [(-0.35, 0.3), (0.35, 0.3), (-0.35, -0.3), (0.35, -0.3)]
        spots2 = [(-0.6, 0.3), (0.6, 0.3), (-0.6, -0.3), (0.6, -0.3)]
        if style == 'mbanza':
            for (x, y) in spots1 + [(-0.62, 0.05), (0.62, 0.05), (0.0, 0.55), (0.0, -0.55)]:
                hut(x, y, g, 0.1, 0.09)
            return
        for i, (x, y) in enumerate(spots1):
            house(x, y, g, 0.28, 0.24, 0.14 + 0.02 * (i % 2), wall=walls[i], roof=roofs[i % 4], rh=0.1, floors=1)
        if lvl >= 2:
            for i, (x, y) in enumerate(spots2):
                house(x, y, g, 0.18, 0.16, 0.22, wall=walls[(i + 2) % 6], roof=roofs[(i + 1) % 4], rh=0.1, floors=2)
        if lvl >= 3:
            for (x, y) in [(-0.2, 0.58), (0.2, 0.58)]:
                box(x, y, g, 0.2, 0.18, 0.42, M('concrete'))
                windows(x, y, g, 0.2, 0.18, 0.42, 3, 4)
                box(x, y, g + 0.42, 0.21, 0.19, 0.015, M('slate'))
        trees_around([(-0.12, -0.12), (0.12, 0.12), (0.12, -0.12), (-0.12, 0.12)], s=0.6, z=g)
    return b


def spaceport(lvl):
    def b():
        g = plot('concreteD', top='concrete', rim='stone2')
        cyl(0.0, 0.0, g, 0.38, 0.015, M('asphalt'), n=24)
        cyl(0.0, 0.0, g + 0.015, 0.2, 0.004, M('yellow'), n=24)
        cyl(0.0, 0.0, g + 0.019, 0.15, 0.004, M('asphalt'), n=24)
        box(-0.5, -0.38, g, 0.3, 0.22, 0.14, M('white'))
        box(-0.5, -0.38, g + 0.14, 0.18, 0.14, 0.1, M('glass', rough=0.1))
        windows(-0.5, -0.38, g, 0.3, 0.22, 0.14, 4, 1)
        if lvl >= 2:
            rocket(0.0, 0.05, g + 0.015, h=0.62, r=0.065)
            for i in range(10):
                z = g + 0.05 + i * 0.06
                box(0.14, 0.05, z, 0.025, 0.025, 0.06, M('red'))
                if i % 2 == 0:
                    box(0.08, 0.05, z + 0.03, 0.14, 0.01, 0.01, M('red'))
        else:
            for k in range(4):
                a = math.radians(45 + 90 * k)
                cyl(0.28 * math.cos(a), 0.28 * math.sin(a), g + 0.015, 0.012, 0.25, M('red'), n=6)
            box(0.0, 0.0, g + 0.2, 0.12, 0.12, 0.02, M('steel'))
            cyl(0.0, 0.0, g + 0.22, 0.04, 0.25, M('white'), n=10, r2=0.0)
        if lvl >= 3:
            for (x, y) in [(0.52, -0.3), (0.58, -0.12)]:
                cyl(x, y, g, 0.07, 0.2, M('white'), n=12)
                dome(x, y, g + 0.2, 0.07, M('white'), h=0.05, n=12)
            box(0.45, 0.4, g, 0.34, 0.2, 0.32, M('white'))
            box(0.45, 0.4, g + 0.32, 0.36, 0.22, 0.02, M('steel'))
            dish(-0.45, 0.35, g, 0.09, 0)
            dish(-0.32, 0.45, g, 0.07, 0)
    return b


def aerodrome(lvl):
    def b():
        g = plot('grassD', top='grass', rim='stone2')
        with at(0, 0, g, 28):
            box(0, 0, 0, 1.55, 0.24, 0.012, M('asphalt'))
            for i in range(9):
                box(-0.62 + i * 0.155, 0, 0.012, 0.07, 0.012, 0.003, M('white'))
        with at(-0.4, 0.4, g, 28):
            n = 12
            rings = []
            for xx in (-0.17, 0.17):
                rings.append([(xx, 0.14 * math.cos(math.pi * i / n), 0.1 * math.sin(math.pi * i / n)) for i in range(n + 1)])
            loft(rings, M('steel'), smooth=True, closed=False, cap0=True, cap1=True)
            box(-0.17, 0, 0.0, 0.01, 0.2, 0.09, M('dark'))
        cyl(0.42, 0.3, g, 0.03, 0.22, M('white'), n=8)
        box(0.42, 0.3, g + 0.22, 0.1, 0.1, 0.06, M('glass'))
        box(0.42, 0.3, g + 0.28, 0.12, 0.12, 0.012, M('red'))
        plane(0.1, -0.1, g + 0.02, rot=28, s=1.2, body='white', wing='blue')
        if lvl >= 2:
            plane(-0.35, 0.05, g + 0.02, rot=28, s=1.0, body='cream', wing='red')
            box(0.5, -0.2, g, 0.12, 0.18, 0.08, M('concrete'))
        if lvl >= 3:
            house(0.3, -0.5, g, 0.34, 0.2, 0.12, wall='concrete', roof='slate', style='flat')
            plane(0.5, 0.0, g + 0.02, rot=28, s=0.9, body='steel', wing='dark')
    return b


def diplomatic(lvl):
    def b():
        g = plot('paveL')
        marble = M('marble')
        path(0, -0.6, 0, -0.05, g, w=0.12)
        lawn_patch(-0.4, -0.35, g, 0.4, 0.3)
        lawn_patch(0.4, -0.35, g, 0.4, 0.3)
        box(0, 0.25, g, 0.7, 0.32, 0.3, M('cream'))
        windows(0, 0.25, g, 0.7, 0.32, 0.3, 5, 2, z0=0.3, z1=0.7)
        for i in range(6):
            col(-0.3 + 0.12 * i, 0.02, g, 0.017, 0.28, marble)
        box(0, 0.03, g + 0.28, 0.76, 0.09, 0.025, marble)
        gable(0, 0.03, g + 0.305, 0.76, 0.1, 0.08, M('slate'), mat_end=marble)
        box(0, 0.25, g + 0.3, 0.66, 0.28, 0.02, M('stone'))
        if lvl >= 2:
            cyl(0, 0.28, g + 0.32, 0.1, 0.1, M('cream'), n=14)
            dome(0, 0.28, g + 0.42, 0.1, M('verdi'), h=0.1, n=14)
        for k, c in enumerate(['red', 'blue', 'green', 'yellow', 'purple', 'orange']):
            flag(-0.55 + 0.22 * k, -0.18, g, 0.3, M(c), w=0.1)
        if lvl >= 3:
            for sx in (-1, 1):
                tower(sx * 0.5, 0.32, g, 0.07, 0.3, 'cream', roof='verdi', rh=0.16)
        cyl(0, -0.45, g, 0.09, 0.04, M('stone'), n=14)
        cyl(0, -0.45, g + 0.04, 0.07, 0.004, M('water', rough=0.05), n=14)
    return b


def preserve(lvl):
    def b():
        g = plot('grass', top='grass', rim='dirt')
        cyl(0.18, -0.12, g, 0.2, 0.004, M('water', rough=0.05), n=16)
        kinds = ['round', 'pine', 'round', 'pine', 'round']
        spots = [(-0.5, 0.3), (-0.35, 0.5), (0.5, 0.35), (0.3, 0.52), (-0.55, -0.1), (0.0, 0.45), (-0.2, 0.2)]
        if lvl >= 2:
            spots += [(-0.55, -0.4), (0.55, -0.35), (-0.25, -0.5), (0.1, -0.55), (0.55, 0.0), (-0.05, 0.05)]
        if lvl >= 3:
            spots += [(-0.7, 0.1), (0.68, 0.15), (-0.4, 0.05), (0.38, 0.1), (-0.15, -0.35)]
        for i, (x, y) in enumerate(spots):
            tree(x, y, 1.0 + 0.1 * (i % 3), kind=kinds[i % 5], z=g)
        for (x, y) in [(-0.1, -0.4), (0.4, 0.15)]:
            sphere(x, y, g, 0.05, M('rock'), n=7, rings=4)
        if lvl >= 2:
            house(-0.28, -0.25, g, 0.22, 0.16, 0.1, wall='woodL', roof='thatch', rh=0.1, floors=1)
        for (x, y, r) in [(0.35, -0.38, 20), (0.45, -0.3, 70)]:
            with at(x, y, g, r):
                box(0, 0, 0.04, 0.07, 0.025, 0.025, M('adobe'))
                box(0.04, 0, 0.065, 0.02, 0.016, 0.03, M('adobe'))
                for lx in (-0.025, 0.025):
                    for ly in (-0.008, 0.008):
                        cyl(lx, ly, 0, 0.004, 0.04, M('woodD'), n=4)
    return b


def datacenter(lvl):
    def b():
        g = plot('concreteD', top='concrete', rim='stone2')
        blue = '#37d4ff'
        w = 0.9 if lvl > 1 else 0.7
        box(-0.05, 0.18, g, w, 0.3, 0.2, M('concreteD'))
        n = 6 if lvl > 1 else 4
        for i in range(n):
            x = -0.05 - w / 2 + (i + 0.5) * w / n
            glow(x, 0.18 - 0.151, g + 0.04, w / n * 0.5, 0.004, 0.12, blue, strength=6)
        box(-0.05, 0.18, g + 0.2, w + 0.02, 0.32, 0.015, M('dark'))
        nf = 3 if lvl == 1 else 5
        for i in range(nf):
            fan(-0.05 - w / 2 + 0.15 + i * (w - 0.3) / (nf - 1), 0.2, g + 0.215, 0.06)
        for k in range(2 if lvl == 1 else 3):
            box(-0.4 + k * 0.28, -0.28, g, 0.2, 0.14, 0.12, M('steel'))
            fan(-0.4 + k * 0.28, -0.28, g + 0.12, 0.055)
        for k in range(3):
            server_rack(0.3 + k * 0.08, -0.25, g, h=0.14)
        glow(0.5, -0.1, g, 0.05, 0.05, 0.04, blue, strength=8)
        point_light(0.0, -0.1, g + 0.15, (0.3, 0.8, 1.0), power=1.5, size=0.1)
        if lvl >= 2:
            for (x, y) in [(0.55, 0.45), (0.35, 0.55)]:
                lathe(x, y, g, [(0.09, 0), (0.065, 0.14), (0.075, 0.26)], M('concrete'), n=14, cap0=False, cap1=False)
                smokestack_puff(x, y, g + 0.26, 0.9)
            for k in range(4):
                solar_panel(-0.55 + k * 0.17, -0.55, g, 0.13, 0.08)
        if lvl >= 3:
            box(0.5, -0.05, g, 0.2, 0.2, 0.45, M('dark'))
            for i in range(6):
                glow(0.5, -0.05 - 0.101, g + 0.04 + i * 0.07, 0.16, 0.004, 0.02, blue, strength=9)
            box(0.5, -0.05, g + 0.45, 0.22, 0.22, 0.015, M('steel'))
            sphere(0.5, -0.05, g + 0.47, 0.05, M('led_c', rgb=blue, emit=blue, emit_strength=7), n=10, rings=6)
    return b


# ---------------------------------------------------------------------------

SPRITES = {}
for _lv in (1, 2, 3):
    SPRITES[f'district_campus_{_lv}'] = (campus(_lv), False)
    SPRITES[f'district_holy_site_{_lv}'] = (holy_site(_lv), False)
    SPRITES[f'district_encampment_{_lv}'] = (encampment(_lv), False)
    SPRITES[f'district_commercial_hub_{_lv}'] = (commercial(_lv), False)
    SPRITES[f'district_harbor_{_lv}'] = (harbor(_lv), False)
    SPRITES[f'district_theater_{_lv}'] = (theater(_lv), False)
    SPRITES[f'district_industrial_zone_{_lv}'] = (industrial(_lv), False)
    SPRITES[f'district_entertainment_complex_{_lv}'] = (entertainment(_lv), False)
    SPRITES[f'district_neighborhood_{_lv}'] = (neighborhood(_lv), False)
    SPRITES[f'district_spaceport_{_lv}'] = (spaceport(_lv), False)
    SPRITES[f'district_aerodrome_{_lv}'] = (aerodrome(_lv), False)
    SPRITES[f'district_diplomatic_quarter_{_lv}'] = (diplomatic(_lv), False)
    SPRITES[f'district_preserve_{_lv}'] = (preserve(_lv), False)
    SPRITES[f'district_datacenter_{_lv}'] = (datacenter(_lv), False)
SPRITES['district_aqueduct_1'] = (aqueduct(), False)
SPRITES['district_acropolis_1'] = (acropolis(), False)
SPRITES['district_bath_1'] = (aqueduct('bath'), False)
SPRITES['district_hansa_1'] = (industrial(2, 'hansa'), False)
SPRITES['district_lavra_1'] = (lavra(), False)
SPRITES['district_mbanza_1'] = (neighborhood(1, 'mbanza'), False)
SPRITES['district_street_carnival_1'] = (street_carnival(), False)
SPRITES['district_royal_navy_dockyard_1'] = (royal_navy_dockyard(), False)
SPRITES['district_thanh_1'] = (thanh(), False)
SPRITES['district_oppidum_1'] = (oppidum(), False)
