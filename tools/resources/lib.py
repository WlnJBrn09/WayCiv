"""Shared Blender helpers for the WayCiv resource / improvement sprites.

Run through Blender:  blender -b --factory-startup -t 4 -P render.py -- <args>
Everything is procedural and original; nothing is derived from Civ VI art.

Conventions: Z is up, the tile centre is the origin, a hex has radius 1
(pointy top, matching the map), models stay inside radius ~0.8.
"""
import math
import random

import bmesh
import bpy
from mathutils import Matrix, Vector

# ---------------------------------------------------------------- scene


def reset_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    for coll in (bpy.data.meshes, bpy.data.materials, bpy.data.curves):
        for item in list(coll):
            coll.remove(item)
    MATS.clear()


def setup_render(res=256, samples=24, top_down=False, ortho=1.7, elev=52.0, target_z=0.25, shift=-0.09):
    sc = bpy.context.scene
    sc.render.engine = "CYCLES"
    sc.cycles.device = "CPU"
    sc.cycles.samples = samples
    sc.cycles.use_adaptive_sampling = True
    sc.cycles.use_denoising = True
    sc.cycles.denoiser = "OPENIMAGEDENOISE"
    sc.cycles.max_bounces = 3
    sc.cycles.diffuse_bounces = 2
    sc.cycles.glossy_bounces = 2
    sc.cycles.transmission_bounces = 2
    sc.cycles.transparent_max_bounces = 4
    sc.cycles.use_fast_gi = False
    sc.render.threads_mode = "FIXED"
    sc.render.threads = 4
    sc.render.resolution_x = res
    sc.render.resolution_y = res
    sc.render.resolution_percentage = 100
    sc.render.film_transparent = True
    sc.render.image_settings.file_format = "PNG"
    sc.render.image_settings.color_mode = "RGBA"
    sc.render.filter_size = 1.2
    sc.view_settings.view_transform = "Standard"
    sc.view_settings.look = "None"
    sc.view_settings.exposure = 0.0
    sc.view_settings.gamma = 1.0

    # World: soft sky fill.
    w = bpy.data.worlds.new("w")
    w.use_nodes = True
    bg = w.node_tree.nodes["Background"]
    bg.inputs[0].default_value = (0.72, 0.80, 0.95, 1)
    bg.inputs[1].default_value = 0.65
    sc.world = w

    # Sun from the upper left front (shadows fall right and back).
    sun = bpy.data.lights.new("sun", "SUN")
    sun.energy = 3.0
    sun.angle = math.radians(14)
    sun.color = (1.0, 0.95, 0.86)
    so = bpy.data.objects.new("sun", sun)
    bpy.context.collection.objects.link(so)
    so.rotation_euler = (math.radians(48), math.radians(8), math.radians(-32))

    # Camera.
    cam = bpy.data.cameras.new("cam")
    cam.type = "ORTHO"
    cam.ortho_scale = ortho
    co = bpy.data.objects.new("cam", cam)
    bpy.context.collection.objects.link(co)
    e = math.radians(90.0 if top_down else elev)
    d = 12.0
    tz = 0.0 if top_down else target_z
    co.location = (0, -d * math.cos(e), tz + d * math.sin(e))
    co.rotation_euler = (math.radians(90) - e, 0, 0)
    cam.shift_y = 0.0 if top_down else shift
    cam.clip_start = 0.1
    cam.clip_end = 60
    sc.camera = co
    return sc


def shadow_catcher(radius=1.4, z=0.0):
    """A big ground disc that only catches shadows (alpha in the render)."""
    bpy.ops.mesh.primitive_circle_add(vertices=48, radius=radius, fill_type="NGON", location=(0, 0, z))
    o = bpy.context.active_object
    o.is_shadow_catcher = True
    return o


# ------------------------------------------------------------ materials

MATS = {}


def _principled(nt):
    return next(n for n in nt.nodes if n.type == "BSDF_PRINCIPLED")


