"""Wonder sprites: wonder_<id without BUILDING_, lower case>."""
import math

import bl
import bmesh
from bl import _b, _face, _hex_pts, _stack
from mathutils import Vector
from parts import *  # noqa: F401,F403


# ---------------------------------------------------------------------------
# helpers

def ring(cx, cy, z, r, n, fn):
    for k in range(n):
        a = 2 * math.pi * k / n
        fn(cx + r * math.cos(a), cy + r * math.sin(a), math.degrees(a), k)


def statue(x, y, z, s=1.0, mat='marble', arms='down', rot=0.0, torch=False):
    m = M(mat)
    with at(x, y, z, rot, s):
        box(-0.035, 0, 0, 0.05, 0.05, 0.2, m)
        box(0.035, 0, 0, 0.05, 0.05, 0.2, m)
        box(0, 0, 0.2, 0.15, 0.08, 0.2, m, taper=0.85)
        sphere(0, 0, 0.4, 0.045, m, n=10, rings=6)
        if arms == 'down':
            box(-0.1, 0, 0.22, 0.04, 0.04, 0.16, m)
            box(0.1, 0, 0.22, 0.04, 0.04, 0.16, m)
        elif arms == 'up':
            box(-0.09, 0, 0.3, 0.04, 0.04, 0.2, m, rot=20)
            box(0.09, 0, 0.3, 0.04, 0.04, 0.2, m, rot=-20)
        elif arms == 'wide':
            box(0, 0, 0.34, 0.5, 0.035, 0.035, m)
        if torch:
            cyl(0.12, 0, 0.45, 0.012, 0.1, m, n=6)
            glow(0.12, 0, 0.55, 0.03, 0.03, 0.05, '#ffcf50', strength=10)


def minaret(x, y, z, h=0.5, r=0.03, mat='marble', cap='gold'):
    cyl(x, y, z, r * 1.5, 0.05, M(mat), n=10)
    cyl(x, y, z + 0.05, r, h, M(mat), n=10, r2=r * 0.8)
    cyl(x, y, z + 0.05 + h * 0.7, r * 1.5, 0.02, M(mat), n=10)
    cyl(x, y, z + 0.05 + h, r * 1.3, 0.03, M(mat), n=10)
    cone(x, y, z + 0.08 + h, r * 1.3, r * 4.5, M(cap), n=10)


def icosphere(x, y, z, r, mat, sub=2):
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=sub, radius=r)
    T = _stack[-1]
    tgt = _b(mat)
    vmap = {}
    for v in bm.verts:
        vmap[v.index] = tgt.verts.new(T @ (v.co + Vector((x, y, z))))
    for f in bm.faces:
        fn = tgt.faces.new([vmap[v.index] for v in f.verts])
        fn.smooth = False
    bm.free()


def sail(x, y, z, w, h, t, mat, rot=0.0, lean=0.0):
    """Opera-house shell: curved sail in the local x-z plane."""
    pts = [(-w / 2, 0)]
    n = 10
    for i in range(1, n):
        u = i / n
        pts.append((-w / 2 + w * u * 0.85, h * math.sin(u * math.pi / 2) ** 0.8))
    pts.append((w / 2 - w * 0.06, h * 0.98))
    for i in range(n, 0, -1):
        u = i / n
        pts.append((w / 2 - w * 0.02 * (1 - u) - w * 0.05, h * 0.98 * (u ** 1.5)))
    pts.append((w / 2, 0))
    with at(x, y, z, rot):
        a = [(p[0], -t / 2, p[1]) for p in pts]
        b = [(p[0] + lean, t / 2, p[1]) for p in pts]
        loft([a, b], mat)


def arena(cx, cy, z, R, tiers, n=16, mat='stone', top='stone2', arch_h=0.17, pitch=True):
    if pitch:
        cyl(cx, cy, z, R * 0.86, 0.03, M('sandD'), n=28)
    for tier in range(tiers):
        for k in range(n):
            a = 2 * math.pi * k / n
            w = 2 * R * math.sin(math.pi / n) * 1.02
            arch_wall(cx + R * math.cos(a), cy + R * math.sin(a), z + arch_h * tier, w, arch_h, 0.05,
                      M(mat if tier % 2 == 0 else 'cream'), rot=math.degrees(a) - 90 + 180, iw=w * 0.5, ih=arch_h * 0.6)
    cyl(cx, cy, z + arch_h * tiers, R * 1.0, 0.03, M(top), n=28, r2=R * 0.82, cap1=False)


def wp(*a, **kw):
    return plot(*a, **kw)


def mountain(x, y, z, r, h, mat='rock', n=9, seed=1, snow=False):
    import random
    rnd = random.Random(seed)
    rings = []
    levels = [(1.0, 0.0), (0.72, 0.38), (0.4, 0.72), (0.14, 0.93), (0.0, 1.0)]
    for rr, hh in levels:
        ring_ = []
        for i in range(n):
            a = 2 * math.pi * i / n
            j = 1 + (rnd.random() - 0.5) * 0.35 * (1 if rr > 0.05 else 0)
            ring_.append((x + r * rr * j * math.cos(a), y + r * rr * j * math.sin(a), z + h * hh))
        rings.append(ring_)
    loft(rings, M(mat), cap0=True, cap1=False)
    if snow:
        pass


# ---------------------------------------------------------------------------
# wonders

def stonehenge():
    g = wp('grass')
    tree(-0.6, 0.45, 1.0, 'round', z=g)
    tree(0.62, 0.4, 0.9, 'round', z=g)
    ring(0, 0, g, 0.55, 12, lambda x, y, a, k: box(x, y, g, 0.1, 0.06, 0.34, M('stone2' if k % 2 else 'rock'), rot=a + 90))
    for k in range(0, 12, 2):
        a0, a1 = 2 * math.pi * k / 12, 2 * math.pi * (k + 1) / 12
        am = (a0 + a1) / 2
        box(0.55 * math.cos(am), 0.55 * math.sin(am), g + 0.34, 0.26, 0.07, 0.05, M('stone'), rot=math.degrees(am) + 90)
    for k in range(5):
        a = math.radians(180 + 36 * (k - 2) + 90)
        px, py = 0.26 * math.cos(a + math.pi / 2 * 0), 0.26 * math.sin(a)
    for (x, y, r) in [(-0.2, 0.0, 90), (0.2, 0.0, 90), (0.0, 0.2, 0), (0.0, -0.2, 0)]:
        box(x, y, g, 0.07, 0.07, 0.42, M('rock'), rot=r)
    box(0, 0.0, g + 0.42, 0.45, 0.08, 0.05, M('stone'))
    box(0, 0.0, g, 0.16, 0.08, 0.05, M('stone2'))


