"""Ships and submarines (bow towards +X)."""
import math
from mathutils import Vector
from lib import *
from vehicles import _flag


def wake(L, B, y=0.0):
    sph((0, y, 0.015), 1.0, "#eaf6fa", scale=(L * 0.54, B * 0.8, 0.012), seg=16, rings=4, name="foam")


def std_hull(L, B, H, col, bow=1.0, stern=0.6, sheer=0.25, zb=0.12, wm=1.0, name="hull", n=11, deckcol=None):
    """Hull of length L, beam B, gunwale height H above the waterline plane z=0."""
    secs = []
    for i in range(n):
        t = -1 + 2 * i / (n - 1)
        x = t * L / 2
        if t >= 0:
            f = max(1 - t ** 2.4, 0.0) ** 0.55 * bow + (1 - bow) * 0.0
        else:
            f = stern + (1 - stern) * (1 - (-t) ** 2.0)
        f = max(f, 0.03)
        wt = B / 2 * f
        wm_ = wt * 0.93 * wm
        wb = wt * 0.55
        zt = H * (1 + sheer * abs(t) ** 2.5 * (1.0 if t > 0 else 0.5))
        zbb = zb + 0.08 * abs(t) ** 2
        secs.append((x, wb, wm_, wt, zbb, (zbb + zt) * 0.45, zt))
    hull(secs, col, smooth=False, name=name)
    return secs


def deck(L, B, z, col="deck", f=0.9):
    box((0, 0, z), (L * 0.78, B * f * 0.9, 0.05), col, bevel=0.01, name="deck")


def mast(x, h, r=0.05, col="dkwood", z0=0.3):
    limb((x, 0, z0), (x, 0, z0 + h), r, col, r1=r * 0.7, seg=6)


def square_sail(x, z, w, h, col="team", bulge=0.2, yard=True):
    cloth((x, -w / 2, z), (x, w / 2, z), (x, -w / 2, z - h), col, bulge=bulge, axis=(1, 0, 0), nu=6, nv=5, solid=0.02)
    if yard:
        limb((x, -w / 2 - 0.05, z), (x, w / 2 + 0.05, z), 0.03, "dkwood", seg=5)


def lateen(x, z, L, h, col="team", tilt=0.0):
    # triangular sail along a diagonal yard
    p_top = (x + L * 0.5, 0, z + h)
    p_bot = (x - L * 0.5, 0, z - 0.1)
    limb(p_bot, p_top, 0.035, "dkwood", seg=5)
    cloth((x - L * 0.5, 0.0, z - 0.1), (x + L * 0.5, 0.0, z + h), (x - L * 0.5, 0.0, z - 0.1 - h * 0.1), col, bulge=0.0, nu=5, nv=2, solid=0.02)
    # fill the triangle below the yard
    poly([(x - L * 0.5, z - 0.1), (x + L * 0.5, z + h), (x + L * 0.45, z - 0.15)], col, plane="XZ", offset=0.12, thick=0.03, name="lateen")


def oars(L, B, z, n, rows=1, length=0.9, col="ltwood"):
    for r in range(rows):
        for i in range(n):
            x = -L * 0.35 + (L * 0.7) * i / max(n - 1, 1)
            for sy in (-1, 1):
                p0 = (x, sy * B * 0.5, z + 0.18 * r)
                p1 = (x - 0.12, sy * (B * 0.5 + length * 0.8), z - 0.3 - 0.05 * r)
                limb(p0, p1, 0.02, col, r1=0.03, seg=4)
                box(p1, (0.16, 0.1, 0.03), col, bevel=0.005)


def shields(L, B, z, n, cols=("team", "#e8d8a0"), r=0.12):
    for i in range(n):
        x = -L * 0.38 + (L * 0.7) * i / max(n - 1, 1)
        c = cols[i % len(cols)]
        for sy in (-1, 1):
            cyl((x, sy * (B * 0.47), z), r, 0.03, c, rot=(90, 0, 0), seg=10)


def flagpole(x, z, h=0.7, L=0.4):
    _flag((x, 0, z), h=h, L=L)


