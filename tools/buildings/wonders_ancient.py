"""Ancient and classical wonders."""
import math
import random

import bl
from bl import _face
from districts import brazier, portico
from ships import dhow
from town import *  # noqa: F401,F403


def around(cx, cy, r, n, fn, a0=0.0, ry=None):
    """Call fn(x, y, angle_deg, k) for n points on a circle/ellipse."""
    ry = r if ry is None else ry
    for k in range(n):
        a = a0 + 2 * math.pi * k / n
        fn(cx + r * math.cos(a), cy + ry * math.sin(a), math.degrees(a), k)


def figure(x, y, z, h, mat, rot=0.0, arms='down', torch=False, skirt=False, legs_apart=0.0):
    """A standing statue about `h` tall, built from tapered limbs."""
    m = M(mat) if isinstance(mat, str) else mat
    with at(x, y, z, rot):
        lw = h * 0.1
        for sx in (-1, 1):
            rod((sx * h * 0.07 + sx * legs_apart, 0, 0), (sx * h * 0.06, 0, h * 0.5), lw * 0.62, m, r1=lw * 0.5)
        if skirt:
            lathe(0, 0, h * 0.4, [(h * 0.12, 0), (h * 0.1, h * 0.12)], m, n=10)
        rod((0, 0, h * 0.5), (0, 0, h * 0.78), h * 0.1, m, r1=h * 0.13)
        sphere(0, 0, h * 0.78, h * 0.065, m, n=10, rings=6)
        shoulder = h * 0.74
        if arms == 'down':
            for sx in (-1, 1):
                rod((sx * h * 0.14, 0, shoulder), (sx * h * 0.17, 0, h * 0.42), h * 0.04, m)
        elif arms == 'wide':
            rod((-h * 0.55, 0, shoulder), (h * 0.55, 0, shoulder), h * 0.04, m)
        elif arms == 'torch':
            rod((-h * 0.14, 0, shoulder), (-h * 0.2, 0, h * 0.5), h * 0.04, m)
            rod((h * 0.14, 0, shoulder), (h * 0.3, 0, h * 1.0), h * 0.04, m)
            if torch:
                cone(h * 0.3, 0, h * 1.0, h * 0.04, h * 0.1, M('gold'), n=6)
                cyl(h * 0.3, 0, h * 1.06, h * 0.035, h * 0.1, M('flame', rgb='#ffcf50', emit='#ffb840', emit_strength=9.0), n=6, r2=h * 0.005)


def stonehenge():
    g = bl.plot('grass')
    rnd = random.Random(3)
    rock = M('rock', rgb='#9a948a', style='rock')
    cyl(0, 0, g, 0.64, 0.004, M('dirt', rgb='#a88c64'), n=40)
    cyl(0, 0, g + 0.002, 0.58, 0.004, G('grass', 0.95, 0.6, 0.9), n=40)
    n = 12
    for k in range(n):
        a = 2 * math.pi * k / n
        x, y = 0.52 * math.cos(a), 0.52 * math.sin(a)
        if k == 8:                                                     # one has fallen
            box(x, y, g, 0.28, 0.1, 0.07, rock, rot=math.degrees(a) + 20)
            continue
        box(x, y, g, 0.1, 0.06, 0.32 + rnd.uniform(-0.02, 0.03), rock, rot=math.degrees(a) + 90, taper=0.88)
    for k in range(n):
        if k in (7, 8):
            continue
        a0, a1 = 2 * math.pi * k / n, 2 * math.pi * (k + 1) / n
        am = (a0 + a1) / 2
        box(0.52 * math.cos(am), 0.52 * math.sin(am), g + 0.32, 0.27, 0.07, 0.05, rock, rot=math.degrees(am) + 90)
    for k, a in enumerate([200, 250, 290, 340, 90]):
        ar = math.radians(a)
        cx, cy = 0.27 * math.cos(ar), 0.27 * math.sin(ar)
        r = math.degrees(ar) + 90
        for s in (-1, 1):
            with at(cx, cy, g, r):
                box(s * 0.065, 0, 0, 0.07, 0.07, 0.46, rock, taper=0.85)
        with at(cx, cy, g + 0.46, r):
            box(0, 0, 0, 0.21, 0.075, 0.05, rock)
    box(0, 0, g, 0.16, 0.07, 0.04, M('stone2', style='rock'), rot=10)
    box(0.0, -0.78, g, 0.06, 0.06, 0.2, rock, rot=20, taper=0.8)
    for (x, y, rr) in [(-0.62, 0.12, 0.8), (0.64, -0.2, 0.7), (0.4, 0.62, 0.9)]:
        blob(x, y, g, 0.035 * rr, M('white', rgb='#ece8de', rough=1.0), seed=int(x * 50), sub=1, squash=0.7)
    road(0.0, -0.86, 0.0, -0.58, g, 0.07, 'dirt')
    tree(-0.7, -0.3, 0.9, 'round', g)