def hanging_gardens():
    g = wp('grass')
    n = 4
    sizes = [1.1, 0.88, 0.66, 0.44]
    for i in range(n):
        s = sizes[i]
        box(0, 0.0, g + i * 0.2, s, s * 0.85, 0.2, M('stone'))
        box(0, 0.0, g + i * 0.2 + 0.2, s * 0.96, s * 0.82, 0.012, M('grassD'))
        for k in range(int(s * 6)):
            x = -s / 2 + 0.06 + k * (s - 0.12) / max(int(s * 6) - 1, 1)
            sphere(x, -s * 0.85 / 2 + 0.03, g + i * 0.2 + 0.2, 0.05, M('leaf'), n=8, rings=5)
        if i > 0:
            for k in range(3):
                x = -s / 2 + 0.1 + k * (s - 0.2) / 2
                col(x, -s * 0.85 / 2 - 0.0, g + i * 0.2, 0.014, 0.2, M('marble'), n=8)
    stair_ramp(0, -0.55, g, 0.14, 0.14, 0.2, 5, 'stone2')
    tree(0.0, 0.0, 1.8, 'round', z=g + 0.8)
    tree(-0.12, 0.05, 1.3, 'round', z=g + 0.8)
    box(0.52, -0.05, g + 0.02, 0.05, 0.6, 0.014, M('water', rough=0.05))
    for k in range(4):
        sphere(-0.35 + 0.23 * k, -0.62, g, 0.06, M('leafL'), n=8, rings=5)


def pyramids():
    g = wp('sand', side='sandD')
    pyr(0.0, 0.18, g, 0.92, 0.92, 0.95, M('sand'), rot=0)
    pyr(0.42, -0.2, g, 0.42, 0.42, 0.42, M('stone'))
    pyr(-0.4, -0.3, g, 0.3, 0.3, 0.3, M('stone'))
    tree(-0.62, 0.0, 1.0, 'palm', z=g)
    tree(0.62, 0.25, 1.0, 'palm', z=g)
    for k in range(3):
        sphere(-0.15 + 0.15 * k, -0.55, g, 0.04, M('sandD'), n=6, rings=4)


def oracle():
    g = wp('stone', side='stoneD')
    ext(_hex_pts(0.7), g, 0.1, M('rock'))
    z = g + 0.1
    ring(0, 0, z, 0.3, 12, lambda x, y, a, k: col(x, y, z, 0.022, 0.28, M('marble'), n=10))
    cyl(0, 0, z, 0.26, 0.28, M('cream'), n=24)
    cyl(0, 0, z + 0.28, 0.36, 0.035, M('marble'), n=24)
    cone(0, 0, z + 0.315, 0.35, 0.18, M('terra'), n=24)
    glow(0, 0, z + 0.4, 0.04, 0.04, 0.05, '#ff9a3a', strength=7)
    cyl(0.0, -0.55, g, 0.07, 0.07, M('bronze'), n=10)
    glow(0.0, -0.55, g + 0.07, 0.06, 0.06, 0.03, '#ffa040', strength=8)
    tree(-0.55, 0.3, 1.0, 'pine', z=g)
    tree(0.55, 0.32, 1.0, 'pine', z=g)


def great_lighthouse():
    g = wp('stone', side='stoneD')
    box(0, 0.05, g, 0.5, 0.5, 0.4, M('stone'), taper=0.85)
    box(0, 0.05, g + 0.4, 0.4, 0.4, 0.35, M('cream'), taper=0.82)
    cyl(0, 0.05, g + 0.75, 0.15, 0.3, M('stone'), n=8, r2=0.12)
    cyl(0, 0.05, g + 1.05, 0.12, 0.03, M('stone2'), n=8)
    for k in range(4):
        a = math.pi / 2 * k + math.pi / 4
        col(0.1 * math.cos(a), 0.05 + 0.1 * math.sin(a), g + 1.08, 0.012, 0.14, M('marble'), n=6)
    glow(0, 0.05, g + 1.1, 0.07, 0.07, 0.1, '#ffd070', strength=10)
    cone(0, 0.05, g + 1.22, 0.1, 0.1, M('bronze'), n=8)
    box(0, -0.25, g + 0.0, 0.14, 0.07, 0.4, M('stone2'), taper=0.7)
    for (x, y) in [(-0.4, -0.35), (0.4, -0.35)]:
        box(x, y, g, 0.15, 0.15, 0.12, M('stone2'))


def colossus():
    g = wp('stone', side='stoneD')
    box(0.0, 0.0, g, 0.9, 0.18, 0.012, M('water', rough=0.05))
    for sx in (-1, 1):
        box(sx * 0.33, 0.0, g, 0.28, 0.3, 0.2, M('stone'), taper=0.9)
    with at(0, 0, g + 0.2):
        for sx in (-1, 1):
            box(sx * 0.3, 0, 0, 0.1, 0.12, 0.2, M('bronze'), rot=0)
            box(sx * 0.18, 0, 0.18, 0.12, 0.12, 0.2, M('bronze'), rot=sx * -25)
        box(0, 0, 0.36, 0.3, 0.14, 0.3, M('bronze'), taper=0.8)
        sphere(0, 0, 0.66, 0.07, M('bronze'), n=10, rings=6)
        for k in range(7):
            a = math.radians(-70 + 20 * k)
            box(0.07 * math.cos(math.radians(90 + 40 * k)), 0, 0.72 + 0.03 * abs(k - 3) * -0.0, 0.01, 0.01, 0.04, M('gold'), rot=0)
        box(0.18, 0, 0.5, 0.05, 0.05, 0.28, M('bronze'), rot=-10)
        box(-0.2, 0, 0.45, 0.05, 0.05, 0.18, M('bronze'), rot=10)
        glow(0.2, 0, 0.78, 0.04, 0.04, 0.07, '#ffcf50', strength=10)
    tree(-0.62, 0.45, 1.0, 'palm', z=g)
    tree(0.62, 0.45, 1.0, 'palm', z=g)


def petra():
    g = wp('sand', side='sandD')
    mountain(0, 0.5, g, 0.7, 0.85, 'rose', n=10, seed=3)
    box(0, 0.18, g, 0.55, 0.3, 0.7, M('rose'), taper=0.97)
    # carved facade
    box(0, 0.025, g, 0.5, 0.05, 0.35, M('terra'))
    for i in range(4):
        col(-0.19 + i * 0.127, -0.01, g, 0.025, 0.32, M('rose'), n=10)
    box(0, -0.01, g + 0.32, 0.52, 0.07, 0.03, M('terra'))
    box(0, 0.01, g + 0.35, 0.42, 0.05, 0.22, M('terra'))
    for i in range(4):
        col(-0.15 + i * 0.1, -0.0, g + 0.35, 0.02, 0.17, M('rose'), n=8)
    gable(0, -0.01, g + 0.52, 0.46, 0.06, 0.07, M('rose'))
    cyl(0, 0.0, g + 0.59, 0.07, 0.07, M('rose'), n=12)
    cone(0, 0.0, g + 0.66, 0.07, 0.07, M('terra'), n=12)
    box(0, -0.02, g, 0.1, 0.03, 0.12, M('dark'))
    tree(-0.55, -0.45, 1.0, 'palm', z=g)
    tree(0.5, -0.4, 0.9, 'palm', z=g)


def colosseum():
    g = wp('paveL')
    arena(0, 0.02, g, 0.66, 3, n=18, mat='cream', top='stone2', arch_h=0.17)
    for k in range(6):
        a = math.radians(60 * k + 30)
        flag(0.62 * math.cos(a), 0.02 + 0.62 * math.sin(a), g + 0.51, 0.16, M('red'), w=0.08)
    stair_ramp(0, -0.78, g, 0.2, 0.1, 0.05, 2, 'stone')


