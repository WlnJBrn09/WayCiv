"""Procedural materials for the building sprites.

Every material is a Principled BSDF with a style-specific pattern (brick,
masonry, roof tiles, plaster, wood, foliage ...), a little colour mottling and a
render-time bevel that catches the sun on edges.  Textures are driven by the
object-space position, and walls are projected "along the wall" (a cheap
triplanar), so one merged mesh per material still gets a sensible pattern.

`M(name)` is a building material, `G(kind)` a ground-patch material whose alpha
fades out towards the plot edge so the sprite melts into the terrain below.
"""
import bpy

# name -> (hex colour, roughness[, metallic])
PAL = {
    'stone': ('#d2c2a0', 0.85), 'stone2': ('#b0a283', 0.9), 'stoneD': ('#837762', 0.9),
    'marble': ('#f1ebdc', 0.42), 'sand': ('#e6cf96', 0.95), 'sandD': ('#c9aa6e', 0.95),
    'brick': ('#b8613f', 0.9), 'brickD': ('#8f4a33', 0.9), 'adobe': ('#dba96d', 0.95),
    'terra': ('#c8502e', 0.75), 'terraD': ('#9e3a22', 0.75),
    'wood': ('#7e5532', 0.8), 'woodL': ('#ad7f50', 0.8), 'woodD': ('#4f3622', 0.85),
    'thatch': ('#cfae5a', 0.95), 'slate': ('#566176', 0.65), 'slateL': ('#8190a6', 0.6),
    'grass': ('#74a842', 1.0), 'grassD': ('#52862d', 1.0), 'dirt': ('#92704a', 1.0),
    'leaf': ('#45912f', 0.9), 'leafD': ('#2f6e23', 0.9), 'leafL': ('#82b947', 0.9),
    'pave': ('#bcb199', 0.9), 'paveD': ('#9c917c', 0.9), 'paveL': ('#dbd1bc', 0.9),
    'gold': ('#eab93c', 0.28, 0.95), 'goldD': ('#b8861e', 0.38, 0.9),
    'bronze': ('#a8703a', 0.4, 0.8), 'copper': ('#c26a45', 0.45, 0.8), 'verdi': ('#4fa592', 0.5, 0.5),
    'iron': ('#555a62', 0.5, 0.7), 'steel': ('#9aa4b2', 0.35, 0.8), 'dark': ('#2b2d33', 0.7),
    'white': ('#f6f3ec', 0.7), 'red': ('#c0392b', 0.7), 'blue': ('#2f6fb8', 0.7),
    'royal': ('#274a9a', 0.6), 'purple': ('#7a3f9a', 0.7), 'green': ('#2f8a4a', 0.7),
    'water': ('#3f90c6', 0.12), 'waterD': ('#2a6a9a', 0.12), 'foam': ('#d8eef2', 0.6),
    'glass': ('#8ec8e8', 0.08), 'glassD': ('#34627f', 0.08),
    'concrete': ('#bdbdb8', 0.9), 'concreteD': ('#8f8f8b', 0.9), 'asphalt': ('#3a3b40', 0.95),
    'yellow': ('#e8c030', 0.7), 'orange': ('#e07a28', 0.7), 'jade': ('#5aa88a', 0.5),
    'cream': ('#f0e4c2', 0.8), 'rose': ('#d98a8a', 0.8), 'turq': ('#3aa6b0', 0.6),
    'acc': ('#ffffff', 0.6), 'accD': ('#ffffff', 0.6),
    'earth': ('#9a7a52', 1.0), 'snow': ('#f0f4f8', 0.9), 'rock': ('#8c8679', 0.95),
}

