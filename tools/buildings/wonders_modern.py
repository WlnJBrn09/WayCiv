"""Industrial and modern wonders (and the generic fallback)."""
import math
import random

import bl
import bmesh
from bl import _b, _face, _stack
from districts import lawn
from mathutils import Matrix, Vector
from ships import dhow
from town import *  # noqa: F401,F403
from wonders_ancient import around


def eiffel_tower():
    g = bl.plot('grass')
    iron = M('iron', rgb='#7a6350', rough=0.55, metal=0.5)
    lawn(0, 0.0, g, 1.3, 1.1)
    road(-0.9, -0.55, 0.9, 0.5, g, 0.12, 'paveL')
    road(-0.9, 0.5, 0.9, -0.55, g, 0.12, 'paveL')

    def half(z):
        return 0.36 * math.exp(-z / 0.42) + 0.025

    levels = [0.0, 0.14, 0.3, 0.46, 0.62, 0.76, 0.9, 1.04, 1.2, 1.36]
    nodes = []
    for z in levels:
        w = half(z)
        nodes.append([(-w, -w), (w, -w), (w, w), (-w, w)])
    for i, z in enumerate(levels):
        pts = nodes[i]
        for k in range(4):
            a, b = pts[k], pts[(k + 1) % 4]
            rod((a[0], a[1], g + z), (b[0], b[1], g + z), 0.006, iron, n=4)
        if i + 1 < len(levels):
            for k in range(4):
                a, b = pts[k], nodes[i + 1][k]
                rod((a[0], a[1], g + z), (b[0], b[1], g + levels[i + 1]), 0.01 if z < 0.7 else 0.007, iron, n=5)
            # X bracing on each face
            for k in range(4):
                a, b = pts[k], pts[(k + 1) % 4]
                c, d = nodes[i + 1][k], nodes[i + 1][(k + 1) % 4]
                rod((a[0], a[1], g + z), (d[0], d[1], g + levels[i + 1]), 0.0045, iron, n=4)
                rod((b[0], b[1], g + z), (c[0], c[1], g + levels[i + 1]), 0.0045, iron, n=4)
    # platforms
    for z, w in [(0.46, 0.17), (0.9, 0.095)]:
        box(0, 0, g + z, w * 2 + 0.03, w * 2 + 0.03, 0.022, iron)
        box(0, 0, g + z + 0.022, w * 2 + 0.06, w * 2 + 0.06, 0.008, M('stone2'))
    # the great arches at the base
    for k in range(4):
        a = math.radians(90 * k)
        cx, cy = math.cos(a), math.sin(a)
        w0 = half(0.0)
        for j in range(8):
            t0, t1 = j / 8, (j + 1) / 8
            xs = [-w0 + 2 * w0 * t0, -w0 + 2 * w0 * t1]
            zs = [0.17 * math.sin(math.pi * t0) ** 0.7, 0.17 * math.sin(math.pi * t1) ** 0.7]
            p = [(cx * w0 + (-cy) * xs[i], cy * w0 + cx * xs[i], g + zs[i] + 0.0) for i in range(2)]
            rod(p[0], p[1], 0.006, iron, n=4)
    cyl(0, 0, g + 1.36, 0.012, 0.16, iron, n=6, r2=0.003)
    sphere(0, 0, g + 1.52, 0.01, M('gold'), n=6, rings=4)
    for (x, y) in [(-0.7, 0.5), (0.7, 0.5), (-0.7, -0.5), (0.7, -0.5), (0.0, 0.78)]:
        tree(x, y, 0.9, 'round', g)
    crowd(g, [(-0.5, -0.05), (0.5, 0.0), (0.0, -0.55), (0.1, 0.55)], ('red', 'blue', 'white', 'orange'))


