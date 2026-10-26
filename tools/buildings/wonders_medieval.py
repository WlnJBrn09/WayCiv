"""Medieval and renaissance wonders."""
import math

import bl
from bl import _face
from districts import lawn, portico
from ships import crane, galleon
from town import *  # noqa: F401,F403
from wonders_ancient import figure


def minaret(x, y, z, h=0.6, r=0.026, mat='white', cap='slateL'):
    cyl(x, y, z, r * 1.8, 0.05, M(mat), n=10)
    cyl(x, y, z + 0.05, r, h, M(mat), n=12, r2=r * 0.8)
    for f in (0.45, 0.82):
        cyl(x, y, z + 0.05 + h * f, r * 1.6, 0.018, M(mat), n=12)
        cyl(x, y, z + 0.068 + h * f, r * 1.5, 0.012, M('stone2'), n=12)
    cyl(x, y, z + 0.05 + h, r * 1.2, 0.02, M(mat), n=12)
    cone(x, y, z + 0.07 + h, r * 1.3, r * 5.0, M(cap), n=12)
    cyl(x, y, z + 0.07 + h + r * 5.0, 0.003, 0.03, M('gold'), n=4)


def hagia_sophia():
    g = bl.plot('paveL')
    cream = M('cream', rgb='#e8d4a8', style='plaster')
    lead = M('slateL', rgb='#8f9bab', rough=0.4, style='plain')
    box(0, 0.0, g, 0.86, 0.7, 0.3, cream)
    for k in range(5):
        box(0, 0.0, g + 0.04 + k * 0.055, 0.874, 0.714, 0.018, M('terra', rgb='#c25a40'))
    windows(0, 0.0, g, 0.86, 0.7, 0.3, 7, 1, ww=0.04, wh=0.07, z0=0.6, z1=0.6, sides='f')
    for sx in (-1, 1):
        for k in range(3):
            box(sx * 0.44, -0.22 + k * 0.22, g, 0.07, 0.07, 0.3, cream)
    # semi-domes front and back, drum and grey lead dome
    for sy in (-1, 1):
        dome(0, sy * 0.25, g + 0.3, 0.2, lead, h=0.12, n=18)
    for sx in (-1, 1):
        dome(sx * 0.3, 0.0, g + 0.3, 0.16, lead, h=0.1, n=16)
    cyl(0, 0.0, g + 0.3, 0.27, 0.12, cream, n=32)
    for k in range(20):
        a = 2 * math.pi * k / 20
        box(0.27 * math.cos(a), 0.27 * math.sin(a), g + 0.35, 0.025, 0.02, 0.05, M('dark'), rot=math.degrees(a) + 90)
    dome(0, 0.0, g + 0.42, 0.28, lead, h=0.27, n=32)
    cyl(0, 0.0, g + 0.69, 0.006, 0.07, M('gold'), n=4)
    for (sx, sy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)]:
        minaret(sx * 0.55, sy * 0.4, g, h=0.66, r=0.025)
    fountain(0.0, -0.58, g, 0.6)
    for (x, y) in [(-0.4, -0.58), (0.4, -0.58)]:
        tree(x, y, 0.8, 'pine', g)


