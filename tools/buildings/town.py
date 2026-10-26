"""A kit of small, detailed buildings and street props for city centres and districts.

Every function builds in its own local frame (`at(x, y, g, rot)`), faces -y (towards the camera),
and takes the ground height `g`.  `ACC` parts are painted in the owner's colour.
"""
import math
import random

import bl
from bl import _face
from parts import *  # noqa: F401,F403

ACC = 'acc'


def strip(a, b, w, mat, y=0.0, side='f'):
    """A thin flat bar from a to b ((x, z) pairs) on a front face at depth y (timber braces)."""
    (x0, z0), (x1, z1) = a, b
    L = math.hypot(x1 - x0, z1 - z0)
    nx, nz = -(z1 - z0) / L * w / 2, (x1 - x0) / L * w / 2
    _face(mat, [(x0 - nx, y, z0 - nz), (x1 - nx, y, z1 - nz), (x1 + nx, y, z1 + nz), (x0 + nx, y, z0 + nz)])


def hexd(x, y):
    """Distance of a point from the centre, measured to the hex outline (flat-to-flat = 1.73)."""
    return max(abs(x), abs(0.5 * x + 0.866 * y), abs(0.5 * x - 0.866 * y)) * 1.1547


def layout(rnd, n, keep, extent=0.74, sx=0.25, sy=0.21, jitter=0.03, bias=0.0):
    """Jittered grid positions inside the plot, closest to the centre first, avoiding `keep` rectangles
    given as (cx, cy, half_w, half_h)."""
    cells = []
    for iy in range(-5, 6):
        for ix in range(-5, 6):
            x = ix * sx + (iy % 2) * sx / 2 + rnd.uniform(-jitter, jitter)
            y = iy * sy + rnd.uniform(-jitter, jitter) - 0.04
            if hexd(x, y) > extent:
                continue
            if any(abs(x - kx) < hw and abs(y - ky) < hh for kx, ky, hw, hh in keep):
                continue
            cells.append((math.hypot(x, y * 1.2) + rnd.uniform(0, 0.12) + bias * y, x, y))
    cells.sort()
    return [(x, y) for _d, x, y in cells[:n]]


def facing(x, y):
    """Rotation that turns a house's front towards its street."""
    if abs(x) > 0.42 and abs(y) < 0.42:
        return 90 if x < 0 else -90
    return 0


def road(x0, y0, x1, y1, g, w=0.09, mat='dirt'):
    """A street strip that fades out with the ground patch."""
    kind, tone = bl.GROUND_KIND.get(mat, ('dirt', 1.0))
    dx, dy = x1 - x0, y1 - y0
    L = math.hypot(dx, dy)
    road.count += 1                       # stagger heights so crossing streets never z-fight
    box((x0 + x1) / 2, (y0 + y1) / 2, g - 0.0045 + 0.0006 * (road.count % 5), L, w, 0.007,
        G(kind, tone, 0.7, 0.92), rot=math.degrees(math.atan2(dy, dx)))


road.count = 0


def banner(x, y, g, h=0.3, w=0.07, rot=0.0, mat=None, pole='woodD'):
    """A pole with a hanging cloth banner (owner colour by default)."""
    mat = mat or M(ACC)
    with at(x, y, g, rot):
        cyl(0, 0, 0, 0.007, h, M(pole), n=5)
        cyl(0, 0, h, 0.012, 0.012, M('gold'), n=6)
        box(w / 2 * 0.0, 0, h - 0.012, w * 1.1, 0.006, 0.006, M(pole))
        _face(mat, [(-w / 2, -0.003, h - 0.014), (w / 2, -0.003, h - 0.014), (w / 2, -0.003, h - 0.014 - w * 1.9),
                    (0, -0.003, h - 0.014 - w * 1.55), (-w / 2, -0.003, h - 0.014 - w * 1.9)])


