"""Blender side of the terrain sprite renderer (see `texlib.render_many`).

    blender -b --factory-startup -P blrender.py -- <dir> <count> [samples]

Each job `<dir>/job_<i>.npz` holds a heightfield (z-units), an albedo
(sRGB) and a footprint alpha over the sprite canvas; the result is written
to `<dir>/out_<i>.npy` as an unpremultiplied RGBA float image.  The camera
and the foreshortening mirror the numpy `render_height`, so the game's
sprite placement stays valid; Cycles adds real sun, sky light and cast
shadows (onto a shadow catcher, so they fall on the tile's ground).
"""
import math
import sys

import bpy
import numpy as np
from mathutils import Vector

ELEV = 38.0          # camera elevation, degrees
EXT = 1.3            # canvas covers [-EXT, EXT] z-units
CANVAS = 512


def lin(a):
    a = np.clip(a, 0, 1)
    return np.where(a <= 0.04045, a / 12.92, ((a + 0.055) / 1.055) ** 2.4)


def clear():
    for ob in list(bpy.data.objects):
        bpy.data.objects.remove(ob, do_unlink=True)
    for coll in (bpy.data.meshes, bpy.data.materials, bpy.data.lights,
                 bpy.data.cameras, bpy.data.worlds):
        for d in list(coll):
            coll.remove(d)


def make_mesh(H, rgb, alpha, ky, kh, step):
    n = H.shape[0] // step
    H = H[::step, ::step][:n, :n]
    rgb = rgb[::step, ::step][:n, :n]
    alpha = alpha[::step, ::step][:n, :n]
    xs = np.linspace(-EXT, EXT, n, dtype=np.float32)
    gx, gy = np.meshgrid(xs, xs)
    # y up the screen is north; the canvas grid grows toward the viewer
    s, c = math.sin(math.radians(ELEV)), math.cos(math.radians(ELEV))
    co = np.stack([gx, -gy * ky / s, H * kh / c], -1).reshape(-1, 3).astype(np.float32)
    mesh = bpy.data.meshes.new("m")
    mesh.vertices.add(n * n)
    mesh.vertices.foreach_set("co", co.ravel())
    idx = np.arange(n * n, dtype=np.int32).reshape(n, n)
    quads = np.stack([idx[:-1, :-1], idx[:-1, 1:], idx[1:, 1:], idx[1:, :-1]], -1).reshape(-1, 4)
    # grid rows run south, so flip winding to face up
    quads = quads[:, ::-1]
    nq = len(quads)
    mesh.loops.add(nq * 4)
    mesh.polygons.add(nq)
    mesh.loops.foreach_set("vertex_index", quads.ravel())
    mesh.polygons.foreach_set("loop_start", np.arange(nq, dtype=np.int32) * 4)
    mesh.polygons.foreach_set("loop_total", np.full(nq, 4, np.int32))
    mesh.polygons.foreach_set("use_smooth", np.ones(nq, np.bool_))
    mesh.update()
    mesh.validate()
    col = mesh.attributes.new("col", 'FLOAT_COLOR', 'POINT')
    col.data.foreach_set("color", np.concatenate(
        [lin(rgb).reshape(-1, 3), np.ones((n * n, 1))], 1).astype(np.float32).ravel())
    al = mesh.attributes.new("alpha", 'FLOAT', 'POINT')
    al.data.foreach_set("value", alpha.reshape(-1).astype(np.float32))
    return mesh


def make_material(rough, spec):
    mat = bpy.data.materials.new("terrain")
    mat.use_nodes = True
    nt = mat.node_tree
    for nd in list(nt.nodes):
        nt.nodes.remove(nd)
    out = nt.nodes.new('ShaderNodeOutputMaterial')
    p = nt.nodes.new('ShaderNodeBsdfPrincipled')
    ac = nt.nodes.new('ShaderNodeAttribute')
    ac.attribute_name = "col"
    aa = nt.nodes.new('ShaderNodeAttribute')
    aa.attribute_name = "alpha"
    nt.links.new(ac.outputs['Color'], p.inputs['Base Color'])
    nt.links.new(aa.outputs['Fac'], p.inputs['Alpha'])
    p.inputs['Roughness'].default_value = rough
    if 'Specular IOR Level' in p.inputs:
        p.inputs['Specular IOR Level'].default_value = spec
    nt.links.new(p.outputs[0], out.inputs[0])
    return mat