def alhambra():
    g = bl.plot('sand')
    red = M('terra', rgb='#c4694a', style='blocks')
    # Alcazaba in the back
    pts = [(-0.7, 0.2), (0.7, 0.2), (0.7, 0.7), (-0.7, 0.7)]
    for i in range(4):
        a, b = pts[i], pts[(i + 1) % 4]
        if i == 0:
            continue
        wall_seg(a[0], a[1], b[0], b[1], g, h=0.2, t=0.07, wall='terra')
    wall_seg(-0.7, 0.2, 0.7, 0.2, g, h=0.2, t=0.07, wall='terra')
    for (x, y, r, h) in [(-0.7, 0.2, 0.09, 0.4), (0.7, 0.2, 0.09, 0.36), (-0.7, 0.7, 0.09, 0.34), (0.7, 0.7, 0.1, 0.5)]:
        tower(x, y, g, r, h, 'terra', crenel=True)
    # Comares tower and the palace
    box(0.3, 0.45, g, 0.2, 0.2, 0.44, red)
    windows(0.3, 0.45, g, 0.2, 0.2, 0.44, 2, 2, ww=0.03, wh=0.05, z0=0.5, z1=0.8, sides='f')
    box(0.3, 0.45, g + 0.44, 0.23, 0.23, 0.025, M('terraD'))
    dome(0.3, 0.45, g + 0.465, 0.09, M('terra', rgb='#8a3d2c'), h=0.07, n=14)
    box(-0.15, 0.45, g, 0.6, 0.24, 0.22, red)
    windows(-0.15, 0.45, g, 0.6, 0.24, 0.22, 5, 2, ww=0.03, wh=0.05, z0=0.3, z1=0.75, sides='f')
    gable(-0.15, 0.45, g + 0.22, 0.62, 0.26, 0.09, M('terraD'), overhang=0.01, mat_end=red)
    # court of the lions: arcaded square with a fountain
    cyl(0.0, -0.05, g, 0.34, 0.012, M('marble'), n=4, r2=0.34)
    box(0.0, -0.05, g, 0.6, 0.4, 0.012, M('marble'))
    box(0.0, -0.05, g + 0.012, 0.5, 0.3, 0.006, M('water', rough=0.04))
    for k in range(9):
        col(-0.28 + k * 0.07, -0.25, g, 0.009, 0.13, M('marble'), n=8)
        col(-0.28 + k * 0.07, 0.15, g, 0.009, 0.13, M('marble'), n=8)
    for k in range(1, 5):
        col(-0.3, -0.25 + k * 0.08, g, 0.009, 0.13, M('marble'), n=8)
        col(0.3, -0.25 + k * 0.08, g, 0.009, 0.13, M('marble'), n=8)
    box(0.0, -0.25, g + 0.13, 0.64, 0.02, 0.025, M('cream'))
    box(0.0, 0.15, g + 0.13, 0.64, 0.02, 0.025, M('cream'))
    cyl(0.0, -0.05, g + 0.018, 0.05, 0.03, M('marble'), n=12)
    for k in range(5):
        tree(-0.62 + k * 0.31, -0.58, 0.7, 'pine', g)
    for (x, y) in [(-0.72, -0.2), (0.72, -0.2)]:
        tree(x, y, 1.0, 'pine', g)


def mont_st_michel():
    g = bl.plot('sand')
    # tidal flats and shallow water
    cyl(0.0, 0.0, g + 0.002, 0.85, 0.004, M('water', rough=0.05), n=40)
    cyl(0.0, 0.0, g + 0.006, 0.7, 0.002, G('sand', 0.95, 0.5, 0.95), n=40)
    top = rock_hill(0, 0.05, g, 0.6, 0.46, seed=7, mat='rock', top='rock', plateau=0.34)
    # village houses wind up the rock
    def slope(r):
        pts = [(0.2, 0.46), (0.48, 0.33), (0.54, 0.16), (0.6, 0.0)]
        for (r0, z0), (r1, z1) in zip(pts, pts[1:]):
            if r <= r1:
                return max(z1 + (z0 - z1) * (r1 - r) / (r1 - r0), 0.0) + 0.03
        return 0.0
    for ring, (r, zz, n) in enumerate([(0.55, 0.06, 8), (0.45, 0.17, 7), (0.35, 0.28, 6)]):
        for k in range(n):
            a = 2 * math.pi * (k + 0.3 * ring) / n + 0.3
            x, y = 0.05 + r * math.cos(a), 0.05 + r * 0.9 * math.sin(a)
            if math.sin(a) > 0.55 and ring == 2:
                continue
            w = 0.12
            with at(x, y, g + slope(r), math.degrees(a) + 90):
                box(0, 0, 0, w, 0.09, 0.07, M('cream'))
                gable(0, 0, 0.07, w + 0.01, 0.1, 0.06, M(['slate', 'terra', 'slate'][k % 3]), overhang=0.01, mat_end=M('cream'))
    # walls
    for k in range(10):
        a0, a1 = 2 * math.pi * k / 10, 2 * math.pi * (k + 1) / 10
        if 0.5 < a0 < 1.2:
            continue
        wall_seg(0.05 + 0.55 * math.cos(a0), 0.05 + 0.5 * math.sin(a0), 0.05 + 0.55 * math.cos(a1), 0.05 + 0.5 * math.sin(a1), g + 0.02, h=0.06, t=0.03, wall='stone', crenel=True)
    # the abbey
    z = top
    box(0.05, 0.05, z, 0.34, 0.28, 0.14, M('stone', rgb='#d9cdb0', style='blocks'))
    for k in range(4):
        box(-0.1 + k * 0.1 + 0.05, 0.05 - 0.15, z, 0.012, 0.03, 0.12, M('stone2'))
    gable(0.05, 0.05, z + 0.14, 0.36, 0.3, 0.1, M('slate'), mat_end=M('stone'))
    box(0.05, 0.05, z + 0.14, 0.12, 0.12, 0.16, M('stone', rgb='#d9cdb0', style='blocks'))
    pyr(0.05, 0.05, z + 0.3, 0.1, 0.1, 0.34, M('stone2'))
    figure_pos = (0.05, 0.05, z + 0.64)
    cyl(figure_pos[0], figure_pos[1], figure_pos[2], 0.004, 0.06, M('gold'), n=4)
    sphere(figure_pos[0], figure_pos[1], figure_pos[2] + 0.06, 0.01, M('gold'), n=6, rings=4)
    road(0.0, -0.9, 0.0, -0.5, g, 0.07, 'sand')
    for k in range(6):
        box(0.0, -0.9 + k * 0.08, g + 0.004, 0.06, 0.07, 0.006, M('stone2'))


