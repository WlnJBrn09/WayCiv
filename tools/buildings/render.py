"""Render the WayCiv building sprites.

    blender -b --factory-startup -t 4 -P tools/buildings/render.py -- \
        --out assets/buildings [--only campus_1,wonder_pyramids] [--samples 40]

Sprites are registered by the modules districts.py, wonders.py and cities.py
(each exposes SPRITES = {name: (builder, tint)}).  See build.sh.
"""
import importlib
import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import bl  # noqa: E402


def main():
    argv = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
    out = 'assets/buildings'
    only = None
    samples = 40
    i = 0
    while i < len(argv):
        if argv[i] == '--out':
            out = argv[i + 1]; i += 2
        elif argv[i] == '--only':
            only = argv[i + 1].split(','); i += 2
        elif argv[i] == '--samples':
            samples = int(argv[i + 1]); i += 2
        else:
            i += 1
    os.makedirs(out, exist_ok=True)
    sprites = {}
    for modname in ('districts', 'wonders', 'cities'):
        try:
            mod = importlib.import_module(modname)
        except ModuleNotFoundError as e:
            if e.name == modname:
                continue
            raise
        sprites.update(mod.SPRITES)
    for name, (builder, tint) in sprites.items():
        if only and not any(name == o or name.startswith(o) for o in only):
            continue
        t0 = time.time()
        bl.render(builder, os.path.join(out, name), samples=samples, tint=tint, tmp=os.environ.get('TMPDIR', '/tmp'))
        print('rendered %s in %.1fs' % (name, time.time() - t0), flush=True)


main()