def mat(name, col, rough=0.65, metal=0.0, col2=None, scale=6.0, bump=0.0, emit=0.0, emit_col=None,
        alpha=1.0, coat=0.0, sss=0.0, wave=False, ior=None, transmission=0.0, stripes=None, patch=None):
    """Cached Principled material. col2 adds noise colour variation; wave makes
    wood-grain bands; stripes=(count) adds a sine banding between col and col2."""
    key = (name, tuple(col), rough, metal, tuple(col2) if col2 else None, scale, bump, emit, alpha, wave, patch, coat, transmission, stripes)
    if key in MATS:
        return MATS[key]
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    nt = m.node_tree
    bsdf = _principled(nt)
    rgba = (col[0], col[1], col[2], 1)
    bsdf.inputs["Base Color"].default_value = rgba
    bsdf.inputs["Roughness"].default_value = rough
    bsdf.inputs["Metallic"].default_value = metal
    if alpha < 1.0:
        bsdf.inputs["Alpha"].default_value = alpha
    if coat:
        bsdf.inputs["Coat Weight"].default_value = coat
    if sss:
        bsdf.inputs["Subsurface Weight"].default_value = sss
        bsdf.inputs["Subsurface Radius"].default_value = (0.4, 0.2, 0.1)
    if transmission:
        bsdf.inputs["Transmission Weight"].default_value = transmission
    if ior:
        bsdf.inputs["IOR"].default_value = ior
    if emit > 0:
        ec = emit_col or col
        bsdf.inputs["Emission Color"].default_value = (ec[0], ec[1], ec[2], 1)
        bsdf.inputs["Emission Strength"].default_value = emit
    if col2 is not None or bump > 0 or wave or stripes:
        tc = nt.nodes.new("ShaderNodeTexCoord")
        mp = nt.nodes.new("ShaderNodeMapping")
        nt.links.new(tc.outputs["Object"], mp.inputs["Vector"])
        noise = nt.nodes.new("ShaderNodeTexNoise")
        noise.inputs["Scale"].default_value = scale
        noise.inputs["Detail"].default_value = 6.0
        noise.inputs["Roughness"].default_value = 0.6
        nt.links.new(mp.outputs["Vector"], noise.inputs["Vector"])
        fac = noise.outputs["Fac"]
        if wave:
            wv = nt.nodes.new("ShaderNodeTexWave")
            wv.inputs["Scale"].default_value = scale
            wv.inputs["Distortion"].default_value = 3.0
            wv.wave_type = "BANDS"
            wv.bands_direction = "X"
            nt.links.new(mp.outputs["Vector"], wv.inputs["Vector"])
            fac = wv.outputs["Fac"]
        if stripes:
            wv = nt.nodes.new("ShaderNodeTexWave")
            wv.inputs["Scale"].default_value = stripes
            wv.inputs["Distortion"].default_value = 0.5
            wv.wave_type = "BANDS"
            wv.bands_direction = "Y"
            nt.links.new(mp.outputs["Vector"], wv.inputs["Vector"])
            fac = wv.outputs["Fac"]
        if patch is not None:
            gt = nt.nodes.new("ShaderNodeMath")
            gt.operation = "GREATER_THAN"
            gt.inputs[1].default_value = patch
            nt.links.new(fac, gt.inputs[0])
            fac = gt.outputs[0]
        if col2 is not None:
            mix = nt.nodes.new("ShaderNodeMix")
            mix.data_type = "RGBA"
            mix.inputs[6].default_value = rgba
            mix.inputs[7].default_value = (col2[0], col2[1], col2[2], 1)
            nt.links.new(fac, mix.inputs[0])
            nt.links.new(mix.outputs[2], bsdf.inputs["Base Color"])
        if bump > 0:
            b = nt.nodes.new("ShaderNodeBump")
            b.inputs["Strength"].default_value = bump
            b.inputs["Distance"].default_value = 0.05
            nt.links.new(noise.outputs["Fac"], b.inputs["Height"])
            nt.links.new(b.outputs["Normal"], bsdf.inputs["Normal"])
    MATS[key] = m
    return m


# Common palette ----------------------------------------------------------
def c(h):
    """'#rrggbb' -> linear rgb tuple."""
    h = h.lstrip("#")
    v = [int(h[i:i + 2], 16) / 255 for i in (0, 2, 4)]
    return tuple(((x + 0.055) / 1.055) ** 2.4 if x > 0.04045 else x / 12.92 for x in v)


def rock_mat(base="#8a847a", dark="#4f4a44"):
    return mat("rock" + base, c(base), 0.85, col2=c(dark), scale=5, bump=0.5)


def wood_mat(base="#8a5a2c", dark="#5a3718"):
    return mat("wood" + base, c(base), 0.8, col2=c(dark), scale=4, bump=0.2, wave=True)


def grass_mat(base="#5d9a3a", dark="#3d7428"):
    return mat("grass" + base, c(base), 0.9, col2=c(dark), scale=10, bump=0.1)


def leaf_mat(base="#4f8f2e"):
    return mat("leaf" + base, c(base), 0.7, sss=0.2)


def cloth_mat(base="#d8c8a0"):
    return mat("cloth" + base, c(base), 0.95)


# --------------------------------------------------------------- builder