def venetian_arsenal():
    g = bl.plot('paveL')
    basin = M('water', rough=0.04)
    box(0.0, -0.05, g - 0.001, 0.5, 0.9, 0.012, basin)
    brickc = M('brick', rgb='#c07a58', style='brick')
    for sx in (-1, 1):
        # long shipbuilding sheds with arched fronts
        box(sx * 0.5, 0.05, g, 0.26, 0.9, 0.16, brickc)
        for k in range(6):
            yy = -0.36 + k * 0.145
            box(sx * 0.5 - sx * 0.1315, yy + 0.05, g, 0.004, 0.08, 0.1, M('dark'))
            dome(sx * 0.5 - sx * 0.1315, yy + 0.05, g + 0.1, 0.04, M('dark'), h=0.0001, n=6)
        gable(sx * 0.5, 0.05, g + 0.16, 0.9, 0.28, 0.1, M('terraD'), rot=90, overhang=0.01, mat_end=brickc)
    # gate with two crenellated towers
    for sx in (-1, 1):
        box(sx * 0.2, -0.58, g, 0.16, 0.16, 0.34, M('white', rgb='#efe6d4', style='blocks'))
        box(sx * 0.2, -0.58, g + 0.34, 0.2, 0.2, 0.025, M('white'))
        for k in range(5):
            box(sx * 0.2 - 0.08 + k * 0.04, -0.58 - 0.09, g + 0.365, 0.025, 0.02, 0.03, M('white'))
        pyr(sx * 0.2, -0.58, g + 0.37, 0.14, 0.14, 0.14, M('terraD'))
    box(0.0, -0.58, g + 0.2, 0.24, 0.12, 0.13, M('white', rgb='#efe6d4', style='blocks'))
    # a ship on the slip and a crane
    galleon(0.0, 0.1, g + 0.01, L=0.6, rot=88, masts=2, hull_mat='woodD', sail_mat='cream', flag_mat='red')
    crane(-0.22, 0.45, g, rot=0, h=0.34)
    crane(0.24, 0.5, g, rot=180, h=0.3)
    cyl(-0.0, -0.78, g, 0.02, 0.2, M('stone2'), n=8)
    sphere(0.0, -0.78, g + 0.22, 0.02, M('gold'), n=6, rings=4)