def great_library():
    g = wp('paveL')
    marble = M('marble')
    box(0, 0.12, g, 0.9, 0.5, 0.06, M('stone'))
    box(0, 0.12, g + 0.06, 0.78, 0.4, 0.3, M('cream'))
    windows(0, 0.12, g + 0.06, 0.78, 0.4, 0.3, 6, 2)
    box(0, 0.12, g + 0.36, 0.84, 0.46, 0.03, marble)
    gable(0, 0.12, g + 0.39, 0.84, 0.46, 0.2, M('terra'), mat_end=marble)
    stair_ramp(0, -0.42, g, 0.5, 0.12, 0.06, 3, 'stone')
    for i in range(7):
        col(-0.36 + i * 0.12, -0.14, g + 0.06, 0.022, 0.3, marble)
    box(0, -0.14, g + 0.36, 0.84, 0.08, 0.03, marble)
    gable(0, -0.14, g + 0.39, 0.84, 0.1, 0.1, M('terra'), mat_end=marble)
    cyl(0, 0.2, g + 0.39, 0.15, 0.12, M('cream'), n=16)
    dome(0, 0.2, g + 0.51, 0.15, M('gold'), h=0.14, n=16)
    for sx in (-1, 1):
        box(sx * 0.5, 0.2, g, 0.2, 0.2, 0.42, M('cream'))
        pyr(sx * 0.5, 0.2, g + 0.42, 0.24, 0.24, 0.14, M('terra'))
    tree(-0.62, -0.35, 0.8, 'round', z=g)
    tree(0.62, -0.35, 0.8, 'round', z=g)


def mahabodhi_temple():
    g = wp('pave')
    stepped(0, 0.0, g, 0.7, 3, 0.05, 0.05, 'stone')
    z = g + 0.15
    n = 7
    for i in range(n):
        s = 0.42 * (1 - i / n * 0.75)
        box(0, 0, z + i * 0.1, s, s, 0.1, M('sand' if i % 2 else 'stone'), taper=0.93)
        if i in (1, 3):
            for sx in (-1, 1):
                box(sx * s * 0.5, -s * 0.5 - 0.0, z + i * 0.1 + 0.01, 0.04, 0.012, 0.07, M('dark'))
    cyl(0, 0, z + n * 0.1, 0.05, 0.08, M('sand'), n=10, r2=0.04)
    sphere(0, 0, z + n * 0.1 + 0.08, 0.05, M('gold'), n=10, rings=6)
    cone(0, 0, z + n * 0.1 + 0.16, 0.012, 0.1, M('gold'), n=6)
    for (sx, sy) in [(-1, 1), (1, 1), (-1, -1), (1, -1)]:
        box(sx * 0.3, sy * 0.3, g + 0.15, 0.1, 0.1, 0.3, M('stone'), taper=0.7)
        cone(sx * 0.3, sy * 0.3, g + 0.45, 0.05, 0.1, M('gold'), n=8)
    tree(0.6, -0.3, 1.2, 'round', z=g)
    tree(-0.62, -0.3, 1.2, 'round', z=g)


def terracotta_army():
    g = wp('earth', top='dirt')
    for (x, y, w, d) in [(0, 0.46, 1.0, 0.07), (0, -0.46, 1.0, 0.07), (-0.5, 0, 0.07, 0.9), (0.5, 0, 0.07, 0.9)]:
        box(x, y, g, w, d, 0.06, M('stoneD'))
    box(0, 0, g - 0.0, 0.95, 0.85, 0.01, M('dirt', rgb='#6c5236'))
    for r in range(4):
        for c in range(7):
            x = -0.36 + c * 0.12
            y = -0.3 + r * 0.2
            m = M('terra', rgb='#b4724c')
            cyl(x, y, g + 0.01, 0.03, 0.1, m, n=7, r2=0.025)
            sphere(x, y, g + 0.1, 0.022, m, n=7, rings=4)
    for k in range(5):
        x = -0.36 + 0.18 * k
        box(x, 0.0, g + 0.05, 0.04, 0.8, 0.012, M('woodD'), rot=0)
    flag(0.42, 0.38, g, 0.3, M('red'), w=0.12)
    flag(-0.42, 0.38, g, 0.3, M('red'), w=0.12)
    house(0.0, 0.62, g, 0.5, 0.14, 0.13, wall='stone', roof='terraD', rh=0.1, floors=1)


def hagia_sophia():
    g = wp('paveL')
    box(0, 0.0, g, 0.86, 0.72, 0.3, M('cream'))
    for i in range(7):
        box(-0.36 + i * 0.12, -0.361, g + 0.03, 0.05, 0.012, 0.2, M('terra'))
    for i in range(6):
        box(0, 0.0, g + 0.04 + i * 0.05, 0.875, 0.73, 0.016, M('terra'))
    windows(0, 0.0, g, 0.86, 0.72, 0.3, 6, 1, z0=0.6)
    # domes
    for dy in (-0.2, 0.2):
        dome(0, dy, g + 0.3, 0.2, M('sand'), h=0.1, n=18)
    cyl(0, 0.0, g + 0.3, 0.26, 0.1, M('cream'), n=24)
    dome(0, 0.0, g + 0.4, 0.28, M('slateL', rough=0.4), h=0.26, n=24)
    cone(0, 0.0, g + 0.66, 0.012, 0.09, M('gold'), n=6)
    for (sx, sy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)]:
        minaret(sx * 0.52, sy * 0.42, g, h=0.6, r=0.028, mat='white', cap='slateL')
        box(sx * 0.4, sy * 0.3, g, 0.1, 0.1, 0.22, M('stone'))
    tree(-0.4, -0.65, 0.8, 'pine', z=g)
    tree(0.4, -0.65, 0.8, 'pine', z=g)


def alhambra():
    g = wp('sand', side='sandD', top='sand')
    wall = 'terra'
    pts = [(-0.7, -0.45), (0.7, -0.45), (0.7, 0.55), (-0.7, 0.55)]
    for i in range(4):
        a, b = pts[i], pts[(i + 1) % 4]
        wall_seg(a[0], a[1], b[0], b[1], g, h=0.2, t=0.07, wall=wall)
    for p in pts:
        tower(p[0], p[1], g, 0.09, 0.4, wall, crenel=True)
    pool(0.0, 0.05, g, 0.5, 0.2, rim='marble', water='water')
    for i in range(5):
        col(-0.24 + i * 0.12, -0.17, g, 0.014, 0.12, M('marble'), n=8)
        col(-0.24 + i * 0.12, 0.27, g, 0.014, 0.12, M('marble'), n=8)
    box(0, 0.27, g + 0.12, 0.6, 0.04, 0.02, M('cream'))
    box(0, 0.4, g, 0.5, 0.2, 0.36, M('cream'))
    windows(0, 0.4, g, 0.5, 0.2, 0.36, 4, 2)
    dome(0, 0.4, g + 0.36, 0.16, M('terraD'), h=0.14, n=14)
    for x in (-0.35, 0.35):
        tree(x, -0.2, 1.0, 'pine', z=g)
    box(0, -0.45, g, 0.18, 0.1, 0.24, M('terraD'))


def chichen_itza():
    g = wp('grass')
    stepped(0, 0.05, g, 0.95, 9, 0.085, 0.045, 'stone', alt='stone2')
    z = g + 0.765
    box(0, 0.05, z, 0.22, 0.2, 0.14, M('cream'))
    box(0, 0.05, z + 0.14, 0.26, 0.24, 0.025, M('stone2'))
    box(0, -0.065, z, 0.06, 0.01, 0.1, M('dark'))
    stair_ramp(0, -0.42, g, 0.2, 0.42, 0.74, 9, 'stone2')
    # steps ramp is local +y toward center: rotate so it faces front
    for (x, y) in [(-0.65, -0.35), (0.65, -0.35), (0.6, 0.45), (-0.62, 0.42)]:
        tree(x, y, 1.1, 'round', z=g)