# material name -> pattern style
STYLE = {
    'stone': 'blocks', 'stone2': 'blocks', 'stoneD': 'blocks', 'rock': 'rock', 'marble': 'marble',
    'sand': 'plaster', 'sandD': 'plaster', 'adobe': 'plaster', 'cream': 'plaster', 'white': 'plaster',
    'rose': 'plaster', 'brick': 'brick', 'brickD': 'brick',
    'terra': 'tiles', 'terraD': 'tiles', 'slate': 'tiles', 'slateL': 'tiles', 'verdi': 'tiles',
    'wood': 'wood', 'woodL': 'wood', 'woodD': 'wood', 'thatch': 'thatch',
    'grass': 'grass', 'grassD': 'grass', 'leaf': 'foliage', 'leafD': 'foliage', 'leafL': 'foliage',
    'dirt': 'dirt', 'earth': 'dirt', 'water': 'water', 'waterD': 'water',
    'pave': 'flags', 'paveD': 'flags', 'paveL': 'flags', 'concrete': 'concrete', 'concreteD': 'concrete',
    'gold': 'metal', 'goldD': 'metal', 'bronze': 'metal', 'copper': 'metal', 'iron': 'metal', 'steel': 'metal',
    'snow': 'snow',
}

_mats = {}
MASK_MODE = False


def lin(c):
    """sRGB 0..1 -> linear."""
    return tuple((x / 12.92 if x <= 0.04045 else ((x + 0.055) / 1.055) ** 2.4) for x in c)


def hexc(h):
    h = h.lstrip('#')
    return tuple(int(h[i:i + 2], 16) / 255.0 for i in (0, 2, 4))


def reset():
    _mats.clear()


# --------------------------------------------------------------------------
# node helpers
# --------------------------------------------------------------------------

class Tree:
    def __init__(self, mat):
        mat.use_nodes = True
        self.nt = mat.node_tree
        for n in list(self.nt.nodes):
            self.nt.nodes.remove(n)
        tc = self.nt.nodes.new('ShaderNodeTexCoord')
        self.obj = tc.outputs['Object']
        self.nrm = tc.outputs['Normal']
        self._wall = None

    def node(self, idname, props=None, **inputs):
        n = self.nt.nodes.new(idname)
        for k, v in (props or {}).items():
            setattr(n, k, v)
        for k, v in inputs.items():
            self.put(n.inputs[k.replace('_', ' ')], v)
        return n

    def put(self, sock, v):
        if isinstance(v, bpy.types.NodeSocket):
            self.nt.links.new(v, sock)
        elif v is not None:
            sock.default_value = v

    def math(self, op, a, b=None, c=None, clamp=False):
        n = self.nt.nodes.new('ShaderNodeMath')
        n.operation = op
        n.use_clamp = clamp
        self.put(n.inputs[0], a)
        if b is not None:
            self.put(n.inputs[1], b)
        if c is not None:
            self.put(n.inputs[2], c)
        return n.outputs[0]

    def sep(self, vec):
        n = self.nt.nodes.new('ShaderNodeSeparateXYZ')
        self.nt.links.new(vec, n.inputs[0])
        return n.outputs

    def comb(self, x, y, z):
        n = self.nt.nodes.new('ShaderNodeCombineXYZ')
        self.put(n.inputs[0], x)
        self.put(n.inputs[1], y)
        self.put(n.inputs[2], z)
        return n.outputs[0]

    def scale(self, vec, s):
        n = self.nt.nodes.new('ShaderNodeVectorMath')
        n.operation = 'SCALE'
        self.nt.links.new(vec, n.inputs[0])
        n.inputs['Scale'].default_value = s
        return n.outputs[0]

    def mixc(self, fac, a, b, blend='MIX'):
        n = self.nt.nodes.new('ShaderNodeMix')
        n.data_type = 'RGBA'
        n.blend_type = blend
        self.put(n.inputs[0], fac)
        self.put(n.inputs[6], a)
        self.put(n.inputs[7], b)
        return n.outputs[2]

    def remap(self, v, a0, a1, b0, b1, smooth=False, clamp=True):
        n = self.nt.nodes.new('ShaderNodeMapRange')
        n.clamp = clamp
        n.interpolation_type = 'SMOOTHSTEP' if smooth else 'LINEAR'
        self.put(n.inputs[0], v)
        n.inputs[1].default_value = a0
        n.inputs[2].default_value = a1
        n.inputs[3].default_value = b0
        n.inputs[4].default_value = b1
        return n.outputs[0]

    def noise(self, scale, detail=3.0, rough=0.55, vec=None, distortion=0.0):
        n = self.nt.nodes.new('ShaderNodeTexNoise')
        self.nt.links.new(vec if vec is not None else self.obj, n.inputs['Vector'])
        n.inputs['Scale'].default_value = scale
        n.inputs['Detail'].default_value = detail
        n.inputs['Roughness'].default_value = rough
        n.inputs['Distortion'].default_value = distortion
        return n.outputs['Factor']

    def ramp(self, fac, stops):
        n = self.nt.nodes.new('ShaderNodeValToRGB')
        el = n.color_ramp.elements
        while len(el) < len(stops):
            el.new(0.5)
        for e, (p, c) in zip(el, stops):
            e.position = p
            e.color = (*lin(c), 1.0)
        self.nt.links.new(fac, n.inputs[0])
        return n.outputs[0]

    def wall_vec(self):
        """(distance along the wall, height, 0): a cheap triplanar for vertical faces."""
        if self._wall is None:
            x, y, z = self.sep(self.obj)
            nx, ny, _nz = self.sep(self.nrm)
            u = self.math('ADD', self.math('MULTIPLY', x, self.math('ABSOLUTE', ny)),
                          self.math('MULTIPLY', y, self.math('ABSOLUTE', nx)))
            self._wall = self.comb(u, z, 0.0)
        return self._wall

    def brick(self, vec, c1, c2, mortar, scale, width, height, msize=0.03, offset=0.5, bias=0.0):
        n = self.nt.nodes.new('ShaderNodeTexBrick')
        n.offset = offset
        n.offset_frequency = 2
        n.squash = 1.0
        self.nt.links.new(vec, n.inputs['Vector'])
        n.inputs['Color1'].default_value = (*lin(c1), 1)
        n.inputs['Color2'].default_value = (*lin(c2), 1)
        n.inputs['Mortar'].default_value = (*lin(mortar), 1)
        n.inputs['Scale'].default_value = scale
        n.inputs['Mortar Size'].default_value = msize
        n.inputs['Mortar Smooth'].default_value = 0.1
        n.inputs['Bias'].default_value = bias
        n.inputs['Brick Width'].default_value = width
        n.inputs['Row Height'].default_value = height
        return n.outputs['Color'], n.outputs['Factor']