def great_zimbabwe():
    g = bl.plot('grass')
    gran = M('stone2', rgb='#a89c86', style='blocks')

    def wall_ring(rx, ry, h, t, n, gap=None, wave=0.0):
        for k in range(n):
            if gap and gap[0] <= k < gap[1]:
                continue
            a0, a1 = 2 * math.pi * k / n, 2 * math.pi * (k + 1) / n
            p = (rx * math.cos(a0), ry * math.sin(a0))
            q = (rx * math.cos(a1), ry * math.sin(a1))
            L = math.hypot(q[0] - p[0], q[1] - p[1])
            rot = math.degrees(math.atan2(q[1] - p[1], q[0] - p[0]))
            box((p[0] + q[0]) / 2, (p[1] + q[1]) / 2, g, L * 1.04, t, h, gran, rot=rot, taper=1.0, tapery=0.8)
            box((p[0] + q[0]) / 2, (p[1] + q[1]) / 2, g + h, L * 1.04, t * 0.85, 0.02, M('stone', rgb='#8a7e69'), rot=rot)
    wall_ring(0.74, 0.56, 0.19, 0.1, 30, gap=(4, 6))
    wall_ring(0.48, 0.34, 0.15, 0.07, 22, gap=(15, 17))
    # the conical tower
    cyl(0.16, 0.1, g, 0.1, 0.36, gran, n=18, r2=0.074)
    cone(0.16, 0.1, g + 0.36, 0.08, 0.07, M('stone2'), n=18)
    for k in range(3):
        for j in range(8):
            a = 2 * math.pi * j / 8 + k * 0.4
            box(0.16 + 0.095 * math.cos(a) * (1 - 0.1 * k), 0.1 + 0.095 * math.sin(a) * (1 - 0.1 * k), g + 0.12 + k * 0.08, 0.02, 0.01, 0.02, M('stoneD'), rot=math.degrees(a) + 90)
    for (x, y) in [(-0.2, 0.0), (-0.1, -0.18), (0.3, -0.12), (-0.28, 0.14)]:
        hut(x, y, g, 0.07, 0.07)
    for (x, y) in [(-0.7, -0.5), (0.7, -0.5), (0.0, -0.72)]:
        tree(x, y, 1.0, 'round', g)
    tree(0.66, 0.5, 1.1, 'palm', g)


def forbidden_city():
    g = bl.plot('paveL')
    red = M('red', rgb='#b02a22', style='plaster')
    gold = M('gold', rgb='#e8b83a', rough=0.35, metal=0.5)
    # walls and corner towers
    wp = [(-0.75, -0.55), (0.75, -0.55), (0.75, 0.65), (-0.75, 0.65)]
    for i in range(4):
        a, b = wp[i], wp[(i + 1) % 4]
        if i == 0:
            wall_seg(a[0], a[1], -0.22, a[1], g, h=0.14, t=0.07, wall='red', crenel=False)
            wall_seg(0.22, a[1], b[0], b[1], g, h=0.14, t=0.07, wall='red', crenel=False)
        else:
            wall_seg(a[0], a[1], b[0], b[1], g, h=0.14, t=0.07, wall='red', crenel=False)
    for p in wp:
        box(p[0], p[1], g, 0.12, 0.12, 0.16, red)
        hip(p[0], p[1], g + 0.16, 0.18, 0.18, 0.08, gold, ridge=0.4)
        hip(p[0], p[1], g + 0.22, 0.12, 0.12, 0.07, gold, ridge=0.4)
    # Meridian gate
    box(0, -0.55, g, 0.5, 0.16, 0.14, red)
    box(0, -0.55, g + 0.14, 0.54, 0.18, 0.02, M('marble'))
    box(0, -0.55, g + 0.16, 0.4, 0.12, 0.1, red)
    hip(0, -0.55, g + 0.26, 0.5, 0.18, 0.1, gold, ridge=0.7)
    for k in range(5):
        box(-0.16 + k * 0.08, -0.635, g, 0.06, 0.012, 0.1, M('dark'))
    for sx in (-1, 1):
        box(sx * 0.3, -0.5, g, 0.1, 0.1, 0.18, red)
        hip(sx * 0.3, -0.5, g + 0.18, 0.15, 0.15, 0.08, gold, ridge=0.4)
    # the great terrace and Hall of Supreme Harmony
    for i in range(3):
        box(0.0, 0.16, g + i * 0.04, 0.84 - i * 0.05, 0.54 - i * 0.04, 0.04, M('marble'))
    z = g + 0.12
    box(0.0, 0.16, z, 0.58, 0.32, 0.16, red)
    for k in range(9):
        col(-0.26 + k * 0.065, -0.02, z, 0.011, 0.16, M('red', rgb='#9a241d'), n=8)
    box(0.0, 0.16, z + 0.16, 0.66, 0.38, 0.018, M('marble'))
    hip(0.0, 0.16, z + 0.178, 0.7, 0.42, 0.1, gold, ridge=0.7)
    box(0.0, 0.16, z + 0.278, 0.5, 0.28, 0.02, red)
    hip(0.0, 0.16, z + 0.298, 0.52, 0.3, 0.11, gold, ridge=0.65)
    stair_ramp(0.0, -0.18, g, 0.2, 0.12, 0.12, 4, 'marble')
    # courtyard bridges
    box(0.0, -0.3, g - 0.002, 0.7, 0.05, 0.012, M('water', rough=0.04))
    box(0.0, -0.3, g, 0.14, 0.07, 0.025, M('marble'))
    for (x, y) in [(-0.55, 0.0), (0.55, 0.0)]:
        box(x, y, g, 0.16, 0.24, 0.08, red)
        hip(x, y, g + 0.08, 0.2, 0.28, 0.07, gold, ridge=0.6)


