"""Low-poly modelling helpers for the WayCiv unit sprites (Blender 5.x, bpy).

Conventions: Z is up, models face +X, the origin is the ground point under the
unit.  Everything is built from simple solids (boxes, tapered cylinders,
spheres, lofted hulls) with flat colours; one special material, "team", marks
the parts that get the owner's colour at runtime.
"""
import math
import bpy
import bmesh
from mathutils import Vector, Matrix, Euler

SC = None
_STACK = [Matrix.Identity(4)]
_MATS = {}


def d(deg):
    return math.radians(deg)


def V(*a):
    if len(a) == 1:
        return Vector(a[0])
    return Vector(a)


# ----------------------------------------------------------------- colour
def srgb_to_lin(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def hexcol(h):
    h = h.lstrip("#")
    return tuple(srgb_to_lin(int(h[i:i + 2], 16) / 255.0) for i in (0, 2, 4)) + (1.0,)


PAL = dict(
    skin="#e2b48c", tan="#c58c5a", brown_skin="#8d5a3a", dark_skin="#5e3b26",
    undyed="#d8c79e", brown="#7a5a3c", green="#4a6b3a", dkgreen="#2f4a2a", blue="#3a5a8c",
    dkblue="#243a66", red="#b0382c", dkred="#7a2420", white="#ece7da", black="#2a2a2f",
    grey="#7d8186", ltgrey="#b4b8bc", dkgrey="#4a4e54", khaki="#b0a070", olive="#5d6140",
    bronze="#c08a3e", iron="#8d949b", steel="#bcc3ca", dkiron="#4b5058", gold="#e0b53a",
    silver="#cfd4d8", wood="#8a5a32", dkwood="#5d3d22", ltwood="#b98a55", leather="#6b4528",
    rope="#bda877", stone="#9a9488", canvas="#eadfc2", orange="#d9772c", yellow="#e6c84a",
    purple="#6b3f8c", hair_brown="#4a3020", hair_blond="#d2b062", hair_black="#1c1a1a",
    hair_red="#8a3a1c", hair_grey="#b5b2ac", navy="#1f2d4d", sea="#2f5f7a", hullgrey="#8f979e",
    haze="#aab4bd", pink="#d46a8a", teal="#2e8a8a", sand="#d2b878", fur="#8c6a48",
    redcoat="#b8231f", linen="#e9e2cf", glass="#7fb4cc", rust="#8a4a2a", ivory="#efe6cc",
    deck="#a8794a", water="#4a86a8",
)


SKIN = {"skin", "tan", "brown_skin", "dark_skin"}
WOOD = {"wood", "dkwood", "ltwood", "deck"}
LEATHER = {"leather", "fur", "rope"}


def _kind(name, metal_):
    if metal_ > 0.3:
        return "metal"
    if name in SKIN:
        return "skin"
    if name in WOOD:
        return "wood"
    if name in LEATHER:
        return "leather"
    if name in ("stone", "sand", "haze"):
        return "stone"
    if name in ("glass", "water", "sea", "gold", "silver", "ivory"):
        return "smooth"
    return "cloth"


# kind -> (noise scale, colour variation, roughness variation, bump strength, specular level)
_SURF = dict(metal=(30.0, 0.10, 0.10, 0.06, 0.5), skin=(14.0, 0.05, 0.05, 0.0, 0.35),
             wood=(1.0, 0.16, 0.10, 0.25, 0.3), leather=(42.0, 0.14, 0.12, 0.30, 0.4),
             stone=(26.0, 0.18, 0.10, 0.35, 0.25), smooth=(20.0, 0.04, 0.04, 0.0, 0.5),
             cloth=(90.0, 0.09, 0.05, 0.22, 0.2))


def _surface(m, col, rough, metal_, kind, emit):
    """Principled shader with a hint of surface: bevelled edges that catch the
    light, fine noise breaking up flat colour and roughness, bump grain."""
    nt = m.node_tree
    nodes, links = nt.nodes, nt.links
    bsdf = nodes.get("Principled BSDF")
    scale, cvar, rvar, bump, spec = _SURF[kind]
    bsdf.inputs["Metallic"].default_value = metal_
    bsdf.inputs["Specular IOR Level"].default_value = spec
    if kind == "metal":
        rough = min(rough, 0.34)
    elif kind == "cloth" and rough == 0.65:
        rough = 0.86
        bsdf.inputs["Sheen Weight"].default_value = 0.35
        bsdf.inputs["Sheen Roughness"].default_value = 0.5
    elif kind == "leather" and rough == 0.65:
        rough = 0.55
    elif kind == "wood" and rough == 0.65:
        rough = 0.72
    elif kind == "skin":
        rough = 0.52
        bsdf.inputs["Subsurface Weight"].default_value = 0.35
        bsdf.inputs["Subsurface Radius"].default_value = (1.0, 0.32, 0.2)
        bsdf.inputs["Subsurface Scale"].default_value = 0.05
    elif kind == "smooth" and rough == 0.65:
        rough = 0.35
    coord = nodes.new("ShaderNodeTexCoord")
    mp = nodes.new("ShaderNodeMapping")
    mp.inputs["Scale"].default_value = (scale, scale, scale) if kind != "wood" else (3.0, 38.0, 38.0)
    links.new(coord.outputs["Object"], mp.inputs["Vector"])
    noise = nodes.new("ShaderNodeTexNoise")
    noise.inputs["Scale"].default_value = 1.0
    noise.inputs["Detail"].default_value = 5.0
    noise.inputs["Roughness"].default_value = 0.6
    links.new(mp.outputs["Vector"], noise.inputs["Vector"])
    # colour variation (multiply by 1 +- cvar)
    tone = nodes.new("ShaderNodeMapRange")
    tone.inputs["To Min"].default_value = 1.0 - cvar
    tone.inputs["To Max"].default_value = 1.0 + cvar * 0.4
    links.new(noise.outputs["Fac"], tone.inputs["Value"])
    mix = nodes.new("ShaderNodeMix")
    mix.data_type = "RGBA"
    mix.blend_type = "MULTIPLY"
    mix.inputs["Factor"].default_value = 1.0
    mix.inputs["A"].default_value = col
    links.new(tone.outputs["Result"], mix.inputs["B"])
    links.new(mix.outputs["Result"], bsdf.inputs["Base Color"])
    # roughness variation
    rg = nodes.new("ShaderNodeMapRange")
    rg.inputs["To Min"].default_value = max(0.0, rough - rvar)
    rg.inputs["To Max"].default_value = min(1.0, rough + rvar)
    links.new(noise.outputs["Fac"], rg.inputs["Value"])
    links.new(rg.outputs["Result"], bsdf.inputs["Roughness"])
    # bevelled normal, then grain bump on top
    bev = nodes.new("ShaderNodeBevel")
    bev.samples = 4
    bev.inputs["Radius"].default_value = 0.018
    nrm = bev.outputs["Normal"]
    if bump > 0:
        bp = nodes.new("ShaderNodeBump")
        bp.inputs["Strength"].default_value = bump
        bp.inputs["Distance"].default_value = 0.02
        links.new(noise.outputs["Fac"], bp.inputs["Height"])
        links.new(bev.outputs["Normal"], bp.inputs["Normal"])
        nrm = bp.outputs["Normal"]
    links.new(nrm, bsdf.inputs["Normal"])
    if emit > 0:
        bsdf.inputs["Emission Color"].default_value = col
        bsdf.inputs["Emission Strength"].default_value = emit


def mat(name, rough=0.65, metal=0.0, emit=0.0):
    """Material by palette name or '#rrggbb'.  'team' is the tintable one."""
    key = (name, rough, metal, emit)
    if key in _MATS:
        return _MATS[key]
    if name == "team":
        col = (0.55, 0.55, 0.55)
        col = tuple(srgb_to_lin(c) for c in col) + (1.0,)
    elif name.startswith("#"):
        col = hexcol(name)
    else:
        col = hexcol(PAL[name])
    m = bpy.data.materials.new(name if len(_MATS) < 1000 else "m")
    m.name = name + "_%d" % len(_MATS) if name != "team" else "team_%d" % len(_MATS)
    m.use_nodes = True
    _surface(m, col, rough, metal, "cloth" if name == "team" else _kind(name, metal), emit)
    m.diffuse_color = col
    _MATS[key] = m
    return m


def metal(name):
    return mat(name, rough=0.34, metal=1.0)


# -------------------------------------------------------------- transform
class xf:
    """with xf(loc, rot_deg, s): ... — nested transforms (uniform scale only)."""

    def __init__(self, loc=(0, 0, 0), rot=(0, 0, 0), s=1.0):
        self.m = (Matrix.Translation(Vector(loc))
                  @ Euler((d(rot[0]), d(rot[1]), d(rot[2])), "XYZ").to_matrix().to_4x4()
                  @ Matrix.Scale(s, 4))

    def __enter__(self):
        _STACK.append(_STACK[-1] @ self.m)

    def __exit__(self, *a):
        _STACK.pop()


def W(p):
    """Point to world through the stack."""
    return _STACK[-1] @ Vector(p)


def _scale_factor():
    return _STACK[-1].to_scale().x


# ------------------------------------------------------------------- core
def reset(scene=None):
    global SC
    for coll in (bpy.data.objects, bpy.data.meshes, bpy.data.materials, bpy.data.cameras,
                 bpy.data.lights, bpy.data.worlds):
        for x in list(coll):
            coll.remove(x)
    _MATS.clear()
    del _STACK[1:]
    SC = bpy.context.scene


def _mk(name, verts, faces, m, smooth=None, bevel=0.0, solid=0.0, recalc=True):
    me = bpy.data.meshes.new(name)
    me.from_pydata([tuple(v) for v in verts], [], faces)
    if recalc:
        bm = bmesh.new()
        bm.from_mesh(me)
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        bm.to_mesh(me)
        bm.free()
    me.update()
    if smooth is not None:
        me.polygons.foreach_set("use_smooth", smooth)
    ob = bpy.data.objects.new(name, me)
    SC.collection.objects.link(ob)
    if m is not None:
        me.materials.append(m)
    if bevel > 0:
        b = ob.modifiers.new("bev", "BEVEL")
        b.width = bevel
        b.segments = 2
        b.limit_method = "ANGLE"
        b.angle_limit = d(35)
    if solid > 0:
        s = ob.modifiers.new("sol", "SOLIDIFY")
        s.thickness = solid
        s.offset = 0
    return ob


def _m(m):
    return mat(m) if isinstance(m, str) else m


def _box_geom(sx, sy, sz):
    x, y, z = sx / 2, sy / 2, sz / 2
    v = [(-x, -y, -z), (x, -y, -z), (x, y, -z), (-x, y, -z),
         (-x, -y, z), (x, -y, z), (x, y, z), (-x, y, z)]
    f = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4), (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)]
    return v, f


