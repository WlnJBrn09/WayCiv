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
import struct
import zlib
from contextlib import contextmanager

import bmesh
import bpy
import numpy as np
from mathutils import Matrix, Vector

PLOT_R = 0.90          # circumradius of the plot hex (world 1.0 = tile radius)
CAM_ELEV = 42.0        # camera elevation, degrees
VIEW_W = 2.2           # sprite width in world units
SPRITE_PX = (256, 320)  # final sprite size
GROUND_FRAC = 0.60     # ground centre sits this far down the sprite
SS = 2                 # supersampling factor
STRETCH = 1.0 / math.sin(math.radians(CAM_ELEV))


# --------------------------------------------------------------------------
# colours and materials
# --------------------------------------------------------------------------

def lin(c):
    """sRGB 0..1 -> linear."""
    return tuple((x / 12.92 if x <= 0.04045 else ((x + 0.055) / 1.055) ** 2.4) for x in c)


def hexc(h):
    h = h.lstrip('#')
    return tuple(int(h[i:i + 2], 16) / 255.0 for i in (0, 2, 4))


PAL = {
    'stone': ('#cdbd9c', 0.85), 'stone2': ('#a89a7e', 0.9), 'stoneD': ('#7f735e', 0.9),
    'marble': ('#ece6d8', 0.5), 'sand': ('#e0c88f', 0.95), 'sandD': ('#c4a56a', 0.95),
    'brick': ('#b5603f', 0.9), 'brickD': ('#8a4630', 0.9), 'adobe': ('#d9a46a', 0.95),
    'terra': ('#c4492a', 0.8), 'terraD': ('#9a3320', 0.8),
    'wood': ('#7b5230', 0.8), 'woodL': ('#a8794a', 0.8), 'woodD': ('#4d3320', 0.85),
    'thatch': ('#c9a652', 0.95), 'slate': ('#4f5a6b', 0.7), 'slateL': ('#7d8aa0', 0.6),
    'grass': ('#6fa23c', 1.0), 'grassD': ('#4d8029', 1.0), 'dirt': ('#8c6a45', 1.0),
    'leaf': ('#3f8a2c', 0.9), 'leafD': ('#2c6a20', 0.9), 'leafL': ('#7bb544', 0.9),
    'pave': ('#b8ad98', 0.9), 'paveD': ('#988d7a', 0.9), 'paveL': ('#d6ccb8', 0.9),
    'gold': ('#e8b83a', 0.3, 0.9), 'goldD': ('#b8861e', 0.4, 0.9),
    'bronze': ('#a8703a', 0.4, 0.8), 'copper': ('#c26a45', 0.45, 0.8), 'verdi': ('#4fa592', 0.5, 0.5),
    'iron': ('#555a62', 0.5, 0.7), 'steel': ('#9aa4b2', 0.35, 0.8), 'dark': ('#2b2d33', 0.7),
    'white': ('#f4f1ea', 0.7), 'red': ('#c0392b', 0.7), 'blue': ('#2f6fb8', 0.7),
    'royal': ('#274a9a', 0.6), 'purple': ('#7a3f9a', 0.7), 'green': ('#2f8a4a', 0.7),
    'water': ('#3f8fc4', 0.15), 'waterD': ('#2a6a9a', 0.15), 'foam': ('#d8eef2', 0.6),
    'glass': ('#8ec8e8', 0.1), 'glassD': ('#3a6e96', 0.1),
    'concrete': ('#b9b9b4', 0.9), 'concreteD': ('#8c8c88', 0.9), 'asphalt': ('#3a3b40', 0.95),
    'yellow': ('#e8c030', 0.7), 'orange': ('#e07a28', 0.7), 'jade': ('#5aa88a', 0.5),
    'cream': ('#efe3c0', 0.8), 'rose': ('#d98a8a', 0.8), 'turq': ('#3aa6b0', 0.6),
    'acc': ('#ffffff', 0.6), 'accD': ('#ffffff', 0.6),
    'earth': ('#9a7a52', 1.0), 'snow': ('#f0f4f8', 0.9), 'rock': ('#8a8478', 0.95),
}

_mats = {}
MASK_MODE = False


def M(name, rgb=None, rough=None, metal=None, emit=None, emit_strength=6.0):
    """A material by palette name (or custom `rgb`); cached per scene."""
    key = (name, rgb, rough, metal, emit, MASK_MODE)
    if key in _mats:
        return _mats[key]
    spec = PAL.get(name, ('#c0c0c0', 0.8))
    base = hexc(spec[0]) if rgb is None else (hexc(rgb) if isinstance(rgb, str) else rgb)
    r = spec[1] if rough is None else rough
    m = spec[2] if (metal is None and len(spec) > 2) else (metal or 0.0)
    mat = bpy.data.materials.new(f"{name}{len(_mats)}")
    mat.use_nodes = True
    nt = mat.node_tree
    for n in list(nt.nodes):
        nt.nodes.remove(n)
    out = nt.nodes.new('ShaderNodeOutputMaterial')
    if MASK_MODE:
        e = nt.nodes.new('ShaderNodeEmission')
        v = 1.0 if name.startswith('acc') else 0.0
        e.inputs['Color'].default_value = (v, v, v, 1)
        e.inputs['Strength'].default_value = 1.0
        nt.links.new(e.outputs[0], out.inputs[0])
    elif emit:
        e = nt.nodes.new('ShaderNodeEmission')
        ec = hexc(emit) if isinstance(emit, str) else emit
        e.inputs['Color'].default_value = (*lin(ec), 1)
        e.inputs['Strength'].default_value = emit_strength
        nt.links.new(e.outputs[0], out.inputs[0])
    else:
        p = nt.nodes.new('ShaderNodeBsdfPrincipled')
        p.inputs['Base Color'].default_value = (*lin(base), 1)
        p.inputs['Roughness'].default_value = r
        p.inputs['Metallic'].default_value = m
        if 'Specular IOR Level' in p.inputs:
            p.inputs['Specular IOR Level'].default_value = 0.3
        nt.links.new(p.outputs[0], out.inputs[0])
    _mats[key] = mat
    return mat