def potala_palace():
    g = bl.plot('dirt')
    rock_hill(0, 0.15, g, 0.82, 0.5, seed=7, mat='earth', top='earth', plateau=0.62)
    white = M('white', rgb='#f4efe4', style='plaster')
    red = M('red', rgb='#8a2a22', style='plaster')
    gold = M('gold', rgb='#e8b83a', rough=0.35, metal=0.5)
    # the White Palace in tiers
    box(0, -0.0, g + 0.18, 1.0, 0.34, 0.28, white, taper=0.92)
    windows(0, 0.0, g + 0.18, 1.0, 0.34, 0.28, 10, 3, ww=0.025, wh=0.03, z0=0.25, z1=0.85, sides='f')
    box(0, 0.12, g + 0.46, 0.7, 0.28, 0.22, white)
    windows(0, 0.12, g + 0.46, 0.7, 0.28, 0.22, 7, 2, ww=0.025, wh=0.03, z0=0.3, z1=0.8, sides='f')
    # the Red Palace at the centre
    box(0.0, 0.14, g + 0.68, 0.36, 0.26, 0.3, red, taper=0.94)
    box(0.0, 0.14 - 0.14, g + 0.76, 0.28, 0.01, 0.1, M('dark'))
    box(0.0, 0.14, g + 0.98, 0.4, 0.3, 0.025, M('stone'))
    for x in (-0.12, 0.0, 0.12):
        hip(x, 0.14, g + 1.0, 0.13, 0.22, 0.08, gold, ridge=0.5)
    for sx in (-1, 1):
        box(sx * 0.42, 0.1, g + 0.46, 0.2, 0.22, 0.26, white)
        hip(sx * 0.42, 0.1, g + 0.72, 0.22, 0.24, 0.07, gold, ridge=0.5)
        box(sx * 0.62, -0.02, g + 0.18, 0.16, 0.2, 0.32, red)
        hip(sx * 0.62, -0.02, g + 0.5, 0.19, 0.23, 0.07, gold, ridge=0.5)
    # a zigzag stair up the front
    for k in range(5):
        with at(0.0 if k % 2 == 0 else 0.0, -0.64 + k * 0.08, g + 0.03 * k, 0):
            box(-0.28 + (k % 2) * 0.28, 0.0, 0.0, 0.34, 0.05, 0.03, M('stone2'))
    for k in range(5):
        tree(-0.64 + k * 0.32, -0.62, 0.7, 'pine', g)
    for k in range(6):
        pennant(-0.4 + k * 0.16, -0.46, g + 0.04, 0.12, 0.05, mat=M(['blue', 'white', 'red', 'green', 'yellow', 'blue'][k]))


