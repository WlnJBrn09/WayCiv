"""District sprites: space, air, nature and the unique districts."""
import math
import random

import bl
from bl import _face, _hex_pts
from districts import lawn, portico
from town import *  # noqa: F401,F403


def gantry(x, y, g, h=0.5, w=0.08):
    for sx in (-1, 1):
        for sy in (-1, 1):
            cyl(x + sx * w / 2, y + sy * w / 2, g, 0.006, h, M('red'), n=4)
    n = int(h / 0.07)
    for i in range(n + 1):
        z = g + i * h / n
        box(x, y - w / 2, z, w, 0.005, 0.005, M('red'))
        box(x, y + w / 2, z, w, 0.005, 0.005, M('red'))
        box(x - w / 2, y, z, 0.005, w, 0.005, M('red'))
        box(x + w / 2, y, z, 0.005, w, 0.005, M('red'))


def rocket(x, y, g, h=0.62, r=0.05):
    lathe(x, y, g, [(r * 0.85, 0), (r, 0.04), (r, h * 0.72), (r * 0.95, h * 0.76), (r * 0.7, h * 0.9), (0.0, h)], M('white', rough=0.35), n=16)
    cyl(x, y, g + h * 0.3, r * 1.01, h * 0.05, M('black', rgb='#25272c', style='plain'), n=16)
    cyl(x, y, g + h * 0.68, r * 1.01, h * 0.04, M('red'), n=16)
    for k in range(4):
        with at(x, y, g, rot=90 * k + 45):
            _face(M('red'), [(r, 0, 0.0), (r * 2.3, 0, 0.0), (r * 1.0, 0, h * 0.22)])
            _face(M('red'), [(r * 1.0, 0, h * 0.22), (r * 2.3, 0, 0.0), (r, 0, 0.0)])


def fuselage(x, y, z, rot=0.0, s=1.0, body='white', wing='red', kind='mono'):
    with at(x, y, z + 0.03 * s, rot, s):
        n = 10
        prof = [(-0.16, 0.004), (-0.12, 0.016), (-0.04, 0.026), (0.05, 0.028), (0.12, 0.02), (0.17, 0.008)]
        rings = []
        for px, rr in prof:
            rings.append([(px, rr * math.cos(2 * math.pi * k / n), rr * math.sin(2 * math.pi * k / n)) for k in range(n)])
        loft(rings, M(body, rough=0.3), smooth=True)
        box(0.02, 0, -0.004, 0.07, 0.34, 0.008, M(wing))
        if kind == 'bi':
            box(0.02, 0, 0.032, 0.06, 0.32, 0.007, M(wing))
            for sy in (-1, 1):
                cyl(0.02, sy * 0.1, 0.0, 0.003, 0.034, M('woodD'), n=4)
        box(-0.15, 0, 0.0, 0.05, 0.14, 0.006, M(wing))
        box(-0.15, 0, 0.0, 0.05, 0.006, 0.06, M(wing))
        if kind != 'jet':
            cyl(0.172, 0, -0.025, 0.002, 0.05, M('dark'), n=4)
            box(0.18, 0, -0.0, 0.004, 0.07, 0.008, M('dark'))


def hangar(x, y, g, w=0.34, d=0.24, h=0.13, rot=0.0, mat='steel'):
    with at(x, y, g, rot):
        n = 10
        rings = []
        for yy in (-d / 2, d / 2):
            rings.append([(w / 2 * math.cos(math.pi * i / n), yy, h * math.sin(math.pi * i / n) + 0.0) for i in range(n + 1)])
        loft(rings, M(mat, rough=0.4, metal=0.4), smooth=True, closed=False, cap0=True, cap1=True)
        box(0, -d / 2 - 0.002, 0, w * 0.55, 0.004, h * 0.65, M('dark'))