def hanging_gardens():
    g = bl.plot('grass')
    rnd = random.Random(9)
    sizes = [(1.06, 0.8), (0.88, 0.64), (0.7, 0.5), (0.52, 0.38)]
    for i, (w, d) in enumerate(sizes):
        z = g + i * 0.19
        box(0, 0.05, z, w, d, 0.19, M('stone'))
        # arcade on the front
        for k in range(int(w / 0.16)):
            ax = -w / 2 + 0.08 + k * 0.16
            box(ax, 0.05 - d / 2 - 0.002, z + 0.02, 0.07, 0.006, 0.12, M('dark'))
            dome(ax, 0.05 - d / 2 - 0.002, z + 0.14, 0.035, M('dark'), h=0.03, n=8)
        box(0, 0.05, z + 0.19, w + 0.02, d + 0.02, 0.02, M('stone2'))
        box(0, 0.05, z + 0.21, w - 0.03, d - 0.03, 0.01, G('grass', 0.9, 0.9, 1.4))
        # spilling planters along the rim
        for k in range(int(w / 0.09)):
            px = -w / 2 + 0.05 + k * (w - 0.1) / max(int(w / 0.09) - 1, 1)
            blob(px, 0.05 - d / 2 + 0.01, z + 0.21, 0.055, M(['leaf', 'leafL', 'leafD'][k % 3]), seed=k + i * 9, sub=1)
            # hanging vines
            box(px, 0.05 - d / 2 - 0.008, z + 0.05, 0.02, 0.012, 0.15 + 0.03 * (k % 3), M('leafL'))
        for k in range(3):
            blob(rnd.uniform(-w / 2.4, w / 2.4), 0.05 + rnd.uniform(-d / 3, d / 3), z + 0.21, 0.06, M('leafD'), seed=k + 50 * i, sub=1)
    top = g + 4 * 0.19 + 0.21
    tree(-0.1, 0.05, 1.3, 'round', top)
    tree(0.12, 0.08, 1.1, 'palm', top)
    # a waterfall down the middle
    for i in range(4):
        box(0.0, 0.05 - sizes[i][1] / 2 - 0.01, g + i * 0.19 + 0.02, 0.05, 0.008, 0.19, M('water', rough=0.05))
    box(0.0, -0.55, g, 0.14, 0.06, 0.012, M('water', rough=0.05))
    stair_ramp(0.0, -0.66, g, 0.16, 0.14, 0.19, 5, 'stone2')
    for (x, y) in [(-0.62, -0.5), (0.62, -0.5), (-0.7, 0.2), (0.7, 0.25)]:
        tree(x, y, 1.0, 'palm', g)


def pyramids():
    g = bl.plot('sand')
    blk = M('sand', rgb='#e6cf96', style='blocks')
    blk2 = M('sand', rgb='#d8be86', style='blocks')
    pyr(0.08, 0.2, g, 0.96, 0.96, 0.9, blk)
    pyr(0.08, 0.2, g + 0.88, 0.05, 0.05, 0.03, M('gold'))
    pyr(0.54, -0.2, g, 0.42, 0.42, 0.4, blk2)
    pyr(0.04, -0.34, g, 0.3, 0.3, 0.28, blk2)
    # the sphinx
    with at(-0.45, -0.3, g, 15):
        sand = M('sandD', rgb='#d4b378', style='plaster')
        box(0, 0, 0, 0.26, 0.1, 0.07, sand)
        box(0.1, 0, 0.0, 0.08, 0.1, 0.075, sand, taper=0.9)
        for sy in (-1, 1):
            box(0.2, sy * 0.032, 0.0, 0.1, 0.03, 0.025, sand)
        box(-0.1, 0, 0.07, 0.1, 0.07, 0.05, sand)
        sphere(-0.1, 0, 0.1, 0.03, sand, n=8, rings=6)
        box(-0.1, 0, 0.115, 0.07, 0.07, 0.016, M('blue'))
    for (x, y) in [(-0.66, 0.3), (0.72, 0.15), (-0.2, -0.62)]:
        tree(x, y, 1.0, 'palm', g)
    road(0.0, -0.1, 0.0, -0.8, g, 0.06, 'sand')


def oracle():
    g = bl.plot('grass')
    top = rock_hill(0, 0.0, g, 0.75, 0.14, seed=2, mat='rock', top='paveL', plateau=0.7)
    # the temple of Apollo
    portico(0.0, 0.2, top, 0.62, 0.4, 0.2, 6, roof='terra')
    box(0, 0.2, top + 0.036, 0.46, 0.3, 0.2, M('cream'))
    # the round tholos in front
    tx, ty = -0.28, -0.3
    cyl(tx, ty, top, 0.15, 0.025, M('stone'), n=20)
    cyl(tx, ty, top + 0.025, 0.1, 0.17, M('marble'), n=20)
    around(tx, ty, 0.125, 12, lambda x, y, a, k: col(x, y, top + 0.025, 0.012, 0.17, M('marble'), n=8))
    cyl(tx, ty, top + 0.195, 0.14, 0.02, M('marble'), n=20)
    cone(tx, ty, top + 0.215, 0.14, 0.1, M('terra'), n=20)
    stair_ramp(0.0, -0.74, g, 0.18, 0.46, 0.13, 6, 'stone2')
    brazier(0.3, -0.2, top, 1.3)
    for (x, y) in [(-0.62, 0.15), (0.62, 0.1), (0.5, -0.4), (-0.55, -0.55)]:
        tree(x, y, 0.9, 'pine', g)