def torre_de_belem():
    g = bl.plot('sand')
    lime = M('white', rgb='#ece3cc', style='blocks')
    box(0.0, -0.55, g - 0.002, 2.2, 0.4, 0.012, G('water', 1.0, 0.7, 0.92))
    # bastion terrace with cannon embrasures and shield battlements
    box(0.0, 0.0, g, 0.86, 0.5, 0.12, lime)
    for k in range(9):
        box(-0.4 + k * 0.1, -0.265, g + 0.12, 0.07, 0.035, 0.05, lime)
        sphere(-0.4 + k * 0.1, -0.285, g + 0.15, 0.018, M('red', rgb='#b8302c'), n=6, rings=4)
    for k in range(5):
        box(-0.3 + k * 0.15, -0.255, g + 0.04, 0.04, 0.012, 0.04, M('dark'))
    # the tower
    tx, ty = 0.1, 0.12
    box(tx, ty, g + 0.12, 0.34, 0.34, 0.5, lime)
    windows(tx, ty, g + 0.12, 0.34, 0.34, 0.5, 2, 3, ww=0.04, wh=0.07, z0=0.2, z1=0.8, sides='f', frame=M('stone2'))
    box(tx, ty, g + 0.62, 0.4, 0.4, 0.03, lime)
    for sx in (-1, 1):
        for sy in (-1, 1):
            x, y = tx + sx * 0.19, ty + sy * 0.19
            cyl(x, y, g + 0.62, 0.045, 0.1, lime, n=10)
            dome(x, y, g + 0.72, 0.05, M('stone2'), h=0.08, n=10)
            sphere(x, y, g + 0.8, 0.012, M('gold'), n=5, rings=3)
    box(tx, ty, g + 0.65, 0.22, 0.22, 0.14, lime)
    dome(tx, ty, g + 0.79, 0.11, M('stone2'), h=0.1, n=12)
    pennant(tx, ty, g + 0.88, 0.15, 0.07, mat=M('red'))
    # arcaded loggia
    for k in range(4):
        col(tx - 0.12 + k * 0.08, ty - 0.19, g + 0.3, 0.01, 0.12, M('marble'), n=8)
    for (x, y) in [(-0.62, 0.3), (0.7, 0.3)]:
        tree(x, y, 0.9, 'palm', g)


def oxford_university():
    g = bl.plot('grass')
    stone = M('sand', rgb='#d8c490', style='blocks')
    lawn(0, 0.0, g, 0.7, 0.55)
    for (x, y, w, d, ang) in [(0, 0.5, 0.96, 0.2, 0), (0, -0.46, 0.96, 0.2, 0), (-0.56, 0.0, 0.2, 0.8, 0), (0.56, 0.0, 0.2, 0.8, 0)]:
        box(x, y, g, w, d, 0.26, stone)
        box(x, y, g + 0.26, w * 1.01, d * 1.01, 0.02, M('stone2'))
        gable(x, y, g + 0.28, max(w, d) * 0.98, min(w, d) * 0.98, min(w, d) * 0.4, M('slate'), rot=0 if w > d else 90)
    windows(0, -0.46, g, 0.96, 0.2, 0.26, 8, 2, ww=0.03, wh=0.05, z0=0.3, z1=0.75, sides='f', frame=M('stone2'))
    windows(0, 0.5 - 0.1, g, 0.96, 0.2, 0.26, 8, 2, ww=0.03, wh=0.05, z0=0.3, z1=0.75, sides='f', frame=M('stone2'))
    # towers at the corners with pinnacles
    for (x, y) in [(-0.56, 0.5), (0.56, 0.5), (-0.56, -0.46), (0.56, -0.46)]:
        box(x, y, g, 0.14, 0.14, 0.42, stone)
        for sx in (-1, 1):
            for sy in (-1, 1):
                cone(x + sx * 0.06, y + sy * 0.06, g + 0.42, 0.016, 0.09, M('stone2'), n=5)
        pyr(x, y, g + 0.42, 0.1, 0.1, 0.12, M('slate'))
    # tom tower gatehouse
    box(0, -0.46, g + 0.26, 0.16, 0.18, 0.26, stone)
    pyr(0, -0.46, g + 0.52, 0.18, 0.2, 0.2, M('slate'))
    # Radcliffe Camera in the quad
    cyl(0.0, 0.0, g, 0.17, 0.12, M('cream', rgb='#e0cfa0', style='blocks'), n=16)
    for k in range(16):
        a = 2 * math.pi * k / 16
        col(0.18 * math.cos(a), 0.18 * math.sin(a), g, 0.01, 0.12, M('marble'), n=6, base=False, cap=False)
    cyl(0.0, 0.0, g + 0.12, 0.19, 0.012, M('stone2'), n=16)
    dome(0.0, 0.0, g + 0.132, 0.16, M('slateL', rgb='#9aa4b2', rough=0.4), h=0.12, n=16)
    cyl(0.0, 0.0, g + 0.252, 0.035, 0.05, M('cream'), n=8)
    cone(0.0, 0.0, g + 0.302, 0.04, 0.05, M('slateL'), n=8)
    crowd(g, [(-0.28, -0.2), (0.3, 0.2), (0.2, -0.28)], ('blue', 'white', 'purple'))