def spaceport(lvl):
    def b():
        g = bl.plot('concrete')
        cyl(0.0, 0.1, g, 0.34, 0.012, M('asphalt'), n=28)
        cyl(0.0, 0.1, g + 0.012, 0.26, 0.003, M('yellow'), n=28)
        cyl(0.0, 0.1, g + 0.015, 0.245, 0.003, M('asphalt'), n=28)
        box(-0.5, -0.38, g, 0.32, 0.2, 0.12, M('white'))
        box(-0.5, -0.38, g + 0.12, 0.2, 0.14, 0.09, M('glass', rough=0.08, metal=0.4))
        windows(-0.5, -0.38, g, 0.32, 0.2, 0.12, 5, 1, ww=0.04, wh=0.04, z0=0.5, z1=0.5, sides='f')
        dish(-0.4, -0.5, g, 0.06, 0)
        road(-0.3, -0.38, 0.0, -0.38, g, 0.05, 'asphalt')
        road(0.0, -0.38, 0.0, -0.08, g, 0.05, 'asphalt')
        for k in range(3):
            cyl(0.45, -0.3 + k * 0.14, g, 0.04, 0.12, M('white'), n=10)
            dome(0.45, -0.3 + k * 0.14, g + 0.12, 0.04, M('steel'), h=0.03, n=10)
        if lvl == 1:
            gantry(0.12, 0.1, g + 0.012, h=0.4)
            sphere(0.4, 0.35, g, 0.06, M('steel', rough=0.2), n=14, rings=9)
            sphere(0.52, 0.28, g, 0.05, M('steel', rough=0.2), n=14, rings=9)
        else:
            gantry(0.15, 0.1, g + 0.012, h=0.62)
            rocket(0.0, 0.1, g + 0.012, h=0.66, r=0.05)
            box(0.07, 0.1, g + 0.35, 0.09, 0.008, 0.008, M('red'))
            box(0.07, 0.1, g + 0.5, 0.09, 0.008, 0.008, M('red'))
            sphere(0.42, 0.38, g, 0.06, M('steel', rough=0.2), n=14, rings=9)
            sphere(0.54, 0.3, g, 0.05, M('steel', rough=0.2), n=14, rings=9)
        if lvl >= 3:
            box(-0.4, 0.4, g, 0.34, 0.2, 0.34, M('white'))
            box(-0.4, 0.4, g + 0.34, 0.36, 0.22, 0.014, M('steel'))
            for k in range(4):
                box(-0.4, 0.4 - 0.101, g + 0.06 + k * 0.07, 0.3, 0.004, 0.03, M('glassD'))
            box(-0.52, 0.29, g + 0.0, 0.07, 0.003, 0.2, M('red'))
            dish(-0.55, 0.0, g, 0.09, 0)
            dish(-0.66, 0.1, g, 0.07, 0)
            for k in range(3):
                solar_panel(0.2 + k * 0.14, 0.58, g, 0.12, 0.08, 25)
    return b


def aerodrome(lvl):
    def b():
        g = bl.plot('grass')
        with at(0, 0, g, 28):
            box(0, 0, 0, 1.5, 0.22, 0.008, M('asphalt'))
            for i in range(8):
                box(-0.6 + i * 0.17, 0, 0.008, 0.08, 0.012, 0.002, M('white'))
            for sx in (-1, 1):
                box(sx * 0.72, 0, 0.008, 0.02, 0.2, 0.002, M('white'))
        hangar(-0.4, 0.42, g, 0.34, 0.24, 0.13, rot=28)
        # control tower
        cyl(0.46, 0.3, g, 0.028, 0.2, M('white'), n=10, r2=0.022)
        box(0.46, 0.3, g + 0.2, 0.11, 0.11, 0.05, M('glass', rough=0.06, metal=0.4))
        box(0.46, 0.3, g + 0.25, 0.125, 0.125, 0.012, M('red'))
        cyl(0.46, 0.3, g + 0.262, 0.003, 0.05, M('steel'), n=4)
        # windsock
        cyl(0.2, 0.52, g, 0.003, 0.14, M('steel'), n=4)
        cone(0.2, 0.52, g + 0.13, 0.014, 0.05, M('orange'), n=6)
        kind = 'bi' if lvl == 1 else ('mono' if lvl == 2 else 'jet')
        fuselage(0.02, -0.08, g + 0.01, rot=28 + 15, s=1.25, body='cream', wing='red', kind=kind)
        if lvl >= 2:
            fuselage(-0.38, 0.12, g + 0.01, rot=28, s=1.1, body='white', wing='blue', kind=kind)
            box(0.5, -0.2, g, 0.14, 0.2, 0.06, M('concrete'))
        if lvl >= 3:
            box(0.1, -0.55, g, 0.4, 0.18, 0.1, M('concrete'))
            hangar(0.5, 0.0, g, 0.26, 0.2, 0.1, rot=28)
            fuselage(0.3, -0.25, g + 0.01, rot=28 - 20, s=1.1, body='steel', wing='dark', kind='jet')
    return b