def galley(o=None):
    L, B, H = 3.6, 1.0, 0.5
    wake(L, B)
    std_hull(L, B, H, "#8a5a32", stern=0.5, sheer=0.35)
    deck(L, B, H - 0.02)
    box((0.0, 0, H + 0.04), (L * 0.75, 0.04, 0.03), "ltwood")
    oars(L, B, H - 0.1, 8, length=1.0)
    shields(L, B, H + 0.1, 7)
    mast(0.2, 2.2, z0=H)
    square_sail(0.2, 2.5, 1.6, 1.4, bulge=0.25)
    # ram
    limb((L / 2 - 0.15, 0, 0.18), (L / 2 + 0.35, 0, 0.12), 0.1, metal("bronze"), r1=0.04, seg=7)
    box((-L / 2 + 0.1, 0, H + 0.25), (0.2, 0.3, 0.35), "dkwood")
    flagpole(-L / 2 + 0.1, H + 0.4, 0.5, 0.35)


def longship(o=None):
    L, B, H = 3.8, 0.95, 0.42
    wake(L, B)
    std_hull(L, B, H, "#7a4e2c", stern=0.35, sheer=0.45)
    # clinker strakes
    for sy in (-1, 1):
        for k, z in enumerate((0.22, 0.34)):
            box((0, sy * B * 0.44, z), (L * 0.75, 0.025, 0.04), "dkwood")
    deck(L, B, H - 0.02)
    # prows
    limb((L / 2 - 0.1, 0, H + 0.2), (L / 2 + 0.1, 0, H + 0.75), 0.07, "dkwood", seg=6)
    limb((L / 2 + 0.1, 0, H + 0.75), (L / 2 + 0.38, 0, H + 0.92), 0.06, "dkwood", seg=6)
    sph((L / 2 + 0.42, 0, H + 0.95), 0.09, "dkred", scale=(1.3, 0.7, 0.9))
    limb((-L / 2 + 0.1, 0, H + 0.2), (-L / 2 - 0.1, 0, H + 0.7), 0.06, "dkwood", seg=6)
    shields(L, B, H + 0.12, 9, cols=("team", "#e8d8a0", "team", "#2a2a2a"))
    oars(L, B, H - 0.05, 8, length=1.0)
    mast(0.0, 2.3, z0=H)
    cloth((0.0, -0.8, 2.6), (0.0, 0.8, 2.6), (0.0, -0.8, 1.35), "team", bulge=0.25, axis=(1, 0, 0), nu=6, nv=5)
    for k in range(2):
        box((0.04, -0.4 + 0.8 * k, 1.95), (0.02, 0.22, 1.15), "white", bevel=0.005)
    limb((0.0, -0.9, 2.6), (0.0, 0.9, 2.6), 0.03, "dkwood", seg=5)


def quadrireme(o=None, dromon=False):
    L, B, H = 4.0, 1.2, 0.6
    wake(L, B)
    std_hull(L, B, H, "#7a5030" if not dromon else "#8a4030", stern=0.5, sheer=0.3)
    deck(L, B, H - 0.02)
    oars(L, B, H - 0.15, 9, rows=2, length=1.1)
    box((0.2, 0, H + 0.02), (L * 0.6, B * 0.7, 0.2), "ltwood", bevel=0.03, name="cabin") if not dromon else None
    if dromon:
        box((-1.1, 0, H + 0.2), (0.8, 0.8, 0.4), "ltwood", bevel=0.03)
        limb((L / 2 - 0.2, 0, H + 0.1), (L / 2 + 0.45, 0, H + 0.25), 0.09, metal("bronze"), r1=0.12, seg=8)  # siphon
        sph((L / 2 + 0.46, 0, H + 0.26), 0.07, "#ffb030", seg=6, rings=4)
        mast(0.4, 2.4, z0=H)
        lateen(0.4, 1.4, 2.6, 1.7, "team")
        mast(-0.9, 1.4, r=0.04, z0=H)
        lateen(-0.9, 1.1, 1.3, 0.9, "white")
    else:
        for sx in (-0.9, 0.9):
            box((sx, 0, H + 0.3), (0.5, 0.6, 0.45), "ltwood", bevel=0.03)
            box((sx, 0, H + 0.58), (0.56, 0.66, 0.06), "team")
        mast(0.0, 2.2, z0=H)
        square_sail(0.0, 2.55, 1.5, 1.25, bulge=0.25)
    limb((L / 2 - 0.15, 0, 0.2), (L / 2 + 0.4, 0, 0.14), 0.1, metal("bronze"), r1=0.04, seg=7)
    shields(L, B, H + 0.1, 7, cols=("team", "#e8d8a0"))
    flagpole(-L / 2 + 0.05, H + 0.2, 0.6, 0.35)


