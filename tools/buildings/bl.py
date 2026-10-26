"""Tiny procedural-modelling kit for the WayCiv building sprites.

Everything is built from a handful of primitives that append faces to one
bmesh per material (so a whole scene is a few dozen objects), then rendered
with Cycles from a fixed 3/4 camera.  Runs inside Blender only:

    blender -b --factory-startup -t 4 -P tools/buildings/render.py -- ...

Coordinates: x east, y north (up the screen), z up.  A hex plot is
pointy-top with circumradius PLOT_R and sits at the origin.
"""
import math
import os
import random
import struct
import zlib
from contextlib import contextmanager

import bmesh
import bpy
import numpy as np
from mathutils import Matrix, Vector

import matlib
from matlib import G, M  # noqa: F401

PLOT_R = 0.90          # circumradius of the plot hex (world 1.0 = tile radius)
CAM_ELEV = 42.0        # camera elevation, degrees
VIEW_W = 2.2           # sprite width in world units
SPRITE_PX = (320, 400)  # final sprite size
GROUND_FRAC = 0.60     # ground centre sits this far down the sprite
SS = 2                 # supersampling factor
STRETCH = 1.0 / math.sin(math.radians(CAM_ELEV))


# --------------------------------------------------------------------------
# colours and materials
# --------------------------------------------------------------------------

# --------------------------------------------------------------------------
# geometry accumulation
# --------------------------------------------------------------------------

_bm = {}
_stack = [Matrix.Identity(4)]
_lights = []   # (location, color, power, size) point lights in local scene space
CONTENT_SCALE = 1.0  # builders draw in plot units; districts are drawn a little larger


def reset():
    global _bm, _stack, _lights
    matlib.reset()
    _bm = {}
    _stack = [Matrix.Diagonal((CONTENT_SCALE, CONTENT_SCALE, CONTENT_SCALE, 1.0))]
    _lights = []


@contextmanager
def at(x=0.0, y=0.0, z=0.0, rot=0.0, s=1.0, sz=None):
    """Local frame: translate, rotate about z (degrees), scale."""
    sz = s if sz is None else sz
    m = Matrix.Translation((x, y, z)) @ Matrix.Rotation(math.radians(rot), 4, 'Z') @ \
        Matrix.Diagonal((s, s, sz, 1.0))
    _stack.append(_stack[-1] @ m)
    try:
        yield
    finally:
        _stack.pop()


def _b(mat):
    if mat not in _bm:
        _bm[mat] = bmesh.new()
    return _bm[mat]


def _face(mat, pts, smooth=False):
    bm = _b(mat)
    T = _stack[-1]
    vs = []
    last = None
    for p in pts:
        w = T @ Vector(p)
        if last is not None and (w - last).length < 1e-6:
            continue
        vs.append(w)
        last = w
    if len(vs) > 1 and (vs[0] - vs[-1]).length < 1e-6:
        vs.pop()
    if len(vs) < 3:
        return
    try:
        f = bm.faces.new([bm.verts.new(v) for v in vs])
        f.smooth = smooth
    except ValueError:
        pass


def loft(rings, mat, smooth=False, cap0=True, cap1=True, closed=True):
    """Skin between consecutive point rings (lists of (x,y,z))."""
    n = len(rings[0])
    for a, b in zip(rings, rings[1:]):
        for i in range(n):
            j = (i + 1) % n
            if not closed and j == 0:
                continue
            _face(mat, [a[i], a[j], b[j], b[i]], smooth)
    if cap0:
        _face(mat, list(reversed(rings[0])))
    if cap1:
        _face(mat, list(rings[-1]))


def _circle(r, z, n, x=0.0, y=0.0, a0=0.0, ry=None):
    ry = r if ry is None else ry
    return [(x + r * math.cos(a0 + 2 * math.pi * i / n), y + ry * math.sin(a0 + 2 * math.pi * i / n), z)
            for i in range(n)]


def cyl(x, y, z, r, h, mat, n=20, r2=None, smooth=True, cap1=True):
    r2 = r if r2 is None else r2
    loft([_circle(r, z, n, x, y), _circle(r2, z + h, n, x, y)], mat, smooth=smooth, cap1=cap1)