def great_lighthouse():
    g = bl.plot('paveL')
    sea = M('stone2')
    box(0, 0.05, g, 0.9, 0.7, 0.05, sea)
    z = g + 0.05
    box(0, 0.1, z, 0.46, 0.46, 0.5, M('stone', rgb='#e6dcc0'), taper=0.84)
    box(0, 0.1 - 0.2, z, 0.1, 0.01, 0.16, M('dark'))
    for k in range(3):
        box(0, 0.1 - 0.215 * (1 - 0.1 * k), z + 0.26 + 0.08 * k, 0.04, 0.008, 0.045, M('dark'))
    z2 = z + 0.5
    cyl(0, 0.1, z2, 0.17, 0.36, M('cream'), n=8, r2=0.145)
    box(0, 0.1 - 0.15, z2 + 0.2, 0.04, 0.01, 0.05, M('dark'))
    z3 = z2 + 0.36
    cyl(0, 0.1, z3, 0.12, 0.26, M('stone'), n=14, r2=0.1)
    cyl(0, 0.1, z3 + 0.26, 0.13, 0.025, M('stone2'), n=14)
    z4 = z3 + 0.285
    for k in range(8):
        a = 2 * math.pi * k / 8 + 0.2
        col(0.085 * math.cos(a), 0.1 + 0.085 * math.sin(a), z4, 0.011, 0.12, M('marble'), n=8, base=False)
    cyl(0, 0.1, z4 + 0.0, 0.05, 0.1, M('flame', rgb='#ffcf50', emit='#ffb840', emit_strength=10.0), n=8)
    cone(0, 0.1, z4 + 0.12, 0.115, 0.07, M('bronze'), n=12)
    figure(0, 0.1, z4 + 0.19, 0.12, 'gold')
    point_light(0, 0.1, z4 + 0.08, (1.0, 0.8, 0.5), power=3.0, size=0.05)
    for (x, y) in [(-0.4, -0.3), (0.42, -0.28)]:
        box(x, y, z, 0.18, 0.14, 0.1, M('stone2'))
    stair_ramp(0.0, -0.58, g, 0.16, 0.18, 0.05, 3, 'stone2')


def colossus():
    g = bl.plot('paveL')
    # a sea gate with the bronze giant striding over it
    box(0, 0.0, g, 0.9, 0.22, 0.012, M('water', rough=0.04))
    for sx in (-1, 1):
        box(sx * 0.36, 0.0, g, 0.28, 0.34, 0.2, M('stone'), taper=0.88)
        box(sx * 0.36, 0.0, g + 0.2, 0.3, 0.36, 0.025, M('stone2'))
    bronze = M('bronze', rgb='#b87a44', rough=0.38, metal=0.85)
    zb = g + 0.225
    h = 1.0
    with at(0, 0, zb):
        for sx in (-1, 1):
            rod((sx * 0.36, 0, 0), (sx * 0.15, 0, 0.36 * h), 0.075, bronze, r1=0.055)
            rod((sx * 0.15, 0, 0.36 * h), (sx * 0.07, 0, 0.5 * h), 0.055, bronze, r1=0.06)
        lathe(0, 0, 0.44 * h, [(0.12, 0), (0.1, 0.1), (0.075, 0.3)], bronze, n=12)
        rod((0, 0, 0.5 * h), (0, 0, 0.76 * h), 0.11, bronze, r1=0.12)
        sphere(0, 0, 0.77 * h, 0.075, bronze, n=12, rings=8)
        for k in range(9):
            a = math.radians(-80 + 20 * k)
            rod((0.07 * math.sin(a), -0.0, 0.83 * h), (0.16 * math.sin(a), 0, 0.83 * h + 0.11 * math.cos(a)), 0.007, M('gold'))
        rod((0.1, 0, 0.7 * h), (0.2, 0, 0.88 * h), 0.045, bronze)
        rod((0.2, 0, 0.88 * h), (0.28, 0, 1.08 * h), 0.04, bronze)
        cone(0.28, 0, 1.06 * h, 0.04, 0.08, M('gold'), n=6)
        cyl(0.28, 0, 1.12 * h, 0.032, 0.09, M('flame', rgb='#ffcf50', emit='#ffb840', emit_strength=10.0), n=6, r2=0.004)
        rod((-0.1, 0, 0.7 * h), (-0.2, 0, 0.5 * h), 0.045, bronze)
        for sx in (-0.2,):
            box(sx - 0.0, 0.0, 0.3 * h, 0.12, 0.012, 0.3 * h, M('bronze', rgb='#9a643a'), taper=1.0)
    dhow(-0.25, -0.42, g, L=0.3, rot=90)
    for sx in (-1, 1):
        tree(sx * 0.62, 0.5, 0.9, 'palm', g)


def petra():
    g = bl.plot('sand')
    rosy = M('rose', rgb='#cf8566', style='rock')
    rose2 = M('rose', rgb='#b86f52', style='rock')
    # the rose-red canyon wall, with crags on top
    cliff = M('rose', rgb='#cf8566', style='rock')
    box(0, 0.7, g, 1.8, 0.5, 0.78, cliff, taper=0.84)
    box(-0.72, 0.3, g, 0.34, 0.5, 0.55, cliff, taper=0.7, rot=10)
    box(0.72, 0.3, g, 0.34, 0.5, 0.55, cliff, taper=0.7, rot=-10)
    for k, (x, h) in enumerate([(-0.5, 0.8), (-0.15, 0.86), (0.25, 0.82), (0.55, 0.78)]):
        blob(x, 0.7, g + h - 0.1, 0.16, cliff, seed=k + 20, sub=1, squash=1.0, jitter=0.3)
    # Al-Khazneh: the carved facade, drawn in its own frame and scaled up
    with at(0.0, 0.2, g, 0, 1.3):
        box(0, 0, 0, 0.54, 0.06, 0.62, rosy)
        for k in range(6):
            col(-0.2 + k * 0.08, -0.045, 0, 0.017, 0.26, rosy, n=10)
        box(0, -0.03, 0.26, 0.5, 0.06, 0.035, rose2)
        gable(0, -0.03, 0.295, 0.5, 0.07, 0.07, rosy, mat_end=rose2)
        box(0, -0.01, 0.34, 0.4, 0.05, 0.22, rosy)
        for k in range(4):
            col(-0.14 + k * 0.093, -0.045, 0.36, 0.013, 0.17, rosy, n=8)
        cyl(0, -0.03, 0.36, 0.07, 0.18, rosy, n=14)
        cone(0, -0.03, 0.54, 0.075, 0.08, rosy, n=14)
        sphere(0, -0.03, 0.62, 0.02, rose2, n=8, rings=5)
        box(0, -0.034, 0, 0.1, 0.01, 0.16, M('dark'))
        for sx in (-1, 1):
            box(sx * 0.16, -0.034, 0.03, 0.06, 0.008, 0.1, M('dark'))
        stair_ramp(0.0, -0.17, 0, 0.2, 0.12, 0.035, 2, 'rose')
    road(0.0, -0.8, 0.0, 0.1, g, 0.22, 'sand')
    tree(-0.5, -0.3, 1.0, 'palm', g)
    tree(0.52, -0.4, 0.9, 'palm', g)
    with at(0.3, -0.35, g, 15):                                           # a camel
        box(0, 0, 0.05, 0.1, 0.035, 0.035, M('adobe'))
        sphere(0.0, 0, 0.1, 0.022, M('adobe'), n=6, rings=4)
        rod((0.05, 0, 0.07), (0.085, 0, 0.12), 0.01, M('adobe'))
        for sx in (-1, 1):
            for sy in (-1, 1):
                cyl(sx * 0.035, sy * 0.012, 0, 0.004, 0.055, M('woodD'), n=3)