def big_ben():
    g = bl.plot('paveL')
    stone = M('sand', rgb='#d8c490', style='blocks')
    box(0.0, -0.5, g - 0.002, 2.2, 0.3, 0.012, G('water', 1.0, 0.7, 0.92))
    # Palace of Westminster along the river
    box(0.05, -0.1, g, 1.0, 0.26, 0.2, stone)
    windows(0.05, -0.1, g, 1.0, 0.26, 0.2, 10, 2, ww=0.028, wh=0.05, z0=0.3, z1=0.75, sides='f', frame=M('stone2'))
    gable(0.05, -0.1, g + 0.2, 1.0, 0.26, 0.1, M('slate'))
    for k in range(7):
        pyr(-0.35 + k * 0.15, -0.1, g + 0.2, 0.04, 0.04, 0.13, M('stone2'))
    box(0.1, 0.12, g, 0.8, 0.2, 0.17, stone)
    gable(0.1, 0.12, g + 0.17, 0.8, 0.2, 0.09, M('slate'))
    box(0.1, -0.1, g, 0.08, 0.34, 0.24, stone)
    tower(0.5, 0.0, g, 0.07, 0.3, 'sand', roof='slate', rh=0.17)
    # Elizabeth Tower
    x, y = -0.5, 0.0
    box(x, y, g, 0.15, 0.15, 0.62, stone, taper=0.94)
    for sx in (-1, 1):
        for sy in (-1, 1):
            box(x + sx * 0.07, y + sy * 0.07, g, 0.025, 0.025, 0.64, M('stone2'))
    box(x, y, g + 0.62, 0.19, 0.19, 0.26, stone)
    for (dx, dy, rot) in [(0, -0.1, 0), (0.1, 0.0, 90)]:
        with at(x + dx, y + dy, g + 0.76, rot):
            _face(M('cream', rgb='#f4f1e0'), [(-0.065, -0.001, -0.065), (0.065, -0.001, -0.065), (0.065, -0.001, 0.065), (-0.065, -0.001, 0.065)])
            _face(M('dark'), [(-0.004, -0.0022, -0.05), (0.004, -0.0022, -0.05), (0.004, -0.0022, 0.0), (-0.004, -0.0022, 0.0)])
            _face(M('dark'), [(0.0, -0.0022, -0.004), (0.045, -0.0022, -0.004), (0.045, -0.0022, 0.004), (0.0, -0.0022, 0.004)])
    box(x, y, g + 0.88, 0.21, 0.21, 0.025, M('stone2'))
    box(x, y, g + 0.905, 0.15, 0.15, 0.08, stone)
    pyr(x, y, g + 0.985, 0.17, 0.17, 0.3, M('slate', rgb='#6a7a8c'))
    cyl(x, y, g + 1.28, 0.005, 0.1, M('gold'), n=4)
    for sx in (-1, 1):
        for sy in (-1, 1):
            cone(x + sx * 0.085, y + sy * 0.085, g + 0.88, 0.014, 0.1, M('stone2'), n=5)
    bridge_y = -0.5
    box(0.0, bridge_y, g, 1.5, 0.1, 0.03, M('stone2'))
    for k in range(5):
        box(-0.6 + k * 0.3, bridge_y - 0.052, g + 0.03, 0.12, 0.01, 0.04, M('green'))
    for k in range(2):
        pass
    tree(0.6, 0.35, 0.9, 'round', g)