def ruhr_valley():
    g = bl.plot('paveD')
    brick = M('brickD', rgb='#7a3c2c', style='brick')
    iron = M('iron', rgb='#4a4e56', rough=0.6, metal=0.6)
    road(0, -0.9, 0, 0.4, g, 0.1, 'asphalt')
    # blast furnaces
    for k, x in enumerate([-0.5, -0.24]):
        h = 0.62 - 0.05 * k
        lathe(x, 0.42, g, [(0.09, 0), (0.1, 0.12), (0.06, 0.4), (0.05, h)], iron, n=16)
        for j in range(4):
            box(x, 0.42, g + 0.1 + j * 0.12, 0.17 - j * 0.02, 0.012, 0.012, iron)
        glow(x, 0.42, g + h, 0.06, 0.06, 0.012, '#ff7a30', strength=9)
        smoke_plume(x, 0.42, g + h + 0.03, 4, 1.5)
        cyl(x + 0.14, 0.42, g, 0.014, 0.4, iron, n=6)
        for z in (0.1, 0.25, 0.38):
            rod((x + 0.14, 0.42, g + z), (x, 0.42, g + z + 0.02), 0.01, iron)
    # coking plant and sawtooth halls
    box(0.2, 0.3, g, 0.56, 0.3, 0.2, brick)
    windows(0.2, 0.3, g, 0.56, 0.3, 0.2, 5, 1, ww=0.04, wh=0.06, z0=0.55, z1=0.55, sides='f')
    for k in range(4):
        x0 = 0.2 - 0.28 + k * 0.14
        _face(M('slate'), [(x0, 0.3 - 0.16, g + 0.2), (x0 + 0.14, 0.3 - 0.16, g + 0.3), (x0 + 0.14, 0.3 + 0.16, g + 0.3), (x0, 0.3 + 0.16, g + 0.2)])
        _face(M('glass'), [(x0 + 0.14, 0.3 - 0.16, g + 0.3), (x0 + 0.14, 0.3 - 0.16, g + 0.2), (x0 + 0.14, 0.3 + 0.16, g + 0.2), (x0 + 0.14, 0.3 + 0.16, g + 0.3)])
    # tall chimneys
    for (x, y, h) in [(0.58, 0.5, 0.7), (0.5, 0.2, 0.55)]:
        lathe(x, y, g, [(0.05, 0), (0.035, h)], brick, n=12, cap0=False, cap1=False)
        cyl(x, y, g + h, 0.038, 0.02, M('dark'), n=12)
        smoke_plume(x, y, g + h + 0.03, 5, 1.8, tone='#bdb9b4')
    # gas holder, cooling tower, conveyor and coal heaps
    cyl(-0.55, -0.1, g, 0.12, 0.2, iron, n=18)
    cyl(-0.55, -0.1, g + 0.2, 0.12, 0.012, M('steel'), n=18)
    lathe(0.5, -0.12, g, [(0.13, 0), (0.09, 0.2), (0.11, 0.38)], M('concrete'), n=20, cap0=False, cap1=False)
    smoke_plume(0.5, -0.12, g + 0.38, 3, 1.5, tone='#f0f0f0')
    rod((-0.1, 0.1, g + 0.08), (-0.1, -0.3, g + 0.3), 0.012, M('orange'), n=4)
    for k in range(3):
        blob(-0.2 + k * 0.1, -0.42, g, 0.07, M('dark'), seed=k + 3, sub=1, squash=0.7)
    # railway
    for yy in (-0.62, -0.68):
        box(0.0, yy, g, 1.3, 0.012, 0.012, M('iron'))
    for k in range(18):
        box(-0.6 + k * 0.07, -0.65, g, 0.012, 0.1, 0.006, M('woodD'))
    box(0.3, -0.65, g + 0.012, 0.16, 0.05, 0.03, M('dark'))
    box(0.4, -0.65, g + 0.012, 0.06, 0.05, 0.05, M('red'))