def cone(x, y, z, r, h, mat, n=20):
    cyl(x, y, z, r, h, mat, n=n, r2=0.0)


def box(x, y, z, sx, sy, h, mat, rot=0.0, taper=1.0, tapery=None):
    """Axis-aligned (before `rot`) block centred at x,y, base at z."""
    ty = taper if tapery is None else tapery
    with at(x, y, z, rot):
        a = [(-sx / 2, -sy / 2, 0), (sx / 2, -sy / 2, 0), (sx / 2, sy / 2, 0), (-sx / 2, sy / 2, 0)]
        b = [(-sx / 2 * taper, -sy / 2 * ty, h), (sx / 2 * taper, -sy / 2 * ty, h),
             (sx / 2 * taper, sy / 2 * ty, h), (-sx / 2 * taper, sy / 2 * ty, h)]
        loft([a, b], mat)


def pyr(x, y, z, sx, sy, h, mat, rot=0.0):
    box(x, y, z, sx, sy, h, mat, rot, taper=0.0)


def hip(x, y, z, sx, sy, h, mat, rot=0.0, ridge=0.5):
    """Hip roof: ridge fraction is the length of the top ridge relative to the base."""
    with at(x, y, z, rot):
        a = [(-sx / 2, -sy / 2, 0), (sx / 2, -sy / 2, 0), (sx / 2, sy / 2, 0), (-sx / 2, sy / 2, 0)]
        rx = sx / 2 * ridge
        b = [(-rx, 0, h), (rx, 0, h), (rx, 0, h), (-rx, 0, h)]
        loft([a, b], mat)


def gable(x, y, z, sx, sy, h, mat, rot=0.0, overhang=0.0, mat_end=None):
    """Gable roof; ridge runs along local x."""
    mat_end = mat_end or mat
    with at(x, y, z, rot):
        hx, hy = sx / 2 + overhang, sy / 2 + overhang * 0.5
        _face(mat, [(-hx, -hy, 0), (hx, -hy, 0), (hx, 0, h), (-hx, 0, h)])
        _face(mat, [(hx, hy, 0), (-hx, hy, 0), (-hx, 0, h), (hx, 0, h)])
        _face(mat_end, [(hx, -hy, 0), (hx, hy, 0), (hx, 0, h)])
        _face(mat_end, [(-hx, hy, 0), (-hx, -hy, 0), (-hx, 0, h)])
        _face(mat, [(-hx, hy, 0), (hx, hy, 0), (hx, -hy, 0), (-hx, -hy, 0)])


def ext(pts, z, h, mat, rot=0.0, x=0.0, y=0.0, smooth=False):
    """Extrude a 2D polygon [(x,y),..] upward."""
    with at(x, y, z, rot):
        a = [(p[0], p[1], 0) for p in pts]
        b = [(p[0], p[1], h) for p in pts]
        loft([a, b], mat, smooth=smooth)


def lathe(x, y, z, prof, mat, n=24, smooth=True, cap0=True, cap1=True):
    """Revolve [(radius, height), ...] about the vertical axis."""
    rings = [_circle(r, z + hh, n, x, y) for r, hh in prof]
    loft(rings, mat, smooth=smooth, cap0=cap0 and prof[0][0] > 0, cap1=cap1 and prof[-1][0] > 0)


def dome(x, y, z, r, mat, h=None, n=24, rings=8, smooth=True):
    h = r if h is None else h
    prof = []
    for i in range(rings + 1):
        a = math.pi / 2 * i / rings
        prof.append((r * math.cos(a), h * math.sin(a)))
    lathe(x, y, z, prof, mat, n=n, smooth=smooth)


def sphere(x, y, z, r, mat, n=16, rings=10):
    prof = []
    for i in range(rings + 1):
        a = -math.pi / 2 + math.pi * i / rings
        prof.append((r * math.cos(a), r + r * math.sin(a)))
    lathe(x, y, z, prof, mat, n=n)