def box(loc, size, m, rot=(0, 0, 0), bevel=0.025, name="box"):
    """Box centred on loc with full extents size, rotated by rot degrees."""
    v, f = _box_geom(*size)
    R = Euler((d(rot[0]), d(rot[1]), d(rot[2])), "XYZ").to_matrix().to_4x4()
    T = _STACK[-1] @ Matrix.Translation(Vector(loc)) @ R
    bev = min(bevel, min(size) * 0.3) * _scale_factor()
    return _mk(name, [T @ Vector(p) for p in v], f, _m(m), bevel=bev)


def _cyl_geom(r0, r1, h, seg, cap=True):
    v, f = [], []
    for i in range(seg):
        a = 2 * math.pi * i / seg
        c, s = math.cos(a), math.sin(a)
        v.append((r0 * c, r0 * s, -h / 2))
    for i in range(seg):
        a = 2 * math.pi * i / seg
        c, s = math.cos(a), math.sin(a)
        v.append((r1 * c, r1 * s, h / 2))
    sides = []
    for i in range(seg):
        j = (i + 1) % seg
        f.append((i, j, seg + j, seg + i))
        sides.append(True)
    if cap:
        f.append(tuple(reversed(range(seg))))
        sides.append(False)
        f.append(tuple(range(seg, 2 * seg)))
        sides.append(False)
    return v, f, sides