def preserve(lvl):
    def b():
        g = bl.plot('grass')
        rnd = random.Random(14)
        # a pond with a muddy shore
        for (r, m) in [(0.27, 'dirt'), (0.23, 'water')]:
            with at(0.2, -0.12, g + 0.001 + (0.004 if m == 'water' else 0.0), 0, 1.0):
                cyl(0, 0, 0, r, 0.003, M(m, style='water' if m == 'water' else None), n=24)
        spots = [(-0.52, 0.32), (-0.34, 0.54), (0.5, 0.4), (0.28, 0.58), (-0.58, -0.1), (0.02, 0.46), (-0.2, 0.2)]
        if lvl >= 2:
            spots += [(-0.55, -0.4), (0.58, -0.38), (-0.25, -0.52), (0.1, -0.58), (0.62, 0.0), (-0.1, 0.0)]
        if lvl >= 3:
            spots += [(-0.7, 0.1), (0.68, 0.18), (-0.4, 0.02), (0.4, 0.15), (-0.15, -0.35), (0.0, 0.7)]
        kinds = ['round', 'pine', 'round', 'pine', 'pine']
        for i, (x, y) in enumerate(spots):
            tree(x, y, 1.0 + 0.12 * (i % 3), kinds[i % 5], g)
        for (x, y, s_) in [(-0.1, -0.4, 1.0), (0.45, 0.1, 1.3), (-0.4, 0.25, 0.8)]:
            blob(x, y, g, 0.05 * s_, M('rock'), seed=int(x * 100), sub=1, squash=0.7)
        for k in range(10):
            a, rr = rnd.random() * 6.28, 0.25 + rnd.random() * 0.4
            sphere(0.0 + rr * math.cos(a) * 1.1, -0.1 + rr * math.sin(a) * 0.9, g, 0.008, M(rnd.choice(['yellow', 'rose', 'white', 'purple'])), n=5, rings=3)
        # a deer
        with at(-0.3, -0.25, g, 30):
            box(0, 0, 0.03, 0.06, 0.022, 0.022, M('bronze'))
            box(0.035, 0, 0.045, 0.016, 0.014, 0.03, M('bronze'))
            sphere(0.05, 0, 0.065, 0.011, M('bronze'), n=6, rings=4)
            for sx in (-1, 1):
                for sy in (-1, 1):
                    cyl(sx * 0.02, sy * 0.007, 0, 0.002, 0.032, M('woodD'), n=3)
        if lvl >= 2:
            box(-0.28, 0.0, g, 0.2, 0.15, 0.09, M('woodL'))
            gable(-0.28, 0.0, g + 0.09, 0.22, 0.17, 0.09, M('thatch'), overhang=0.012, mat_end=M('woodL'))
            door(-0.28, 0.0, g, 0.2, 0.15, 0.035, 0.06)
            chimney(-0.2, 0.03, g + 0.1, 0.01, 0.07, mat='stone2')
        if lvl >= 3:
            with at(0.26, -0.42, g + 0.01, -20):
                for k in range(6):
                    box(-0.12 + k * 0.05, 0, 0, 0.045, 0.05, 0.008, M('woodL'))
            pennant(0.5, -0.2, g, 0.2, 0.07, mat=M('green'))
    return b