def colosseum():
    g = bl.plot('paveL')
    rx, ry = 0.74, 0.58
    n = 26
    with at(0, 0, g):
        pts = [(0.62 * math.cos(2 * math.pi * k / 40), 0.46 * math.sin(2 * math.pi * k / 40)) for k in range(40)]
        ext(pts, 0.0, 0.012, M('sandD', rgb='#d9c28a'))
        box(0, 0, 0.012, 0.9, 0.016, 0.004, M('woodD'))
        box(0, 0, 0.012, 0.016, 0.66, 0.004, M('woodD'))
    stone = M('stone', rgb='#e6d6b0', style='blocks')
    for tier in range(3):
        for k in range(n):
            a = 2 * math.pi * (k + 0.5) / n
            x, y = rx * math.cos(a), ry * math.sin(a)
            tang = math.degrees(math.atan2(ry * math.cos(a), -rx * math.sin(a)))
            w = 2 * math.pi * 0.66 / n * 1.04
            arch_wall(x, y, g + 0.012 + 0.17 * tier, w, 0.17, 0.06, stone, rot=tang + 180 + 90 - 90, iw=w * 0.55, ih=0.115)
    # attic and the rim
    top = g + 0.012 + 0.17 * 3
    for k in range(n):
        a = 2 * math.pi * (k + 0.5) / n
        x, y = rx * math.cos(a), ry * math.sin(a)
        tang = math.degrees(math.atan2(ry * math.cos(a), -rx * math.sin(a)))
        box(x, y, top, 2 * math.pi * 0.66 / n * 1.04, 0.06, 0.1, stone, rot=tang + 180)
        box(x, y, top + 0.06, 0.02, 0.064, 0.025, M('dark'), rot=tang + 180)
    around(0, 0, rx * 0.99, 12, lambda x, y, a, k: cyl(x, y, top + 0.1, 0.004, 0.1, M('woodD'), n=4), ry=ry * 0.99)
    around(0, 0, rx * 0.99, 8, lambda x, y, a, k: banner(x, y, top + 0.1, 0.12, 0.05, mat=M('red')), a0=0.2, ry=ry * 0.99)
    # sloping seating inside
    rings = []
    for (sx_, z_) in [(0.62, 0.012), (0.69, 0.25), (0.72, 0.42)]:
        rings.append([(sx_ * math.cos(2 * math.pi * k / 40) * 1.0, 0.46 / 0.62 * sx_ * math.sin(2 * math.pi * k / 40), g + z_) for k in range(40)])
    loft(rings, M('stone2'), cap0=False, cap1=False)
    crowd(g, [(-0.2, -0.78), (0.0, -0.8), (0.2, -0.76)], ('red', 'cream', 'blue'))
    stair_ramp(0.0, -0.86, g, 0.2, 0.1, 0.02, 2, 'stone')


def great_library():
    g = bl.plot('paveL')
    marble = M('marble')
    box(0, 0.12, g, 1.0, 0.62, 0.05, M('stone'))
    z = g + 0.05
    box(0, 0.2, z, 0.78, 0.4, 0.3, M('cream'))
    windows(0, 0.2, z, 0.78, 0.4, 0.3, 6, 1, ww=0.04, wh=0.09, z0=0.55, z1=0.55, sides='f')
    box(0, 0.2, z + 0.3, 0.84, 0.46, 0.03, marble)
    gable(0, 0.2, z + 0.33, 0.84, 0.46, 0.17, M('terra'), mat_end=marble)
    portico(0.0, -0.05, z, 0.84, 0.16, 0.3, 8, roof='terra')
    cyl(0.0, 0.25, z + 0.46, 0.15, 0.08, M('cream'), n=16)
    dome(0.0, 0.25, z + 0.54, 0.15, M('gold'), h=0.15, n=16)
    for sx in (-1, 1):
        box(sx * 0.58, 0.2, z, 0.2, 0.34, 0.2, M('cream'))
        gable(sx * 0.58, 0.2, z + 0.2, 0.2, 0.34, 0.1, M('terra'), rot=90, overhang=0.012)
        for k in range(4):
            col(sx * 0.58 + (-0.06 + 0.04 * k), -0.1, z, 0.01, 0.2, marble, n=8)
        tree(sx * 0.66, -0.4, 1.0, 'pine', g)
    stair_ramp(0, -0.45, g, 0.5, 0.18, 0.05, 3, 'stone')
    crowd(g, [(-0.12, -0.6), (0.1, -0.56), (0.0, -0.66), (0.26, -0.52)], ('white', 'cream', 'blue', 'purple'))