def _shade(c, f):
    return tuple(min(1.0, max(0.0, v * f)) for v in c)


# --------------------------------------------------------------------------
# building materials
# --------------------------------------------------------------------------

def _pattern(t, style, base):
    """(colour socket-or-tuple, bump height socket or None, bevel radius) for a style."""
    bl = lin(base)
    if style == 'brick':
        col, fac = t.brick(t.wall_vec(), _shade(base, 1.12), _shade(base, 0.82), _shade(base, 1.55), 9.0, 0.5, 0.2,
                           msize=0.05)
        col = t.mixc(t.remap(t.noise(7, 2), 0.3, 0.7, 0.0, 0.45), col, (*_shade(bl, 0.7), 1), 'MULTIPLY')
        return col, fac, 0.006
    if style == 'blocks':
        col, fac = t.brick(t.wall_vec(), _shade(base, 1.08), _shade(base, 0.9), _shade(base, 0.7), 4.4, 0.5, 0.35,
                           msize=0.035)
        mott = t.remap(t.noise(11, 3), 0.3, 0.7, 0.86, 1.1)
        return t.mixc(1.0, col, mott, 'MULTIPLY'), fac, 0.007
    if style == 'tiles':
        col, fac = t.brick(t.obj, _shade(base, 1.1), _shade(base, 0.86), _shade(base, 0.5), 7.5, 0.5, 0.25,
                           msize=0.06, offset=0.5)
        return col, fac, 0.004
    if style == 'flags':
        col, fac = t.brick(t.obj, _shade(base, 1.06), _shade(base, 0.92), _shade(base, 0.7), 5.0, 0.7, 0.45,
                           msize=0.04)
        return col, fac, 0.0
    if style == 'wood':
        w = t.node('ShaderNodeTexWave', {'wave_type': 'BANDS', 'bands_direction': 'X'}, Scale=26.0, Distortion=2.5,
                   Detail=2.0)
        t.nt.links.new(t.obj, w.inputs['Vector'])
        g = w.outputs['Factor']
        return t.mixc(t.remap(g, 0, 1, 0.0, 0.45), (*bl, 1), (*_shade(bl, 0.72), 1)), t.math('MULTIPLY', g, 0.5), 0.004
    if style == 'thatch':
        n = t.noise(46, 3, 0.6, vec=t.comb(t.sep(t.obj)[0], t.math('MULTIPLY', t.sep(t.obj)[1], 0.25), t.sep(t.obj)[2]))
        return t.mixc(t.remap(n, 0.3, 0.7, 0, 1), (*_shade(bl, 0.72), 1), (*_shade(bl, 1.12), 1)), n, 0.0
    if style in ('grass', 'foliage'):
        big = t.remap(t.noise(7 if style == 'grass' else 16, 3, 0.55), 0.3, 0.7, 0.0, 1.0)
        fine = t.noise(70, 2, 0.6)
        col = t.mixc(big, (*_shade(bl, 0.72), 1), (*_shade(bl, 1.18), 1))
        col = t.mixc(t.remap(fine, 0.35, 0.65, 0.0, 0.45), col, (*(bl[0] * 1.5, bl[1] * 1.3, bl[2] * 0.6), 1))
        return col, fine, 0.0
    if style == 'dirt':
        big = t.remap(t.noise(6, 4, 0.6), 0.3, 0.7, 0.0, 1.0)
        return t.mixc(big, (*_shade(bl, 0.75), 1), (*_shade(bl, 1.15), 1)), t.noise(60, 3), 0.0
    if style == 'marble':
        n = t.noise(5, 4, 0.7, distortion=2.0)
        return t.mixc(t.remap(n, 0.35, 0.65, 0.0, 0.35), (*bl, 1), (*_shade(bl, 0.86), 1)), None, 0.005
    if style == 'rock':
        big = t.remap(t.noise(8, 5, 0.65), 0.3, 0.7, 0.0, 1.0)
        return t.mixc(big, (*_shade(bl, 0.7), 1), (*_shade(bl, 1.2), 1)), t.noise(18, 5, 0.7), 0.0
    if style == 'plaster':
        big = t.remap(t.noise(9, 3), 0.3, 0.7, 0.0, 1.0)
        return t.mixc(big, (*_shade(bl, 0.88), 1), (*_shade(bl, 1.05), 1)), t.noise(80, 2), 0.006
    if style == 'concrete':
        big = t.remap(t.noise(5, 4), 0.3, 0.7, 0.0, 1.0)
        return t.mixc(big, (*_shade(bl, 0.88), 1), (*_shade(bl, 1.06), 1)), t.noise(60, 2), 0.005
    if style == 'water':
        big = t.remap(t.noise(3.5, 3, 0.5, distortion=1.0), 0.3, 0.7, 0.0, 1.0)
        return t.mixc(big, (*_shade(bl, 0.82), 1), (*_shade(bl, 1.15), 1)), t.noise(22, 2), 0.0
    if style == 'snow':
        return (*bl, 1), t.noise(30, 3), 0.0
    if style == 'metal':
        return (*bl, 1), t.noise(40, 2), 0.004
    big = t.remap(t.noise(12, 3), 0.3, 0.7, 0.0, 1.0)
    return t.mixc(big, (*_shade(bl, 0.9), 1), (*_shade(bl, 1.06), 1)), None, 0.005