def cyl(loc, r, h, m, rot=(0, 0, 0), r2=None, seg=10, cap=True, name="cyl"):
    """Tapered cylinder along local Z (centred on loc); r2 = top radius."""
    r2 = r if r2 is None else r2
    seg = max(seg, 18) if seg >= 8 else seg
    v, f, sm = _cyl_geom(r, r2, h, seg, cap)
    R = Euler((d(rot[0]), d(rot[1]), d(rot[2])), "XYZ").to_matrix().to_4x4()
    T = _STACK[-1] @ Matrix.Translation(Vector(loc)) @ R
    return _mk(name, [T @ Vector(p) for p in v], f, _m(m), smooth=sm)


def limb(p0, p1, r0, m, r1=None, seg=8, cap=True, name="limb"):
    """Tapered cylinder from point p0 to p1 (radius r0 -> r1)."""
    a, b = W(p0), W(p1)
    s = _scale_factor()
    r1 = r0 if r1 is None else r1
    dv = b - a
    L = dv.length
    if L < 1e-6:
        return None
    q = dv.normalized().to_track_quat("Z", "Y")
    seg = max(seg, 10) if seg >= 8 else seg
    v, f, sm = _cyl_geom(r0 * s, r1 * s, L, seg, cap)
    M = Matrix.Translation((a + b) / 2) @ q.to_matrix().to_4x4()
    return _mk(name, [M @ Vector(p) for p in v], f, _m(m), smooth=sm)