def pennant(x, y, g, h=0.24, w=0.09, rot=0.0, mat=None):
    """A flagpole with a swallow-tailed streamer."""
    mat = mat or M(ACC)
    with at(x, y, g, rot):
        cyl(0, 0, 0, 0.006, h, M('woodD'), n=5)
        _face(mat, [(0, 0, h), (w, 0.0, h - 0.012), (w * 0.8, 0.0, h - w * 0.28), (w, 0.0, h - w * 0.55), (0, 0, h - w * 0.5)])


def smoke_plume(x, y, z, n=4, s=1.0, tone='#cfccc8'):
    for i in range(n):
        blob(x + 0.018 * i * s, y + 0.008 * i, z + 0.04 * i * s, (0.026 + 0.014 * i) * s,
             M('white', rgb=tone, rough=1.0, style='plain'), seed=i + 3, sub=1, squash=1.0, jitter=0.2)


def person(x, y, g, tunic='red', s=1.0, rot=0.0):
    """A tiny townsperson."""
    cyl(x, y, g, 0.016 * s, 0.05 * s, M(tunic), n=6, r2=0.011 * s)
    sphere(x, y, g + 0.046 * s, 0.014 * s, M('cream', rgb='#e8c9a0'), n=6, rings=4)


def crowd(g, spots, tunics=('red', 'blue', 'cream', 'orange', 'green', 'purple')):
    for k, (x, y) in enumerate(spots):
        person(x, y, g, tunics[k % len(tunics)])


def well(x, y, g, s=1.0):
    cyl(x, y, g, 0.04 * s, 0.045 * s, M('stone2'), n=10)
    cyl(x, y, g + 0.04 * s, 0.03 * s, 0.004, M('waterD'), n=10)
    for sx in (-1, 1):
        cyl(x + sx * 0.034 * s, y, g + 0.04 * s, 0.004, 0.06 * s, M('woodD'), n=4)
    box(x, y, g + 0.095 * s, 0.09 * s, 0.01, 0.01, M('woodD'))
    gable(x, y, g + 0.1 * s, 0.1 * s, 0.07 * s, 0.03 * s, M('terra'), overhang=0.005)


def fountain(x, y, g, s=1.0, mat='marble'):
    cyl(x, y, g, 0.1 * s, 0.03 * s, M(mat), n=18)
    cyl(x, y, g + 0.03 * s, 0.085 * s, 0.006, M('water', rough=0.05), n=18)
    lathe(x, y, g + 0.03 * s, [(0.03 * s, 0), (0.014 * s, 0.05 * s), (0.04 * s, 0.09 * s), (0.04 * s, 0.1 * s), (0.0, 0.1 * s)],
          M(mat), n=12, cap0=False)
    cyl(x, y, g + 0.12 * s, 0.006, 0.05 * s, M('foam', rough=0.3), n=6, r2=0.012)


def hay(x, y, g, s=1.0):
    blob(x, y, g, 0.06 * s, M('thatch'), seed=int(x * 100), sub=1, squash=0.8)


def cart(x, y, g, rot=0.0, load='thatch'):
    with at(x, y, g, rot):
        box(0, 0, 0.025, 0.11, 0.06, 0.02, M('wood'))
        box(0, 0, 0.045, 0.1, 0.055, 0.025, M(load))
        for sy in (-1, 1):
            cyl(-0.0, sy * 0.034, 0.0, 0.024, 0.008, M('woodD'), n=10)


def lamp(x, y, g, h=0.12, glowc='#ffd890'):
    cyl(x, y, g, 0.004, h, M('dark'), n=5)
    box(x, y, g + h, 0.016, 0.016, 0.02, M('glow_' + glowc, rgb=glowc, emit=glowc, emit_strength=5.0))


# ---------------------------------------------------------------------------
# ancient
# ---------------------------------------------------------------------------