def mahabodhi_temple():
    g = bl.plot('paveL')
    sand = M('sand', rgb='#d9b878', style='blocks')
    stepped(0, 0.0, g, 0.74, 2, 0.05, 0.05, 'stone', d=0.74)
    z = g + 0.1
    n = 7
    for i in range(n):
        s = 0.44 * (1 - i / n * 0.72)
        box(0, 0, z + i * 0.11, s, s, 0.11, sand, taper=0.94)
        if i < 5:
            for sx in (-1, 0, 1):
                box(sx * s * 0.28, -s / 2 * 0.94 - 0.003, z + i * 0.11 + 0.022, 0.05, 0.008, 0.062, M('dark'))
                dome(sx * s * 0.28, -s / 2 * 0.94 - 0.003, z + i * 0.11 + 0.08, 0.025, M('dark'), h=0.02, n=6)
    zt = z + n * 0.11
    cyl(0, 0, zt, 0.07, 0.06, sand, n=12, r2=0.055)
    lathe(0, 0, zt + 0.06, [(0.06, 0), (0.07, 0.04), (0.04, 0.1), (0.0, 0.14)], M('gold'), n=12)
    cyl(0, 0, zt + 0.2, 0.004, 0.08, M('gold'), n=4)
    for (sx, sy) in [(-1, 1), (1, 1), (-1, -1), (1, -1)]:
        x, y = sx * 0.33, sy * 0.33
        for i in range(4):
            box(x, y, z + i * 0.06 - 0.04, 0.11 - i * 0.022, 0.11 - i * 0.022, 0.06, sand, taper=0.95)
        cone(x, y, z + 0.2, 0.025, 0.07, M('gold'), n=8)
    # railings, the bodhi tree and prayer flags
    for k in range(10):
        box(-0.5 + k * 0.11, -0.52, g, 0.012, 0.012, 0.05, M('stone2'))
    box(0, -0.52, g + 0.05, 1.0, 0.012, 0.01, M('stone2'))
    tree(0.6, 0.15, 1.8, 'round', g)
    tree(0.54, 0.25, 1.3, 'round', g)
    cols = ['blue', 'white', 'red', 'green', 'yellow']
    for k in range(10):
        with at(-0.6 + k * 0.05, 0.5, g + 0.2 - 0.03 * math.sin(math.pi * k / 9), 0):
            _face(M(cols[k % 5]), [(0, 0, 0), (0.04, 0, 0), (0.04, 0, -0.04), (0, 0, -0.04)])
    cyl(-0.62, 0.5, g, 0.006, 0.22, M('woodD'), n=4)
    cyl(-0.12, 0.5, g, 0.006, 0.22, M('woodD'), n=4)
    crowd(g, [(-0.2, -0.4), (0.2, -0.36), (0.0, -0.62)], ('orange', 'orange', 'red'))


def terracotta_army():
    g = bl.plot('dirt')
    earth = M('earth', rgb='#8a6a44', style='dirt')
    # three trenches separated by rammed-earth walls
    for k in range(4):
        box(-0.54 + k * 0.36, 0.0, g, 0.07, 0.82, 0.08, earth)
    box(0, 0.42, g, 1.0, 0.08, 0.08, earth)
    box(0, -0.42, g, 1.0, 0.08, 0.08, earth)
    box(0, 0.0, g - 0.003, 0.98, 0.76, 0.01, M('dirt', rgb='#6a4f33'))
    clay = M('terra', rgb='#b0714b', rough=0.85, style='plain')
    clay2 = M('terra', rgb='#9a6240', rough=0.85, style='plain')
    for c in range(3):
        for r in range(7):
            x = -0.36 + c * 0.36
            y = -0.34 + r * 0.115
            for sx in (-1, 1):
                xx = x + sx * 0.08
                mm = clay if (r + c) % 2 else clay2
                cyl(xx, y, g + 0.005, 0.034, 0.1, mm, n=8, r2=0.026)
                sphere(xx, y, g + 0.1, 0.025, mm, n=8, rings=5)
                box(xx, y, g + 0.005, 0.045, 0.03, 0.01, M('stoneD'))
    # a bronze chariot and horses
    for sx in (-1, 1):
        box(0.0 + sx * 0.04, 0.42, g + 0.08, 0.07, 0.03, 0.04, M('bronze'))
    # a viewing hall over the east end
    box(0.0, 0.62, g, 0.9, 0.16, 0.12, M('stone'))
    gable(0.0, 0.62, g + 0.12, 0.96, 0.2, 0.13, M('terraD'), overhang=0.02, mat_end=M('stone'))
    for k in range(7):
        col(-0.4 + k * 0.133, 0.53, g, 0.01, 0.12, M('red'), n=6, base=False, cap=False)
    banner(0.5, 0.45, g, 0.34, 0.08, mat=M('red'))
    banner(-0.5, 0.45, g, 0.34, 0.08, mat=M('red'))