def beam(p0, p1, w, h, m, up=(0, 0, 1), bevel=0.02, name="beam"):
    """Box from p0 to p1 with cross-section w (horizontal) by h (vertical-ish)."""
    a, b = W(p0), W(p1)
    s = _scale_factor()
    dv = b - a
    L = dv.length
    if L < 1e-6:
        return None
    upv = _STACK[-1].to_3x3() @ Vector(up)
    dn = dv.normalized()
    if abs(dn.dot(upv.normalized())) > 0.98:
        upv = Vector((1, 0, 0))
    q = dn.to_track_quat("X", "Z") if upv == Vector((0, 0, 1)) else _track_up(dn, upv)
    v, f = _box_geom(L, w * s, h * s)
    M = Matrix.Translation((a + b) / 2) @ q.to_matrix().to_4x4()
    bev = min(bevel * s, min(w, h) * s * 0.3)
    return _mk(name, [M @ Vector(p) for p in v], f, _m(m), bevel=bev)


def _track_up(x, up):
    x = x.normalized()
    z = (up - x * up.dot(x)).normalized()
    y = z.cross(x)
    M = Matrix((x, y, z)).transposed()
    return M.to_quaternion()


def sph(loc, r, m, scale=(1, 1, 1), seg=10, rings=6, rot=(0, 0, 0), name="sph"):
    seg, rings = max(seg, 20), max(rings, 12)
    v, f = [], []
    v.append((0, 0, -1))
    for j in range(1, rings):
        t = math.pi * j / rings
        for i in range(seg):
            a = 2 * math.pi * i / seg
            v.append((math.sin(t) * math.cos(a), math.sin(t) * math.sin(a), -math.cos(t)))
    v.append((0, 0, 1))
    top = len(v) - 1
    for i in range(seg):
        f.append((0, 1 + (i + 1) % seg, 1 + i))
    for j in range(rings - 2):
        for i in range(seg):
            a = 1 + j * seg + i
            b = 1 + j * seg + (i + 1) % seg
            f.append((a, b, b + seg, a + seg))
    base = 1 + (rings - 2) * seg
    for i in range(seg):
        f.append((top, base + i, base + (i + 1) % seg))
    S = Matrix.Diagonal((r * scale[0], r * scale[1], r * scale[2], 1))
    R = Euler((d(rot[0]), d(rot[1]), d(rot[2])), "XYZ").to_matrix().to_4x4()
    T = _STACK[-1] @ Matrix.Translation(Vector(loc)) @ R @ S
    return _mk(name, [T @ Vector(p) for p in v], f, _m(m), smooth=[True] * len(f))