def mont_st_michel():
    g = wp('sand', side='sandD', top='sandD')
    for (r, h0, h1) in [(0.7, 0.0, 0.14), (0.54, 0.14, 0.3), (0.38, 0.3, 0.46)]:
        ext(_hex_pts(r * 1.0), g + h0, h1 - h0, M('rock'))
    ext(_hex_pts(0.62), g + 0.14, 0.01, M('grassD'))
    z = g + 0.46
    for (x, y, rot, h) in [(-0.38, -0.3, 20, 0.12), (0.4, -0.28, -20, 0.1), (-0.42, 0.18, 0, 0.1), (0.38, 0.22, 10, 0.12), (0.0, -0.42, 0, 0.1)]:
        house(x, y, g + 0.14, 0.16, 0.11, h, wall='cream', roof='slate', rot=rot, rh=0.08, floors=1)
    for (x, y, rot) in [(-0.2, -0.22, 10), (0.2, -0.2, -10), (-0.24, 0.12, 0), (0.22, 0.14, 0)]:
        house(x, y, g + 0.3, 0.14, 0.1, 0.1, wall='cream', roof='terra', rot=rot, rh=0.07, floors=1)
    box(0, 0, z, 0.3, 0.26, 0.2, M('cream'))
    gable(0, 0, z + 0.2, 0.3, 0.26, 0.1, M('slate'))
    box(0, 0, z + 0.2, 0.14, 0.14, 0.14, M('cream'))
    cyl(0.0, 0.0, z + 0.34, 0.04, 0.1, M('stone'), n=8)
    cone(0.0, 0.0, z + 0.44, 0.045, 0.4, M('slateL'), n=8)
    cyl(0.0, 0.0, z + 0.84, 0.003, 0.06, M('gold'), n=4)
    for k in range(10):
        a = 2 * math.pi * k / 10
        wall_seg(0.76 * math.cos(a), 0.76 * math.sin(a), 0.76 * math.cos(a + 0.6), 0.76 * math.sin(a + 0.6), g, h=0.1, t=0.04, wall='stone2', crenel=False)


def venetian_arsenal():
    bl.USE_CATCHER = False
    g = wp('stone')
    box(0, -0.3, g, 0.9, 0.3, 0.011, M('water', rough=0.05))
    ship(-0.2, -0.28, g + 0.01, rot=0, s=1.1, hull='brick', sail=False)
    ship(0.28, -0.3, g + 0.01, rot=0, s=0.95, hull='brickD', sail=False)
    box(0, 0.32, g, 0.95, 0.26, 0.26, M('brick'))
    windows(0, 0.32, g, 0.95, 0.26, 0.26, 8, 1, z0=0.7)
    box(0, 0.32, g + 0.26, 0.97, 0.28, 0.02, M('stone'))
    for k in range(5):
        x = -0.38 + 0.19 * k
        box(x, 0.19, g, 0.12, 0.02, 0.15, M('dark'))
    for sx in (-1, 1):
        tower(sx * 0.46, 0.2, g, 0.07, 0.38, 'brick', crenel=True)
    flag(0.0, 0.32, g + 0.28, 0.2, M('red'), w=0.1)
    cyl(-0.55, -0.1, g, 0.015, 0.25, M('iron'), n=6)
    box(-0.45, -0.1, g + 0.25, 0.22, 0.012, 0.015, M('iron'))


def great_zimbabwe():
    g = wp('grass')
    n = 28
    def segs(R, h, t, gap=(3, 5)):
        for k in range(n):
            if gap[0] <= k < gap[1]:
                continue
            a0, a1 = 2 * math.pi * k / n, 2 * math.pi * (k + 1) / n
            wall_seg(R * math.cos(a0), R * math.sin(a0) * 0.92, R * math.cos(a1), R * math.sin(a1) * 0.92, g, h=h, t=t, wall='stone', crenel=False)
    segs(0.7, 0.18, 0.09)
    segs(0.46, 0.15, 0.07, gap=(12, 15))
    cyl(0.1, 0.12, g, 0.1, 0.34, M('stone2'), n=16, r2=0.075)
    cone(0.1, 0.12, g + 0.34, 0.075, 0.06, M('stone'), n=16)
    for (x, y) in [(-0.2, 0.0), (-0.15, -0.2), (0.3, -0.12)]:
        hut(x, y, g, 0.08, 0.08)
    tree(0.55, 0.5, 1.0, 'round', z=g)
    tree(-0.6, 0.45, 1.0, 'round', z=g)


def forbidden_city():
    g = wp('paveL')
    wall_pts = [(-0.74, -0.5), (0.74, -0.5), (0.74, 0.6), (-0.74, 0.6)]
    for i in range(4):
        a, b = wall_pts[i], wall_pts[(i + 1) % 4]
        if i == 0:
            continue
        wall_seg(a[0], a[1], b[0], b[1], g, h=0.14, t=0.06, wall='red', crenel=False)
    wall_seg(-0.74, -0.5, -0.2, -0.5, g, h=0.14, t=0.06, wall='red', crenel=False)
    wall_seg(0.2, -0.5, 0.74, -0.5, g, h=0.14, t=0.06, wall='red', crenel=False)
    for p in wall_pts:
        box(p[0], p[1], g, 0.12, 0.12, 0.2, M('red'))
        hip(p[0], p[1], g + 0.2, 0.17, 0.17, 0.1, M('gold'), ridge=0.35)
    # gate
    box(0, -0.5, g, 0.4, 0.14, 0.22, M('red'))
    box(0, -0.5, g + 0.22, 0.44, 0.16, 0.02, M('stone'))
    hip(0, -0.5, g + 0.24, 0.42, 0.18, 0.12, M('gold'), ridge=0.6)
    box(0, -0.57, g, 0.1, 0.02, 0.12, M('dark'))
    # terrace and hall
    stepped(0, 0.15, g, 0.8, 3, 0.04, 0.03, 'marble', d=0.52)
    z = g + 0.12
    box(0, 0.15, z, 0.56, 0.34, 0.2, M('red'))
    for i in range(7):
        col(-0.24 + i * 0.08, -0.04, z, 0.014, 0.19, M('red'), n=8)
    hip(0, 0.15, z + 0.2, 0.66, 0.44, 0.1, M('gold'), ridge=0.7)
    hip(0, 0.15, z + 0.3, 0.46, 0.3, 0.1, M('gold'), ridge=0.6)
    stair_ramp(0, -0.34, g, 0.2, 0.12, 0.1, 3, 'marble')