def dromon(o=None):
    quadrireme(o, dromon=True)


def _sailing_hull(L, B, H, col, stern_h, bow=1.0, name="hull"):
    wake(L, B)
    std_hull(L, B, H, col, stern=0.7, sheer=0.3, bow=bow)
    # stern castle
    box((-L * 0.33, 0, H + stern_h / 2 - 0.02), (L * 0.28, B * 0.82, stern_h), col, bevel=0.03, name="sterncastle")
    box((-L * 0.33, 0, H + stern_h), (L * 0.3, B * 0.86, 0.05), "deck")
    for sy in (-1, 1):
        box((0.0, sy * B * 0.42, H + 0.1), (L * 0.55, 0.035, 0.22), "dkwood")


def caravel(o=None):
    L, B, H = 3.4, 1.05, 0.55
    _sailing_hull(L, B, H, "#7a5030", 0.4)
    for (x, h, w) in ((0.8, 2.4, 1.3), (-0.2, 2.8, 1.5)):
        mast(x, h, z0=H)
    lateen(0.8, 1.7, 2.2, 1.3, "team")
    lateen(-0.2, 1.9, 2.4, 1.5, "team")
    mast(-1.15, 1.5, r=0.04, z0=H + 0.4)
    lateen(-1.15, 1.4, 1.1, 0.7, "white")
    limb((L / 2 - 0.1, 0, H + 0.1), (L / 2 + 0.55, 0, H + 0.55), 0.04, "dkwood", seg=5)
    flagpole(-1.15, H + 1.8, 0.0001, 0.0001)
    for sy in (-1, 1):
        for x in (0.6, 1.0, -0.4):
            box((x, sy * (B * 0.47), H - 0.1), (0.1, 0.03, 0.1), "black")


def nau(o=None):
    L, B, H = 3.8, 1.5, 0.7
    _sailing_hull(L, B, H, "#6a4428", 0.55)
    for x, h in ((0.9, 3.0), (-0.1, 3.4), (-1.2, 2.0)):
        mast(x, h, r=0.06, z0=H)
    square_sail(0.9, 3.4, 1.7, 1.0)
    square_sail(0.9, 2.3, 2.0, 1.2)
    square_sail(-0.1, 3.8, 1.8, 1.1)
    square_sail(-0.1, 2.6, 2.3, 1.5)
    lateen(-1.2, 2.2, 1.5, 0.9, "white")
    for k in range(2):
        box((0.9 + 0.01, 0, 2.8 - 1.1 * k), (0.02, 0.2, 0.9), "white", bevel=0.005)
    limb((L / 2 - 0.1, 0, H + 0.1), (L / 2 + 0.7, 0, H + 0.5), 0.05, "dkwood", seg=5)
    box((-L * 0.33, B * 0.0, H + 0.3), (0.05, B * 0.7, 0.0001), "team")
    for sy in (-1, 1):
        box((0.1, sy * B * 0.5, H - 0.25), (L * 0.55, 0.03, 0.14), "team")
        for x in (-0.3, 0.2, 0.7):
            box((x, sy * B * 0.5, H - 0.1), (0.14, 0.03, 0.12), "black")
    flagpole(-L * 0.38, H + 0.55, 0.7, 0.45)