def broadway():
    g = bl.plot('asphalt')
    signs = ['#ff3f8a', '#37d4ff', '#ffd23a', '#7dff5a', '#ff7a30', '#c86bff']
    # the avenue across the front, with yellow cabs
    box(0, -0.2, g, 1.7, 0.3, 0.004, M('asphalt'))
    for k in range(7):
        box(-0.6 + k * 0.2, -0.2, g + 0.005, 0.1, 0.012, 0.002, M('yellow'))
    for (x, y, rot) in [(-0.35, -0.26, 0), (0.15, -0.14, 180), (0.5, -0.26, 0)]:
        with at(x, y, g, rot):
            box(0, 0, 0.004, 0.11, 0.05, 0.03, M('yellow', rough=0.3))
            box(0.005, 0, 0.034, 0.06, 0.045, 0.022, M('glassD'))
    # the theatres along the back, their lit fronts toward the street
    rnd = random.Random(7)
    for i, (x, w, h) in enumerate([(-0.58, 0.28, 0.75), (-0.27, 0.3, 0.95), (0.05, 0.3, 0.8), (0.37, 0.3, 1.05), (0.66, 0.24, 0.7)]):
        y = 0.28
        box(x, y, g, w, 0.4, h, M('brick' if i % 2 else 'brickD'))
        box(x, y, g + h, w * 1.03, 0.41, 0.02, M('stone'))
        front = y - 0.201
        for k in range(int(h / 0.14)):  # rows of windows
            box(x, front, g + 0.28 + k * 0.14, w * 0.8, 0.006, 0.05, M('glassD'))
        c = signs[i % len(signs)]
        glow(x, front - 0.035, g + 0.14, w * 0.95, 0.05, 0.06, c, strength=10)   # the marquee
        glow(x + w * 0.38, front - 0.02, g + 0.26, 0.04, 0.03, h * 0.5, signs[(i + 2) % 6], strength=12)  # blade sign
        glow(x, y, g + h + 0.02, w * 0.7, 0.03, 0.12, signs[(i + 3) % 6], strength=9)  # rooftop billboard
        box(x, front - 0.01, g, w * 0.4, 0.01, 0.12, M('glow_door', rgb='#ffe2a0', emit='#ffd27a', emit_strength=4))
    # the near side of the street: lamps and the crowd
    for k in range(6):
        lamp(-0.7 + k * 0.28, -0.42, g, h=0.12)
    crowd(g, [(-0.5, -0.02), (-0.2, -0.0), (0.12, -0.03), (0.4, 0.0), (0.6, -0.04), (-0.3, -0.5), (0.25, -0.52)],
          ('white', 'red', 'orange', 'cream', 'blue'))
    point_light(0, -0.1, g + 0.4, (1.0, 0.55, 0.8), power=3.0, size=0.2)


def cristo_redentor():
    g = bl.plot('grass')
    # Corcovado: a steep granite peak with forest on its flanks
    granite = M('rock', rgb='#8e8a80', style='rock')
    prof = [(0.66, 0.0), (0.6, 0.08), (0.48, 0.24), (0.34, 0.42), (0.2, 0.58), (0.1, 0.66), (0.0, 0.68)]
    lathe(0.0, 0.1, g, prof, granite, n=12, smooth=False)
    rnd = random.Random(3)
    for k in range(16):
        a = 2 * math.pi * k / 16 + rnd.random() * 0.3
        r = 0.52 + rnd.random() * 0.12
        tree(r * math.cos(a), 0.1 + r * math.sin(a) * 0.95, 0.85, 'round', g + 0.02)
    for k in range(9):
        a = 2 * math.pi * k / 9 + 0.2
        tree(0.36 * math.cos(a), 0.1 + 0.36 * math.sin(a), 0.7, 'round', g + 0.27)
    peak = g + 0.68
    box(0, 0.1, peak - 0.02, 0.14, 0.14, 0.14, M('stone', rgb='#d6cfbf', style='blocks'), taper=0.85)
    z = peak + 0.12
    white = M('white', rgb='#f4f1e8', rough=0.5)
    h = 0.62
    with at(0, 0.1, z):
        # the robe, the body, arms spread wide, the head
        lathe(0, 0, 0, [(0.07, 0), (0.085, 0.12 * h), (0.075, 0.5 * h), (0.06, 0.62 * h)], white, n=14)
        rod((0, 0, 0.55 * h), (0, 0, 0.72 * h), 0.07, white, r1=0.06)
        sphere(0, 0, 0.8 * h, 0.045, white, n=12, rings=8)
        for sx in (-1, 1):
            rod((sx * 0.04, 0, 0.7 * h), (sx * 0.36 * h, 0, 0.71 * h), 0.032, white, r1=0.026)
            box(sx * 0.2 * h, 0, 0.62 * h, 0.2 * h, 0.03, 0.08 * h, white, taper=0.6)  # the hanging sleeves
    for k in range(4):
        a = 2 * math.pi * k / 4 + 0.6
        blob(0.42 * math.cos(a), 0.1 + 0.3 * math.sin(a), g + 0.62 + 0.05 * k, 0.07,
             M('white', rgb='#f4f6f8', rough=1.0, style='plain'), seed=k, sub=1, squash=0.45)