def potala_palace():
    g = wp('earth', top='dirt', side='stoneD')
    mountain(0, 0.1, g, 0.8, 0.5, 'earth', n=9, seed=7)
    z = g + 0.45
    box(0, 0.0, g + 0.15, 0.9, 0.3, 0.3, M('white'), taper=0.9)
    windows(0, 0.0, g + 0.15, 0.9, 0.3, 0.3, 8, 3)
    box(0, 0.05, z, 0.55, 0.26, 0.3, M('white'))
    box(0, 0.05, z + 0.3, 0.5, 0.24, 0.03, M('stone'))
    box(0, 0.05, z + 0.3, 0.3, 0.24, 0.3, M('red'), taper=0.97)
    box(0, -0.075, z + 0.18, 0.32, 0.01, 0.12, M('red'))
    box(0, 0.05, z + 0.6, 0.36, 0.28, 0.02, M('stone'))
    for x in (-0.18, 0.0, 0.18):
        hip(x, 0.05, z + 0.62, 0.14, 0.18, 0.08, M('gold'), ridge=0.4)
    for sx in (-1, 1):
        box(sx * 0.36, 0.05, g + 0.3, 0.2, 0.2, 0.3, M('white'))
        hip(sx * 0.36, 0.05, g + 0.6, 0.22, 0.22, 0.07, M('gold'), ridge=0.5)
    for k in range(5):
        tree(-0.6 + k * 0.3, -0.5, 0.7, 'pine', z=g)


def ruhr_valley():
    g = wp('concreteD', top='asphalt', side='stoneD')
    for (x, y, h) in [(-0.45, 0.35, 0.7), (-0.2, 0.42, 0.6)]:
        cyl(x, y, g, 0.07, h, M('steel'), n=14, r2=0.055)
        glow(x, y - 0.0, g + h, 0.08, 0.08, 0.02, '#ff7a30', strength=6)
        smokestack_puff(x, y, g + h + 0.02, 1.4)
    for (x, y) in [(0.45, 0.3), (0.65, 0.2)]:
        chimney(x, y, g, 0.04, 0.6, smoke=3, mat='brickD')
    box(0.1, 0.2, g, 0.5, 0.3, 0.22, M('brick'))
    sawtooth(0.1, 0.2, g + 0.22, 0.5, 0.3, 0)
    lathe(0.62, -0.1, g, [(0.13, 0), (0.095, 0.2), (0.11, 0.38)], M('concrete'), n=16, cap0=False, cap1=False)
    smokestack_puff(0.62, -0.1, g + 0.38, 1.4)
    for k in range(3):
        cyl(-0.45 + 0.17 * k, -0.2, g, 0.07, 0.24, M('steel'), n=12)
        cyl(-0.45 + 0.17 * k, -0.2, g + 0.24, 0.07, 0.02, M('iron'), n=12)
    box(0.0, -0.1, g, 0.9, 0.04, 0.03, M('iron'))
    box(-0.2, -0.0, g + 0.02, 0.04, 0.5, 0.04, M('copper'))
    for k in range(5):
        box(-0.3 + 0.15 * k, -0.4, g, 0.1, 0.05, 0.05, M('dark'))
    glow(-0.25, 0.1, g, 0.12, 0.05, 0.07, '#ff7a30', strength=7)
    box(0.0, -0.55, g, 0.95, 0.1, 0.014, M('iron'))


def bolshoi_theatre():
    g = wp('paveL')
    box(0, 0.15, g, 0.9, 0.55, 0.36, M('cream'))
    windows(0, 0.15, g, 0.9, 0.55, 0.36, 7, 2, z0=0.2, z1=0.6)
    stair_ramp(0, -0.38, g, 0.6, 0.12, 0.07, 3, 'stone')
    for i in range(8):
        col(-0.35 + i * 0.1, -0.12, g + 0.07, 0.022, 0.32, M('marble'))
    box(0, -0.12, g + 0.39, 0.82, 0.12, 0.03, M('marble'))
    gable(0, -0.12, g + 0.42, 0.82, 0.14, 0.12, M('marble'), mat_end=M('stone'))
    box(0, 0.15, g + 0.36, 0.92, 0.57, 0.03, M('marble'))
    gable(0, 0.2, g + 0.39, 0.4, 0.7, 0.16, M('green'), rot=90, overhang=0.02)
    # quadriga on top
    for k in range(4):
        box(-0.045 + k * 0.03, -0.12, g + 0.54, 0.02, 0.04, 0.06, M('gold'))
    box(0.0, -0.1, g + 0.54, 0.05, 0.04, 0.03, M('gold'))
    sphere(0.0, -0.1, g + 0.57, 0.02, M('gold'), n=6, rings=4)
    tree(-0.65, -0.35, 0.8, 'round', z=g)
    tree(0.65, -0.35, 0.8, 'round', z=g)


def oxford_university():
    g = wp('grass')
    stone = M('sand')
    for (x, y, w, d) in [(0, 0.45, 0.95, 0.2), (0, -0.4, 0.95, 0.2), (-0.5, 0.0, 0.2, 0.7), (0.5, 0.0, 0.2, 0.7)]:
        box(x, y, g, w, d, 0.26, stone)
        box(x, y, g + 0.26, w * 1.01, d * 1.01, 0.02, M('stone2'))
        gable(x, y, g + 0.28, w * 0.98, d * 0.98, min(w, d) * 0.4, M('slate'))
    windows(0, -0.4, g, 0.95, 0.2, 0.26, 6, 2)
    windows(0, 0.45, g, 0.95, 0.2, 0.26, 6, 2)
    lawn_patch(0, 0.0, g, 0.7, 0.55)
    for (x, y) in [(-0.5, 0.45), (0.5, 0.45), (-0.5, -0.4), (0.5, -0.4)]:
        box(x, y, g, 0.14, 0.14, 0.42, M('sand'))
        for sx in (-1, 1):
            for sy in (-1, 1):
                cone(x + sx * 0.06, y + sy * 0.06, g + 0.42, 0.018, 0.1, M('stone2'), n=5)
        pyr(x, y, g + 0.42, 0.1, 0.1, 0.12, M('slate'))
    box(0, 0.45, g + 0.26, 0.15, 0.16, 0.3, M('sand'))
    pyr(0, 0.45, g + 0.56, 0.17, 0.17, 0.22, M('slate'))
    cyl(0.0, 0.0, g, 0.04, 0.05, M('stone2'), n=10)


def big_ben():
    g = wp('paveL')
    box(0.0, -0.35, g, 1.0, 0.07, 0.012, M('water', rough=0.05))
    box(0.0, 0.15, g, 0.96, 0.24, 0.2, M('sand'))
    windows(0, 0.15, g, 0.96, 0.24, 0.2, 8, 2)
    gable(0, 0.15, g + 0.2, 0.96, 0.26, 0.12, M('slate'))
    for k in range(5):
        pyr(-0.4 + 0.2 * k, 0.15, g + 0.2, 0.05, 0.05, 0.14, M('stone2'))
    x, y = -0.4, 0.0
    box(x, y, g, 0.15, 0.15, 0.7, M('sand'), taper=0.92)
    box(x, y, g + 0.7, 0.19, 0.19, 0.22, M('sand'))
    box(x, y, g + 0.92, 0.2, 0.2, 0.025, M('stone2'))
    with at(x, y - 0.0975, g + 0.82):
        _face(M('white'), [(-0.065, 0, -0.065), (0.065, 0, -0.065), (0.065, 0, 0.065), (-0.065, 0, 0.065)])
        _face(M('dark'), [(0.0, -0.001, -0.05), (0.008, -0.001, -0.05), (0.008, -0.001, 0.0), (0.0, -0.001, 0.0)])
        _face(M('dark'), [(0.0, -0.001, 0.0), (0.04, -0.001, 0.0), (0.04, -0.001, 0.008), (0.0, -0.001, 0.008)])
    pyr(x, y, g + 0.945, 0.17, 0.17, 0.3, M('slateL'))
    cone(x, y, g + 1.24, 0.01, 0.1, M('gold'), n=4)
    tower(0.4, 0.2, g, 0.07, 0.34, 'sand', roof='slate', rh=0.18)
    tree(0.55, -0.15, 0.8, 'round', z=g)