def ring_loft(rings, m, cap_start=True, cap_end=True, smooth=False, name="loft", close=False):
    """Loft closed rings (lists of 3-tuples in local coordinates) into a solid."""
    n = len(rings[0])
    v, f = [], []
    for r in rings:
        for p in r:
            v.append(W(p))
    K = len(rings)
    for k in range(K if close else K - 1):
        k2 = (k + 1) % K
        for i in range(n):
            j = (i + 1) % n
            f.append((k * n + i, k * n + j, k2 * n + j, k2 * n + i))
    if close:
        cap_start = cap_end = False
    if cap_start:
        f.append(tuple(reversed(range(n))))
    if cap_end:
        f.append(tuple(range((len(rings) - 1) * n, len(rings) * n)))
    sm = [smooth] * len(f)
    return _mk(name, v, f, _m(m), smooth=sm)


def hull(secs, m, smooth=False, name="hull"):
    """Ship/boat hull: secs = [(x, wb, wm, wt, zb, zm, zt)] half-widths at bottom,
    middle and top (gunwale) of each cross-section, lofted along X."""
    rings = []
    for (x, wb, wm, wt, zb, zm, zt) in secs:
        rings.append([(x, -wb, zb), (x, -wm, zm), (x, -wt, zt),
                      (x, wt, zt), (x, wm, zm), (x, wb, zb)])
    return ring_loft(rings, m, smooth=smooth, name=name)


def tube(secs, m, n=10, smooth=True, name="tube"):
    """Elliptical tube lofted along X: secs = [(x, ry, rz, zc, yc?)]."""
    rings = []
    for s in secs:
        x, ry, rz, zc = s[:4]
        yc = s[4] if len(s) > 4 else 0
        ring = []
        for i in range(n):
            a = 2 * math.pi * i / n
            ring.append((x, yc + ry * math.cos(a), zc + rz * math.sin(a)))
        rings.append(ring)
    return ring_loft(rings, m, smooth=smooth, name=name)


def torus(loc, R, r, m, plane="XZ", seg=16, tseg=6, name="torus"):
    """Ring of major radius R and tube radius r; plane XZ means the ring faces Y."""
    rings = []
    for i in range(seg):
        a = 2 * math.pi * i / seg
        ring = []
        for k in range(tseg):
            b = 2 * math.pi * k / tseg
            rr = R + r * math.cos(b)
            zz = r * math.sin(b)
            if plane == "XZ":
                p = (rr * math.cos(a), zz, rr * math.sin(a))
            elif plane == "XY":
                p = (rr * math.cos(a), rr * math.sin(a), zz)
            else:
                p = (zz, rr * math.cos(a), rr * math.sin(a))
            ring.append((loc[0] + p[0], loc[1] + p[1], loc[2] + p[2]))
        rings.append(ring)
    return ring_loft(rings, m, smooth=True, close=True, name=name)


def poly(pts, m, plane="XY", offset=0.0, thick=0.04, name="poly", bevel=0.0):
    """Flat polygon extruded by thick.  plane='XY' extrudes along Z, 'XZ' along Y,
    'YZ' along X (pts are 2D in that plane)."""
    verts, faces = [], []
    n = len(pts)
    for side in (-0.5, 0.5):
        for (a, b) in pts:
            if plane == "XY":
                verts.append(W((a, b, offset + side * thick)))
            elif plane == "XZ":
                verts.append(W((a, offset + side * thick, b)))
            else:
                verts.append(W((offset + side * thick, a, b)))
    faces.append(tuple(reversed(range(n))))
    faces.append(tuple(range(n, 2 * n)))
    for i in range(n):
        j = (i + 1) % n
        faces.append((i, j, n + j, n + i))
    bev = bevel * _scale_factor()
    return _mk(name, verts, faces, _m(m), bevel=bev)


