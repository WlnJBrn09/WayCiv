"""Reusable building parts shared by districts, wonders and cities."""
import math

from bl import *  # noqa: F401,F403
from bl import _face


def _win_quad(mat, frame, x, cz, w, h, plane, off, side):
    """One window: a pale frame with the dark pane inset, on the front (-y) or right (+x) face."""
    for m, grow, o in ((frame, 0.012, 0.0015), (mat, 0.0, 0.0028)):
        hw, hh = w / 2 + grow, h / 2 + grow
        if side == 'f':
            _face(m, [(x - hw, plane - o, cz - hh), (x + hw, plane - o, cz - hh), (x + hw, plane - o, cz + hh),
                      (x - hw, plane - o, cz + hh)])
        else:
            _face(m, [(plane + o, x - hw, cz - hh), (plane + o, x + hw, cz - hh), (plane + o, x + hw, cz + hh),
                      (plane + o, x - hw, cz + hh)])


def windows(x, y, z, w, d, h, cols, rows, mat=None, rot=0.0, sides='fr', ww=None, wh=None, z0=0.25, z1=0.82,
            frame=None):
    """Window quads (pale frame, dark pane) on the front (-y) and/or right (+x) faces of a block."""
    mat = mat or M('glassD', rough=0.1)
    frame = frame or M('cream')
    if cols <= 0 or rows <= 0:
        return
    with at(x, y, z, rot):
        if 'f' in sides:
            ww_ = ww or min(0.05, w / cols * 0.45)
            wh_ = wh or min(0.07, h / rows * 0.5)
            for i in range(cols):
                cx = (-0.5 + (i + 0.5) / cols) * w * 0.86
                for j in range(rows):
                    cz = h * (z0 + (z1 - z0) * (j / max(rows - 1, 1) if rows > 1 else 0.5))
                    _win_quad(mat, frame, cx, cz, ww_, wh_, -d / 2, 0, 'f')
        if 'r' in sides:
            n = max(1, int(d / (w / cols))) if cols else 1
            ww_ = ww or min(0.05, d / n * 0.45)
            wh_ = wh or min(0.07, h / rows * 0.5)
            for i in range(n):
                cy = (-0.5 + (i + 0.5) / n) * d * 0.86
                for j in range(rows):
                    cz = h * (z0 + (z1 - z0) * (j / max(rows - 1, 1) if rows > 1 else 0.5))
                    _win_quad(mat, frame, cy, cz, ww_, wh_, w / 2, 0, 'r')


def door(x, y, z, w, d, dw=0.06, dh=0.1, mat=None, rot=0.0):
    mat = mat or M('woodD')
    with at(x, y, z, rot):
        _face(mat, [(-dw / 2, -d / 2 - 0.003, 0), (dw / 2, -d / 2 - 0.003, 0), (dw / 2, -d / 2 - 0.003, dh),
                    (-dw / 2, -d / 2 - 0.003, dh)])


def house(x, y, z, w, d, h, wall='stone', roof='terra', rot=0.0, rh=None, style='gable', win=True, floors=None):
    box(x, y, z, w, d, h, M(wall), rot)
    rh = rh if rh is not None else min(w, d) * 0.45
    if style == 'gable':
        gable(x, y, z + h, w, d, rh, M(roof), rot, overhang=0.015)
    elif style == 'hip':
        hip(x, y, z + h, w + 0.02, d + 0.02, rh, M(roof), rot, ridge=0.45)
    elif style == 'flat':
        box(x, y, z + h, w + 0.02, d + 0.02, 0.015, M(roof), rot)
    elif style == 'pyr':
        pyr(x, y, z + h, w + 0.02, d + 0.02, rh, M(roof), rot)
    if win:
        f = floors or max(1, int(round(h / 0.11)))
        windows(x, y, z, w, d, h, max(1, int(w / 0.12)), f, rot=rot)
        door(x, y, z, w, d, rot=rot)


def hut(x, y, z, r=0.09, h=0.09, wall='adobe', roof='thatch'):
    cyl(x, y, z, r, h, M(wall), n=12)
    cone(x, y, z + h, r * 1.3, r * 1.1, M(roof), n=12)