def frigate(o=None, col="#5a3a24", sails="team", trim="#d8c070", priv=False):
    L, B, H = 4.2, 1.2, 0.65
    _sailing_hull(L, B, H, col, 0.35)
    for sy in (-1, 1):
        box((0.0, sy * B * 0.46, H - 0.2), (L * 0.6, 0.03, 0.05), trim)
        for x in (-1.0, -0.6, -0.2, 0.2, 0.6, 1.0):
            box((x, sy * B * 0.49, H - 0.13), (0.14, 0.03, 0.11), "black")
    masts = ((1.1, 3.0), (0.0, 3.5), (-1.1, 2.6))
    for x, h in masts:
        mast(x, h, z0=H)
    for x, h in masts:
        square_sail(x, h + H - 0.1, 1.4, h * 0.3, col=sails)
        square_sail(x, h + H - 0.1 - h * 0.34, 1.7, h * 0.34, col=sails)
        if x < 1.0:
            square_sail(x, h + H - 0.1 - h * 0.72, 1.9, h * 0.3, col=sails)
    limb((L / 2 - 0.1, 0, H + 0.1), (L / 2 + 0.6, 0, H + 0.5), 0.04, "dkwood", seg=5)
    cloth((L / 2 + 0.3, 0, H + 0.3), (L / 2 + 0.3, 0, H + 0.3), (L / 2 - 0.2, 0, H + 0.1), sails, bulge=0.0, nu=2, nv=2) if False else None
    flagpole(-L * 0.42, H + 0.35, 0.9, 0.5)


def privateer(o=None):
    frigate(o, col="#2e2a30", sails="#3a3a42", trim="#8a2a20")


def sea_dog(o=None):
    frigate(o, col="#7a2a20", sails="#d8c8a0", trim="#d8b040")


def jong(o=None):
    L, B, H = 4.2, 1.6, 0.75
    wake(L, B)
    std_hull(L, B, H, "#7a3a28", stern=0.9, sheer=0.35, bow=1.0)
    box((-0.4, 0, H + 0.25), (1.6, 1.2, 0.5), "#8a4a30", bevel=0.04, name="cabin")
    box((-0.4, 0, H + 0.55), (1.8, 1.4, 0.06), "team")
    for x, h in ((1.15, 3.2), (0.1, 3.8), (-1.15, 2.8)):
        mast(x, h, r=0.06, z0=H)
        # battened tanja sail
        cloth((x, -0.2, h + H - 0.2), (x, 0.9, h + H - 0.2 - 0.5), (x, -0.2, H + 0.6), "team", bulge=0.12, axis=(1, 0, 0), nu=5, nv=6)
        for k in range(5):
            z = H + 0.7 + (h - 0.8) * k / 4
            limb((x + 0.02, -0.25, z), (x + 0.02, 0.95, z - 0.1 * (k > 2) - 0.0), 0.02, "dkwood", seg=4)
    limb((L / 2 - 0.2, 0, H + 0.1), (L / 2 + 0.5, 0, H + 0.35), 0.05, "dkwood", seg=5)
    flagpole(-L * 0.4, H + 0.6, 0.7, 0.45)


def _flat_deck(L, B, z, col="#7a7f84"):
    box((0, 0, z), (L, B, 0.04), col, bevel=0.01)


def _turret(x, z, scale=1.0, barrels=2, col="#7a8086", fwd=1, elev=8):
    with xf((x, 0, z), (0, 0, 0 if fwd > 0 else 180), scale):
        cyl((0, 0, 0.0), 0.34, 0.1, "#5a6066", seg=10)
        box((0.0, 0, 0.18), (0.8, 0.62, 0.3), col, bevel=0.05, name="turret")
        box((0.0, 0, 0.36), (0.6, 0.5, 0.06), "team", bevel=0.01) if scale > 0.9 and False else None
        for k in range(barrels):
            y = (k - (barrels - 1) / 2) * 0.22
            with xf((0.3, y, 0.2), (0, -elev, 0)):
                limb((0, 0, 0), (0.95, 0, 0), 0.05, "#4a5056", seg=7)


