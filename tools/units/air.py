"""Aircraft (nose towards +X, wings across Y, origin at the centre of mass)."""
import math
from lib import *


def _prop(x, y=0.0, z=0.0, r=0.55, col="#333333"):
    sph((x, y, z), 0.12, "#c8ccd0", scale=(1.4, 1, 1), seg=8, rings=5, name="spinner")
    for k in range(3):
        a = math.radians(k * 120 + 20)
        limb((x, y, z), (x, y + r * math.cos(a), z + r * math.sin(a)), 0.045, col, r1=0.025, seg=5)


def _wing(pts_right, col, z=0.0, thick=0.07, stripe=None):
    """Mirror-symmetric wing from planform points for the +Y side (x,y)."""
    left = [(x, -y) for (x, y) in reversed(pts_right)]
    right = list(pts_right)
    poly(right + [(x, -y) for (x, y) in reversed(right)][0:0], col, plane="XY", offset=z, thick=thick, bevel=0.01, name="wingR")
    poly([(x, -y) for (x, y) in pts_right], col, plane="XY", offset=z, thick=thick, bevel=0.01, name="wingL")


def biplane(o=None):
    fus = "#d8c8a0"
    tube([(-1.5, 0.03, 0.04, 0.0), (-0.8, 0.12, 0.16, 0.02), (0.0, 0.2, 0.24, 0.05), (0.7, 0.2, 0.22, 0.05), (1.0, 0.16, 0.18, 0.04)], fus, n=10, name="fuselage")
    sph((1.0, 0, 0.05), 0.2, "#7a7f84", scale=(0.8, 1, 1), name="cowl")
    _prop(1.2, 0, 0.05, 0.6)
    for (z, off) in ((0.55, 0.0), (-0.15, 0.1)):
        w = [(0.5 + off, 0.0), (0.5 + off, 1.6), (0.0 + off, 1.6), (0.0 + off, 0.0)]
        _wing(w, "team" if z > 0.3 else fus, z=z, thick=0.06)
        for sy in (-1, 1):
            box((0.25 + off, sy * 1.5, z), (0.5, 0.04, 0.08), "team" if z <= 0.3 else "white", bevel=0.005)
    for sy in (-1, 1):
        for sx in (0.05, 0.45):
            limb((sx, sy * 0.9, 0.55), (sx + 0.05, sy * 0.9, -0.12), 0.025, "dkwood", seg=4)
        limb((0.25, sy * 0.2, 0.15), (0.3, sy * 0.4, 0.5), 0.02, "dkwood", seg=4)
    # tail
    poly([(-1.1, 0.0), (-1.1, 0.5), (-1.5, 0.45), (-1.55, 0.0)], fus, plane="XY", thick=0.05, name="tailplane")
    poly([(-1.1, 0.0), (-1.1, -0.5), (-1.5, -0.45), (-1.55, 0.0)], fus, plane="XY", thick=0.05)
    poly([(-1.0, 0.1), (-1.5, 0.05), (-1.55, 0.55), (-1.25, 0.55)], "team", plane="XZ", thick=0.05, name="fin")
    cyl((0.35, 0, 0.3), 0.12, 0.03, "black", rot=(0, 0, 0), seg=8)
    sph((0.3, 0, 0.28), 0.1, "skin")


def _fuselage(L, r, col, nose=1.0):
    tube([(-L, 0.03, 0.03, 0.0), (-L * 0.6, r * 0.5, r * 0.55, 0.02), (-L * 0.1, r, r, 0.0), (L * 0.35, r * 0.95, r * 0.95, 0.0),
          (L * nose * 0.8, r * 0.6, r * 0.6, 0.0), (L * nose, r * 0.2, r * 0.2, 0.0)], col, n=12, name="fuselage")


def fighter(o=None, mustang=False):
    col = "#c8ced4" if not mustang else "#d4d8dc"
    L = 1.5
    _fuselage(L, 0.2, col, nose=0.95)
    _prop(L * 0.95 + 0.05, 0, 0, 0.62)
    cyl((L * 0.88, 0, 0), 0.19, 0.2, "team" if mustang else "#3a3f46", rot=(0, 90, 0), seg=10, name="nosecowl")
    w = [(0.45, 0.0), (0.4, 0.5), (0.1, 1.55), (-0.1, 1.55), (-0.25, 0.5), (-0.35, 0.0)]
    _wing(w, col, z=-0.1, thick=0.07)
    for sy in (-1, 1):
        box((0.0, sy * 1.45, -0.1), (0.25, 0.1, 0.09), "team", bevel=0.01, name="wingtip")
        box((0.15, sy * 0.9, -0.17), (0.5, 0.1, 0.06), "dkgrey")
    # canopy
    if mustang:
        sph((0.05, 0, 0.22), 0.2, "glass", scale=(1.6, 0.8, 1.0), seg=10, rings=6, name="canopy")
        box((-0.2, 0, -0.22), (0.5, 0.2, 0.15), "dkgrey", bevel=0.04)
    else:
        sph((0.2, 0, 0.2), 0.2, "glass", scale=(1.4, 0.8, 0.9), seg=10, rings=6, name="canopy")
    # tail
    poly([(-1.1, 0.0), (-1.05, 0.55), (-1.4, 0.5), (-1.5, 0.0)], col, plane="XY", thick=0.05)
    poly([(-1.1, 0.0), (-1.05, -0.55), (-1.4, -0.5), (-1.5, 0.0)], col, plane="XY", thick=0.05)
    poly([(-0.9, 0.05), (-1.4, 0.05), (-1.55, 0.6), (-1.25, 0.6)], "team", plane="XZ", thick=0.05, name="fin")