def setup(sc, samples):
    sc.render.engine = 'CYCLES'
    cy = sc.cycles
    cy.device = 'CPU'
    cy.samples = samples
    cy.use_denoising = True
    cy.use_adaptive_sampling = True
    cy.max_bounces = 4
    cy.diffuse_bounces = 3
    cy.glossy_bounces = 1
    cy.transparent_max_bounces = 6
    cy.filter_width = 1.2
    sc.render.film_transparent = True
    sc.render.resolution_x = CANVAS
    sc.render.resolution_y = CANVAS
    sc.render.resolution_percentage = 100
    sc.render.image_settings.file_format = 'PNG'
    sc.render.image_settings.color_mode = 'RGBA'
    sc.render.image_settings.color_depth = '8'
    sc.view_settings.view_transform = 'Standard'
    sc.view_settings.look = 'None'
    sc.display_settings.display_device = 'sRGB'
    sc.render.threads_mode = 'AUTO'


def build(job, samples, tmp, i):
    clear()
    sc = bpy.context.scene
    setup(sc, samples)
    ky, kh = float(job['ky']), float(job['kh'])
    mesh = make_mesh(job['H'], job['rgb'], job['alpha'], ky, kh, int(job['step']))
    ob = bpy.data.objects.new("terrain", mesh)
    sc.collection.objects.link(ob)
    mesh.materials.append(make_material(float(job['rough']), float(job['spec'])))

    # a ground plane that only catches the cast shadows
    plane = bpy.data.meshes.new("plane")
    plane.from_pydata([(-6, -6, -0.002), (6, -6, -0.002), (6, 6, -0.002), (-6, 6, -0.002)],
                      [], [(0, 1, 2, 3)])
    po = bpy.data.objects.new("catcher", plane)
    po.is_shadow_catcher = True
    sc.collection.objects.link(po)

    cam = bpy.data.cameras.new("cam")
    cam.type = 'ORTHO'
    cam.ortho_scale = 2 * EXT
    cam.clip_start = 0.1
    cam.clip_end = 200
    cam.shift_y = (float(job['cy']) - CANVAS / 2) / CANVAS
    co = bpy.data.objects.new("cam", cam)
    sc.collection.objects.link(co)
    e = math.radians(ELEV)
    co.location = Vector((0, -math.cos(e) * 30, math.sin(e) * 30))
    co.rotation_euler = (math.radians(90 - ELEV), 0, 0)
    sc.camera = co

    w = bpy.data.worlds.new("w")
    sc.world = w
    w.use_nodes = True
    bg = w.node_tree.nodes.get('Background')
    bg.inputs['Color'].default_value = (0.62, 0.72, 0.9, 1)
    bg.inputs['Strength'].default_value = float(job['sky'])
    sun = bpy.data.lights.new("sun", 'SUN')
    sun.energy = float(job['sun'])
    sun.angle = math.radians(5)
    sun.color = (1.0, 0.94, 0.84)
    so = bpy.data.objects.new("sun", sun)
    sc.collection.objects.link(so)
    # light from the upper left (north-west), fairly high
    so.rotation_euler = Vector((0.62, -0.5, -0.78)).normalized().to_track_quat('-Z', 'Y').to_euler()
    fill = bpy.data.lights.new("fill", 'SUN')
    fill.energy = 0.6
    fill.angle = math.radians(40)
    fill.color = (0.75, 0.82, 1.0)
    fo = bpy.data.objects.new("fill", fill)
    sc.collection.objects.link(fo)
    fo.rotation_euler = Vector((-0.2, 0.9, -0.55)).normalized().to_track_quat('-Z', 'Y').to_euler()

    path = f"{tmp}/render_{i}.png"
    sc.render.filepath = path
    bpy.ops.render.render(write_still=True)
    img = bpy.data.images.load(path)
    img.colorspace_settings.name = 'Non-Color'
    img.alpha_mode = 'STRAIGHT'
    wd, ht = img.size
    px = np.empty(wd * ht * 4, dtype=np.float32)
    img.pixels.foreach_get(px)
    bpy.data.images.remove(img)
    return px.reshape(ht, wd, 4)[::-1]


def main():
    argv = sys.argv[sys.argv.index('--') + 1:]
    tmp, count = argv[0], int(argv[1])
    samples = int(argv[2]) if len(argv) > 2 else 48
    for i in range(count):
        job = np.load(f"{tmp}/job_{i}.npz")
        out = build(job, samples, tmp, i)
        np.save(f"{tmp}/out_{i}.npy", out)
        print("rendered", i, flush=True)


main()