def hermitage():
    g = wp('paveL')
    box(0, 0.1, g, 1.05, 0.46, 0.34, M('green', rgb='#8fd1b0', rough=0.8))
    windows(0, 0.1, g, 1.05, 0.46, 0.34, 9, 2)
    for i in range(9):
        box(-0.46 + i * 0.115, -0.125, g, 0.03, 0.025, 0.34, M('white'))
    box(0, 0.1, g + 0.34, 1.08, 0.48, 0.03, M('white'))
    box(0, 0.1, g + 0.37, 0.98, 0.4, 0.05, M('green', rgb='#8fd1b0'))
    box(0, 0.1, g + 0.34, 0.98, 0.5, 0.0, M('white'))
    for i in range(11):
        person(-0.5 + i * 0.1, -0.1, g + 0.42, M('white'), s=1.2)
    box(0, -0.12, g + 0.32, 0.4, 0.04, 0.06, M('white'))
    for sx in (-1, 1):
        cyl(sx * 0.3, 0.1, g + 0.42, 0.1, 0.12, M('white'), n=14)
        dome(sx * 0.3, 0.1, g + 0.54, 0.1, M('gold'), h=0.12, n=14)
    cyl(0.0, 0.1, g + 0.42, 0.1, 0.1, M('white'), n=14)
    dome(0.0, 0.1, g + 0.52, 0.1, M('green'), h=0.14, n=14)
    lawn_patch(0, -0.5, g, 0.9, 0.2)
    path(0.0, -0.6, 0.0, -0.14, g)


def eiffel_tower():
    g = wp('grass')
    iron = M('iron', rgb='#6d5a48', rough=0.6, metal=0.4)
    lawn_patch(0, 0, g, 1.2, 1.1, mat='grassD')
    path(-0.7, -0.5, 0.7, 0.5, g, w=0.1)
    # four legs curve inward
    prof = [(0.34, 0.0), (0.26, 0.2), (0.17, 0.42), (0.1, 0.62)]
    for (sx, sy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)]:
        rings_ = []
        for r_, h_ in prof:
            for_pts = [(sx * r_ * 1.0 + dx, sy * r_ * 1.0 + dy, g + h_) for (dx, dy) in [(-0.025, -0.025), (0.025, -0.025), (0.025, 0.025), (-0.025, 0.025)]]
            rings_.append(for_pts)
        loft(rings_, iron)
    # platforms
    box(0, 0, g + 0.42, 0.42, 0.42, 0.03, iron)
    box(0, 0, g + 0.45, 0.36, 0.36, 0.03, M('steel'))
    for h_, w_ in [(0.0, 0.0)]:
        pass
    # arches between legs
    for sx in (-1, 1):
        box(sx * 0.26, 0, g + 0.2, 0.025, 0.3, 0.02, iron)
        box(0, sx * 0.26, g + 0.2, 0.3, 0.025, 0.02, iron)
    # upper shaft
    box(0, 0, g + 0.48, 0.24, 0.24, 0.5, iron, taper=0.4)
    box(0, 0, g + 0.98, 0.1, 0.1, 0.02, iron)
    box(0, 0, g + 0.7, 0.2, 0.2, 0.02, M('steel'))
    cyl(0, 0, g + 1.0, 0.018, 0.34, iron, n=6, r2=0.004)
    for k in range(6):
        box(0, 0, g + 0.5 + k * 0.08, 0.2 - k * 0.025, 0.012, 0.012, iron)
        box(0, 0, g + 0.5 + k * 0.08, 0.012, 0.2 - k * 0.025, 0.012, iron)
    tree(-0.6, 0.45, 1.0, 'round', z=g)
    tree(0.6, 0.4, 1.0, 'round', z=g)
    tree(0.62, -0.4, 0.9, 'round', z=g)


def broadway():
    g = wp('asphalt', side='stoneD', top='asphalt')
    road_strip(0, -0.8, 0, 0.8, 0.12, g, M('concreteD'))
    cols = ['#ff3f8a', '#37d4ff', '#ffd23a', '#7dff5a']
    for i, (x, y, w, h) in enumerate([(-0.35, 0.3, 0.4, 0.5), (0.35, 0.3, 0.4, 0.4), (-0.35, -0.2, 0.4, 0.3), (0.35, -0.2, 0.4, 0.45)]):
        box(x, y, g, w, 0.3, h, M('brick' if i % 2 else 'brickD'))
        windows(x, y, g, w, 0.3, h, 3, 3)
        glow(x, y - 0.151, g + h * 0.5, w * 0.8, 0.012, 0.06, cols[i], strength=8)
        glow(x, y - 0.151, g + h * 0.2, w * 0.5, 0.012, 0.03, cols[(i + 1) % 4], strength=8)
        box(x, y, g + h, w * 1.02, 0.31, 0.015, M('stone'))
    box(0.0, 0.0, g + 0.4, 0.5, 0.012, 0.12, M('dark'))
    glow(0.0, -0.0075, g + 0.4, 0.46, 0.003, 0.09, '#ffdf50', strength=9)
    for k in range(4):
        cyl(-0.12 + k * 0.08, 0.0, g, 0.01, 0.4, M('iron'), n=5)
    point_light(0, -0.3, g + 0.3, (1.0, 0.6, 0.8), power=2.0, size=0.1)


def cristo_redentor():
    g = wp('grass')
    mountain(0, 0.0, g, 0.62, 0.55, 'rock', n=11, seed=11)
    for k in range(7):
        a = 2 * math.pi * k / 7
        tree(0.68 * math.cos(a), 0.68 * math.sin(a) - 0.05, 0.8, 'round', z=g)
    z = g + 0.58
    box(0, 0, z, 0.2, 0.2, 0.18, M('stone'), taper=0.8)
    statue(0, 0, z + 0.18, 1.1, mat='white', arms='wide')


def maracana():
    g = wp('paveL')
    R = 0.66
    n = 20
    cyl(0, 0.0, g, R * 0.7, 0.01, M('grass'), n=28)
    for t in range(3):
        cyl(0, 0.0, g, R - 0.0 + 0.0, 0.0, M('white'))
    lathe(0, 0.0, g, [(R * 0.62, 0.02), (R * 0.84, 0.16), (R * 1.0, 0.3), (R * 1.0, 0.34)], M('white'), n=28, cap0=False, cap1=False)
    lathe(0, 0.0, g, [(R * 1.02, 0.34), (R * 0.86, 0.37), (R * 0.76, 0.35)], M('blue'), n=28, cap0=False, cap1=False)
    for k in range(n):
        a = 2 * math.pi * k / n
        box(R * 0.98 * math.cos(a), R * 0.98 * math.sin(a), g, 0.04, 0.04, 0.34, M('concrete'), rot=math.degrees(a))
    for k in range(4):
        a = math.radians(45 + 90 * k)
        cyl(R * 1.05 * math.cos(a), R * 1.05 * math.sin(a), g, 0.012, 0.55, M('steel'), n=6)
        box(R * 1.05 * math.cos(a), R * 1.05 * math.sin(a), g + 0.55, 0.08, 0.03, 0.04, M('yellow', emit='#fff2a0', emit_strength=3))
    box(0, 0.0, g + 0.012, 0.2, 0.02, 0.003, M('white'))
    for sx in (-1, 1):
        box(sx * 0.28, 0.0, g + 0.012, 0.01, 0.22, 0.003, M('white'))
    flag(0.0, -R * 0.95, g + 0.34, 0.18, M('green'), w=0.1)