def adobe_house(x, y, g, rnd, rot=0.0, variant=None, awning=False):
    v = variant if variant is not None else rnd.randrange(4)
    wall = rnd.choice(['adobe', 'sand', 'adobe', 'sandD'])
    with at(x, y, g, rot):
        if v == 0:
            w, d, h = 0.17, 0.15, 0.1
            box(0, 0, 0, w, d, h, M(wall))
            box(0, 0, h, w + 0.02, d + 0.02, 0.014, M('sand'))
            box(0, 0, h + 0.014, w - 0.02, d - 0.02, 0.004, M('sandD'))
            door(0, 0, 0, w, d, 0.04, 0.065)
            windows(0, 0, 0, w, d, h, 1, 1, ww=0.03, wh=0.03, z0=0.62, z1=0.62, sides='f')
            if awning:
                with at(0.0, -d / 2 - 0.04, 0.07):
                    _face(M(ACC), [(-0.06, 0.0, 0.0), (0.06, 0.0, 0.0), (0.06, -0.055, -0.03), (-0.06, -0.055, -0.03)])
        elif v == 1:
            box(0, 0, 0, 0.2, 0.16, 0.085, M(wall))
            box(0, 0, 0.085, 0.215, 0.175, 0.012, M('sand'))
            box(-0.025, 0.01, 0.097, 0.12, 0.1, 0.08, M(wall))
            box(-0.025, 0.01, 0.177, 0.135, 0.115, 0.012, M('sand'))
            door(0, 0, 0, 0.2, 0.16, 0.04, 0.06)
            windows(-0.025, 0.01, 0.097, 0.12, 0.1, 0.08, 1, 1, ww=0.026, wh=0.03, z0=0.5, z1=0.5, sides='f')
            if awning:
                box(0.07, -0.085, 0.07, 0.07, 0.04, 0.008, M(ACC))
        elif v == 2:
            r = 0.085
            cyl(0, 0, 0, r, 0.08, M(wall), n=14)
            cone(0, 0, 0.08, r * 1.3, r * 1.2, M('thatch'), n=14)
            cyl(0, -r + 0.005, 0, 0.002, 0.0, M('woodD'), n=3)
            door(0, 0, 0, r * 2 - 0.01, r * 2 - 0.01, 0.035, 0.055)
        else:
            r = 0.075
            cyl(0, 0, 0, r, 0.07, M(wall), n=12)
            dome(0, 0, 0.07, r * 1.02, M('sand'), h=r * 1.05, n=14)
            door(0, 0, 0, r * 2 - 0.01, r * 2 - 0.01, 0.035, 0.055)


def ziggurat(x, y, g, w=0.5, n=3, step=0.07, shrink=0.075, mats=('sand', 'adobe', 'sandD'), rot=0.0, ramp=True):
    with at(x, y, g, rot):
        for i in range(n):
            ww = w - 2 * shrink * i
            box(0, 0, i * step, ww, ww * 0.82, step, M(mats[i % len(mats)]))
            box(0, 0, i * step + step, ww + 0.012, ww * 0.82 + 0.012, 0.008, M('sand'))
        if ramp:
            stair_ramp(0, -w * 0.41 - 0.0, 0, 0.09, w * 0.41 - shrink * (n - 1) * 0.5, step * n, 3 * n, 'sandD')
    return g + step * n


def ancient_hall(g, capital, tier):
    top = ziggurat(0, 0.06, g, w=0.56 if capital else 0.4, n=3 if capital else 2, step=0.075 if capital else 0.065,
                   shrink=0.075 if capital else 0.065)
    if capital:
        # columned pavilion with a tiled gable roof and a great banner
        box(0, 0.06, top, 0.22, 0.17, 0.02, M('stone'))
        for i in range(4):
            for sy in (-1, 1):
                col(-0.085 + i * 0.057, 0.06 + sy * 0.07, top + 0.02, 0.012, 0.1, M('marble'), n=8)
        box(0, 0.06, top + 0.12, 0.23, 0.18, 0.015, M('marble'))
        gable(0, 0.06, top + 0.135, 0.24, 0.19, 0.075, M('terra'), overhang=0.01, mat_end=M('sand'))
        banner(0.0, 0.06, top + 0.15, 0.17, 0.07)
        for sx in (-1, 1):
            box(sx * 0.37, -0.12, g, 0.04, 0.04, 0.2, M('sand'), taper=0.55)
            pyr(sx * 0.37, -0.12, g + 0.2, 0.022, 0.022, 0.03, M('gold'))
    else:
        box(0, 0.06, top, 0.16, 0.12, 0.07, M('adobe'))
        box(0, 0.06, top + 0.07, 0.18, 0.14, 0.012, M('sand'))
        door(0, 0.06, top, 0.16, 0.12, 0.04, 0.05)
        banner(0.0, 0.06, top + 0.08, 0.15, 0.06)