def p51(o=None):
    fighter(o, mustang=True)


def bomber(o=None):
    col = "#8a9078"
    L = 2.2
    _fuselage(L, 0.32, col, nose=0.9)
    sph((L * 0.88, 0, 0.0), 0.28, "glass", scale=(0.9, 0.9, 0.9), seg=10, rings=6, name="nose")
    w = [(0.5, 0.0), (0.45, 0.5), (0.15, 2.5), (-0.15, 2.5), (-0.4, 0.5), (-0.5, 0.0)]
    _wing(w, col, z=-0.12, thick=0.12)
    for sy in (-1, 1):
        for ey in (0.8, 1.6):
            cyl((0.55, sy * ey, -0.12), 0.14, 0.7, "#6a7060", rot=(0, 90, 0), seg=8)
            _prop(0.93, sy * ey, -0.12, 0.5)
        box((0.0, sy * 2.35, -0.12), (0.3, 0.2, 0.12), "team", bevel=0.01, name="wingtip")
    poly([(-1.6, 0.0), (-1.6, 0.9), (-2.1, 0.8), (-2.2, 0.0)], col, plane="XY", thick=0.07)
    poly([(-1.6, 0.0), (-1.6, -0.9), (-2.1, -0.8), (-2.2, 0.0)], col, plane="XY", thick=0.07)
    poly([(-1.3, 0.05), (-2.2, 0.05), (-2.3, 0.9), (-1.8, 0.95)], "team", plane="XZ", thick=0.07, name="fin")
    box((0.1, 0, 0.32), (0.4, 0.3, 0.14), col)
    sph((0.1, 0, 0.4), 0.14, "glass", scale=(1.2, 1, 0.8))
    box((0.1, 0, -0.1), (1.1, 0.07, 0.07), "team", name="belly stripe") if False else None


def jet_fighter(o=None):
    col = "#8a949e"
    L = 1.9
    tube([(-L, 0.14, 0.14, 0.0), (-L * 0.5, 0.22, 0.2, 0.0), (0.0, 0.26, 0.2, 0.0), (L * 0.5, 0.18, 0.14, 0.0), (L * 0.9, 0.08, 0.07, -0.02), (L, 0.02, 0.02, -0.03)], col, n=10, name="fuselage")
    sph((0.55, 0, 0.18), 0.22, "glass", scale=(2.0, 0.8, 0.9), seg=10, rings=6, name="canopy")
    w = [(0.5, 0.0), (0.3, 0.3), (-0.3, 1.45), (-0.65, 1.45), (-0.65, 0.0)]
    _wing(w, col, z=-0.05, thick=0.06)
    for sy in (-1, 1):
        box((-0.55, sy * 1.45, -0.05), (0.25, 0.1, 0.07), "team", bevel=0.01, name="wingtip")
        box((0.15, sy * 0.32, 0.0), (0.7, 0.28, 0.3), "#555c64", bevel=0.05, name="intake")
        cyl((-L - 0.02, sy * 0.14, 0.0), 0.12, 0.2, "#3a3e44", rot=(0, 90, 0), seg=8, name="nozzle")
        poly([(-1.1, sy * 0.2), (-1.55, sy * 0.2), (-1.7, sy * 0.85), (-1.45, sy * 0.85)], col, plane="XY", thick=0.04)
        poly([(-1.0, sy * 0.25), (-1.6, sy * 0.25), (-1.65, sy * 0.4), (-1.1, sy * 0.35)], "team", plane="XZ", offset=0, thick=0.04) if False else None
        with xf((0, sy * 0.28, 0.2), (sy * 18, 0, 0)):
            poly([(-1.0, 0.0), (-1.6, 0.0), (-1.6, 0.6), (-1.35, 0.6)], "team", plane="XZ", thick=0.04, name="fin")


def jet_bomber(o=None):
    col = "#5f6770"
    L = 2.4
    tube([(-L * 0.9, 0.2, 0.12, 0.0), (-L * 0.4, 0.3, 0.2, 0.0), (0.2, 0.34, 0.22, 0.0), (L * 0.6, 0.2, 0.14, 0.0), (L, 0.04, 0.04, -0.02)], col, n=10, name="fuselage")
    sph((L * 0.55, 0, 0.16), 0.2, "glass", scale=(1.6, 0.8, 0.8))
    w = [(0.8, 0.0), (0.2, 0.7), (-1.2, 2.6), (-1.5, 2.6), (-1.5, 0.0)]
    _wing(w, col, z=-0.06, thick=0.09)
    for sy in (-1, 1):
        box((-1.35, sy * 2.5, -0.06), (0.4, 0.2, 0.1), "team", bevel=0.01, name="wingtip")
        for ey in (0.9, 1.5):
            cyl((-0.15, sy * ey, -0.2), 0.17, 1.1, "#454c54", rot=(0, 90, 0), seg=8, name="engine")
            cyl((0.42, sy * ey, -0.2), 0.14, 0.08, "#1c1e22", rot=(0, 90, 0), seg=8)
        poly([(-1.5, sy * 0.2), (-2.0, sy * 0.2), (-2.2, sy * 0.9), (-1.9, sy * 0.9)], col, plane="XY", thick=0.05)
    with xf((0, 0, 0.0)):
        poly([(-1.4, 0.0), (-2.2, 0.0), (-2.3, 0.9), (-1.9, 0.9)], "team", plane="XZ", thick=0.06, name="fin")


def drone_air(o=None):
    pass