def sydney_opera_house():
    bl.USE_CATCHER = True
    g = 0.0
    box(0.0, 0.0, g, 1.2, 0.8, 0.07, M('stone'))
    stair_ramp(0.0, -0.55, g, 0.5, 0.15, 0.07, 4, 'stone')
    for i, (x, y, w, h, r) in enumerate([(-0.38, 0.1, 0.55, 0.42, 20), (-0.1, 0.1, 0.5, 0.52, 20), (0.2, 0.1, 0.46, 0.38, 20),
                                         (-0.3, -0.12, 0.28, 0.22, 15), (0.28, -0.1, 0.26, 0.2, 15)]):
        sail(x, y, g + 0.07, w, h, 0.14, M('white', rough=0.35), rot=r - 20)
        sail(x - 0.0, y - 0.0, g + 0.07, w * 0.82, h * 0.8, 0.2, M('cream', rough=0.35), rot=r - 20)
    box(0.0, -0.32, g + 0.07, 0.9, 0.1, 0.05, M('glass'))


def huey_teocalli():
    g = wp('stone')
    stepped(0, 0.0, g, 0.98, 6, 0.1, 0.07, 'stone', alt='stone2')
    z = g + 0.6
    for sx in (-1, 1):
        box(sx * 0.14, 0.0, z, 0.2, 0.22, 0.16, M('stone2'))
        box(sx * 0.14, 0.0, z + 0.16, 0.24, 0.26, 0.03, M('red' if sx < 0 else 'blue'))
        box(sx * 0.14, -0.112, z, 0.06, 0.01, 0.1, M('dark'))
    stair_ramp(0, -0.48, g, 0.16, 0.5, 0.58, 12, 'stone2')
    flag(0.0, 0.0, z + 0.2, 0.2, M('red'), w=0.1)
    for (x, y) in [(-0.62, -0.4), (0.62, -0.4)]:
        tree(x, y, 1.0, 'palm', z=g)


def apadana():
    g = wp('stone')
    stepped(0, 0.1, g, 1.1, 2, 0.05, 0.03, 'stone', d=0.9)
    z = g + 0.1
    for i in range(6):
        for j in range(4):
            col(-0.4 + i * 0.16, -0.2 + j * 0.2, z, 0.025, 0.4 - 0.0, M('sand'), n=10)
    box(0, 0.45, z + 0.4, 0.95, 0.3, 0.04, M('terra'))
    stair_ramp(0, -0.5, g, 0.3, 0.12, 0.1, 4, 'stone2')
    for sx in (-1, 1):
        box(sx * 0.62, -0.3, g, 0.1, 0.1, 0.3, M('sandD'))
        box(sx * 0.62, -0.3, g + 0.3, 0.14, 0.12, 0.04, M('sand'))
    tree(-0.65, 0.5, 0.9, 'palm', z=g)
    tree(0.65, 0.5, 0.9, 'palm', z=g)


def mausoleum():
    g = wp('paveL')
    stepped(0, 0, g, 0.9, 2, 0.06, 0.04, 'stone')
    z = g + 0.12
    box(0, 0, z, 0.6, 0.6, 0.2, M('marble'))
    for i in range(6):
        col(-0.3 + i * 0.12, -0.34, z, 0.02, 0.2, M('marble'))
        col(-0.3 + i * 0.12, 0.34, z, 0.02, 0.2, M('marble'))
    for j in range(1, 5):
        col(-0.34, -0.3 + j * 0.12, z, 0.02, 0.2, M('marble'))
        col(0.34, -0.3 + j * 0.12, z, 0.02, 0.2, M('marble'))
    for (bx, by, bw, bd) in [(0, -0.34, 0.76, 0.06), (0, 0.34, 0.76, 0.06), (-0.34, 0, 0.06, 0.76), (0.34, 0, 0.06, 0.76)]:
        box(bx, by, z + 0.2, bw, bd, 0.03, M('marble'))
    box(0, 0, z + 0.2, 0.62, 0.62, 0.03, M('marble'))
    stepped(0, 0, z + 0.23, 0.6, 7, 0.045, 0.03, 'marble')
    box(0, 0, z + 0.545, 0.2, 0.12, 0.03, M('gold'))
    for k in range(4):
        box(-0.045 + k * 0.03, 0, z + 0.575, 0.02, 0.05, 0.05, M('gold'))
    box(0.0, 0.0, z + 0.575, 0.03, 0.04, 0.06, M('gold'))
    tree(-0.62, -0.45, 0.8, 'pine', z=g)
    tree(0.62, -0.45, 0.8, 'pine', z=g)


def jebel_barkal():
    g = wp('sand', side='sandD')
    mountain(0.1, 0.25, g, 0.7, 0.85, 'adobe', n=9, seed=21)
    cyl(0.15, 0.28, g + 0.7, 0.05, 0.4, M('adobe'), n=7, r2=0.035)
    cone(0.15, 0.28, g + 1.1, 0.035, 0.05, M('sandD'), n=7)
    for k in range(4):
        pyr(-0.55 + 0.28 * k, -0.45, g, 0.2, 0.2, 0.28, M('stone'))
    box(-0.3, -0.1, g, 0.34, 0.2, 0.18, M('sand'))
    for i in range(4):
        col(-0.42 + i * 0.08, -0.22, g, 0.014, 0.18, M('cream'), n=8)
    tree(0.55, -0.2, 1.0, 'palm', z=g)


def angkor_wat():
    g = wp('grass')
    # moat
    ext(_hex_pts(0.86), g - 0.0, 0.002, M('water', rough=0.05))
    ext(_hex_pts(0.66), g, 0.012, M('grass'))
    stepped(0, 0.0, g, 0.7, 3, 0.07, 0.07, 'stone', alt='stone2')
    z = g + 0.21
    for (x, y, h, s) in [(0, 0, 0.5, 0.15), (-0.2, 0.0, 0.34, 0.1), (0.2, 0.0, 0.34, 0.1), (0, 0.2, 0.34, 0.1), (0, -0.2, 0.34, 0.1)]:
        box(x, y, z, s, s, h * 0.6, M('stone2'), taper=0.85)
        lathe(x, y, z + h * 0.6, [(s * 0.42, 0), (s * 0.5, h * 0.18), (s * 0.36, h * 0.38), (s * 0.18, h * 0.55), (0.0, h * 0.7)], M('stone2'), n=12)
    box(0, -0.78, g, 0.12, 0.3, 0.02, M('stone'))
    road_strip(0, -0.8, 0, -0.5, 0.1, g, M('stone'))
    tree(-0.65, 0.2, 1.0, 'palm', z=g)
    tree(0.65, 0.2, 1.0, 'palm', z=g)