def arch_wall(x, y, z, w, h, t, mat, rot=0.0, iw=None, ih=None, n=10):
    """A wall slab with one round-topped opening (extruded along local y)."""
    iw = w * 0.5 if iw is None else iw
    ih = h * 0.6 if ih is None else ih
    spring = max(ih - iw / 2, 0.0)
    pts = [(-w / 2, 0), (-iw / 2, 0), (-iw / 2, spring)]
    for i in range(1, n):
        a = math.pi - math.pi * i / n
        pts.append((iw / 2 * math.cos(a), spring + iw / 2 * math.sin(a)))
    pts += [(iw / 2, spring), (iw / 2, 0), (w / 2, 0), (w / 2, h), (-w / 2, h)]
    with at(x, y, z, rot):
        # polygon lives in the x-z plane, thickness along y
        a = [(p[0], -t / 2, p[1]) for p in pts]
        b = [(p[0], t / 2, p[1]) for p in pts]
        loft([a, b], mat)


def col(x, y, z, r, h, mat, base=True, cap=True, n=14):
    """Classical column with base and capital."""
    bh = r * 0.5
    if base:
        cyl(x, y, z, r * 1.35, bh, mat, n=n)
    sh = h - (bh if base else 0) - (bh if cap else 0)
    z0 = z + (bh if base else 0)
    lathe(x, y, z0, [(r, 0), (r * 0.9, sh * 0.5), (r * 0.82, sh)], mat, n=n)
    if cap:
        cyl(x, y, z0 + sh, r * 1.3, bh, mat, n=n)


def colonnade(cx, cy, z, rx, ry, count_x, count_y, r, h, mat):
    """Columns around the rim of a rectangle (rx, ry half sizes)."""
    for i in range(count_x):
        px = -rx + 2 * rx * i / max(count_x - 1, 1)
        col(cx + px, cy - ry, z, r, h, mat)
        col(cx + px, cy + ry, z, r, h, mat)
    for j in range(1, count_y - 1):
        py = -ry + 2 * ry * j / max(count_y - 1, 1)
        col(cx - rx, cy + py, z, r, h, mat)
        col(cx + rx, cy + py, z, r, h, mat)


def steps(x, y, z, sx, sy, n, step_h, inset, mat, rot=0.0):
    for i in range(n):
        box(x, y, z + i * step_h, sx - 2 * inset * i, sy - 2 * inset * i, step_h, mat, rot)


def _hex_pts(r):
    return [(r * math.cos(math.radians(90 + 60 * i)), r * math.sin(math.radians(90 + 60 * i))) for i in range(6)]


def blob(x, y, z, r, mat, seed=0, sub=2, squash=0.85, jitter=0.16):
    """A lumpy smooth ball (foliage, bushes, haystacks) resting on z."""
    rnd = random.Random(seed)
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=sub, radius=1.0)
    for v in bm.verts:
        v.co *= 1.0 + (rnd.random() - 0.5) * 2 * jitter
    T = _stack[-1]
    tgt = _b(mat)
    vmap = {}
    for v in bm.verts:
        vmap[v.index] = tgt.verts.new(T @ Vector((x + v.co.x * r, y + v.co.y * r, z + r * squash + v.co.z * r * squash)))
    for f in bm.faces:
        fn = tgt.faces.new([vmap[v.index] for v in f.verts])
        fn.smooth = True
    bm.free()


def tree(x, y, s=1.0, kind='round', z=0.0):
    seed = int(abs(x * 977 + y * 613)) % 1000
    rnd = random.Random(seed)
    if kind == 'round':
        cyl(x, y, z, 0.02 * s, 0.1 * s, M('woodD'), n=6, r2=0.014 * s)
        pick = ['leaf', 'leafD', 'leafL'][seed % 3]
        blob(x, y, z + 0.05 * s, 0.1 * s, M(pick), seed)
        for k in range(3):
            a = rnd.random() * 6.28
            blob(x + 0.07 * s * math.cos(a), y + 0.07 * s * math.sin(a), z + (0.08 + 0.04 * k) * s, 0.065 * s,
                 M(['leafL', 'leaf', 'leafD'][(seed + k) % 3]), seed + k + 1, sub=1)
    elif kind == 'bush':
        blob(x, y, z, 0.07 * s, M(['leaf', 'leafD'][seed % 2]), seed, sub=1)
    elif kind == 'pine':
        cyl(x, y, z, 0.02 * s, 0.06 * s, M('woodD'), n=6)
        for i, (rr, hh, zz) in enumerate([(0.1, 0.11, 0.04), (0.08, 0.1, 0.11), (0.056, 0.1, 0.18)]):
            cone(x, y, z + zz * s, rr * s, hh * s, M('leafD' if i % 2 == 0 else 'leaf'), n=10)
    elif kind == 'palm':
        lathe(x, y, z, [(0.025 * s, 0), (0.015 * s, 0.2 * s)], M('woodL'), n=6)
        for k in range(7):
            with at(x, y, z + 0.2 * s, rot=360 / 7 * k):
                for sgn in (-1, 1):
                    _face(M('leaf'), [(0, sgn * 0.02 * s, 0), (0.17 * s, sgn * 0.012 * s, -0.05 * s), (0.22 * s, 0.0, -0.1 * s),
                                      (0.17 * s, -sgn * 0.012 * s, -0.05 * s), (0, -sgn * 0.02 * s, 0)])