def tent(x, y, z, s=0.14, mat='cream', rot=0.0, h=None):
    """A square pavilion tent with a coloured skirt and a door flap."""
    h = h or s * 0.9
    with at(x, y, z, rot):
        box(0, 0, 0, s, s, h * 0.28, M(mat))
        pyr(0, 0, h * 0.28, s * 1.04, s * 1.04, h * 0.72, M(mat))
        box(0, 0, h * 0.2, s * 1.01, s * 1.01, h * 0.08, M('red'))
        _face(M('dark'), [(-s * 0.1, -s / 2 - 0.002, 0), (s * 0.1, -s / 2 - 0.002, 0), (s * 0.07, -s / 2 - 0.002, h * 0.26), (-s * 0.07, -s / 2 - 0.002, h * 0.26)])
        cyl(0, 0, h, 0.004, 0.05, M('woodD'), n=4)


def tower(x, y, z, r, h, wall='stone', roof='terra', crenel=False, rh=None):
    cyl(x, y, z, r, h, M(wall), n=14, r2=r * 0.92)
    if crenel:
        cyl(x, y, z + h, r * 1.12, 0.035, M(wall), n=14)
        for k in range(8):
            a = 2 * math.pi * k / 8
            box(x + math.cos(a) * r * 1.0, y + math.sin(a) * r * 1.0, z + h + 0.035, r * 0.35, r * 0.35, 0.04, M(wall),
                rot=math.degrees(a))
    else:
        cone(x, y, z + h, r * 1.25, rh if rh is not None else r * 2.2, M(roof), n=14)


def chimney(x, y, z, r=0.03, h=0.3, smoke=0, mat='brickD'):
    cyl(x, y, z, r, h, M(mat), n=10, r2=r * 0.8)
    cyl(x, y, z + h, r * 0.95, 0.02, M('dark'), n=10)
    for i in range(smoke):
        blob(x + 0.02 * i, y + 0.006 * i, z + h + 0.025 + 0.05 * i, 0.028 + 0.013 * i, M('white', rgb='#cfccc8', rough=1.0, style='plain'),
             seed=i + 3, sub=1, squash=1.0, jitter=0.2)


def crate(x, y, z, s=0.05, mat='woodL', rot=0.0):
    box(x, y, z, s, s, s, M(mat), rot)


def barrel(x, y, z, r=0.022, h=0.045, mat='wood'):
    lathe(x, y, z, [(r * 0.8, 0), (r, h * 0.5), (r * 0.8, h)], M(mat), n=8)


def fence(x0, y0, x1, y1, z, h=0.05, mat='woodL', posts=True, step=0.07):
    dx, dy = x1 - x0, y1 - y0
    L = math.hypot(dx, dy)
    n = max(1, int(L / step))
    rot = math.degrees(math.atan2(dy, dx))
    box((x0 + x1) / 2, (y0 + y1) / 2, z + h * 0.55, L, 0.012, 0.012, M(mat), rot)
    for i in range(n + 1):
        t = i / n
        cyl(x0 + dx * t, y0 + dy * t, z, 0.009, h, M(mat), n=5)


def palisade(x0, y0, x1, y1, z, h=0.1, mat='woodL', step=0.035):
    dx, dy = x1 - x0, y1 - y0
    L = math.hypot(dx, dy)
    n = max(1, int(L / step))
    for i in range(n + 1):
        t = i / n
        cyl(x0 + dx * t, y0 + dy * t, z, 0.014, h, M(mat), n=5)
        cone(x0 + dx * t, y0 + dy * t, z + h, 0.014, 0.03, M(mat), n=5)


def wall_seg(x0, y0, x1, y1, z, h=0.12, t=0.05, wall='stone', crenel=True):
    dx, dy = x1 - x0, y1 - y0
    L = math.hypot(dx, dy)
    rot = math.degrees(math.atan2(dy, dx))
    box((x0 + x1) / 2, (y0 + y1) / 2, z, L, t, h, M(wall), rot)
    if crenel:
        n = max(1, int(L / 0.07))
        for i in range(n):
            tt = (i + 0.5) / n
            box(x0 + dx * tt, y0 + dy * tt, z + h, L / n * 0.55, t * 0.8, 0.03, M(wall), rot)