def diplomatic(lvl):
    def b():
        g = bl.plot('paveL')
        road(0, -0.85, 0, 0.1, g, 0.12, 'paveL')
        for sx in (-1, 1):
            lawn(sx * 0.4, -0.3, g, 0.38, 0.34)
            for k in range(4):
                tree(sx * (0.3 + 0.08 * k), -0.45, 0.5, 'bush', g)
        box(0, 0.3, g, 0.74, 0.3, 0.26, M('cream'))
        windows(0, 0.3, g, 0.74, 0.3, 0.26, 7, 2, ww=0.032, wh=0.05, z0=0.3, z1=0.72, sides='f')
        box(0, 0.3, g + 0.26, 0.78, 0.32, 0.02, M('marble'))
        hip(0, 0.3, g + 0.28, 0.76, 0.3, 0.09, M('slate'), ridge=0.6)
        portico(0, 0.1, g, 0.4, 0.14, 0.24, 6, roof='slate')
        if lvl >= 2:
            cyl(0, 0.34, g + 0.28, 0.12, 0.08, M('cream'), n=16)
            dome(0, 0.34, g + 0.36, 0.12, M('verdi'), h=0.13, n=16)
            cyl(0, 0.34, g + 0.49, 0.005, 0.06, M('gold'), n=4)
        for k, c in enumerate(['red', 'blue', 'green', 'yellow', 'purple', 'orange', 'white']):
            pennant(-0.57 + 0.19 * k, -0.04, g, 0.3, 0.08, mat=M(c))
        if lvl >= 3:
            for sx in (-1, 1):
                tower(sx * 0.46, 0.46, g, 0.07, 0.34, 'cream', roof='verdi', rh=0.16)
        fountain(0, -0.52, g, 0.8)
    return b


def datacenter(lvl):
    def b():
        g = bl.plot('concrete')
        blue = '#37d4ff'
        w = 0.9 if lvl > 1 else 0.7
        box(-0.05, 0.2, g, w, 0.3, 0.2, M('concreteD'))
        n = 6 if lvl > 1 else 4
        for i in range(n):
            x = -0.05 - w / 2 + (i + 0.5) * w / n
            glow(x, 0.2 - 0.151, g + 0.04, w / n * 0.5, 0.004, 0.12, blue, strength=6)
        box(-0.05, 0.2, g + 0.2, w + 0.02, 0.32, 0.015, M('dark'))
        nf = 3 if lvl == 1 else 5
        for i in range(nf):
            fan(-0.05 - w / 2 + 0.15 + i * (w - 0.3) / (nf - 1), 0.22, g + 0.215, 0.06)
        for k in range(2 if lvl == 1 else 3):
            box(-0.4 + k * 0.28, -0.28, g, 0.2, 0.14, 0.12, M('steel'))
            fan(-0.4 + k * 0.28, -0.28, g + 0.12, 0.055)
        for k in range(3):
            server_rack(0.3 + k * 0.08, -0.25, g, h=0.14)
        glow(0.5, -0.1, g, 0.05, 0.05, 0.04, blue, strength=8)
        if lvl >= 2:
            for k in range(4):
                solar_panel(-0.55 + k * 0.17, -0.55, g, 0.13, 0.08)
            for (x, y) in [(0.55, 0.45), (0.35, 0.55)]:
                lathe(x, y, g, [(0.09, 0), (0.065, 0.14), (0.075, 0.26)], M('concrete'), n=14, cap0=False, cap1=False)
                smoke_plume(x, y, g + 0.26, 3, 1.0, tone='#e8eef2')
        if lvl >= 3:
            box(0.5, -0.05, g, 0.2, 0.2, 0.45, M('dark'))
            for i in range(6):
                glow(0.5, -0.05 - 0.101, g + 0.04 + i * 0.07, 0.16, 0.004, 0.02, blue, strength=9)
            box(0.5, -0.05, g + 0.45, 0.22, 0.22, 0.015, M('steel'))
            sphere(0.5, -0.05, g + 0.47, 0.05, M('led_c', rgb=blue, emit=blue, emit_strength=7), n=10, rings=6)
    return b