def ironclad(o=None):
    L, B, H = 4.0, 1.3, 0.34
    wake(L, B)
    std_hull(L, B, H, "#4a4e54", stern=0.6, sheer=0.1)
    _flat_deck(L * 0.9, B * 0.9, H, "#6a5a40")
    # sloped casemate
    poly([(-0.9, 0.0), (0.9, 0.0), (0.65, 0.5), (-0.65, 0.5)], "#555a60", plane="XZ", thick=1.0, offset=0.0, bevel=0.03, name="casemate")
    with xf((0, 0, H)):
        poly([(-0.9, 0.0), (0.9, 0.0), (0.65, 0.5), (-0.65, 0.5)], "#555a60", plane="XZ", thick=0.9, bevel=0.03, name="casemate")
        for k in range(3):
            box((-0.4 + 0.4 * k, 0.45, 0.22), (0.18, 0.03, 0.14), "black")
            box((-0.4 + 0.4 * k, -0.45, 0.22), (0.18, 0.03, 0.14), "black")
        cyl((0.0, 0, 0.55), 0.17, 0.12, "dkgrey", seg=8)
        box((0, 0, 0.5), (1.3, 0.9, 0.04), "team", bevel=0.01, name="roofstripe")
    cyl((-0.7, 0, H + 0.9), 0.14, 0.9, "black", r2=0.12, seg=8)
    cyl((-0.7, 0, H + 1.38), 0.15, 0.08, "dkgrey", seg=8)
    limb((L / 2 - 0.1, 0, 0.15), (L / 2 + 0.3, 0, 0.1), 0.08, metal("dkiron"), r1=0.03, seg=6)
    for sy in (-1, 1):
        box((0.0, sy * 0.58, 0.28), (3.0, 0.05, 0.06), "team")
    flagpole(L * 0.4, H, 0.9, 0.4)


def battleship(o=None, funnels=1, mg=False, col="#8f979e"):
    L, B, H = 6.4, 1.3, 0.45
    wake(L, B * 1.2)
    std_hull(L, B, H, col, stern=0.55, sheer=0.12, n=13)
    _flat_deck(L * 0.9, B * 0.9, H, "#8a7a5a")
    for sy in (-1, 1):
        box((L * 0.22, sy * B * 0.45, H - 0.14), (L * 0.2, 0.03, 0.12), "team")
    # turrets: two forward (superfiring), two aft
    _turret(2.2, H, 1.0, 3 if mg else 2)
    _turret(1.55, H + 0.27, 1.0, 3 if mg else 2)
    _turret(-1.55, H, 1.0, 2, fwd=-1)
    _turret(-2.2, H + 0.0, 0.95, 2, fwd=-1) if mg else None
    # superstructure
    box((0.3, 0, H + 0.4), (1.2, 0.8, 0.8), "#a0a6ac", bevel=0.05, name="bridge")
    box((0.35, 0, H + 0.95), (0.8, 0.6, 0.35), "#a0a6ac", bevel=0.05)
    box((0.35, 0, H + 1.2), (0.6, 0.5, 0.1), "#6a7076", bevel=0.02)
    limb((0.35, 0, H + 1.2), (0.35, 0, H + 2.0), 0.04, "#5a6066", seg=6)
    box((0.35, 0, H + 1.6), (0.3, 0.5, 0.04), "#5a6066")
    for k in range(funnels):
        cyl((-0.4 - 0.6 * k, 0, H + 0.7), 0.24, 1.1, "#7a8086", r2=0.2, seg=10)
        cyl((-0.4 - 0.6 * k, 0, H + 1.28), 0.22, 0.08, "black", seg=10)
    for k in range(4):
        for sy in (-1, 1):
            cyl((-0.9 + 0.35 * k, sy * 0.35, H + 0.12), 0.07, 0.2, "#5a6066", seg=6)
    flagpole(-L * 0.45, H, 1.0, 0.5)
    limb((L / 2 - 0.1, 0, H + 0.1), (L / 2 - 0.1, 0, H + 0.4), 0.02, "dkgrey", seg=4)


def minas_geraes(o=None):
    battleship(o, funnels=2, mg=True, col="#7e868e")