def maracana():
    g = bl.plot('paveL')
    R = 0.7
    n = 28
    pts = [(R * 0.62 * math.cos(2 * math.pi * k / n), R * 0.48 * math.sin(2 * math.pi * k / n)) for k in range(n)]
    ext(pts, g, 0.012, M('grass', rgb='#4a9a38'))
    # mowing stripes
    for k in range(8):
        box(-0.35 + k * 0.1, 0.0, g + 0.012, 0.05, 0.5, 0.002, M('grass', rgb='#5aaa44', style='plain'))
    box(0.0, 0.0, g + 0.014, 0.012, 0.5, 0.002, M('white'))
    cyl(0.0, 0.0, g + 0.014, 0.07, 0.002, M('white'), n=20)
    for s in (-1, 1):
        box(s * 0.28, 0.0, g + 0.014, 0.12, 0.2, 0.002, M('white'))
        box(s * 0.285, 0.0, g + 0.016, 0.1, 0.18, 0.002, M('grass', rgb='#4a9a38', style='plain'))
    # the bowl: stands in blue and white with a ring roof
    rows = []
    for (sr, z, c) in [(0.62, 0.012, 'blue'), (0.72, 0.18, 'blue'), (0.84, 0.34, 'white'), (0.9, 0.38, 'white')]:
        rows.append([(sr * math.cos(2 * math.pi * k / n) * 1.0, sr * 0.78 * math.sin(2 * math.pi * k / n), g + z) for k in range(n)])
    loft(rows[:2], M('blue', rgb='#3a68b8', style='plain'), cap0=False, cap1=False)
    loft(rows[1:3], M('white', rgb='#e8ecf2', style='plain'), cap0=False, cap1=False)
    loft(rows[2:4], M('concrete'), cap0=False, cap1=False)
    for k in range(n):
        a = 2 * math.pi * k / n
        box(0.88 * math.cos(a), 0.88 * 0.78 * math.sin(a), g, 0.045, 0.045, 0.38, M('concrete'), rot=math.degrees(a))
    roof = []
    for (sr, z) in [(0.92, 0.4), (0.7, 0.44), (0.6, 0.43)]:
        roof.append([(sr * math.cos(2 * math.pi * k / n), sr * 0.78 * math.sin(2 * math.pi * k / n), g + z) for k in range(n)])
    loft(roof, M('white', rgb='#f2f4f7', rough=0.4), cap0=False, cap1=False)
    for k in range(4):
        a = math.radians(45 + 90 * k)
        x, y = 0.98 * math.cos(a), 0.98 * 0.78 * math.sin(a)
        cyl(x, y, g, 0.012, 0.6, M('steel'), n=6)
        box(x, y, g + 0.6, 0.1, 0.03, 0.05, M('glow_fl', rgb='#fff6c8', emit='#fff2a0', emit_strength=6.0))
    for k in range(6):
        a = 2 * math.pi * k / 6 + 0.3
        pennant(0.95 * math.cos(a), 0.95 * 0.78 * math.sin(a), g + 0.4, 0.14, 0.06, mat=M(['green', 'yellow', 'blue'][k % 3]))


def _shell(x, y, z, w, d, h, mat, rot=0.0, lean=0.0):
    """A sail-shaped shell: a pointed, curved triangle in the x-z plane swollen by an ellipse across y."""
    apex = lean
    prof = []
    m = 10
    for i in range(m + 1):                                   # rising front edge, slightly convex
        t = i / m
        prof.append((-w / 2 + (apex + w / 2) * t ** 0.85, h * t ** 0.9))
    for i in range(1, m + 1):                                # falling back edge, concave
        t = i / m
        prof.append((apex + (w / 2 - apex) * t ** 0.7, h * (1 - t) ** 1.25))
    rings = []
    k = 8
    for j in range(k + 1):
        u = -1 + 2 * j / k
        s = max(math.sqrt(max(1 - u * u, 0.0)), 0.03)
        rings.append([(px * s, u * d / 2, pz * s) for (px, pz) in prof])
    with at(x, y, z, rot):
        loft(rings, mat, smooth=True, cap0=True, cap1=True)