def ancient(tier, capital, rnd_seed=1):
    def build():
        g = bl.plot('dirt', top='dirt')
        rnd = random.Random(40 + tier + (5 if capital else 0))
        s = {1: 1.6, 2: 1.42, 3: 1.3, 4: 1.2}[tier]
        with at(0, 0, 0, 0, s):
            n = {1: 3, 2: 6, 3: 11, 4: 16}[tier]
            road(0, -0.9, 0, -0.1, g, 0.1, 'earth')
            if tier >= 2:
                road(-0.8, -0.05, 0.8, -0.05, g, 0.08, 'earth')
            keep = [(0, 0.06, 0.34 if capital else 0.28, 0.3 if capital else 0.24), (0, -0.46, 0.11, 0.4)]
            for k, (x, y) in enumerate(layout(rnd, n, keep, extent=0.82 / s)):
                adobe_house(x, y, g, rnd, rot=facing(x, y), awning=(k % 3 == 1))
            ancient_hall(g, capital, tier)
            for (x, y, sc) in [(-0.62, 0.34, 1.0), (0.64, 0.4, 0.9), (-0.66, -0.35, 0.9), (0.6, -0.4, 0.8)][:2 + tier]:
                tree(x, y, sc, 'palm', g)
            if tier >= 2:
                well(0.18, -0.3, g)
            if tier >= 3:
                hay(-0.32, -0.42, g)
                hay(0.38, -0.5, g, 0.8)
                cart(0.3, -0.28, g, rot=10)
                crowd(g, [(-0.1, -0.5), (0.1, -0.38), (0.0, -0.22)])
            if tier >= 4:
                for sx in (-1, 1):
                    cyl(sx * 0.6, 0.0, g, 0.06, 0.12, M('adobe'), n=12)
                    dome(sx * 0.6, 0.0, g + 0.12, 0.06, M('sand'), h=0.06, n=12)
                stall(-0.2, -0.55, g, 0.14, 0.1, awning=ACC, goods='orange')
    return build


def rod(p0, p1, r, mat, r1=None, n=6):
    """A round bar between two points (limbs, lattice work, rigging)."""
    from mathutils import Vector
    a, b = Vector(p0), Vector(p1)
    d = b - a
    if d.length < 1e-6:
        return
    d.normalize()
    ref = Vector((0, 0, 1)) if abs(d.z) < 0.9 else Vector((1, 0, 0))
    u = d.cross(ref).normalized()
    v = d.cross(u).normalized()
    r1 = r if r1 is None else r1
    rings = []
    for pt, rr in ((a, r), (b, r1)):
        rings.append([tuple(pt + (u * math.cos(2 * math.pi * k / n) + v * math.sin(2 * math.pi * k / n)) * rr) for k in range(n)])
    loft(rings, mat)


def rock_hill(x, y, z, r, h, seed=1, mat='rock', top='grass', n=14, plateau=0.68):
    """A craggy hill with a flat top (the top material may be None to taper to a point)."""
    rnd = random.Random(seed)
    levels = [(1.0, 0.0), (0.9, 0.35), (0.8, 0.72), (plateau, 1.0)]
    jit = [1 + (rnd.random() - 0.5) * 0.3 for _ in range(n)]
    rings = []
    for rr, hh in levels:
        rings.append([(x + r * rr * jit[i] * math.cos(2 * math.pi * i / n), y + r * rr * jit[i] * math.sin(2 * math.pi * i / n),
                       z + h * hh + (rnd.random() - 0.5) * 0.02 * (hh < 1)) for i in range(n)])
    loft(rings, M(mat), cap0=False, cap1=False)
    if top:
        _face(M(top), list(reversed(rings[-1])))
    else:
        _face(M(mat), list(reversed(rings[-1])))
    return z + h