def lavra():
    def b():
        g = bl.plot('paveL')
        for (x, y) in [(-0.62, -0.42), (0.62, -0.42), (-0.64, 0.3)]:
            tree(x, y, 1.0, 'pine', g)
        road(0, -0.85, 0, -0.1, g, 0.1, 'paveL')
        box(0, 0.12, g, 0.44, 0.36, 0.3, M('white'))
        windows(0, 0.12, g, 0.44, 0.36, 0.3, 3, 1, ww=0.04, wh=0.08, z0=0.5, z1=0.5, sides='f', frame=M('blue'))
        door(0, 0.12, g, 0.44, 0.36, 0.07, 0.1)
        for (x, y, r, h) in [(0, 0.12, 0.11, 0.3), (-0.22, 0.3, 0.065, 0.2), (0.22, 0.3, 0.065, 0.2), (-0.22, -0.04, 0.065, 0.2), (0.22, -0.04, 0.065, 0.2)]:
            if x != 0:
                box(x, y, g, 0.15, 0.15, 0.2, M('white'))
            top = g + h
            cyl(x, y, top, r * 0.85, 0.07, M('white'), n=14)
            lathe(x, y, top + 0.07, [(r * 0.85, 0), (r * 1.3, r * 0.9), (r * 1.15, r * 1.8), (r * 0.5, r * 2.6), (0.0, r * 3.2)],
                  M('gold' if x == 0 else 'verdi'), n=16)
            cyl(x, y, top + 0.07 + r * 3.1, 0.004, 0.06, M('gold'), n=4)
        box(0.5, -0.1, g, 0.07, 0.07, 0.3, M('white'))                       # bell tower
        pyr(0.5, -0.1, g + 0.3, 0.08, 0.08, 0.14, M('verdi'))
    return b


def acropolis():
    def b():
        g = bl.plot('grass')
        top = rock_hill(0, 0.0, g, 0.72, 0.17, seed=4)
        portico(0, 0.05, top, 0.62, 0.34, 0.2, 6, roof='terra')
        box(0, 0.05, top + 0.036, 0.44, 0.24, 0.2, M('marble'))
        stair_ramp(0, -0.76, g, 0.16, 0.5, 0.16, 7, 'stone2')
        for (x, y) in [(-0.55, -0.45), (0.58, -0.4)]:
            tree(x, y, 0.8, 'round', g)
        pennant(0.3, -0.2, top, 0.2, 0.07, mat=M('purple'))
    return b


def street_carnival():
    def b():
        g = bl.plot('paveL')
        colors = ['red', 'yellow', 'green', 'blue', 'purple', 'orange']
        steel = M('steel')
        # a ferris wheel with colourful gondolas
        cx, cy, cz, R = 0.0, 0.18, g + 0.26, 0.22
        for sx in (-1, 1):
            rod((sx * 0.1, cy, g), (0, cy, cz), 0.009, steel)
            rod((sx * 0.1, cy + 0.04, g), (0, cy, cz), 0.009, steel)
        n = 14
        for k in range(n):
            a, a2 = 2 * math.pi * k / n, 2 * math.pi * (k + 1) / n
            p = (cx + R * math.cos(a), cy, cz + R * math.sin(a))
            q = (cx + R * math.cos(a2), cy, cz + R * math.sin(a2))
            rod(p, q, 0.006, steel, n=4)
            rod((cx, cy, cz), p, 0.003, steel, n=4)
            box(p[0], p[1], p[2] - 0.035, 0.036, 0.03, 0.032, M(colors[k % 6]))
        for k in range(6):
            a = math.radians(30 + 60 * k)
            x, y = 0.58 * math.cos(a), 0.5 * math.sin(a) - 0.1
            if abs(x) < 0.3 and y > 0.0:
                continue
            tent(x, y, g, 0.22, colors[k], rot=math.degrees(a) + 90, h=0.2)
        # a carousel
        cyl(0.0, -0.3, g, 0.1, 0.02, M('woodL'), n=16)
        cyl(0.0, -0.3, g + 0.02, 0.012, 0.12, M('gold'), n=6)
        cone(0.0, -0.3, g + 0.14, 0.12, 0.07, M('red'), n=16)
        for k in range(6):
            a = 2 * math.pi * k / 6
            cyl(0.07 * math.cos(a), -0.3 + 0.07 * math.sin(a), g + 0.02, 0.003, 0.1, M('gold'), n=4)
        for k in range(6):
            pennant(-0.5 + 0.2 * k, -0.55, g, 0.24, 0.07, mat=M(colors[k]))
        crowd(g, [(-0.28, -0.4), (0.28, -0.38), (0.12, -0.46), (0.4, -0.1), (-0.4, -0.12)], ('red', 'blue', 'yellow', 'green', 'purple'))
    return b