def sydney_opera_house():
    g = 0.0
    # the podium and wide steps
    box(0.0, 0.05, g, 1.3, 0.8, 0.09, M('stone', rgb='#d8cfba', style='blocks'))
    stair_ramp(0.0, -0.58, g, 0.8, 0.18, 0.09, 6, 'stone2')
    tile = M('white', rgb='#f6f1e4', rough=0.25)
    tile2 = M('cream', rgb='#e8dfc6', rough=0.3)
    z = g + 0.09
    for (x, y, w, d, h, lean, c) in [(-0.42, 0.0, 0.6, 0.32, 0.62, 0.12, tile), (-0.1, 0.0, 0.56, 0.32, 0.74, 0.12, tile),
                                      (0.26, 0.0, 0.52, 0.3, 0.58, 0.1, tile), (-0.38, 0.3, 0.44, 0.24, 0.46, 0.08, tile2),
                                      (0.02, 0.3, 0.4, 0.24, 0.52, 0.08, tile2), (0.34, 0.3, 0.34, 0.2, 0.4, 0.06, tile2)]:
        _shell(x, y, z, w, d, h, c, rot=0, lean=lean)
    box(0.0, -0.17, z, 1.0, 0.03, 0.2, M('glass', rgb='#3a5a70', rough=0.06, metal=0.3))
    dhow(0.2, -0.78, g, L=0.26, rot=0, sail_mat='white')
    dhow(-0.45, -0.8, g, L=0.22, rot=30, sail_mat='cream')


def biosphere():
    g = bl.plot('paveL')
    r = 0.52
    cx, cy, cz = 0.0, 0.05, g + 0.26
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=2, radius=r)
    T = _stack[-1]
    glass = _b(M('glass', rgb='#a9d4ec', rough=0.08, metal=0.2))
    verts = {}
    keep = [f for f in bm.faces if f.calc_center_median().z > -r * 0.45]
    for f in keep:
        vs = []
        for v in f.verts:
            if v.index not in verts:
                verts[v.index] = glass.verts.new(T @ Vector((cx + v.co.x, cy + v.co.y, cz + v.co.z)))
            vs.append(verts[v.index])
        glass.faces.new(vs)
    seen = set()
    for f in keep:
        for e in f.edges:
            if e.index in seen:
                continue
            seen.add(e.index)
            a, b = e.verts
            rod((cx + a.co.x, cy + a.co.y, cz + a.co.z), (cx + b.co.x, cy + b.co.y, cz + b.co.z), 0.005, M('steel'), n=4)
    bm.free()
    cyl(cx, cy, g, r * 0.5, cz - g - r * 0.45, M('concrete'), n=20, r2=r * 0.7)
    # monorail across the foreground
    box(0.0, -0.6, g + 0.12, 1.3, 0.04, 0.03, M('concrete'))
    for k in range(5):
        box(-0.52 + k * 0.26, -0.6, g, 0.03, 0.03, 0.12, M('concrete'))
    box(0.2, -0.6, g + 0.15, 0.18, 0.04, 0.04, M('white'))
    box(0.2, -0.6, g + 0.17, 0.17, 0.042, 0.015, M('glassD'))
    for (x, y) in [(-0.68, -0.2), (0.68, -0.2), (0.0, -0.78), (-0.6, 0.5), (0.65, 0.5)]:
        tree(x, y, 0.9, 'round', g)


def generic():
    g = bl.plot('paveL')
    stepped(0, 0, g, 0.9, 3, 0.05, 0.06, 'stone')
    z = g + 0.15
    around(0, 0, 0.32, 8, lambda x, y, a, k: col(x, y, z, 0.03, 0.35, M('marble')))
    cyl(0, 0, z, 0.2, 0.3, M('cream'), n=16)
    cyl(0, 0, z + 0.35, 0.36, 0.03, M('marble'), n=16)
    dome(0, 0, z + 0.38, 0.26, M('gold'), h=0.24, n=18)
    cone(0, 0, z + 0.62, 0.012, 0.14, M('gold'), n=6)
    for (x, y) in [(-0.62, -0.4), (0.62, -0.4)]:
        box(x, y, g, 0.07, 0.07, 0.3, M('stone'), taper=0.5)
        cone(x, y, g + 0.3, 0.05, 0.08, M('gold'), n=4)


