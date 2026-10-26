"""Sailing ships and harbour furniture."""
import math

from bl import _face
from town import *  # noqa: F401,F403


def hull(L, W, H, mat, bow=0.9, stern_lift=0.4, n=9):
    """A lofted hull along local x (bow at +x): keel at z=0, gunwale at z=H."""
    rings = []
    for i in range(n + 1):
        t = i / n
        px = (t - 0.5) * L
        if t < 0.18:
            wd = W / 2 * (0.55 + 0.45 * (t / 0.18))
        elif t > 0.7:
            wd = W / 2 * max(0.05, 1 - ((t - 0.7) / 0.3) ** 1.6 * bow)
        else:
            wd = W / 2
        lift = stern_lift * H * (1 - t) ** 3 + 0.35 * H * t ** 5
        h = H + lift
        keel = 0.0 + 0.1 * H * (1 - math.sin(math.pi * min(t * 0.9 + 0.05, 1)))
        rings.append([(px, -wd, h), (px, -wd * 0.92, h * 0.55), (px, -wd * 0.55, keel),
                      (px, wd * 0.55, keel), (px, wd * 0.92, h * 0.55), (px, wd, h)])
    loft(rings, mat, closed=False, cap0=True, cap1=True)
    return rings


def sail(x, z0, z1, w, mat, rot=0.0, belly=0.025, billow=0.0):
    """A square sail hung in the y-z plane at local x, belled forward in +x."""
    nx, nz = 5, 4
    with at(x, 0, 0):
        for i in range(nx):
            for j in range(nz):
                def p(ii, jj):
                    u, v = ii / nx, jj / nz
                    return (belly * math.sin(math.pi * u) * math.sin(math.pi * v) + billow * v, -w / 2 + w * u, z0 + (z1 - z0) * v)
                _face(mat, [p(i, j), p(i + 1, j), p(i + 1, j + 1), p(i, j + 1)])


def galleon(x, y, z, L=0.5, rot=0.0, masts=2, hull_mat='wood', sail_mat='white', flag_mat=None, s=1.0):
    W = L * 0.3
    H = L * 0.1
    with at(x, y, z, rot, s):
        hull(L, W, H, M(hull_mat))
        box(0, 0, H * 0.92, L * 0.72, W * 0.8, 0.008, M('woodL'))
        box(-L * 0.36, 0, H * 0.9, L * 0.2, W * 0.9, H * 0.5, M(hull_mat))
        box(-L * 0.36, 0, H * 1.4, L * 0.22, W * 0.95, 0.008, M('woodL'))
        for k in range(masts):
            mx = L * (0.18 - 0.32 * k) if masts > 1 else 0.0
            mh = L * (0.62 - 0.1 * k)
            cyl(mx, 0, H * 0.9, 0.007, mh, M('woodD'), n=5)
            for j, (z0, z1, sw) in enumerate([(0.12, 0.42, 0.9), (0.46, 0.7, 0.62)]):
                zz0, zz1 = H + mh * z0 * 0.9, H + mh * z1 * 0.95
                box(mx + 0.004, 0, zz1, 0.006, W * sw * 1.15, 0.006, M('woodD'))
                sail(mx + 0.01, zz0, zz1, W * sw, M(sail_mat, rough=0.9), belly=0.022 * s)
        box(L * 0.45, 0, H * 1.2, L * 0.16, 0.004, 0.0, M('woodD'))
        cyl(-L * 0.42, 0, H * 1.45, 0.004, L * 0.2, M('woodD'), n=4)
        _face(M(flag_mat or 'red'), [(-L * 0.42, 0.0, H * 1.45 + L * 0.2), (-L * 0.42 - 0.08, 0.0, H * 1.45 + L * 0.19),
                                      (-L * 0.42 - 0.08, 0.0, H * 1.45 + L * 0.15), (-L * 0.42, 0.0, H * 1.45 + L * 0.14)])


def dhow(x, y, z, L=0.34, rot=0.0, s=1.0, sail_mat='cream'):
    """A small lateen-rigged boat."""
    W = L * 0.28
    H = L * 0.1
    with at(x, y, z, rot, s):
        hull(L, W, H, M('woodL'), bow=1.0)
        cyl(0, 0, H * 0.7, 0.006, L * 0.75, M('woodD'), n=5)
        a = (-L * 0.28, H + L * 0.16)
        for sgn in (1,):
            _face(M(sail_mat, rough=0.9), [(a[0], 0.0, a[1]), (0.0, 0.0, H + L * 0.7), (L * 0.32, 0.0, H + L * 0.08)])
            _face(M(sail_mat, rough=0.9), [(L * 0.32, 0.0, H + L * 0.08), (0.0, 0.0, H + L * 0.7), (a[0], 0.0, a[1])])


def rowboat(x, y, z, rot=0.0, s=1.0):
    with at(x, y, z, rot, s):
        hull(0.14, 0.05, 0.02, M('woodL'), bow=1.0, stern_lift=0.0, n=6)


def crane(x, y, g, rot=0.0, h=0.28, mat='woodD'):
    with at(x, y, g, rot):
        for sx in (-1, 1):
            cyl(sx * 0.03, 0, 0, 0.007, h, M(mat), n=5, r2=0.005)
        box(0, 0, h, 0.09, 0.012, 0.012, M(mat))
        box(0.07, 0, h - 0.006, 0.17, 0.01, 0.01, M(mat))
        cyl(0.15, 0, h - 0.07, 0.002, 0.07, M('dark'), n=3)
        box(0.15, 0, h - 0.09, 0.025, 0.025, 0.025, M('woodL'))


def lighthouse(x, y, g, h=0.5, r=0.07):
    cyl(x, y, g, r * 1.4, 0.03, M('stone'), n=14)
    lathe(x, y, g + 0.03, [(r, 0), (r * 0.75, h)], M('white'), n=16)
    for k in range(3):
        cyl(x, y, g + 0.03 + h * (0.15 + 0.3 * k), r * (0.98 - 0.22 * (0.15 + 0.3 * k) / 1.0), h * 0.12, M('red'), n=16, r2=r * (0.98 - 0.22 * (0.27 + 0.3 * k)))
    cyl(x, y, g + 0.03 + h, r * 1.1, 0.014, M('stone2'), n=16)
    cyl(x, y, g + 0.044 + h, r * 0.55, 0.06, M('glow_lh', rgb='#fff0b0', emit='#fff0b0', emit_strength=9.0), n=10)
    dome(x, y, g + 0.104 + h, r * 0.6, M('red'), h=r * 0.55, n=12)
    point_light(x, y, g + 0.08 + h, (1.0, 0.9, 0.6), power=2.0, size=0.04)
