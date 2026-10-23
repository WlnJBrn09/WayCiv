"""Blender entry point: render beauty + team-mask passes for some units.

  blender -b --factory-startup -t 4 -P render.py -- --out DIR [--res 384] [--samples 40] UNIT_ID...

Writes DIR/<id>.png (beauty, transparent) and DIR/<id>_mask.png (alpha = where
the team-colour parts are).  tools/units/build_all.sh runs this for every unit
and post.py turns the raw renders into the final sprites.
"""
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import bpy  # noqa: E402
import lib  # noqa: E402
from lib import xf  # noqa: E402
import catalog  # noqa: E402


def main():
    argv = sys.argv[sys.argv.index("--") + 1:]
    out, res, samples, ids = ".", 384, 40, []
    i = 0
    while i < len(argv):
        if argv[i] == "--out":
            out = argv[i + 1]; i += 2
        elif argv[i] == "--res":
            res = int(argv[i + 1]); i += 2
        elif argv[i] == "--samples":
            samples = int(argv[i + 1]); i += 2
        else:
            ids.append(argv[i]); i += 1
    os.makedirs(out, exist_ok=True)
    for uid in ids:
        t0 = time.time()
        lib.reset()
        spec = catalog.spec_for(uid)
        lib.setup_scene(res=res, samples=samples, ortho=spec["ortho"], elev=spec.get("elev", 45),
                        anchor=spec["anchor"])
        with xf((0, 0, spec.get("lift", 0.0)), (spec.get("pitch", 0), 0, spec["yaw"])):
            catalog.build(uid)
        lib.render_to(os.path.join(out, uid.lower() + ".png"))
        lib.apply_mask_materials()
        lib.render_to(os.path.join(out, uid.lower() + "_mask.png"))
        print("UNIT %s %.1fs" % (uid, time.time() - t0), flush=True)


main()