def _lying(x, y, z, prof, mat, rot=0.0, n=20):
    """A lathed body lying on its side along `rot` degrees (profile z runs along its length)."""
    m = Matrix.Translation((x, y, z)) @ Matrix.Rotation(math.radians(rot), 4, 'Z') @ \
        Matrix.Rotation(math.radians(90), 4, 'Y')
    bl._stack.append(bl._stack[-1] @ m)  # bl.reset() rebinds the stack, so go through the module
    try:
        lathe(0, 0, 0, prof, mat, n=n)
    finally:
        bl._stack.pop()


def _starship(x, y, z, booster=True):
    """A stainless-steel Starship, standing on its Super Heavy booster when `booster`."""
    hull = M('steel', rgb='#d3d8de', rough=0.22, metal=0.9)
    tiles = M('dark', rgb='#26282d', rough=0.6)
    r = 0.068
    if booster:
        cyl(x, y, z, r, 0.66, hull, n=24)
        for k in range(4):  # grid fins near the top of the booster
            a = math.radians(45 + 90 * k)
            box(x + math.cos(a) * (r + 0.02), y + math.sin(a) * (r + 0.02), z + 0.6, 0.035, 0.035, 0.04, M('iron'), rot=45 + 90 * k)
        cyl(x, y, z + 0.66, r * 1.02, 0.015, M('concreteD'), n=24)
        z += 0.675
    cyl(x, y, z, r, 0.4, hull, n=24)
    # the heat shield wraps the windward (front) half of the ship
    lathe(x, y - 0.002, z + 0.02, [(r * 1.01, 0.0), (r * 1.01, 0.36)], tiles, n=24, cap0=False, cap1=False)
    lathe(x, y, z + 0.4, [(r, 0.0), (r * 0.92, 0.08), (r * 0.7, 0.15), (r * 0.38, 0.2), (0.0, 0.23)], hull, n=24)
    for sx in (-1, 1):  # forward and aft flaps
        box(x + sx * (r + 0.012), y, z + 0.04, 0.022, 0.012, 0.11, tiles)
        box(x + sx * (r + 0.01), y, z + 0.47, 0.018, 0.01, 0.07, tiles)


def _lattice_tower(x, y, z, w, h, mat, levels=9):
    """A square truss tower with X bracing."""
    step = h / levels
    for i in range(levels + 1):
        zz = z + i * step
        for k in range(4):
            a = [(-w, -w), (w, -w), (w, w), (-w, w)][k]
            b = [(-w, -w), (w, -w), (w, w), (-w, w)][(k + 1) % 4]
            rod((x + a[0], y + a[1], zz), (x + b[0], y + b[1], zz), 0.005, mat, n=4)
            if i < levels:
                rod((x + a[0], y + a[1], zz), (x + a[0], y + a[1], zz + step), 0.009, mat, n=5)
                rod((x + a[0], y + a[1], zz), (x + b[0], y + b[1], zz + step), 0.004, mat, n=4)


def starbase_tx():
    g = bl.plot('concrete')
    # the orbital launch mount: a ring table on six legs over a flame trench
    mount = M('concrete', rgb='#c9c8c2')
    box(0.0, 0.05, g, 0.42, 0.42, 0.02, M('concreteD'))
    for k in range(6):
        a = math.radians(60 * k + 30)
        box(0.14 * math.cos(a), 0.05 + 0.14 * math.sin(a), g, 0.05, 0.05, 0.2, mount)
    lathe(0.0, 0.05, g + 0.2, [(0.17, 0.0), (0.17, 0.05), (0.09, 0.05), (0.09, 0.0)], M('steel'), n=28)
    _starship(0.0, 0.05, g + 0.25)
    # the catch tower and its chopsticks
    iron = M('iron', rgb='#4b4f57', rough=0.5, metal=0.6)
    _lattice_tower(0.3, 0.2, g, 0.07, 1.25, iron, levels=10)
    for sy in (-1, 1):
        box(0.17, 0.2 + sy * 0.09, g + 0.98, 0.34, 0.03, 0.04, M('steel', rgb='#8f969f'), rot=sy * 12)
    box(0.3, 0.2, g + 1.25, 0.18, 0.18, 0.04, iron)
    cyl(0.3, 0.2, g + 1.29, 0.006, 0.12, M('steel'), n=6)
    # propellant tank farm
    for k, (tx, ty) in enumerate([(-0.48, 0.38), (-0.36, 0.5), (-0.6, 0.18), (-0.24, 0.62)]):
        cyl(tx, ty, g, 0.07, 0.3 - 0.04 * (k % 2), M('white', rgb='#eef0f2', rough=0.35), n=18)
        dome(tx, ty, g + 0.3 - 0.04 * (k % 2), 0.07, M('white', rgb='#eef0f2', rough=0.35), h=0.035, n=18)
    for (tx, ty) in [(-0.62, -0.18), (-0.5, -0.3)]:
        _lying(tx, ty, g + 0.05, [(0.0, -0.18), (0.045, -0.16), (0.045, 0.16), (0.0, 0.18)], M('white'), rot=30, n=14)
    # a low hangar, a second ship waiting on a stand, scrub and palms by the shore
    box(0.58, -0.25, g, 0.32, 0.26, 0.16, M('concreteD'))
    box(0.58, -0.25, g + 0.16, 0.34, 0.28, 0.02, M('steel', rgb='#a9b0b8'))
    for (x, y) in [(-0.2, -0.5), (0.25, -0.55), (0.72, 0.25), (-0.75, -0.48)]:
        tree(x, y, 0.75, 'bush', g)
    for (x, y) in [(0.62, 0.5), (-0.05, 0.72)]:
        tree(x, y, 0.85, 'palm', g)