def person(x, y, z, mat, s=1.0):
    lathe(x, y, z, [(0.03 * s, 0), (0.025 * s, 0.07 * s), (0.02 * s, 0.1 * s)], mat, n=6)
    sphere(x, y, z + 0.1 * s, 0.022 * s, M('cream'), n=6, rings=4)


def point_light(x, y, z, color=(1.0, 0.8, 0.5), power=30.0, size=0.05):
    _lights.append((_stack[-1] @ Vector((x, y, z)), color, power, size))


def glow(x, y, z, w, d, h, color, rot=0.0, strength=6.0):
    box(x, y, z, w, d, h, M('glow_' + color, rgb=color, emit=color, emit_strength=strength), rot)


# --------------------------------------------------------------------------
# plots
# --------------------------------------------------------------------------

GROUND_KIND = {
    'earth': ('dirt', 1.0), 'dirt': ('dirt', 1.0), 'grass': ('grass', 1.0), 'grassD': ('grass', 0.85),
    'pave': ('cobble', 1.0), 'paveL': ('cobble', 1.12), 'paveD': ('cobble', 0.85), 'stone': ('cobble', 1.0),
    'stone2': ('cobble', 0.9), 'stoneD': ('cobble', 0.75), 'sand': ('sand', 1.0), 'sandD': ('sand', 0.88),
    'concrete': ('slab', 1.0), 'concreteD': ('slab', 0.85), 'asphalt': ('dark', 1.0), 'marble': ('marble', 1.0),
    'cream': ('marble', 1.0),
}


def plot(base='pave', top=None, r_in=0.66, r_out=0.9, **_ignored):
    """The ground patch every district/wonder sits on: textured, fading to nothing at its rim so it
    melts into the terrain.  Returns the ground height."""
    kind, tone = GROUND_KIND.get(top or base, ('cobble', 1.0))
    scaled = _stack[:]
    _stack[:] = [Matrix.Identity(4)]
    _face(G(kind, tone, r_in, r_out), [(px, py, 0.004) for px, py in _hex_pts(1.05)])
    _stack[:] = scaled
    return 0.008


# --------------------------------------------------------------------------
# scene realisation + rendering
# --------------------------------------------------------------------------

def realise():
    for mat, bm in _bm.items():
        mesh = bpy.data.meshes.new("m")
        bm.normal_update()
        bm.to_mesh(mesh)
        bm.free()
        ob = bpy.data.objects.new("o", mesh)
        mesh.materials.append(mat)
        bpy.context.scene.collection.objects.link(ob)
    _bm.clear()
    if not matlib.MASK_MODE:  # shadows fall on the transparent terrain around the ground patch
        mesh = bpy.data.meshes.new("catcher")
        bm = bmesh.new()
        r = 0.98
        bm.faces.new([bm.verts.new((r * math.cos(math.radians(90 + 60 * i)), r * math.sin(math.radians(90 + 60 * i)), 0.0))
                      for i in range(6)])
        bm.to_mesh(mesh)
        bm.free()
        ob = bpy.data.objects.new("catcher", mesh)
        ob.is_shadow_catcher = True
        bpy.context.scene.collection.objects.link(ob)


def clear_scene():
    for ob in list(bpy.data.objects):
        bpy.data.objects.remove(ob, do_unlink=True)
    for coll in (bpy.data.meshes, bpy.data.materials, bpy.data.lights, bpy.data.cameras):
        for d in list(coll):
            coll.remove(d)