def M(name, rgb=None, rough=None, metal=None, emit=None, emit_strength=6.0, style=None):
    """A material by palette name (or custom `rgb`); cached per scene."""
    key = (name, rgb, rough, metal, emit, style, MASK_MODE)
    if key in _mats:
        return _mats[key]
    spec = PAL.get(name, ('#c0c0c0', 0.8))
    base = hexc(spec[0]) if rgb is None else (hexc(rgb) if isinstance(rgb, str) else rgb)
    r = spec[1] if rough is None else rough
    m = spec[2] if (metal is None and len(spec) > 2) else (metal or 0.0)
    mat = bpy.data.materials.new(f"{name}{len(_mats)}")
    t = Tree(mat)
    nt = t.nt
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
        st = style or STYLE.get(name, 'plain')
        if name.startswith('acc'):
            st = 'plain'
        p = nt.nodes.new('ShaderNodeBsdfPrincipled')
        col, bump, bev = _pattern(t, st, base)
        t.put(p.inputs['Base Color'], col)
        p.inputs['Roughness'].default_value = r
        p.inputs['Metallic'].default_value = m
        if 'Specular IOR Level' in p.inputs:
            p.inputs['Specular IOR Level'].default_value = 0.3
        normal = None
        if bump is not None:
            normal = t.node('ShaderNodeBump', Strength=0.35 if st not in ('grass', 'foliage', 'thatch') else 0.6,
                            Distance=0.01, Height=bump).outputs[0]
        if bev > 0:
            b = t.node('ShaderNodeBevel', {'samples': 3}, Radius=bev)
            if normal is not None:
                nt.links.new(normal, b.inputs['Normal'])
            normal = b.outputs[0]
        if normal is not None:
            nt.links.new(normal, p.inputs['Normal'])
        nt.links.new(p.outputs[0], out.inputs[0])
    _mats[key] = mat
    return mat