def starbase_la():
    g = bl.plot('concreteD')
    # a bayou inlet at the front, with a barge carrying a finished booster
    _face(M('water', rgb='#4d7f6e', rough=0.1),
          [(0.55 * math.cos(a), -0.56 + 0.17 * math.sin(a), g + 0.004) for a in
           (2 * math.pi * k / 32 for k in range(32))])
    box(-0.05, -0.56, g + 0.004, 0.62, 0.16, 0.04, M('dark', rgb='#3a3d44'))
    _lying(-0.05, -0.56, g + 0.105, [(0.0, -0.28), (0.06, -0.27), (0.06, 0.27), (0.0, 0.28)],
           M('steel', rgb='#d3d8de', rough=0.22, metal=0.9))
    # the mega bays: two tall high bays where ships are stacked
    for (bx, by, h) in [(-0.3, 0.32, 0.95), (0.08, 0.36, 0.8)]:
        box(bx, by, g, 0.26, 0.24, h, M('white', rgb='#eceeef', rough=0.5))
        box(bx, by - 0.121, g + 0.05, 0.16, 0.004, h - 0.12, M('glassD'))
        box(bx, by, g + h, 0.27, 0.25, 0.02, M('concreteD'))
    # the long factory hall
    box(0.48, 0.12, g, 0.36, 0.5, 0.22, M('white', rgb='#e3e6e8', rough=0.5))
    box(0.48, 0.12, g + 0.22, 0.38, 0.52, 0.02, M('steel', rgb='#a9b0b8'))
    for k in range(4):
        box(0.48, -0.131, g + 0.04 + k * 0.045, 0.3, 0.004, 0.02, M('glass'))
    # a ship rolling out on its transporter
    box(0.0, -0.18, g, 0.46, 0.14, 0.04, M('yellow', rgb='#d9a42a'))
    _lying(0.0, -0.18, g + 0.1, [(0.0, -0.22), (0.055, -0.2), (0.06, 0.12), (0.04, 0.18), (0.0, 0.22)],
           M('steel', rgb='#d3d8de', rough=0.22, metal=0.9))
    _starship(-0.62, 0.05, g, booster=False)
    # cypress and woods around the works
    for (x, y) in [(-0.78, -0.28), (0.76, -0.38), (0.82, 0.48), (-0.62, 0.6), (0.4, 0.68), (-0.85, 0.3)]:
        tree(x, y, 0.95, 'pine', g)
    for (x, y) in [(-0.68, -0.6), (0.66, -0.6), (0.7, -0.2)]:
        tree(x, y, 0.8, 'round', g)


WONDERS = {
    'eiffel_tower': eiffel_tower, 'ruhr_valley': ruhr_valley, 'broadway': broadway, 'cristo_redentor': cristo_redentor,
    'estadio_do_maracana': maracana, 'sydney_opera_house': sydney_opera_house, 'biosphere': biosphere, 'generic': generic,
    'starbase_tx': starbase_tx, 'starbase_la': starbase_la,
}
SPRITES = {f'wonder_{k}': (v, False) for k, v in WONDERS.items()}