def destroyer(o=None):
    L, B, H = 4.6, 0.9, 0.4
    wake(L, B)
    std_hull(L, B, H, "#8a9298", stern=0.5, sheer=0.15, n=11)
    _flat_deck(L * 0.9, B * 0.88, H, "#7a7f84")
    _turret(1.5, H, 0.75, 1)
    _turret(-1.3, H, 0.7, 1, fwd=-1)
    box((0.35, 0, H + 0.35), (0.9, 0.55, 0.7), "#a8aeb4", bevel=0.05, name="bridge")
    box((0.35, 0, H + 0.75), (0.5, 0.45, 0.08), "#6a7076")
    limb((0.35, 0, H + 0.75), (0.35, 0, H + 1.6), 0.035, "#5a6066", seg=6)
    cyl((-0.5, 0, H + 0.4), 0.2, 0.7, "#7a8086", r2=0.16, seg=10)
    cyl((-0.5, 0, H + 0.76), 0.17, 0.05, "black", seg=10)
    for k in range(2):
        cyl((-0.1 - 0.15 * k, 0.0, H + 0.25), 0.1, 0.5, "#6a7076", rot=(90, 0, 0), seg=8)
    for sy in (-1, 1):
        box((0.0, sy * 0.4, H - 0.14), (2.0, 0.03, 0.1), "team")
    flagpole(-L * 0.42, H, 0.8, 0.45)


def missile_cruiser(o=None):
    L, B, H = 5.2, 1.0, 0.42
    wake(L, B)
    std_hull(L, B, H, "#8a9298", stern=0.55, sheer=0.12, n=13)
    _flat_deck(L * 0.9, B * 0.88, H, "#7a7f84")
    _turret(1.9, H, 0.6, 1)
    for ix in range(3):
        for iy in range(2):
            box((1.0 - 0.2 * iy - 0.0, -0.2 + 0.2 * ix, H + 0.04), (0.18, 0.18, 0.04), "#4a5056", bevel=0.01)
    box((0.1, 0, H + 0.5), (1.5, 0.75, 0.9), "#a8aeb4", bevel=0.05, name="superstructure")
    box((0.1, 0, H + 1.0), (1.2, 0.6, 0.12), "#6a7076")
    # radar panels in team colour
    for sy in (-1, 1):
        box((0.65, sy * 0.38, H + 0.65), (0.28, 0.03, 0.28), "team", bevel=0.01, name="radar")
        box((-0.45, sy * 0.38, H + 0.65), (0.28, 0.03, 0.28), "team", bevel=0.01, name="radar")
    limb((0.0, 0, H + 1.0), (0.0, 0, H + 1.9), 0.035, "#5a6066", seg=6)
    sph((0.0, 0, H + 1.95), 0.12, "#d8dce0", seg=8, rings=5)
    for k in range(2):
        box((-1.2 + 0.5 * k, 0, H + 0.45), (0.4, 0.45, 0.65), "#9aa0a6", bevel=0.04)
        cyl((-1.2 + 0.5 * k, 0, H + 0.82), 0.14, 0.08, "black", seg=8)
    for ix in range(4):
        for iy in range(2):
            box((-1.9 + 0.2 * ix, -0.12 + 0.24 * iy, H + 0.04), (0.18, 0.2, 0.04), "#4a5056", bevel=0.01)
    flagpole(-L * 0.45, H, 0.7, 0.4)