def statue_of_zeus():
    g = wp('paveL')
    for i in range(5):
        col(-0.4 + i * 0.2, -0.4, g, 0.03, 0.5, M('marble'), n=12)
        col(-0.4 + i * 0.2, 0.4, g, 0.03, 0.5, M('marble'), n=12)
    for j in range(1, 4):
        col(-0.4, -0.4 + j * 0.2, g, 0.03, 0.5, M('marble'), n=12)
        col(0.4, -0.4 + j * 0.2, g, 0.03, 0.5, M('marble'), n=12)
    box(0, 0.4, g + 0.5, 0.95, 0.12, 0.04, M('marble'))
    gable(0, 0.4, g + 0.54, 0.95, 0.14, 0.1, M('terra'), mat_end=M('marble'))
    with at(0, 0.05, g, s=1.5):
        box(0, 0.12, 0, 0.34, 0.26, 0.12, M('stone'))
        box(0, 0.2, 0.12, 0.34, 0.06, 0.32, M('woodL'))
        box(0, 0.0, 0.12, 0.2, 0.2, 0.06, M('gold'))
        box(0, -0.07, 0.0, 0.16, 0.14, 0.12, M('gold'))
        box(0, 0.12, 0.18, 0.2, 0.14, 0.2, M('gold'), taper=0.85)
        sphere(0, 0.12, 0.38, 0.055, M('cream'), n=10, rings=6)
        box(-0.13, 0.05, 0.26, 0.04, 0.12, 0.04, M('gold'))
        box(0.13, 0.05, 0.26, 0.04, 0.12, 0.04, M('gold'))
        cyl(0.18, 0.0, 0, 0.01, 0.5, M('goldD'), n=5)
        sphere(0.18, 0.0, 0.47, 0.025, M('gold'), n=8, rings=5)


def biosphere():
    g = wp('pave')
    pool(0, 0, g, 1.0, 0.9, rim='stone', water='water')
    icosphere(0, 0, g + 0.5, 0.5, M('glass', rough=0.1, metal=0.2), sub=2)
    cyl(0, 0, g + 0.015, 0.45, 0.0, M('steel'), n=3)
    lathe(0, 0, g + 0.015, [(0.38, 0.0), (0.46, 0.0), (0.46, 0.015), (0.38, 0.015)], M('steel'), n=24, cap0=False, cap1=False)
    for k in range(6):
        a = math.pi * k / 3
        cyl(0.42 * math.cos(a), 0.42 * math.sin(a), g + 0.0, 0.015, 0.2, M('steel'), n=6)
    sphere(0, 0, g + 0.3, 0.12, M('leaf'), n=10, rings=6)
    tree(-0.1, 0.0, 1.5, 'round', z=g + 0.02)
    tree(0.12, 0.05, 1.2, 'round', z=g + 0.02)


def etemenanki():
    g = wp('sand', side='sandD')
    stepped(0, 0.0, g, 0.95, 5, 0.13, 0.09, 'adobe', alt='sandD')
    z = g + 0.65
    box(0, 0, z, 0.26, 0.26, 0.14, M('turq', rough=0.3))
    box(0, 0, z + 0.14, 0.3, 0.3, 0.02, M('gold'))
    stair_ramp(0, -0.48, g, 0.14, 0.48, 0.62, 10, 'sand')
    tree(-0.62, 0.4, 1.0, 'palm', z=g)
    tree(0.62, 0.4, 1.0, 'palm', z=g)


def torre_de_belem():
    g = wp('stone')
    box(0.0, -0.3, g, 1.0, 0.25, 0.012, M('water', rough=0.05))
    box(0.0, 0.2, g, 0.8, 0.4, 0.12, M('stone'))
    box(0.1, 0.2, g + 0.12, 0.34, 0.34, 0.5, M('white', rgb='#e8e0cc'))
    windows(0.1, 0.2, g + 0.12, 0.34, 0.34, 0.5, 2, 3)
    box(0.1, 0.2, g + 0.62, 0.38, 0.38, 0.03, M('stone'))
    for (sx, sy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)]:
        cyl(0.1 + sx * 0.18, 0.2 + sy * 0.18, g + 0.62, 0.05, 0.1, M('white', rgb='#e8e0cc'), n=8)
        dome(0.1 + sx * 0.18, 0.2 + sy * 0.18, g + 0.72, 0.05, M('stone2'), h=0.07, n=8)
    box(0.1, 0.2, g + 0.65, 0.2, 0.2, 0.12, M('white', rgb='#e8e0cc'))
    pyr(0.1, 0.2, g + 0.77, 0.22, 0.22, 0.12, M('terra'))
    for k in range(6):
        box(-0.3 + 0.08 * k, 0.04, g + 0.12, 0.05, 0.05, 0.05, M('stone2'))
    box(-0.25, 0.2, g + 0.12, 0.26, 0.2, 0.14, M('cream'))
    flag(0.1, 0.2, g + 0.89, 0.12, M('red'), w=0.08)


def generic():
    g = wp('paveL')
    stepped(0, 0, g, 0.9, 3, 0.05, 0.06, 'stone')
    z = g + 0.15
    ring(0, 0, z, 0.32, 8, lambda x, y, a, k: col(x, y, z, 0.03, 0.35, M('marble')))
    cyl(0, 0, z, 0.2, 0.3, M('cream'), n=16)
    for k in range(4):
        a = math.pi / 2 * k + math.pi / 4
        box(0.32 * math.cos(a), 0.32 * math.sin(a), z + 0.35, 0.22, 0.22, 0.03, M('marble'), rot=math.degrees(a))
    dome(0, 0, z + 0.3, 0.26, M('gold'), h=0.24, n=18)
    cone(0, 0, z + 0.54, 0.012, 0.14, M('gold'), n=6)
    for (x, y) in [(-0.62, -0.4), (0.62, -0.4)]:
        box(x, y, g, 0.07, 0.07, 0.3, M('stone'), taper=0.5)
        cone(x, y, g + 0.3, 0.05, 0.08, M('gold'), n=4)


WONDERS = {
    'stonehenge': stonehenge, 'hanging_gardens': hanging_gardens, 'pyramids': pyramids, 'oracle': oracle,
    'great_lighthouse': great_lighthouse, 'colossus': colossus, 'petra': petra, 'colosseum': colosseum,
    'great_library': great_library, 'mahabodhi_temple': mahabodhi_temple, 'terracotta_army': terracotta_army,
    'hagia_sophia': hagia_sophia, 'alhambra': alhambra, 'chichen_itza': chichen_itza, 'mont_st_michel': mont_st_michel,
    'venetian_arsenal': venetian_arsenal, 'great_zimbabwe': great_zimbabwe, 'forbidden_city': forbidden_city,
    'potala_palace': potala_palace, 'ruhr_valley': ruhr_valley, 'bolshoi_theatre': bolshoi_theatre,
    'oxford_university': oxford_university, 'big_ben': big_ben, 'hermitage': hermitage, 'eiffel_tower': eiffel_tower,
    'broadway': broadway, 'cristo_redentor': cristo_redentor, 'estadio_do_maracana': maracana,
    'sydney_opera_house': sydney_opera_house, 'huey_teocalli': huey_teocalli, 'apadana': apadana,
    'halicarnassus_mausoleum': mausoleum, 'jebel_barkal': jebel_barkal, 'angkor_wat': angkor_wat,
    'statue_of_zeus': statue_of_zeus, 'biosphere': biosphere, 'etemenanki': etemenanki, 'torre_de_belem': torre_de_belem,
    'generic': generic,
}

SPRITES = {f'wonder_{k}': (v, False) for k, v in WONDERS.items()}