def statue_of_zeus():
    g = bl.plot('paveL')
    marble = M('marble')
    gold = M('gold')
    ivory = M('cream', rgb='#f4ead2', rough=0.35)
    # the temple stands behind; the colossal seated Zeus sits on a podium before it
    box(0, 0.4, g, 0.9, 0.34, 0.04, M('stone'))
    z = g + 0.04
    box(0, 0.5, z, 0.7, 0.05, 0.36, M('cream'))
    for k in range(7):
        col(-0.38 + k * 0.127, 0.3, z, 0.024, 0.36, marble, n=12)
    box(0, 0.4, z + 0.36, 0.88, 0.34, 0.03, marble)
    gable(0, 0.4, z + 0.39, 0.88, 0.34, 0.11, M('terra'), overhang=0.01, mat_end=marble)
    box(0, -0.12, g, 0.5, 0.42, 0.06, M('stone'))
    box(0, -0.12, g + 0.06, 0.42, 0.34, 0.04, M('stone2'))
    zz = g + 0.1
    box(0, -0.02, zz, 0.3, 0.2, 0.06, M('woodD'))
    box(0, 0.06, zz + 0.06, 0.28, 0.07, 0.62, gold)
    box(-0.13, -0.04, zz + 0.06, 0.04, 0.2, 0.22, gold)
    box(0.13, -0.04, zz + 0.06, 0.04, 0.2, 0.22, gold)
    box(0, -0.04, zz + 0.06, 0.28, 0.18, 0.12, gold)
    rod((-0.07, -0.02, zz + 0.2), (-0.07, -0.14, zz + 0.22), 0.05, ivory)
    rod((0.07, -0.02, zz + 0.2), (0.07, -0.14, zz + 0.22), 0.05, ivory)
    rod((-0.07, -0.14, zz + 0.22), (-0.07, -0.15, zz + 0.07), 0.04, ivory)
    rod((0.07, -0.14, zz + 0.22), (0.07, -0.15, zz + 0.07), 0.04, ivory)
    rod((0.0, 0.02, zz + 0.2), (0.0, 0.03, zz + 0.52), 0.1, gold, r1=0.11)
    sphere(0.0, 0.03, zz + 0.6, 0.06, ivory, n=10, rings=6)
    sphere(0.0, 0.03, zz + 0.67, 0.062, M('leafD'), n=8, rings=3)
    rod((0.1, 0.03, zz + 0.46), (0.19, -0.1, zz + 0.4), 0.032, ivory)
    figure(0.19, -0.1, zz + 0.4, 0.1, 'gold')
    rod((-0.1, 0.03, zz + 0.46), (-0.2, -0.06, zz + 0.3), 0.032, ivory)
    cyl(-0.2, -0.06, zz + 0.0, 0.007, 0.78, gold, n=5)
    sphere(-0.2, -0.06, zz + 0.8, 0.02, gold, n=6, rings=4)
    stair_ramp(0, -0.55, g, 0.3, 0.12, 0.05, 3, 'stone')
    for (x, y) in [(-0.62, -0.3), (0.62, -0.3), (-0.65, 0.4), (0.65, 0.4)]:
        tree(x, y, 1.0, 'pine', g)


def mausoleum():
    g = bl.plot('paveL')
    box(0, 0.0, g, 0.96, 0.78, 0.06, M('stone'))
    z = g + 0.06
    box(0, 0.0, z, 0.82, 0.64, 0.2, M('marble'))
    # frieze band
    for k in range(12):
        box(-0.38 + k * 0.069, -0.325, z + 0.12, 0.05, 0.006, 0.05, M(['terra', 'blue', 'gold'][k % 3]))
    z2 = z + 0.2
    box(0, 0.0, z2, 0.7, 0.54, 0.022, M('marble'))
    cols_x, cols_y = 8, 6
    for i in range(cols_x):
        for j in (0, cols_y - 1):
            col(-0.3 + i * 0.6 / (cols_x - 1), -0.22 + j * 0.44 / (cols_y - 1), z2 + 0.022, 0.017, 0.24, M('marble'), n=10)
    for j in range(1, cols_y - 1):
        for i in (0, cols_x - 1):
            col(-0.3 + i * 0.6 / (cols_x - 1), -0.22 + j * 0.44 / (cols_y - 1), z2 + 0.022, 0.017, 0.24, M('marble'), n=10)
    box(0, 0.0, z2 + 0.022, 0.5, 0.34, 0.24, M('cream'))
    z3 = z2 + 0.262
    box(0, 0.0, z3, 0.7, 0.54, 0.03, M('marble'))
    for i in range(8):
        box(0, 0.0, z3 + 0.03 + i * 0.03, 0.62 - i * 0.065, 0.46 - i * 0.048, 0.03, M('marble' if i % 2 else 'stone'))
    z4 = z3 + 0.27
    # quadriga on top
    for sx in (-1.5, -0.5, 0.5, 1.5):
        box(sx * 0.026, 0.0, z4, 0.02, 0.06, 0.04, M('white'))
        sphere(sx * 0.026, 0.04, z4 + 0.04, 0.014, M('white'), n=6, rings=4)
    box(0, -0.06, z4, 0.1, 0.05, 0.04, M('gold'))
    figure(0, -0.06, z4 + 0.04, 0.09, 'gold')
    for (sx, sy) in [(-1, -1), (1, -1)]:
        box(sx * 0.42, sy * 0.36, g + 0.06, 0.06, 0.1, 0.05, M('stone2'))
        sphere(sx * 0.42, sy * 0.36, g + 0.11, 0.035, M('sandD'), n=8, rings=5)
    stair_ramp(0, -0.52, g, 0.3, 0.12, 0.06, 3, 'stone')
    for (x, y) in [(-0.7, 0.2), (0.7, 0.2)]:
        tree(x, y, 1.1, 'pine', g)


