"""Blender entry point: renders raw 256px transparent PNGs of models.

  blender -b --factory-startup -t 4 -P render.py -- OUT_DIR [name ...]

Names are model ids (resource_wheat, imp_farm, camp, ...); none = all.
Each model module exposes MODELS = {name: fn(b) -> opts dict | None}.
Options: top_down (bool), water (bool), res, samples, ortho.
"""
import importlib
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import bpy  # noqa: E402

import lib  # noqa: E402

args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
out = args[0]
only = set(args[1:])
os.makedirs(out, exist_ok=True)

models = {}
for modname in ("res_bonus", "res_lux", "res_strat", "improvements"):
    try:
        mod = importlib.import_module(modname)
    except ModuleNotFoundError as e:
        if e.name != modname:
            raise
        continue
    models.update(mod.MODELS)

def fit_scale(objs, opts):
    """Scale + centre the model so small props fill the frame like big ones."""
    if opts.get("fit") is False or not objs:
        return
    from mathutils import Vector
    mn = Vector((1e9, 1e9, 1e9))
    mx = Vector((-1e9, -1e9, -1e9))
    for o in objs:
        for v in o.bound_box:
            w = o.matrix_world @ Vector(v)
            mn = Vector(map(min, mn, w))
            mx = Vector(map(max, mx, w))
    ext = max(mx.x - mn.x, (mx.y - mn.y) * 1.15, (mx.z - mn.z) * 0.9)
    target = opts.get("target", 1.15)
    f = max(0.7, min(1.9, target / max(ext, 1e-3)))
    cx, cy = (mn.x + mx.x) / 2, (mn.y + mx.y) / 2
    for o in objs:
        o.location = Vector((-cx * f, -cy * f, 0))
        o.scale = (f, f, f)


for name, fn in models.items():
    if only and name not in only and not any(name.startswith(o.rstrip("*")) for o in only if o.endswith("*")):
        continue
    t0 = time.time()
    lib.reset_scene()
    # First build with defaults to learn the options, then set the camera.
    b = lib.Builder(name)
    opts = fn(b) or {}
    sc = lib.setup_render(
        res=opts.get("res", 256),
        samples=opts.get("samples", 24),
        top_down=opts.get("top_down", False),
        ortho=opts.get("ortho", 1.7),
        elev=opts.get("elev", 52.0),
        target_z=opts.get("target_z", 0.25),
        shift=opts.get("shift", -0.09),
    )
    objs = b.finish()
    fit_scale(objs, opts)
    if opts.get("ystretch"):
        import math
        for o in objs:
            o.scale.y *= 1.0 / math.sin(math.radians(opts.get("elev", 52.0)))
    if not opts.get("no_catcher"):
        lib.shadow_catcher(radius=opts.get("catcher", 1.3), z=-0.03)
    sc.render.filepath = os.path.join(out, name + ".png")
    bpy.ops.render.render(write_still=True)
    print(f"RENDERED {name} {time.time() - t0:.1f}s")