def stall(x, y, z, w=0.14, d=0.1, awning='red', rot=0.0, goods='orange'):
    with at(x, y, z, rot):
        box(0, 0, 0, w, d, 0.045, M('woodL'))
        box(0, -d * 0.1, 0.045, w * 0.7, d * 0.5, 0.02, M(goods))
        for sx in (-1, 1):
            for sy in (-1, 1):
                cyl(sx * (w / 2 - 0.01), sy * (d / 2 - 0.01), 0, 0.006, 0.12, M('woodD'), n=5)
        box(0, -0.0, 0.12, w + 0.02, d + 0.03, 0.012, M(awning), rot=0, taper=1.0)
        gable(0, 0, 0.125, w + 0.02, d + 0.03, 0.04, M(awning))


def temple(x, y, z, w, d, ncols=6, ch=0.2, wall='marble', roof='terra', rot=0.0, step=2, cella=True, roof_mat=None):
    """Greek temple: stepped base, cella, front/back + side colonnades, pediment."""
    with at(x, y, z, rot):
        steps(0, 0, 0, w, d, step, 0.025, 0.012, M('stone'))
        z0 = 0.025 * step
        if cella:
            box(0, 0, z0, w * 0.62, d * 0.7, ch, M(wall))
        r = min(w, d) * 0.035
        colonnade(0, 0, z0, w / 2 - r * 2.2, d / 2 - r * 2.2, ncols, max(3, int(ncols * d / w) + 1), r, ch, M('marble'))
        box(0, 0, z0 + ch, w * 0.96, d * 0.94, 0.03, M(wall))
        gable(0, 0, z0 + ch + 0.03, w * 0.96, d * 0.94, min(w, d) * 0.22, M(roof_mat or roof), mat_end=M(wall), overhang=0.0)


def pool(x, y, z, w, d, rim='marble', water='water', rot=0.0, h=0.015):
    with at(x, y, z, rot):
        box(0, 0, 0, w, d, h, M(rim))
        box(0, 0, h, w * 0.82, d * 0.8, 0.004, M(water, rough=0.05))


def lawn_patch(x, y, z, w, d, rot=0.0, mat='grass'):
    box(x, y, z, w, d, 0.008, M(mat), rot)


def path(x0, y0, x1, y1, z=0.0, w=0.07, mat='paveL'):
    dx, dy = x1 - x0, y1 - y0
    L = math.hypot(dx, dy)
    box((x0 + x1) / 2, (y0 + y1) / 2, z, L, w, 0.01, M(mat), rot=math.degrees(math.atan2(dy, dx)))


def solar_panel(x, y, z, w=0.14, d=0.09, tilt=25, rot=0.0):
    with at(x, y, z, rot):
        cyl(0, 0, 0, 0.008, 0.03, M('steel'), n=5)
        with at(0, 0, 0.03):
            t = math.radians(tilt)
            pts = [(-w / 2, -d / 2 * math.cos(t), -d / 2 * math.sin(t)), (w / 2, -d / 2 * math.cos(t), -d / 2 * math.sin(t)),
                   (w / 2, d / 2 * math.cos(t), d / 2 * math.sin(t)), (-w / 2, d / 2 * math.cos(t), d / 2 * math.sin(t))]
            _face(M('royal', rough=0.15, metal=0.4), pts)
            _face(M('steel'), list(reversed(pts)))


def dish(x, y, z, r=0.06, rot=0.0, tilt=50):
    with at(x, y, z, rot):
        cyl(0, 0, 0, 0.01, 0.06, M('steel'), n=6)
        with at(0, 0, 0.06, 0):
            for k in range(1):
                lathe(0, 0, 0, [(0, -r * 0.0), (r * 0.5, r * 0.15), (r, r * 0.45)], M('white'), n=14)


def server_rack(x, y, z, w=0.06, d=0.08, h=0.1, rot=0.0, led='#37d4ff'):
    with at(x, y, z, rot):
        box(0, 0, 0, w, d, h, M('dark'))
        n = 5
        for i in range(n):
            box(0, -d / 2 - 0.001, h * (0.12 + 0.78 * i / (n - 1)) - 0.004, w * 0.78, 0.004, 0.008,
                M('led' + led, rgb=led, emit=led, emit_strength=8.0))