def thanh():
    def b():
        g = bl.plot('grass')
        pts = _hex_pts(0.72)
        for i in range(6):
            a, c2 = pts[i], pts[(i + 1) % 6]
            if i == 4:
                continue
            wall_seg(a[0], a[1], c2[0], c2[1], g, h=0.08, t=0.11, wall='earth', crenel=False)
            wall_seg(a[0], a[1], c2[0], c2[1], g + 0.08, h=0.02, t=0.06, wall='earth', crenel=False)
        for (x, y, s_) in [(-0.5, 0.15, 1.0), (0.5, 0.15, 0.9), (0.0, 0.62, 0.8)]:
            tree(x, y, s_, 'palm', g)
        for (x, y) in [(-0.2, 0.15), (0.22, 0.2), (0.0, -0.05)]:
            adobe_house(x, y, g, random.Random(int(x * 10)), variant=2)
        # a two-tier pagoda gate house
        box(0.0, 0.38, g, 0.3, 0.2, 0.1, M('woodD'))
        gable(0.0, 0.38, g + 0.1, 0.38, 0.26, 0.07, M('terra'), overhang=0.02, mat_end=M('woodD'))
        box(0.0, 0.38, g + 0.16, 0.2, 0.14, 0.09, M('woodD'))
        gable(0.0, 0.38, g + 0.25, 0.28, 0.2, 0.07, M('terra'), overhang=0.02, mat_end=M('woodD'))
        banner(0.0, 0.38, g + 0.3, 0.12, 0.06, mat=M('red'))
        pennant(0.22, -0.4, g, 0.3, 0.1, mat=M('red'))
    return b


def oppidum():
    def b():
        g = bl.plot('dirt')
        pts = _hex_pts(0.74)
        for i in range(6):
            if i == 4:
                continue
            a, c2 = pts[i], pts[(i + 1) % 6]
            palisade(a[0], a[1], c2[0], c2[1], g, h=0.14, mat='wood')
        for p in (pts[4], pts[5]):
            cyl(p[0], p[1], g, 0.03, 0.26, M('woodD'), n=6)
            box(p[0], p[1], g + 0.26, 0.1, 0.1, 0.05, M('woodL'))
            cone(p[0], p[1], g + 0.31, 0.08, 0.08, M('thatch'), n=8)
        for (x, y) in [(-0.3, 0.2), (0.25, 0.28), (0.0, -0.08), (-0.35, -0.2), (0.38, -0.12)]:
            hut(x, y, g, 0.1, 0.08, wall='woodL', roof='thatch')
        banner(0.0, 0.5, g, 0.3, 0.09, mat=M('blue'))
        for sx in (-1, 1):
            cyl(sx * 0.2, 0.0, g, 0.005, 0.14, M('woodD'), n=4)
        hay(0.5, 0.2, g)
    return b


SPRITES = {}
for _lv in (1, 2, 3):
    SPRITES[f'district_spaceport_{_lv}'] = (spaceport(_lv), False)
    SPRITES[f'district_aerodrome_{_lv}'] = (aerodrome(_lv), False)
    SPRITES[f'district_preserve_{_lv}'] = (preserve(_lv), False)
    SPRITES[f'district_diplomatic_quarter_{_lv}'] = (diplomatic(_lv), False)
    SPRITES[f'district_datacenter_{_lv}'] = (datacenter(_lv), False)
SPRITES['district_lavra_1'] = (lavra(), False)
SPRITES['district_acropolis_1'] = (acropolis(), False)
SPRITES['district_street_carnival_1'] = (street_carnival(), False)
SPRITES['district_thanh_1'] = (thanh(), False)
SPRITES['district_oppidum_1'] = (oppidum(), False)