def _setup_render(sc, samples, mask):
    sc.render.engine = 'CYCLES'
    cy = sc.cycles
    cy.device = 'CPU'
    cy.samples = samples
    cy.use_denoising = not mask
    cy.use_adaptive_sampling = not mask
    cy.max_bounces = 4
    cy.diffuse_bounces = 3
    cy.glossy_bounces = 2
    cy.transmission_bounces = 2
    cy.transparent_max_bounces = 4
    cy.filter_width = 1.2 if not mask else 0.8
    sc.render.film_transparent = True
    sc.render.resolution_x = SPRITE_PX[0] * SS
    sc.render.resolution_y = SPRITE_PX[1] * SS
    sc.render.resolution_percentage = 100
    sc.render.image_settings.file_format = 'PNG'
    sc.render.image_settings.color_mode = 'RGBA'
    sc.render.image_settings.color_depth = '8'
    sc.view_settings.view_transform = 'Standard'
    sc.view_settings.look = 'None'
    sc.view_settings.exposure = 0.0
    sc.display_settings.display_device = 'sRGB'
    sc.render.threads_mode = 'FIXED'
    sc.render.threads = 4


def _setup_camera(sc):
    cam = bpy.data.cameras.new("cam")
    cam.type = 'ORTHO'
    cam.ortho_scale = VIEW_W
    cam.sensor_fit = 'HORIZONTAL'
    cam.clip_start = 0.1
    cam.clip_end = 100
    ob = bpy.data.objects.new("cam", cam)
    sc.collection.objects.link(ob)
    e = math.radians(CAM_ELEV)
    # frame centre (final-image centre) is GROUND_FRAC-0.5 of the height above the ground centre
    final_unit = SPRITE_PX[0] / VIEW_W  # px per world unit after correction
    dy = (GROUND_FRAC - 0.5) * SPRITE_PX[1] / final_unit  # world units north of ground centre
    target = Vector((0, dy * STRETCH, 0))
    d = 20.0
    ob.location = target + Vector((0, -math.cos(e) * d, math.sin(e) * d))
    ob.rotation_euler = (math.radians(90 - CAM_ELEV), 0, 0)
    sc.camera = ob
    return ob


def _setup_lights(sc, mask):
    w = bpy.data.worlds.new("w")
    sc.world = w
    w.use_nodes = True
    bg = w.node_tree.nodes.get('Background')
    if mask:
        bg.inputs['Color'].default_value = (0, 0, 0, 1)
        bg.inputs['Strength'].default_value = 0.0
        return
    bg.inputs['Color'].default_value = (0.58, 0.70, 0.92, 1)
    bg.inputs['Strength'].default_value = 0.8
    sun = bpy.data.lights.new("sun", 'SUN')
    sun.energy = 3.7
    sun.angle = math.radians(9)
    sun.color = (1.0, 0.93, 0.80)
    so = bpy.data.objects.new("sun", sun)
    sc.collection.objects.link(so)
    # light from the upper left (north-west), a bit toward the viewer's side
    dirv = Vector((0.7, 0.15, -0.8)).normalized()
    so.rotation_euler = dirv.to_track_quat('-Z', 'Y').to_euler()
    fill = bpy.data.lights.new("fill", 'SUN')
    fill.energy = 1.0
    fill.angle = math.radians(40)
    fill.color = (0.75, 0.82, 1.0)
    fo = bpy.data.objects.new("fill", fill)
    sc.collection.objects.link(fo)
    fo.rotation_euler = Vector((-0.2, 0.9, -0.55)).normalized().to_track_quat('-Z', 'Y').to_euler()
    for loc, color, power, size in _lights:
        pl = bpy.data.lights.new("pl", 'POINT')
        pl.energy = power
        pl.color = color
        pl.shadow_soft_size = size
        po = bpy.data.objects.new("pl", pl)
        sc.collection.objects.link(po)
        po.location = (loc.x, loc.y * STRETCH, loc.z)


def _read_png(path):
    img = bpy.data.images.load(path)
    img.colorspace_settings.name = 'Non-Color'
    img.alpha_mode = 'STRAIGHT'
    w, h = img.size
    px = np.empty(w * h * 4, dtype=np.float32)
    img.pixels.foreach_get(px)
    bpy.data.images.remove(img)
    return px.reshape(h, w, 4)[::-1]  # top row first