class Builder:
    """Collects primitives into one mesh per material, then links them."""

    def __init__(self, name="model"):
        self.name = name
        self.bm = {}
        self.rng = random.Random(name)

    def _b(self, m):
        if m not in self.bm:
            self.bm[m] = bmesh.new()
        return self.bm[m]

    # -- primitives --------------------------------------------------
    def _mat(self, pos, scale=(1, 1, 1), yaw=0.0, rot=None):
        m = Matrix.Translation(Vector(pos)) @ Matrix.Rotation(yaw, 4, "Z")
        if rot is not None:
            m = Matrix.Translation(Vector(pos)) @ rot
        s = Matrix.Diagonal((scale[0], scale[1], scale[2], 1.0))
        return m @ s

    def _smooth(self, r, smooth=True):
        for f in {f for v in r["verts"] for f in v.link_faces}:
            f.smooth = smooth

    def blob(self, m, pos, size, yaw=0.0, seg=14, smooth=True, tilt=0.0):
        """Ellipsoid with *half* sizes size=(sx, sy, sz)."""
        bm = self._b(m)
        rot = Matrix.Rotation(yaw, 4, "Z") @ Matrix.Rotation(tilt, 4, "Y")
        r = bmesh.ops.create_uvsphere(bm, u_segments=seg, v_segments=max(6, seg // 2), radius=1.0,
                                      matrix=self._mat(pos, size, rot=rot))
        self._smooth(r, smooth)
        return self

    def box(self, m, pos, size, yaw=0.0, rot=None, smooth=False):
        """Box with *full* sizes; pos is its centre."""
        bm = self._b(m)
        self._n = getattr(self, "_n", 0) + 1
        j = (self._n % 9) * 0.0007  # break coplanar overlaps (they render black)
        pos = (pos[0], pos[1], pos[2] + j)
        size = (size[0] + j, size[1] + j, size[2] + j)
        r = bmesh.ops.create_cube(bm, size=1.0, matrix=self._mat(pos, size, yaw, rot))
        self._smooth(r, False)
        return self

    def cyl(self, m, a, b, r0, r1=None, seg=10, cap=True, smooth=True):
        """Tapered cylinder / cone from point a (radius r0) to point b (radius r1)."""
        r1 = r0 if r1 is None else r1
        a, b = Vector(a), Vector(b)
        d = b - a
        L = d.length
        if L < 1e-6:
            return self
        bm = self._b(m)
        rot = Vector((0, 0, 1)).rotation_difference(d.normalized()).to_matrix().to_4x4()
        mtx = Matrix.Translation((a + b) / 2) @ rot
        r = bmesh.ops.create_cone(bm, cap_ends=cap, cap_tris=False, segments=seg, radius1=r0,
                                  radius2=r1, depth=L, matrix=mtx)
        self._smooth(r, smooth)
        return self

    def tube(self, m, pts, r0, r1=None, seg=8):
        """Chain of tapered cylinders along pts (a curved tusk, banana, tail...)."""
        r1 = r0 if r1 is None else r1
        n = len(pts) - 1
        for i in range(n):
            ra = r0 + (r1 - r0) * i / n
            rb = r0 + (r1 - r0) * (i + 1) / n
            self.cyl(m, pts[i], pts[i + 1], ra, rb, seg=seg)
            if i > 0:
                self.blob(m, pts[i], (ra, ra, ra), seg=seg)
        return self

    def cone(self, m, base, height, r, seg=10, tip_r=0.0):
        return self.cyl(m, base, (base[0], base[1], base[2] + height), r, tip_r, seg=seg)

    def prism(self, m, base, height, r, sides=6, yaw=0.0, tip=None):
        """Regular n-gon prism, optionally capped by a point of height `tip`."""
        bm = self._b(m)
        rot = Matrix.Rotation(yaw, 4, "Z")
        mtx = Matrix.Translation((base[0], base[1], base[2] + height / 2)) @ rot
        rr = bmesh.ops.create_cone(bm, cap_ends=True, segments=sides, radius1=r, radius2=r, depth=height, matrix=mtx)
        self._smooth(rr, False)
        if tip:
            self.cyl(m, (base[0], base[1], base[2] + height), (base[0], base[1], base[2] + height + tip),
                     r, 0.0, seg=sides)
        return self

    def rock(self, m, pos, size, seed=0, subdiv=2, flat_bottom=True, rough=0.28, yaw=None):
        """Lumpy low-poly boulder; size = half extents."""
        rng = random.Random(f"{self.name}{seed}")
        bm = self._b(m)
        tmp = bmesh.new()
        bmesh.ops.create_icosphere(tmp, subdivisions=subdiv, radius=1.0)
        for v in tmp.verts:
            k = 1.0 + (rng.random() - 0.5) * 2 * rough
            v.co *= k
            if flat_bottom and v.co.z < -0.35:
                v.co.z = -0.35
        yaw = rng.random() * 6.28 if yaw is None else yaw
        mtx = self._mat(pos, size, yaw)
        tmp.transform(mtx)
        for f in tmp.faces:
            f.smooth = False
        me = bpy.data.meshes.new("t")
        tmp.to_mesh(me)
        tmp.free()
        bm.from_mesh(me)
        bpy.data.meshes.remove(me)
        return self

    def crystal(self, m, pos, h, r, tilt=(0, 0), sides=6, yaw=0.0, tip=0.35):
        """A pointed hex crystal leaning by tilt=(x, y) radians."""
        bm = self._b(m)
        rot = Matrix.Rotation(yaw, 4, "Z") @ Matrix.Rotation(tilt[1], 4, "X") @ Matrix.Rotation(tilt[0], 4, "Y")
        base = Matrix.Translation(Vector(pos)) @ rot
        tmp = bmesh.new()
        body = h * (1 - tip)
        r1 = bmesh.ops.create_cone(tmp, cap_ends=True, segments=sides, radius1=r, radius2=r, depth=body,
                                   matrix=Matrix.Translation((0, 0, body / 2)))
        r2 = bmesh.ops.create_cone(tmp, cap_ends=False, segments=sides, radius1=r, radius2=0.0,
                                   depth=h * tip, matrix=Matrix.Translation((0, 0, body + h * tip / 2)))
        tmp.transform(base)
        for f in tmp.faces:
            f.smooth = False
        me = bpy.data.meshes.new("t")
        tmp.to_mesh(me)
        tmp.free()
        self._b(m).from_mesh(me)
        bpy.data.meshes.remove(me)
        return self

    def disc(self, m, pos, r, h=0.02, seg=24):
        return self.cyl(m, pos, (pos[0], pos[1], pos[2] + h), r, r, seg=seg)

    def ring(self, m, pos, r, w, h=0.02, seg=40):
        """Flat ring (for water ripples, fences' ground circle)."""
        bm = self._b(m)
        outer = r + w / 2
        inner = r - w / 2
        top = []
        for i in range(seg):
            a0 = 2 * math.pi * i / seg
            a1 = 2 * math.pi * (i + 1) / seg
            p = [
                (pos[0] + math.cos(a0) * inner, pos[1] + math.sin(a0) * inner, pos[2]),
                (pos[0] + math.cos(a0) * outer, pos[1] + math.sin(a0) * outer, pos[2]),
                (pos[0] + math.cos(a1) * outer, pos[1] + math.sin(a1) * outer, pos[2]),
                (pos[0] + math.cos(a1) * inner, pos[1] + math.sin(a1) * inner, pos[2]),
            ]
            vs = [bm.verts.new(q) for q in p]
            bm.faces.new(vs)
        return self

    def poly(self, m, pts, z=0.0, h=0.03):
        """Extruded convex polygon on the ground (pts are (x, y))."""
        bm = self._b(m)
        bot = [bm.verts.new((x, y, z)) for x, y in pts]
        top = [bm.verts.new((x, y, z + h)) for x, y in pts]
        bm.faces.new(bot[::-1])
        bm.faces.new(top)
        n = len(pts)
        for i in range(n):
            j = (i + 1) % n
            bm.faces.new([bot[i], bot[j], top[j], top[i]])
        return self

    def wall_run(self, m, pts, h, t, cap=None, cap_h=0.05):
        """A wall (box segments) along the polyline pts [(x,y)...]."""
        for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
            dx, dy = x1 - x0, y1 - y0
            L = math.hypot(dx, dy)
            ang = math.atan2(dy, dx)
            self.box(m, ((x0 + x1) / 2, (y0 + y1) / 2, h / 2), (L + t * 0.6, t, h), yaw=ang)
            if cap:
                self.box(cap, ((x0 + x1) / 2, (y0 + y1) / 2, h + cap_h / 2), (L + t * 0.6, t * 1.2, cap_h), yaw=ang)
        return self

    def tuft(self, m, pos, h=0.12, n=5, spread=0.04, r=0.008):
        """Grass blades."""
        for i in range(n):
            a = self.rng.random() * 6.28
            s = self.rng.random() * spread
            tx, ty = pos[0] + math.cos(a) * s, pos[1] + math.sin(a) * s
            lean = (self.rng.random() - 0.5) * 0.08
            self.cyl(m, (tx, ty, pos[2]), (tx + lean, ty + lean * 0.5, pos[2] + h * (0.7 + self.rng.random() * 0.5)),
                     r, 0.0, seg=3, smooth=False)
        return self

    def finish(self):
        objs = []
        for m, bm in self.bm.items():
            me = bpy.data.meshes.new(self.name)
            bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
            bm.normal_update()
            bm.to_mesh(me)
            bm.free()
            me.materials.append(m)
            o = bpy.data.objects.new(self.name, me)
            bpy.context.collection.objects.link(o)
            objs.append(o)
        self.bm = {}
        return objs


# ------------------------------------------------------------- compounds

def tree(b, pos, h=0.5, leaf="#4f8f2e", trunk="#6a4423", kind="round", s=1.0):
    x, y, z = pos
    tm = wood_mat(trunk)
    lm = leaf_mat(leaf)
    b.cyl(tm, (x, y, z), (x, y, z + h * 0.55), 0.03 * s, 0.022 * s, seg=6)
    if kind == "pine":
        for i in range(3):
            b.cone(lm, (x, y, z + h * (0.25 + 0.2 * i)), h * 0.42, 0.17 * s * (1 - i * 0.22), seg=7)
    else:
        b.blob(lm, (x, y, z + h * 0.78), (0.15 * s, 0.15 * s, 0.14 * s), seg=8)
        b.blob(lm, (x + 0.07 * s, y, z + h * 0.65), (0.11 * s, 0.1 * s, 0.1 * s), seg=8)
        b.blob(lm, (x - 0.07 * s, y + 0.03 * s, z + h * 0.66), (0.1 * s, 0.1 * s, 0.09 * s), seg=8)


def bush(b, pos, r=0.12, col="#4f8f2e", col2=None):
    lm = leaf_mat(col)
    x, y, z = pos
    b.blob(lm, (x, y, z + r * 0.7), (r, r, r * 0.8), seg=8)
    b.blob(lm, (x + r * 0.6, y + r * 0.2, z + r * 0.45), (r * 0.7, r * 0.7, r * 0.6), seg=8)
    b.blob(lm, (x - r * 0.5, y - r * 0.2, z + r * 0.4), (r * 0.7, r * 0.7, r * 0.55), seg=8)


def sheaf(b, pos, h=0.28, col="#d9b24a", tie="#8a5a2c", n=9):
    """A bundle of grain stalks tied in the middle."""
    x, y, z = pos
    gm = mat("grain" + col, c(col), 0.55, col2=c("#b88a30"), scale=30)
    tm = mat("tie" + tie, c(tie), 0.9)
    for i in range(n):
        a = 2 * math.pi * i / n
        lean = 0.07 + 0.02 * (i % 3)
        top = (x + math.cos(a) * lean * 1.6, y + math.sin(a) * lean * 1.6, z + h * (0.95 + 0.08 * (i % 2)))
        b.cyl(gm, (x + math.cos(a) * 0.012, y + math.sin(a) * 0.012, z), top, 0.011, 0.007, seg=4)
        b.blob(gm, (top[0], top[1], top[2] + 0.02), (0.016, 0.016, 0.05), seg=6)
    b.cyl(tm, (x, y, z + h * 0.38), (x, y, z + h * 0.46), 0.045, 0.045, seg=8)


def hex_pts(r=1.0, n=6, off=30.0):
    return [(r * math.cos(math.radians(off + 60 * i)), r * math.sin(math.radians(off + 60 * i))) for i in range(n)]


def water_rings(b, r=0.7, n=2):
    wm = mat("foam", (0.9, 0.97, 1.0), 0.3, alpha=0.55, emit=0.15)
    for i in range(n):
        b.ring(wm, (0, 0, 0.012 + 0.002 * i), r + 0.16 * i, 0.035, seg=44)


# ---------------------------------------------------------------- animals

def _P(x, y, yaw, s):
    ca, sa = math.cos(yaw), math.sin(yaw)

    def f(lx, ly, lz):
        return (x + (lx * ca - ly * sa) * s, y + (lx * sa + ly * ca) * s, lz * s)
    return f


def quad(b, x, y, yaw=0.0, s=1.0, kind="cow", body="#8a5a36", accent="#f0e8d8", dark="#2a1c14"):
    """Low-poly four-legged animal facing local +X, rotated by yaw."""
    P = _P(x, y, yaw, s)
    bm = mat("fur" + body, c(body), 0.85, col2=c(accent) if kind == "cow" else c(body), scale=5 if kind == "cow" else 20, bump=0.15, patch=0.56 if kind == "cow" else None)
    am = mat("fur" + accent, c(accent), 0.85)
    dm = mat("dk" + dark, c(dark), 0.8)
    sm = s
    if kind == "cow":
        b.blob(bm, P(0, 0, 0.19), (0.27 * sm, 0.13 * sm, 0.12 * sm), yaw)
        for lx in (-0.17, 0.17):
            for ly in (-0.07, 0.07):
                b.cyl(bm, P(lx, ly, 0.11), P(lx, ly, 0.0), 0.028 * sm, 0.022 * sm, seg=6)
        b.blob(bm, P(0.3, 0, 0.25), (0.1 * sm, 0.075 * sm, 0.075 * sm), yaw)
        b.blob(am, P(0.38, 0, 0.22), (0.05 * sm, 0.055 * sm, 0.04 * sm), yaw)
        for ly in (-0.07, 0.07):
            b.tube(am, [P(0.3, ly, 0.31), P(0.3, ly * 1.5, 0.35), P(0.3, ly * 1.9, 0.38)], 0.014 * sm, 0.004 * sm, seg=5)
            b.blob(dm, P(0.27, ly * 1.2, 0.29), (0.025 * sm, 0.014 * sm, 0.018 * sm), yaw)
        b.tube(bm, [P(-0.27, 0, 0.24), P(-0.32, 0, 0.14), P(-0.31, 0, 0.06)], 0.01 * sm, 0.008 * sm, seg=4)
    elif kind == "horse":
        b.blob(bm, P(0, 0, 0.3), (0.3 * sm, 0.12 * sm, 0.12 * sm), yaw)
        for lx in (-0.2, 0.2):
            for ly in (-0.06, 0.06):
                b.cyl(bm, P(lx, ly, 0.24), P(lx + 0.01, ly, 0.02), 0.03 * sm, 0.018 * sm, seg=6)
                b.cyl(dm, P(lx + 0.01, ly, 0.03), P(lx + 0.01, ly, 0.0), 0.022 * sm, 0.022 * sm, seg=6)
        b.tube(bm, [P(0.22, 0, 0.34), P(0.3, 0, 0.46), P(0.35, 0, 0.56)], 0.075 * sm, 0.05 * sm, seg=8)
        b.blob(bm, P(0.42, 0, 0.58), (0.1 * sm, 0.045 * sm, 0.05 * sm), yaw, tilt=0.5)
        b.blob(dm, P(0.5, 0, 0.55), (0.03 * sm, 0.035 * sm, 0.03 * sm), yaw)
        b.tube(dm, [P(0.2, 0, 0.5), P(0.27, 0, 0.58), P(0.32, 0, 0.63)], 0.03 * sm, 0.02 * sm, seg=5)
        for ly in (-0.025, 0.025):
            b.cone(bm, P(0.36, ly, 0.62), 0.07 * sm, 0.015 * sm, seg=4)
        b.tube(dm, [P(-0.3, 0, 0.34), P(-0.38, 0, 0.26), P(-0.4, 0, 0.12)], 0.035 * sm, 0.01 * sm, seg=5)
    elif kind == "sheep":
        wm = mat("wool", c("#f1ede2"), 1.0, col2=c("#d8d2c0"), scale=14, bump=0.4)
        for dx, dy, dz, r in [(0, 0, 0.17, 0.15), (0.1, 0.03, 0.18, 0.11), (-0.1, -0.02, 0.17, 0.12), (0, 0.06, 0.2, 0.1), (0.02, -0.07, 0.19, 0.1), (-0.06, 0.07, 0.17, 0.09)]:
            b.blob(wm, P(dx, dy, dz), (r * sm * 1.1, r * sm, r * sm * 0.95), yaw, seg=10)
        for lx in (-0.1, 0.1):
            for ly in (-0.06, 0.06):
                b.cyl(dm, P(lx, ly, 0.1), P(lx, ly, 0.0), 0.018 * sm, 0.014 * sm, seg=5)
        b.blob(dm, P(0.2, 0, 0.2), (0.065 * sm, 0.045 * sm, 0.055 * sm), yaw)
        for ly in (-0.05, 0.05):
            b.blob(dm, P(0.17, ly * 1.5, 0.22), (0.03 * sm, 0.02 * sm, 0.012 * sm), yaw)
    elif kind == "deer":
        b.blob(bm, P(0, 0, 0.26), (0.22 * sm, 0.09 * sm, 0.09 * sm), yaw)
        b.blob(am, P(0.0, 0, 0.21), (0.2 * sm, 0.07 * sm, 0.06 * sm), yaw)
        for lx in (-0.14, 0.14):
            for ly in (-0.05, 0.05):
                b.cyl(bm, P(lx, ly, 0.24), P(lx, ly, 0.0), 0.016 * sm, 0.01 * sm, seg=5)
        b.tube(bm, [P(0.16, 0, 0.3), P(0.23, 0, 0.42), P(0.27, 0, 0.5)], 0.05 * sm, 0.035 * sm, seg=7)
        b.blob(bm, P(0.31, 0, 0.51), (0.07 * sm, 0.04 * sm, 0.04 * sm), yaw, tilt=0.3)
        b.blob(dm, P(0.37, 0, 0.5), (0.015 * sm, 0.02 * sm, 0.015 * sm), yaw)
        antler = mat("antler", c("#d8c9a8"), 0.7)
        for ly in (-1, 1):
            b.tube(antler, [P(0.27, ly * 0.03, 0.55), P(0.25, ly * 0.07, 0.65), P(0.22, ly * 0.1, 0.74)], 0.012 * sm, 0.006 * sm, seg=4)
            b.tube(antler, [P(0.25, ly * 0.07, 0.64), P(0.31, ly * 0.09, 0.7)], 0.008 * sm, 0.004 * sm, seg=4)
            b.tube(antler, [P(0.24, ly * 0.08, 0.7), P(0.17, ly * 0.1, 0.76)], 0.008 * sm, 0.004 * sm, seg=4)
        b.cone(am, P(-0.22, 0, 0.3), 0.06 * sm, 0.03 * sm, seg=5)


def fish(b, x, y, z=0.04, yaw=0.0, s=1.0, col="#7fb4d8", belly="#e8f0f4"):
    P = _P(x, y, yaw, s)
    fm = mat("fish" + col, c(col), 0.25, metal=0.4, col2=c(belly), scale=12)
    b.blob(fm, P(0, 0, z), (0.15 * s, 0.045 * s, 0.06 * s), yaw, seg=12)
    b.blob(fm, P(-0.15, 0, z), (0.05 * s, 0.012 * s, 0.07 * s), yaw, seg=8, tilt=0)
    b.blob(fm, P(-0.2, 0, z + 0.01), (0.04 * s, 0.01 * s, 0.09 * s), yaw, seg=8)
    b.blob(fm, P(0.0, 0, z + 0.07 * s), (0.06 * s, 0.01 * s, 0.045 * s), yaw, seg=8)
    em = mat("eye", c("#101010"), 0.2)
    b.blob(em, P(0.1, 0.04, z + 0.015), (0.012 * s, 0.01 * s, 0.012 * s), yaw, seg=6)


# ---------------------------------------------------------------- props

def sack(b, pos, s=1.0, col="#c9b078", content=None):
    x, y, z = pos
    m = mat("burlap" + col, c(col), 1.0, col2=c("#a89058"), scale=40, bump=0.3)
    b.blob(m, (x, y, z + 0.1 * s), (0.11 * s, 0.1 * s, 0.11 * s), seg=10)
    b.cyl(m, (x, y, z + 0.17 * s), (x, y, z + 0.23 * s), 0.07 * s, 0.045 * s, seg=8)
    if content:
        cm = mat("sackc" + content, c(content), 0.9)
        b.cyl(cm, (x, y, z + 0.2 * s), (x, y, z + 0.26 * s), 0.065 * s, 0.05 * s, seg=8)
        b.blob(cm, (x, y, z + 0.26 * s), (0.065 * s, 0.065 * s, 0.035 * s), seg=8)


def heap(b, pos, r, h, col, rough=0.9, seg=10):
    m = mat("heap" + col, c(col), rough, col2=c(col), scale=30, bump=0.3)
    b.cyl(m, (pos[0], pos[1], pos[2]), (pos[0], pos[1], pos[2] + h), r, 0.0, seg=seg)
    b.blob(m, (pos[0], pos[1], pos[2] + 0.01), (r, r, h * 0.12), seg=seg)


def barrel(b, pos, s=1.0, col="#8a5a2c"):
    x, y, z = pos
    wm = wood_mat(col)
    hm = mat("hoop", c("#3a3028"), 0.5, metal=0.7)
    b.blob(wm, (x, y, z + 0.13 * s), (0.1 * s, 0.1 * s, 0.13 * s), seg=12)
    for dz in (0.05, 0.21):
        b.ring(hm, (x, y, z + dz * s), 0.095 * s, 0.015 * s, seg=14)
    b.cyl(hm, (x, y, z + 0.255 * s), (x, y, z + 0.262 * s), 0.07 * s, 0.07 * s, seg=12)


def crate(b, pos, s=1.0, yaw=0.0, col="#9a6a38"):
    x, y, z = pos
    wm = wood_mat(col)
    b.box(wm, (x, y, z + 0.08 * s), (0.2 * s, 0.2 * s, 0.16 * s), yaw=yaw)


def pot(b, pos, s=1.0, col="#b0583a", fill=None):
    x, y, z = pos
    m = mat("clay" + col, c(col), 0.7, col2=c(col), scale=8, bump=0.1)
    b.blob(m, (x, y, z + 0.07 * s), (0.08 * s, 0.08 * s, 0.07 * s), seg=12)
    b.cyl(m, (x, y, z + 0.11 * s), (x, y, z + 0.16 * s), 0.05 * s, 0.065 * s, seg=12)
    if fill:
        fm = mat("potf" + fill, c(fill), 0.9)
        b.cyl(fm, (x, y, z + 0.15 * s), (x, y, z + 0.17 * s), 0.06 * s, 0.06 * s, seg=12)


def smoke(b, pos, n=4, s=1.0, col="#e8e8e8", alpha=0.6):
    m = mat("smoke", c(col), 1.0, alpha=alpha)
    for i in range(n):
        b.blob(m, (pos[0] + 0.03 * i * s, pos[1], pos[2] + 0.1 * i * s), ((0.05 + 0.02 * i) * s,) * 3, seg=8)


# ------------------------------------------------------------- buildings

def gable(b, m, x, y, z, w, d, rh, yaw=0.0, over=0.04):
    """Triangular-prism roof; ridge along local X. (x,y,z) is the eave centre."""
    bm = b._b(m)
    hw, hd = w / 2 + over, d / 2 + over
    ca, sa = math.cos(yaw), math.sin(yaw)

    def P(lx, ly, lz):
        return (x + lx * ca - ly * sa, y + lx * sa + ly * ca, z + lz)
    v = [bm.verts.new(P(-hw, -hd, 0)), bm.verts.new(P(hw, -hd, 0)), bm.verts.new(P(hw, hd, 0)), bm.verts.new(P(-hw, hd, 0)),
         bm.verts.new(P(-hw, 0, rh)), bm.verts.new(P(hw, 0, rh))]
    bm.faces.new([v[0], v[1], v[5], v[4]])
    bm.faces.new([v[3], v[4], v[5], v[2]])
    bm.faces.new([v[0], v[4], v[3]])
    bm.faces.new([v[1], v[2], v[5]])
    bm.faces.new([v[0], v[3], v[2], v[1]])


def house(b, x, y, w=0.3, d=0.22, h=0.14, wall=None, roof=None, yaw=0.0, rh=None, z=0.0, door=True):
    wall = wall or mat("plaster", c("#e8dcc0"), 0.9, col2=c("#c8b896"), scale=12, bump=0.2)
    roof = roof or mat("tiles", c("#b8503a"), 0.7, col2=c("#8a3a28"), scale=20, bump=0.3)
    b.box(wall, (x, y, z + h / 2), (w, d, h), yaw=yaw)
    gable(b, roof, x, y, z + h, w, d, rh if rh is not None else h * 0.6, yaw)
    if door:
        dm = mat("door", c("#3a2616"), 0.8)
        ca, sa = math.cos(yaw), math.sin(yaw)
        ox, oy = 0.0, -d / 2 - 0.002
        b.box(dm, (x + ox * ca - oy * sa, y + ox * sa + oy * ca, z + h * 0.32), (w * 0.2, 0.01, h * 0.62), yaw=yaw)


def tent(b, x, y, r=0.2, h=0.3, col="#d8c8a0", yaw=0.0, seg=8, door_col="#2a1c10", poles=True):
    m = mat("tent" + col, c(col), 0.95, col2=c(col), scale=20, bump=0.2)
    dm = mat("tentdoor" + door_col, c(door_col), 0.9)
    b.cyl(m, (x, y, 0), (x, y, h), r, 0.015, seg=seg, smooth=False)
    bm = b._b(dm)
    v = [bm.verts.new((x - r * 0.2, y - r * 1.0 - 0.004, 0.0)), bm.verts.new((x + r * 0.2, y - r * 1.0 - 0.004, 0.0)),
         bm.verts.new((x, y - r * 0.42 - 0.004, h * 0.58))]
    bm.faces.new(v)
    if poles:
        wd = wood_mat()
        b.cyl(wd, (x - 0.02, y, h * 0.9), (x - 0.05, y, h * 1.18), 0.009, 0.009, seg=4)
        b.cyl(wd, (x + 0.02, y, h * 0.9), (x + 0.05, y, h * 1.18), 0.009, 0.009, seg=4)


def fence_line(b, p0, p1, h=0.12, n=None, wd=None, rails=2, post_r=0.012):
    wd = wd or wood_mat("#8a6a3a", "#6a4a24")
    L = math.hypot(p1[0] - p0[0], p1[1] - p0[1])
    n = n or max(2, int(L / 0.12))
    pts = [(p0[0] + (p1[0] - p0[0]) * i / n, p0[1] + (p1[1] - p0[1]) * i / n) for i in range(n + 1)]
    for (x, y) in pts:
        b.cyl(wd, (x, y, 0), (x, y, h), post_r, post_r, seg=5, smooth=False)
    for r in range(rails):
        z = h * (0.45 + 0.4 * r / max(1, rails - 1)) if rails > 1 else h * 0.7
        b.cyl(wd, (p0[0], p0[1], z), (p1[0], p1[1], z), 0.008, 0.008, seg=4, smooth=False)


def palisade(b, p0, p1, h=0.2, n=None, wd=None):
    wd = wd or wood_mat("#7a5a30", "#5a3c1c")
    L = math.hypot(p1[0] - p0[0], p1[1] - p0[1])
    n = n or max(3, int(L / 0.055))
    for i in range(n + 1):
        x = p0[0] + (p1[0] - p0[0]) * i / n
        y = p0[1] + (p1[1] - p0[1]) * i / n
        hh = h * (0.9 + 0.2 * ((i * 7) % 3) / 2)
        b.cyl(wd, (x, y, 0), (x, y, hh), 0.026, 0.012, seg=5, smooth=False)


def flag(b, pos, h=0.3, col="#c02a2a", w=0.14, ph=0.1, yaw=0.0):
    x, y, z = pos
    pm = wood_mat("#5a3c1c")
    fm = mat("flag" + col, c(col), 0.7)
    b.cyl(pm, (x, y, z), (x, y, z + h), 0.008, 0.008, seg=4)
    ca, sa = math.cos(yaw), math.sin(yaw)
    b.box(fm, (x + ca * w / 2, y + sa * w / 2, z + h - ph / 2 - 0.01), (w, 0.006, ph), yaw=yaw)


def battlements(b, m, x, y, w, d, z, n=5, ch=0.04, cw=0.04):
    for i in range(n):
        t = i / (n - 1) - 0.5
        for (px, py) in [(x + t * (w - cw), y - d / 2 + cw / 2), (x + t * (w - cw), y + d / 2 - cw / 2)]:
            b.box(m, (px, py, z + ch / 2), (cw, cw, ch))
    for i in range(1, n - 1):
        t = i / (n - 1) - 0.5
        for (px, py) in [(x - w / 2 + cw / 2, y + t * (d - cw)), (x + w / 2 - cw / 2, y + t * (d - cw))]:
            b.box(m, (px, py, z + ch / 2), (cw, cw, ch))


def tower(b, x, y, w=0.2, h=0.4, wall=None, roof=None, round_=False):
    wall = wall or mat("castle", c("#b8b0a0"), 0.9, col2=c("#8a8274"), scale=10, bump=0.4)
    if round_:
        b.cyl(wall, (x, y, 0), (x, y, h), w / 2, w / 2 * 0.95, seg=12)
    else:
        b.box(wall, (x, y, h / 2), (w, w, h))
    if roof:
        b.cone(roof, (x, y, h), w * 0.9, w * 0.62, seg=8 if round_ else 4)
    else:
        battlements(b, wall, x, y, w, w, h, 4, 0.045, 0.045)


def stepped(b, m, x, y, z, w, tiers=4, th=0.1, shrink=0.78, yaw=0.0, m2=None):
    for i in range(tiers):
        mm = m2 if (m2 and i % 2) else m
        b.box(mm, (x, y, z + th * (i + 0.5)), (w * shrink ** i, w * shrink ** i, th), yaw=yaw)


def ground_patch(b, col="#8a6a44", col2="#6a4e30", r=0.5, z=0.0, seed=1, ry=0.78):
    """A ragged dirt patch under a structure."""
    gm = mat("gpatch" + col, c(col), 1.0, col2=c(col2), scale=8, bump=0.2)
    r *= 0.82
    rng = random.Random(seed)
    b.blob(gm, (0, 0, z - 0.01), (r, r * ry, 0.035), seg=20)
    for i in range(6):
        a = i * 1.05 + rng.random()
        b.blob(gm, (math.cos(a) * r * 0.75, math.sin(a) * r * 0.62, z - 0.012), (r * 0.35, r * 0.3, 0.03), seg=10)


def hut_round(b, x, y, r=0.12, h=0.1, wall=None, roof=None):
    wall = wall or mat("mudwall", c("#c8a878"), 1.0, col2=c("#a88858"), scale=14, bump=0.3)
    roof = roof or mat("thatch", c("#c8a850"), 1.0, col2=c("#9a7c34"), scale=40, bump=0.6)
    b.cyl(wall, (x, y, 0), (x, y, h), r, r, seg=12)
    b.cyl(roof, (x, y, h * 0.95), (x, y, h + r * 1.05), r * 1.25, 0.0, seg=12)
    dm = mat("door", c("#3a2616"), 0.8)
    b.box(dm, (x, y - r + 0.003, h * 0.42), (r * 0.4, 0.01, h * 0.78))


def tri(b, m, p0, p1, p2, double=True):
    bm = b._b(m)
    bm.faces.new([bm.verts.new(p0), bm.verts.new(p1), bm.verts.new(p2)])
    if double:
        bm.faces.new([bm.verts.new(p2), bm.verts.new(p1), bm.verts.new(p0)])