def apadana():
    g = bl.plot('sand')
    stone = M('stone', rgb='#cfc0a0', style='blocks')
    stepped(0, 0.1, g, 1.1, 2, 0.05, 0.03, 'stone', d=0.92)
    z = g + 0.1
    # grand double stair with relief panels
    for sx in (-1, 1):
        stair_ramp(sx * 0.34, -0.5, g, 0.3, 0.14, 0.1, 5, 'stone2')
    # hall of columns
    for i in range(6):
        for j in range(5):
            x, y = -0.38 + i * 0.152, -0.18 + j * 0.17
            cyl(x, y, z, 0.02, 0.46, M('sand', rgb='#e8dcc0'), n=10, r2=0.016)
            box(x, y, z + 0.46, 0.05, 0.05, 0.03, M('stone2'))
            box(x, y, z + 0.49, 0.07, 0.022, 0.04, M('sandD'))
            box(x, y, z + 0.49, 0.022, 0.07, 0.04, M('sandD'))
    # roof beams over the back half
    for j in range(3, 5):
        box(0, -0.18 + j * 0.17, z + 0.53, 0.98, 0.04, 0.03, M('woodD'))
    box(0, 0.37, z + 0.53, 0.98, 0.14, 0.045, M('woodL'))
    box(0, 0.42, z, 1.0, 0.05, 0.5, stone)
    # the gate with winged bulls
    for sx in (-1, 1):
        box(sx * 0.62, -0.1, g, 0.12, 0.2, 0.34, stone)
        box(sx * 0.62, -0.1, g + 0.34, 0.14, 0.22, 0.03, M('sandD'))
        sphere(sx * 0.62, -0.19, g + 0.28, 0.035, M('stone2'), n=8, rings=5)
    for (x, y) in [(-0.55, -0.55), (0.55, -0.55), (0.0, -0.7)]:
        tree(x, y, 0.9, 'palm', g)
    banner(0.0, -0.2, z, 0.35, 0.07, mat=M('purple'))


def jebel_barkal():
    g = bl.plot('sand')
    butte = rock_hill(0.05, 0.3, g, 0.62, 0.8, seed=12, mat='sand', top='sandD', plateau=0.78)
    rod((0.18, 0.2, butte), (0.2, 0.2, butte + 0.28), 0.045, M('sand', style='rock'), r1=0.025)
    for k in range(3):
        blob(0.2, 0.2, butte + 0.26 + 0.0 * k, 0.03, M('sand', style='rock'), seed=3, sub=1, squash=1.3)
    # temple of Amun at the foot of the butte
    for sx in (-1, 1):
        box(sx * 0.2, -0.3, g, 0.16, 0.12, 0.34, M('stone', rgb='#d8c08c', style='blocks'), taper=0.85)
        box(sx * 0.2, -0.3, g + 0.34, 0.17, 0.13, 0.025, M('sandD'))
    box(0.0, -0.3, g + 0.2, 0.24, 0.1, 0.15, M('stone', rgb='#d8c08c', style='blocks'))
    box(0.0, -0.355, g, 0.1, 0.01, 0.2, M('dark'))
    # Nubian pyramids
    for (x, y, s_) in [(-0.62, -0.1, 0.15), (-0.5, 0.15, 0.12), (0.62, -0.25, 0.16), (0.7, 0.05, 0.12)]:
        pyr(x, y, g, s_, s_, s_ * 1.5, M('sand', rgb='#d8c08c', style='blocks'))
    road(0, -0.8, 0, -0.38, g, 0.1, 'sand')
    tree(0.55, -0.55, 1.0, 'palm', g)
    tree(-0.5, -0.55, 1.0, 'palm', g)


def etemenanki():
    g = bl.plot('dirt')
    cols = ['#3a3836', '#f0e8d4', '#c24a2a', '#3a6aa8', '#e08a30', '#c8ccd2', '#e8b83a']
    w = 0.94
    z = g
    for i, c in enumerate(cols):
        ww = w * (1 - 0.115 * i)
        hh = 0.13 if i < 6 else 0.1
        box(0, 0.06, z, ww, ww * 0.88, hh, M('adobe', rgb=c, style='plaster'))
        box(0, 0.06, z + hh, ww + 0.014, ww * 0.88 + 0.014, 0.012, M('sand'))
        z += hh + 0.012
    # glazed blue shrine on the summit
    box(0, 0.06, z, 0.2, 0.17, 0.12, M('turq', rgb='#2a78b8', rough=0.25))
    box(0, 0.06 - 0.0865, z + 0.02, 0.06, 0.004, 0.08, M('gold'))
    gable(0, 0.06, z + 0.12, 0.22, 0.19, 0.06, M('gold'), overhang=0.01)
    # triple stair up the front
    stair_ramp(0.0, -0.42, g, 0.14, 0.5, 0.5, 14, 'sandD')
    for sx in (-1, 1):
        stair_ramp(sx * 0.2, -0.42, g, 0.07, 0.4, 0.26, 9, 'sandD')
    # temple compound wall and palms
    for sx in (-1, 1):
        wall_seg(sx * 0.7, -0.6, sx * 0.7, 0.55, g, h=0.08, t=0.05, wall='adobe', crenel=True)
    wall_seg(-0.7, 0.55, 0.7, 0.55, g, h=0.08, t=0.05, wall='adobe', crenel=True)
    for (x, y) in [(-0.58, -0.55), (0.58, -0.55), (0.0, -0.78)]:
        tree(x, y, 1.0, 'palm', g)


def angkor_wat():
    g = bl.plot('grass')
    blue = M('water', rgb='#4a9ab0', rough=0.04)
    # moat ring and causeway
    for (cx, cy, w, d) in [(0, 0.62, 1.2, 0.14), (0, -0.52, 1.2, 0.14), (-0.62, 0.05, 0.14, 1.1), (0.62, 0.05, 0.14, 1.1)]:
        box(cx, cy, g - 0.002, w, d, 0.01, blue)
    for k in range(7):
        box(-0.5 + k * 0.17, 0.62, g + 0.008, 0.04, 0.04, 0.003, M('leafL'))
    box(0.0, -0.52, g, 0.18, 0.16, 0.025, M('stone'))
    stone = M('stone', rgb='#cdbf9a', style='blocks')
    stepped(0, 0.05, g, 0.92, 3, 0.07, 0.07, 'stone', d=0.72)
    z = g + 0.21
    # corner towers and the central lotus tower
    def lotus(x, y, h, r):
        cyl(x, y, z, r * 1.5, h * 0.35, stone, n=4, r2=r * 1.25)
        for k in range(5):
            rr = r * (1.2 - 0.17 * k)
            cyl(x, y, z + h * 0.35 + k * h * 0.12, rr, h * 0.12, stone, n=12, r2=rr * 0.85)
        lathe(x, y, z + h * 0.35 + 5 * h * 0.12, [(r * 0.5, 0), (r * 0.4, h * 0.1), (r * 0.15, h * 0.2), (0.0, h * 0.26)], M('gold'), n=12)
    for (sx, sy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)]:
        lotus(sx * 0.26, 0.05 + sy * 0.2, 0.42, 0.075)
    lotus(0.0, 0.05, 0.75, 0.12)
    for sx in (-1, 1):
        box(sx * 0.4, 0.05, z - 0.02, 0.05, 0.6, 0.1, stone)
        gable(sx * 0.4, 0.05, z + 0.08, 0.05, 0.6, 0.04, M('stone2'), rot=90)
    stair_ramp(0.0, -0.32, g, 0.2, 0.18, 0.21, 6, 'stone2')
    tree(-0.7, -0.55, 1.0, 'palm', g)
    tree(0.72, -0.55, 1.0, 'palm', g)