def _down(a):
    """Premultiplied box downsample by SS."""
    h, w, _ = a.shape
    p = a.copy()
    p[..., :3] *= p[..., 3:4]
    p = p.reshape(h // SS, SS, w // SS, SS, 4).mean(axis=(1, 3))
    al = p[..., 3:4]
    out = p.copy()
    out[..., :3] = np.where(al > 1e-4, p[..., :3] / np.maximum(al, 1e-4), 0.0)
    return out


def _grade(a):
    """Warm, slightly saturated grade plus a light unsharp mask on the colour channels."""
    rgb = a[..., :3]
    lum = rgb.mean(axis=2, keepdims=True)
    rgb = lum + (rgb - lum) * 1.1
    blur = (np.roll(rgb, 1, 0) + np.roll(rgb, -1, 0) + np.roll(rgb, 1, 1) + np.roll(rgb, -1, 1) + 4 * rgb) / 8.0
    rgb = rgb + (rgb - blur) * 0.55
    out = a.copy()
    out[..., :3] = np.clip(rgb, 0.0, 1.0)
    return out


def write_png(path, rgba8):
    h, w, _ = rgba8.shape
    rows = rgba8.reshape(h, w * 4)
    up = np.vstack([np.zeros((1, w * 4), np.uint8), rows[:-1]])
    sub = rows.copy()
    sub[:, 4:] -= rows[:, :-4]
    cands = [rows, sub, rows - up]                       # PNG filters None, Sub, Up (uint8 wraps)
    cost = np.stack([np.abs(c.astype(np.int8).astype(np.int32)).sum(axis=1) for c in cands])
    best = cost.argmin(axis=0)
    raw = b''.join(bytes([int(best[y])]) + cands[best[y]][y].tobytes() for y in range(h))

    def chunk(t, d):
        c = struct.pack('>I', len(d)) + t + d
        return c + struct.pack('>I', zlib.crc32(t + d) & 0xffffffff)

    data = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 6, 0, 0, 0)) + \
        chunk(b'IDAT', zlib.compress(raw, 9)) + chunk(b'IEND', b'')
    with open(path, 'wb') as f:
        f.write(data)


def render(builder, out_base, samples=40, tint=False, tmp='/tmp'):
    """Build the scene with `builder()`, render, write `out_base.png` (+ `_tint.png`)."""
    passes = [False] + ([True] if tint else [])
    imgs = []
    for mask in passes:
        matlib.MASK_MODE = mask
        clear_scene()
        reset()
        builder()
        realise()
        sc = bpy.context.scene
        _setup_render(sc, 6 if mask else samples, mask)
        _setup_camera(sc)
        _setup_lights(sc, mask)
        # stretch the whole scene north-south so the oblique camera draws the ground undistorted
        for ob in list(sc.collection.objects):
            if ob.type == 'MESH':
                ob.scale = (1, STRETCH, 1)
        tmpf = os.path.join(tmp, 'pass_%d.png' % int(mask))
        sc.render.filepath = tmpf
        bpy.ops.render.render(write_still=True)
        imgs.append(_down(_read_png(tmpf)))
    beauty = _grade(imgs[0])
    a = beauty[..., 3:4]
    if tint:
        m = np.clip(imgs[1][..., 0:1] * imgs[1][..., 3:4], 0, 1)  # accent coverage (premultiplied)
        m = np.minimum(m, a)
        frac = np.where(a > 1e-4, m / np.maximum(a, 1e-4), 0.0)
        base = beauty.copy()
        base[..., 3:4] = a * (1 - frac)
        acc = beauty.copy()
        lum = beauty[..., :3].mean(axis=2, keepdims=True)
        acc[..., :3] = np.clip(lum * 1.1, 0, 1)
        acc[..., 3:4] = a * frac
        write_png(out_base + '.png', (np.clip(base, 0, 1) * 255 + 0.5).astype(np.uint8))
        write_png(out_base + '_tint.png', (np.clip(acc, 0, 1) * 255 + 0.5).astype(np.uint8))
    else:
        write_png(out_base + '.png', (np.clip(beauty, 0, 1) * 255 + 0.5).astype(np.uint8))