# --------------------------------------------------------------------------
# ground patches
# --------------------------------------------------------------------------

# kind -> (ramp colours dark..light, roughness)
GROUND = {
    'cobble': (['#8a7e69', '#b4a88f', '#d3c8b1'], 0.9),
    'dirt': (['#7a5a38', '#9a7a52', '#b89868'], 1.0),
    'grass': (['#4a7e2a', '#6ea23d', '#8cbb52'], 1.0),
    'sand': (['#d3b97c', '#e5cf98', '#f0dfae'], 1.0),
    'slab': (['#8d8d89', '#b3b3ae', '#cfcfca'], 0.9),
    'marble': (['#cfc6b2', '#e6dfcd', '#f4efe2'], 0.6),
    'dark': (['#34353a', '#45464c', '#58595f'], 0.95),
    'water': (['#2f7fb4', '#4a9acb', '#78bbe0'], 0.06),
}


def G(kind='cobble', tone=1.0, r_in=0.60, r_out=0.82, wobble=0.09):
    """A ground-patch material: textured, and fading to transparent towards the plot edge."""
    key = ('ground', kind, tone, r_in, r_out, MASK_MODE)
    if key in _mats:
        return _mats[key]
    mat = bpy.data.materials.new(f"ground_{kind}")
    t = Tree(mat)
    nt = t.nt
    out = nt.nodes.new('ShaderNodeOutputMaterial')
    # fade mask from a noisy, corner-rounded hexagon distance
    x, y, _z = t.sep(t.obj)
    ax = t.math('ABSOLUTE', x)
    d1 = t.math('ABSOLUTE', t.math('ADD', t.math('MULTIPLY', x, 0.5), t.math('MULTIPLY', y, 0.866)))
    d2 = t.math('ABSOLUTE', t.math('SUBTRACT', t.math('MULTIPLY', x, 0.5), t.math('MULTIPLY', y, 0.866)))
    h = t.math('MAXIMUM', ax, t.math('MAXIMUM', d1, d2))
    circ = t.math('POWER', t.math('ADD', t.math('MULTIPLY', x, x), t.math('MULTIPLY', y, y)), 0.5)
    d = t.math('ADD', t.math('MULTIPLY', h, 0.72), t.math('MULTIPLY', circ, 0.28))
    warp = t.math('MULTIPLY', t.math('SUBTRACT', t.noise(4.0, 3, 0.6), 0.5), wobble * 2)
    alpha = t.remap(t.math('ADD', d, warp), r_in, r_out, 1.0, 0.0, smooth=True)
    if MASK_MODE:
        e = nt.nodes.new('ShaderNodeEmission')
        e.inputs['Color'].default_value = (0, 0, 0, 1)
        nt.links.new(e.outputs[0], out.inputs[0])
        _mats[key] = mat
        return mat
    stops, rough = GROUND[kind]
    stops = [tuple(min(1.0, c * tone) for c in hexc(s)) for s in stops]
    p = nt.nodes.new('ShaderNodeBsdfPrincipled')
    p.inputs['Roughness'].default_value = rough
    if 'Specular IOR Level' in p.inputs:
        p.inputs['Specular IOR Level'].default_value = 0.15
    bump = None
    if kind == 'cobble':
        cells = t.node('ShaderNodeTexVoronoi', {'feature': 'F1'}, Scale=15.0, Randomness=0.85)
        nt.links.new(t.obj, cells.inputs['Vector'])
        edge = t.node('ShaderNodeTexVoronoi', {'feature': 'DISTANCE_TO_EDGE'}, Scale=15.0, Randomness=0.85)
        nt.links.new(t.obj, edge.inputs['Vector'])
        sep = t.sep(cells.outputs['Color'])
        tone_f = t.ramp(sep[0], [(0.0, _shade(stops[1], 0.93)), (0.5, stops[1]), (1.0, stops[2])])
        big = t.remap(t.noise(5, 3), 0.3, 0.7, 0.85, 1.1)
        col = t.mixc(1.0, tone_f, big, 'MULTIPLY')
        gap = t.remap(edge.outputs['Distance'], 0.012, 0.045, 0.0, 1.0)
        col = t.mixc(gap, (*lin(_shade(stops[0], 1.1)), 1), col)
        bump = gap
        col = t.mixc(t.remap(t.noise(30, 2), 0.4, 0.7, 0, 0.18), col, (0.05, 0.07, 0.02, 1))
    elif kind in ('slab', 'marble'):
        col, fac = t.brick(t.obj, stops[1], stops[2], stops[0], 4.2, 0.7, 0.5, msize=0.03)
        col = t.mixc(t.remap(t.noise(6, 3), 0.3, 0.7, 0, 0.5), col, (*lin(stops[0]), 1))
        bump = fac
    else:
        f1 = t.remap(t.noise(5, 4, 0.6), 0.25, 0.75, 0.0, 1.0)
        col = t.ramp(f1, [(0.0, stops[0]), (0.5, stops[1]), (1.0, stops[2])])
        if kind == 'grass':
            tuft = t.remap(t.noise(34, 3, 0.65), 0.45, 0.8, 0.0, 0.6)
            col = t.mixc(tuft, col, (*lin(_shade(hexc('#9dc255'), tone)), 1))
        else:
            peb = t.node('ShaderNodeTexVoronoi', {'feature': 'F1'}, Scale=38.0, Randomness=1.0)
            nt.links.new(t.obj, peb.inputs['Vector'])
            dots = t.remap(peb.outputs['Distance'], 0.1, 0.22, 1.0, 0.0)
            col = t.mixc(t.math('MULTIPLY', dots, 0.35), col, (*lin(stops[2]), 1))
        bump = t.noise(55, 3)
    t.put(p.inputs['Base Color'], col)
    nrm = t.node('ShaderNodeBump', Strength=0.4, Distance=0.01, Height=bump).outputs[0]
    nt.links.new(nrm, p.inputs['Normal'])
    tr = nt.nodes.new('ShaderNodeBsdfTransparent')
    mix = nt.nodes.new('ShaderNodeMixShader')
    nt.links.new(alpha, mix.inputs[0])
    nt.links.new(tr.outputs[0], mix.inputs[1])
    nt.links.new(p.outputs[0], mix.inputs[2])
    nt.links.new(mix.outputs[0], out.inputs[0])
    _mats[key] = mat
    return mat