def huey_teocalli():
    g = bl.plot('paveL')
    stone = M('stone', rgb='#cfc2a0', style='blocks')
    stepped(0, 0.08, g, 0.98, 5, 0.12, 0.085, 'stone', alt='stone2')
    z = g + 0.6
    for sx, c, name in ((-1, '#2a7ab8', 'blue'), (1, '#c8392b', 'red')):
        box(sx * 0.17, 0.08, z, 0.2, 0.2, 0.15, stone)
        box(sx * 0.17, 0.08 - 0.101, z, 0.07, 0.004, 0.09, M('dark'))
        box(sx * 0.17, 0.08, z + 0.15, 0.24, 0.24, 0.025, M('stone2'))
        pyr(sx * 0.17, 0.08, z + 0.175, 0.2, 0.2, 0.1, M('adobe', rgb=c))
    # double stairway with balustrades
    for sx in (-1, 1):
        stair_ramp(sx * 0.17, -0.38, g, 0.14, 0.46, 0.6, 12, 'stone2')
        box(sx * 0.17 - 0.1, 0.08 - 0.33, g, 0.04, 0.5, 0.2, stone, taper=0.9)
    # serpent heads at the foot, skull racks, braziers
    for sx in (-1, 1):
        sphere(sx * 0.17, -0.45, g, 0.04, M('green', rgb='#3a8a5a'), n=8, rings=5)
    for k in range(7):
        sphere(-0.2 + k * 0.07, -0.66, g + 0.07, 0.016, M('white'), n=6, rings=4)
    box(0, -0.66, g, 0.5, 0.012, 0.06, M('woodD'))
    for sx in (-1, 1):
        cyl(sx * 0.26, -0.66, g, 0.008, 0.1, M('woodD'), n=4)
    for (x, y) in [(-0.4, -0.34), (0.4, -0.34)]:
        brazier(x, y, g, 1.0)
    for (x, y) in [(-0.68, 0.0), (0.68, 0.05)]:
        tree(x, y, 1.0, 'palm', g)
    banner(0.0, 0.08, z + 0.0, 0.3, 0.06, mat=M('red'))


def chichen_itza():
    g = bl.plot('grass')
    stone = M('stone', rgb='#d4c8a8', style='blocks')
    n = 9
    for i in range(n):
        s = 0.96 - i * 0.092
        box(0, 0.1, g + i * 0.075, s, s, 0.075, stone)
        box(0, 0.1, g + i * 0.075 + 0.075, s + 0.012, s + 0.012, 0.008, M('stone2'))
    z = g + n * 0.075 + 0.008
    box(0, 0.1, z, 0.26, 0.26, 0.15, stone)
    box(0, 0.1 - 0.131, z, 0.12, 0.004, 0.1, M('dark'))
    box(0, 0.1, z + 0.15, 0.3, 0.3, 0.03, M('stone2'))
    for sx in (-1, 1):
        box(sx * 0.09, 0.1 - 0.131, z, 0.02, 0.006, 0.13, M('stone2'))
    # four stairways (three seen) with balustrades and serpent heads
    stair_ramp(0.0, -0.42, g, 0.2, 0.4, n * 0.075, 20, 'stone2')
    for sx in (-1, 1):
        box(sx * 0.12, -0.2, g, 0.05, 0.55, 0.25, stone, taper=0.8)
        sphere(sx * 0.12, -0.5, g, 0.04, M('stone2'), n=8, rings=5)
    for (x, y, s_) in [(-0.7, -0.38, 1.1), (0.7, -0.38, 1.1), (-0.66, 0.5, 1.2), (0.66, 0.48, 1.2), (0.0, 0.78, 1.0)]:
        tree(x, y, s_, 'round', g)
    tree(-0.55, -0.62, 1.0, 'palm', g)
    tree(0.55, -0.64, 1.0, 'palm', g)


WONDERS = {
    'stonehenge': stonehenge, 'hanging_gardens': hanging_gardens, 'pyramids': pyramids, 'oracle': oracle,
    'great_lighthouse': great_lighthouse, 'colossus': colossus, 'petra': petra, 'colosseum': colosseum,
    'great_library': great_library, 'mahabodhi_temple': mahabodhi_temple, 'terracotta_army': terracotta_army,
    'statue_of_zeus': statue_of_zeus, 'halicarnassus_mausoleum': mausoleum, 'apadana': apadana,
    'jebel_barkal': jebel_barkal, 'etemenanki': etemenanki, 'angkor_wat': angkor_wat, 'huey_teocalli': huey_teocalli,
    'chichen_itza': chichen_itza,
}
SPRITES = {f'wonder_{k}': (v, False) for k, v in WONDERS.items()}