def hermitage():
    g = bl.plot('paveL')
    mint = M('green', rgb='#8fd0b4', style='plaster')
    white = M('white', rgb='#f7f4ec')
    box(0, 0.1, g, 1.08, 0.46, 0.34, mint)
    windows(0, 0.1, g, 1.08, 0.46, 0.34, 11, 2, ww=0.03, wh=0.06, z0=0.28, z1=0.75, sides='f', frame=white)
    for i in range(12):
        box(-0.5 + i * 0.0909, -0.125, g, 0.032, 0.026, 0.34, white)
    box(0, 0.1, g + 0.34, 1.12, 0.5, 0.025, white)
    box(0, 0.1, g + 0.365, 1.04, 0.42, 0.05, mint)
    # rooftop statues
    for i in range(14):
        box(-0.5 + i * 0.077, -0.1, g + 0.415, 0.014, 0.014, 0.06, M('gold'))
        sphere(-0.5 + i * 0.077, -0.1, g + 0.475, 0.011, M('gold'), n=5, rings=3)
    # central pediment and domes
    box(0, -0.13, g + 0.34, 0.4, 0.06, 0.09, white)
    gable(0, -0.13, g + 0.43, 0.4, 0.07, 0.08, white, mat_end=white)
    for sx in (-1, 1):
        cyl(sx * 0.34, 0.1, g + 0.415, 0.09, 0.1, white, n=14)
        dome(sx * 0.34, 0.1, g + 0.515, 0.09, M('gold'), h=0.12, n=14)
    cyl(0.0, 0.1, g + 0.415, 0.1, 0.09, white, n=16)
    dome(0.0, 0.1, g + 0.505, 0.1, M('green', rgb='#4aa58a'), h=0.15, n=16)
    # Alexander column and plaza
    cyl(0.0, -0.46, g, 0.035, 0.05, M('stone2'), n=8)
    cyl(0.0, -0.46, g + 0.05, 0.022, 0.5, M('rose', rgb='#c98a8a', rough=0.4), n=10, r2=0.018)
    figure(0.0, -0.46, g + 0.55, 0.12, 'gold')
    lawn(-0.5, -0.5, g, 0.3, 0.2)
    lawn(0.5, -0.5, g, 0.3, 0.2)
    crowd(g, [(-0.2, -0.6), (0.18, -0.58), (0.3, -0.45)], ('red', 'blue', 'white'))


def bolshoi_theatre():
    g = bl.plot('paveL')
    cream = M('cream', rgb='#efe4c8', style='plaster')
    marble = M('marble')
    box(0, 0.15, g, 0.9, 0.55, 0.36, cream)
    windows(0, 0.15, g, 0.9, 0.55, 0.36, 8, 2, ww=0.03, wh=0.07, z0=0.3, z1=0.72, sides='f')
    z = portico(0.0, -0.18, g, 0.8, 0.16, 0.34, 8, roof='stone', steps_n=3)
    # pediment sculpture: the quadriga of Apollo
    for k in range(4):
        box(-0.045 + k * 0.03, -0.18, z + 0.08, 0.02, 0.04, 0.05, M('gold'))
        sphere(-0.045 + k * 0.03, -0.17, z + 0.14, 0.014, M('gold'), n=6, rings=4)
    box(0.0, -0.18, z + 0.08, 0.05, 0.04, 0.03, M('gold'))
    figure(0.0, -0.18, z + 0.1, 0.09, 'gold')
    box(0, 0.15, g + 0.36, 0.94, 0.58, 0.025, marble)
    gable(0, 0.25, g + 0.385, 0.5, 0.78, 0.16, M('green', rgb='#7aa890'), rot=90, overhang=0.02)
    fountain(0.0, -0.5, g, 0.9)
    for (x, y) in [(-0.62, -0.42), (0.62, -0.42)]:
        tree(x, y, 0.9, 'round', g)
    crowd(g, [(-0.2, -0.58), (0.16, -0.62), (0.3, -0.5)], ('red', 'cream', 'purple'))


WONDERS = {
    'hagia_sophia': hagia_sophia, 'alhambra': alhambra, 'mont_st_michel': mont_st_michel,
    'venetian_arsenal': venetian_arsenal, 'great_zimbabwe': great_zimbabwe, 'forbidden_city': forbidden_city,
    'potala_palace': potala_palace, 'torre_de_belem': torre_de_belem, 'oxford_university': oxford_university,
    'big_ben': big_ben, 'hermitage': hermitage, 'bolshoi_theatre': bolshoi_theatre,
}
SPRITES = {f'wonder_{k}': (v, False) for k, v in WONDERS.items()}