def cloth(p00, p10, p01, m, bulge=0.1, nu=6, nv=5, axis=(0, 1, 0), name="cloth", solid=0.02, wave=0.0):
    """Billowing sheet spanned by corner p00, edge to p10 (u) and p01 (v); the
    middle bulges along `axis`.  Used for sails and banners."""
    a, b, c = Vector(p00), Vector(p10), Vector(p01)
    U, Vv = b - a, c - a
    ax = Vector(axis)
    verts, faces = [], []
    for j in range(nv + 1):
        for i in range(nu + 1):
            u, vv = i / nu, j / nv
            p = a + U * u + Vv * vv
            p = p + ax * (bulge * math.sin(math.pi * u) * math.sin(math.pi * vv)
                          + wave * math.sin(u * math.pi * 3) * vv)
            verts.append(W(p))
    for j in range(nv):
        for i in range(nu):
            k = j * (nu + 1) + i
            faces.append((k, k + 1, k + nu + 2, k + nu + 1))
    ob = _mk(name, verts, faces, _m(m), smooth=[True] * len(faces), solid=solid * _scale_factor(), recalc=False)
    return ob


def rep(n, fn):
    for i in range(n):
        fn(i)


def mirror_y(fn):
    fn(1)
    fn(-1)


# ----------------------------------------------------------------- scene
def _sky(sc):
    """Soft studio-outdoors environment: blue zenith, pale warm horizon and a
    grassy bounce from below, so metals and glossy parts have something to reflect."""
    w = bpy.data.worlds.new("w")
    w.use_nodes = True
    nt = w.node_tree
    nodes, links = nt.nodes, nt.links
    bg = nodes["Background"]
    coord = nodes.new("ShaderNodeTexCoord")
    sep = nodes.new("ShaderNodeSeparateXYZ")
    links.new(coord.outputs["Generated"], sep.inputs["Vector"])
    ramp = nodes.new("ShaderNodeValToRGB")
    rm = ramp.color_ramp
    rm.elements[0].position = 0.30
    rm.elements[0].color = hexcol("#5f6a45")
    rm.elements[1].position = 0.85
    rm.elements[1].color = hexcol("#5d8fd6")
    mid = rm.elements.new(0.52)
    mid.color = hexcol("#f1e6d2")
    links.new(sep.outputs["Z"], ramp.inputs["Fac"])
    links.new(ramp.outputs["Color"], bg.inputs["Color"])
    bg.inputs["Strength"].default_value = 0.95
    sc.world = w


def setup_scene(res=512, samples=96, ortho=3.0, elev=45, yaw=-30.0, anchor=0.72,
                key_energy=3.4):
    """Cycles CPU, transparent film, ortho camera at `elev` degrees, warm sun
    from the upper left, soft sky fill and a cool rim light behind."""
    sc = bpy.context.scene
    sc.render.engine = "CYCLES"
    sc.cycles.device = "CPU"
    sc.cycles.samples = samples
    sc.cycles.use_denoising = True
    sc.cycles.max_bounces = 4
    sc.cycles.diffuse_bounces = 3
    sc.cycles.glossy_bounces = 3
    sc.cycles.transmission_bounces = 2
    sc.cycles.sample_clamp_indirect = 5.0
    sc.render.film_transparent = True
    sc.render.resolution_x = sc.render.resolution_y = res
    sc.render.resolution_percentage = 100
    sc.render.image_settings.file_format = "PNG"
    sc.render.image_settings.color_mode = "RGBA"
    sc.view_settings.view_transform = "Standard"
    sc.view_settings.look = "None"
    sc.view_settings.exposure = 0.0
    sc.render.filter_size = 1.1
    _sky(sc)
    # camera
    cam = bpy.data.cameras.new("cam")
    cam.type = "ORTHO"
    cam.ortho_scale = ortho
    cam.clip_start = 0.1
    cam.clip_end = 200
    co = bpy.data.objects.new("cam", cam)
    sc.collection.objects.link(co)
    sc.camera = co
    e = d(elev)
    dist = 40
    co.location = (0, -dist * math.cos(e), dist * math.sin(e))
    co.rotation_euler = (d(90 - elev), 0, 0)
    cam.shift_y = anchor - 0.5  # origin lands `anchor` of the way down the frame
    # key: warm late-morning sun from the upper left front; rim: cool, from behind
    for name, rot, energy, col, ang in (
        ("key", (d(46), d(8), d(-52)), key_energy, (1.0, 0.88, 0.70), 5),
        ("rim", (d(62), 0, d(150)), 1.5, (0.72, 0.84, 1.0), 4),
        ("fill", (d(40), 0, d(60)), 0.45, (1.0, 0.93, 0.85), 40),
    ):
        l = bpy.data.lights.new(name, "SUN")
        l.energy = energy
        l.color = col
        l.angle = d(ang)
        lo = bpy.data.objects.new(name, l)
        sc.collection.objects.link(lo)
        lo.rotation_euler = rot
    # invisible bounce plane so undersides pick up some earthy light
    ground = bpy.data.objects.new("bounce", bpy.data.meshes.new("bounce"))
    gm = ground.data
    gm.from_pydata([(-30, -30, -0.01), (30, -30, -0.01), (30, 30, -0.01), (-30, 30, -0.01)], [], [(0, 1, 2, 3)])
    gm.update()
    gmat = bpy.data.materials.new("bounce")
    gmat.use_nodes = True
    gmat.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = hexcol("#7f8456")
    gmat.node_tree.nodes["Principled BSDF"].inputs["Roughness"].default_value = 1.0
    gm.materials.append(gmat)
    sc.collection.objects.link(ground)
    ground.visible_camera = False
    ground.visible_shadow = False
    return sc