# --------------------------------------------------------------------------
# geometry accumulation
# --------------------------------------------------------------------------

_bm = {}
_stack = [Matrix.Identity(4)]
_lights = []   # (location, color, power, size) point lights in local scene space
USE_CATCHER = False  # shadows on a transparent plot (harbors on water)


def reset():
    global _bm, _stack, _mats, _lights, USE_CATCHER
    USE_CATCHER = False
    _bm = {}
    _stack = [Matrix.Identity(4)]
    _mats = {}
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


def hexslab(r, z, h, mat, x=0.0, y=0.0):
    """Pointy-top hexagonal slab (circumradius r)."""
    ext(_hex_pts(r), z, h, mat, x=x, y=y)


def tree(x, y, s=1.0, kind='round', z=0.0):
    if kind == 'round':
        cyl(x, y, z, 0.025 * s, 0.1 * s, M('woodD'), n=6)
        sphere(x, y, z + 0.07 * s, 0.1 * s, M('leaf'), n=10, rings=6)
    elif kind == 'pine':
        cyl(x, y, z, 0.02 * s, 0.06 * s, M('woodD'), n=6)
        cone(x, y, z + 0.05 * s, 0.1 * s, 0.14 * s, M('leafD'), n=9)
        cone(x, y, z + 0.13 * s, 0.075 * s, 0.13 * s, M('leaf'), n=9)
    elif kind == 'palm':
        lathe(x, y, z, [(0.025 * s, 0), (0.015 * s, 0.2 * s)], M('woodL'), n=6)
        for k in range(6):
            with at(x, y, z + 0.2 * s, rot=60 * k):
                _face(M('leaf'), [(0, -0.02 * s, 0), (0.17 * s, -0.01 * s, -0.06 * s), (0.2 * s, 0.0, -0.09 * s),
                                  (0.17 * s, 0.01 * s, -0.06 * s), (0, 0.02 * s, 0)])
                _face(M('leaf'), [(0, 0.02 * s, 0), (0.17 * s, 0.01 * s, -0.06 * s), (0.2 * s, 0.0, -0.09 * s),
                                  (0.17 * s, -0.01 * s, -0.06 * s), (0, -0.02 * s, 0)])


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

def plot(base='pave', top=None, side='stoneD', r=PLOT_R, h=0.07, rim='stone2', inset=0.045):
    """The hexagonal plot every district/wonder sits on.  Returns the ground height.
    `base` is the surface material (or, when `top` is given, the slab's side colour)."""
    if top is None:
        top = base
    else:
        side = base
    ext(_hex_pts(r), -h, h, M(side))
    ext(_hex_pts(r - inset * 0.3), 0.0, 0.012, M(rim))
    ext(_hex_pts(r - inset), 0.012, 0.006, M(top))
    return 0.018


def road_strip(x0, y0, x1, y1, w, z, mat):
    dx, dy = x1 - x0, y1 - y0
    L = math.hypot(dx, dy)
    box((x0 + x1) / 2, (y0 + y1) / 2, z, L, w, 0.008, mat, rot=math.degrees(math.atan2(dy, dx)))


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
    if USE_CATCHER and not MASK_MODE:
        mesh = bpy.data.meshes.new("catcher")
        bm = bmesh.new()
        r = PLOT_R * 1.05
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
    bg.inputs['Color'].default_value = (0.62, 0.72, 0.9, 1)
    bg.inputs['Strength'].default_value = 0.75
    sun = bpy.data.lights.new("sun", 'SUN')
    sun.energy = 3.4
    sun.angle = math.radians(7)
    sun.color = (1.0, 0.95, 0.86)
    so = bpy.data.objects.new("sun", sun)
    sc.collection.objects.link(so)
    # light from the upper left (north-west), a bit toward the viewer's side
    dirv = Vector((0.55, -0.45, -0.9)).normalized()
    so.rotation_euler = dirv.to_track_quat('-Z', 'Y').to_euler()
    fill = bpy.data.lights.new("fill", 'SUN')
    fill.energy = 0.45
    fill.angle = math.radians(40)
    fill.color = (0.75, 0.82, 1.0)
    fo = bpy.data.objects.new("fill", fill)
    sc.collection.objects.link(fo)
    fo.rotation_euler = Vector((-0.5, 0.6, -0.6)).normalized().to_track_quat('-Z', 'Y').to_euler()
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


def write_png(path, rgba8):
    h, w, _ = rgba8.shape
    raw = b''.join(b'\x00' + rgba8[y].tobytes() for y in range(h))

    def chunk(t, d):
        c = struct.pack('>I', len(d)) + t + d
        return c + struct.pack('>I', zlib.crc32(t + d) & 0xffffffff)

    data = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 6, 0, 0, 0)) + \
        chunk(b'IDAT', zlib.compress(raw, 9)) + chunk(b'IEND', b'')
    with open(path, 'wb') as f:
        f.write(data)


def render(builder, out_base, samples=40, tint=False, tmp='/tmp'):
    """Build the scene with `builder()`, render, write `out_base.png` (+ `_tint.png`)."""
    global MASK_MODE
    passes = [False] + ([True] if tint else [])
    imgs = []
    for mask in passes:
        MASK_MODE = mask
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
    beauty = imgs[0]
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