def aircraft_carrier(o=None):
    L, B, H = 6.8, 1.6, 0.5
    wake(L, B * 1.1)
    std_hull(L, B * 0.9, H, "#7a8288", stern=0.8, sheer=0.0, n=13, wm=1.1)
    # flight deck overhanging
    poly([(-3.3, -0.85), (2.6, -0.85), (3.4, -0.2), (3.4, 0.2), (2.6, 0.85), (-3.3, 0.85)], "#5e6468", plane="XY", offset=H + 0.06, thick=0.1, bevel=0.02, name="flightdeck")
    box((0.2, 0.0, H + 0.12), (5.8, 0.04, 0.01), "white")
    box((-0.6, 0.45, H + 0.12), (4.0, 0.04, 0.01), "#e8d070")
    box((-0.6, -0.45, H + 0.12), (4.0, 0.04, 0.01), "#e8d070")
    box((2.7, 0.0, H + 0.12), (1.0, 0.5, 0.01), "team", name="deckpaint")
    # island
    box((0.5, 0.75, H + 0.5), (0.9, 0.35, 0.85), "#9aa0a6", bevel=0.04, name="island")
    box((0.5, 0.75, H + 1.0), (0.7, 0.4, 0.16), "#6a7076")
    limb((0.5, 0.75, H + 1.05), (0.5, 0.75, H + 1.8), 0.035, "#5a6066", seg=6)
    box((0.3, 0.75, H + 1.5), (0.3, 0.04, 0.35), "team", name="radar")
    # parked aircraft
    for (x, y) in ((-1.9, -0.3), (-1.4, 0.25), (-0.6, -0.3), (-2.4, 0.3)):
        with xf((x, y, H + 0.16), (0, 0, 8), 0.3):
            tube([(-0.9, 0.04, 0.04, 0.0), (-0.3, 0.14, 0.14, 0.0), (0.5, 0.12, 0.12, 0.0), (1.0, 0.04, 0.04, 0.0)], "#4a5056", n=8)
            poly([(0.2, 0.1), (-0.3, 0.1), (-0.5, 1.0), (-0.1, 1.0)], "#4a5056", plane="XY", thick=0.04)
            poly([(0.2, -0.1), (-0.3, -0.1), (-0.5, -1.0), (-0.1, -1.0)], "#4a5056", plane="XY", thick=0.04)
    flagpole(-L * 0.46, H + 0.1, 0.0001, 0.0001)


def _sub(L, rz, col, sailh, sail_x, deckgun=False, nuke=False, team_sail=True):
    wake(L, rz * 2.6)
    z0 = 0.18
    tube([(-L / 2, 0.03, 0.03, z0), (-L / 2 + 0.5, rz * 0.7, rz * 0.7, z0), (-L * 0.15, rz, rz, z0), (L * 0.2, rz, rz, z0),
          (L / 2 - 0.45, rz * 0.8, rz * 0.8, z0), (L / 2, 0.04, 0.04, z0)], col, n=12, smooth=True, name="pressure_hull")
    # conning tower
    box((sail_x, 0, z0 + rz + sailh * 0.3), (0.7 if nuke else 0.55, 0.22, sailh), col, bevel=0.06, name="sail")
    box((sail_x, 0, z0 + rz + sailh * 0.7), (0.5, 0.24, 0.14), "team" if team_sail else col, bevel=0.02, name="sailband")
    limb((sail_x + 0.1, 0, z0 + rz + sailh), (sail_x + 0.1, 0, z0 + rz + sailh + 0.35), 0.025, "dkgrey", seg=5)
    limb((sail_x - 0.1, 0, z0 + rz + sailh), (sail_x - 0.1, 0, z0 + rz + sailh + 0.28), 0.02, "dkgrey", seg=5)
    # tail fins and screw
    for a in (0, 90):
        r = math.radians(a)
        poly([(-L / 2 + 0.15, 0.0), (-L / 2 - 0.05, 0.35), (-L / 2 + 0.3, 0.35)], col, plane="XY" if a == 0 else "XZ", offset=z0 if a == 0 else 0, thick=0.04)
    if deckgun:
        cyl((L * 0.28, 0, z0 + rz), 0.1, 0.25, "dkgrey", seg=8)
        limb((L * 0.28, 0, z0 + rz + 0.15), (L * 0.28 + 0.5, 0, z0 + rz + 0.2), 0.035, "dkiron", seg=6)
    for sx in (-1.0, 0.6, 1.4):
        pass


def submarine(o=None):
    _sub(4.0, 0.32, "#4a525a", 0.55, 0.3)


def uboat(o=None):
    _sub(4.0, 0.32, "#585e62", 0.6, 0.2, deckgun=True)


def nuclear_sub(o=None):
    _sub(5.0, 0.42, "#25282c", 0.8, 0.6, nuke=True)