def fan(x, y, z, r=0.07, mat='steel'):
    cyl(x, y, z, r, 0.04, M('concreteD'), n=16)
    cyl(x, y, z + 0.04, r * 0.92, 0.005, M('dark'), n=16)
    for k in range(3):
        a = 120 * k
        box(x, y, z + 0.046, r * 1.5, r * 0.28, 0.006, M(mat), rot=a)


def ship(x, y, z, L=0.32, W=0.1, rot=0.0, hull='woodD', sail=True, deck='woodL', sail_mat='white', s=1.0):
    with at(x, y, z, rot, s):
        n = 8
        rings = []
        for i in range(n + 1):
            t = i / n
            px = (t - 0.5) * L
            wd = W / 2 * (1 - (2 * t - 1) ** 2 * 0.0) * (1 - max(0, t - 0.55) ** 2 * 3.2) * (0.35 + 0.65 * math.sin(math.pi * min(t * 1.05 + 0.05, 1)))
            hh = 0.06 + 0.02 * t
            rings.append([(px, -wd, hh), (px, -wd * 0.7, 0.0), (px, wd * 0.7, 0.0), (px, wd, hh)])
        loft(rings, M(hull), closed=False, cap0=True, cap1=True)
        box(0, 0, 0.05, L * 0.8, W * 0.7, 0.012, M(deck))
        if sail:
            cyl(0, 0, 0.05, 0.007, 0.28, M('woodD'), n=5)
            box(0.0, 0, 0.12, 0.004, W * 1.3, 0.16, M(sail_mat), taper=1.0)
            box(L * 0.2, 0, 0.1, 0.004, W * 1.0, 0.12, M(sail_mat))


def rocket(x, y, z, h=0.45, r=0.045, body='white', band='red'):
    lathe(x, y, z, [(r, 0), (r, h * 0.7), (r * 0.5, h * 0.92), (0.0, h)], M(body, rough=0.4), n=14)
    cyl(x, y, z + h * 0.35, r * 1.02, h * 0.06, M(band), n=14)
    for k in range(4):
        with at(x, y, z, rot=90 * k + 45):
            _face(M(band), [(r, 0, 0), (r * 2.4, 0, 0), (r * 1.0, 0, h * 0.28)])
            _face(M(band), [(r * 1.0, 0, h * 0.28), (r * 2.4, 0, 0), (r, 0, 0)])


def plane(x, y, z, rot=0.0, s=1.0, body='white', wing='red'):
    with at(x, y, z + 0.03, rot, s):
        loft([[(-0.14, -0.012, -0.012), (-0.14, 0.012, -0.012), (-0.14, 0.012, 0.012), (-0.14, -0.012, 0.012)],
              [(0.05, -0.03, -0.025), (0.05, 0.03, -0.025), (0.05, 0.03, 0.03), (0.05, -0.03, 0.03)],
              [(0.14, -0.012, -0.01), (0.14, 0.012, -0.01), (0.14, 0.012, 0.01), (0.14, -0.012, 0.01)]], M(body))
        box(0.02, 0, 0, 0.07, 0.3, 0.008, M(wing))
        box(-0.12, 0, 0.0, 0.05, 0.12, 0.006, M(wing))
        box(-0.125, 0, 0.0, 0.05, 0.006, 0.07, M(wing))


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


def stepped(x, y, z, w, n, step_h, shrink, mat, d=None, rot=0.0, alt=None):
    d = w if d is None else d
    for i in range(n):
        box(x, y, z + i * step_h, w - 2 * shrink * i, d - 2 * shrink * i, step_h, M(alt if (alt and i % 2) else mat), rot)


def stair_ramp(x, y, z, w, length, rise, n, mat, rot=0.0):
    """Stairs going up toward +y (local), starting at y=0."""
    with at(x, y, z, rot):
        for i in range(n):
            box(0, length * (i + 0.5) / n, 0, w, length / n, rise * (i + 1) / n, M(mat))