def set_ortho(scale, anchor):
    sc = bpy.context.scene
    sc.camera.data.ortho_scale = scale
    sc.camera.data.shift_y = anchor - 0.5


def apply_mask_materials():
    """Switch every material to the mask pass: team -> white emission, rest holdout."""
    for o in bpy.data.objects:
        if o.type == "MESH":
            o.is_holdout = False
            o.is_shadow_catcher = False
            if o.name.startswith("bounce"):
                o.visible_camera = False
    for m in bpy.data.materials:
        if m.name.startswith("bounce"):
            continue
        nt = m.node_tree
        nt.nodes.clear()
        out = nt.nodes.new("ShaderNodeOutputMaterial")
        if m.name.startswith("team"):
            e = nt.nodes.new("ShaderNodeEmission")
            e.inputs["Color"].default_value = (1, 1, 1, 1)
            e.inputs["Strength"].default_value = 1.0
            nt.links.new(e.outputs[0], out.inputs[0])
        else:
            h = nt.nodes.new("ShaderNodeHoldout")
            nt.links.new(h.outputs[0], out.inputs[0])
    sc = bpy.context.scene
    sc.cycles.samples = 16
    sc.cycles.use_denoising = False
    sc.world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.0
    for o in bpy.data.objects:
        if o.type == "LIGHT":
            o.hide_render = True


def render_shadow(path, samples=128):
    """Soft cast shadow on the ground, alone: every model becomes a holdout and
    the bounce plane a shadow catcher, so the alpha channel is the shadow."""
    sc = bpy.context.scene
    for o in bpy.data.objects:
        if o.type == "MESH":
            if o.name.startswith("bounce"):
                o.visible_camera = True
                o.is_shadow_catcher = True
            else:
                o.is_holdout = True
        elif o.type == "LIGHT":
            o.hide_render = True
    # the cast shadow falls to the right and a little toward the viewer, like
    # the terrain's, whatever the model shading does
    sun = bpy.data.lights.new("shadow", "SUN")
    sun.energy = 4.0
    sun.angle = d(7)
    so = bpy.data.objects.new("shadow", sun)
    sc.collection.objects.link(so)
    so.rotation_euler = Vector((0.6, -0.25, -0.76)).to_track_quat("-Z", "Y").to_euler()
    sc.cycles.samples = samples
    sc.cycles.use_denoising = False
    render_to(path)


def render_to(path):
    sc = bpy.context.scene
    sc.render.filepath = path
    bpy.ops.render.render(write_still=True)
