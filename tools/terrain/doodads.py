"""Small ground details scattered over open land: grass tufts, flowers,
bushes, rocks, dry scrub, lichen, snowy stones, reeds, fallen logs.

Run inside Blender to render one cell per detail, then without Blender to
pack them into assets/terrain/doodads.png (8 x 2 cells of 256 px):

  blender -b --factory-startup -t 8 -P tools/terrain/doodads.py -- RAW_DIR
  python3 tools/terrain/doodads.py --pack RAW_DIR assets/terrain/doodads.png

The cell order is the `DOODADS` list below; `render/terrain.rs` picks cells
by biome (keep `DOODAD_SETS` there in sync).
"""
import math
import os
import random
import sys

CELL = 256
COLS = 8
# Every cell shows this many world units (hex radii) across.
SPAN = 0.9


def blender_main(out):
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "resources"))
    import bpy
    import lib
    from lib import Builder, mat, c, rock_mat, grass_mat, wood_mat, bush

    def tufts(b, col, col2, n, seed, h=0.1, r=0.22):
        gm = grass_mat(col, col2)
        rng = random.Random(seed)
        for _ in range(n):
            a, d = rng.random() * math.tau, r * math.sqrt(rng.random())
            b.tuft(gm, (d * math.cos(a), d * math.sin(a) * 0.8, 0), h * (0.7 + 0.6 * rng.random()), 7, 0.035, 0.007)

    def flowers(b, petals, seed):
        tufts(b, "#5d9a3a", "#3d7428", 5, seed, 0.08)
        rng = random.Random(seed + 1)
        for i in range(14):
            a, d = rng.random() * math.tau, 0.22 * math.sqrt(rng.random())
            x, y = d * math.cos(a), d * math.sin(a) * 0.8
            col = petals[i % len(petals)]
            b.cyl(mat("stem", c("#4a8a2a"), 0.7), (x, y, 0), (x, y, 0.07), 0.004, 0.003, seg=4)
            b.blob(mat("petal" + col, c(col), 0.55, sss=0.2), (x, y, 0.075), (0.022, 0.022, 0.012), seg=8)
            b.blob(mat("heart", c("#f0c838"), 0.5), (x, y, 0.083), (0.008, 0.008, 0.005), seg=6)

    def rocks(b, base, dark, seed, n=3, size=0.11, moss=None):
        rm = rock_mat(base, dark)
        rng = random.Random(seed)
        for i in range(n):
            a, d = rng.random() * math.tau, 0.14 * math.sqrt(rng.random()) * (i > 0)
            s = size * (1.0 if i == 0 else 0.45 + 0.3 * rng.random())
            b.rock(rm, (d * math.cos(a), d * math.sin(a), s * 0.45), (s, s * 0.85, s * 0.7), seed + i)
            if moss and i == 0:
                b.blob(mat("moss" + moss, c(moss), 0.9, col2=c("#3d6a28"), scale=30, bump=0.4),
                       (-0.02, 0.02, s * 0.85), (s * 0.7, s * 0.6, s * 0.18), seg=10)

    def shrub(b, col, col2, seed, twig=False):
        if twig:
            wm = wood_mat("#7a5a3a", "#4e3622")
            rng = random.Random(seed)
            for _ in range(11):
                a = rng.random() * math.tau
                tilt = 0.35 + 0.4 * rng.random()
                h = 0.12 + 0.08 * rng.random()
                b.cyl(wm, (0, 0, 0), (math.cos(a) * h * tilt, math.sin(a) * h * tilt, h), 0.007, 0.002, seg=4)
            b.blob(mat("dryleaf", c(col), 0.9, col2=c(col2), scale=20), (0, 0, 0.1), (0.1, 0.09, 0.05), seg=10)
        else:
            bush(b, (0, 0, 0), 0.13, col, col2)
            bush(b, (0.1, -0.05, 0), 0.08, col2, col)

    def dead_tree(b):
        wm = wood_mat("#8a7458", "#5a4a36")
        b.tube(wm, [(0, 0, 0), (0.01, 0.0, 0.14), (-0.03, 0.01, 0.26)], 0.025, 0.012, seg=6)
        b.tube(wm, [(0.005, 0, 0.12), (0.08, 0.02, 0.2), (0.12, 0.03, 0.25)], 0.012, 0.005, seg=5)
        b.tube(wm, [(-0.01, 0, 0.18), (-0.09, -0.02, 0.24)], 0.01, 0.004, seg=5)
        rocks(b, "#c8ac80", "#9a805a", 3, n=2, size=0.05)

    def reeds(b, seed):
        rng = random.Random(seed)
        st = mat("reed", c("#7a9a3a"), 0.7, col2=c("#a8b858"), scale=20)
        hd = mat("cattail", c("#6a4228"), 0.8)
        for _ in range(18):
            a, d = rng.random() * math.tau, 0.13 * math.sqrt(rng.random())
            x, y = d * math.cos(a), d * math.sin(a) * 0.8
            h = 0.14 + 0.1 * rng.random()
            lean = (rng.random() - 0.5) * 0.06
            b.cyl(st, (x, y, 0), (x + lean, y, h), 0.006, 0.002, seg=4)
            if rng.random() < 0.35:
                b.cyl(hd, (x + lean * 0.8, y, h * 0.8), (x + lean * 0.9, y, h * 0.92), 0.012, 0.012, seg=6)

    def log(b):
        wm = wood_mat("#6a4a2c", "#4a321c")
        b.cyl(wm, (-0.17, -0.03, 0.035), (0.15, 0.04, 0.035), 0.035, 0.03, seg=10)
        b.blob(mat("logend", c("#c8a070"), 0.8), (0.152, 0.04, 0.035), (0.004, 0.03, 0.028), seg=10)
        tufts(b, "#5d9a3a", "#3d7428", 3, 9, 0.07, 0.15)
        b.blob(mat("mush", c("#c8402a"), 0.5), (-0.05, -0.06, 0.03), (0.018, 0.018, 0.01), seg=8)

    DOODADS = [
        ("grass", lambda b: tufts(b, "#5d9a3a", "#3d7428", 7, 1)),
        ("flowers_light", lambda b: flowers(b, ["#f4f2ec", "#f0d040", "#f8f4d8"], 2)),
        ("flowers_bright", lambda b: flowers(b, ["#9a5ad0", "#d84a6a", "#5a7ad8"], 3)),
        ("bush", lambda b: shrub(b, "#4f8f2e", "#3a7424", 4)),
        ("pebbles", lambda b: rocks(b, "#9a948a", "#6a645c", 5, n=4, size=0.09)),
        ("boulder", lambda b: rocks(b, "#8a857c", "#5a5650", 6, n=3, size=0.14, moss="#5a8a32")),
        ("dry_grass", lambda b: tufts(b, "#c8b060", "#9a8a40", 7, 7, 0.11)),
        ("dry_shrub", lambda b: shrub(b, "#8a9a50", "#6a7a38", 8, twig=True)),
        ("desert_rocks", lambda b: rocks(b, "#c8a878", "#9a7c52", 9, n=4, size=0.11)),
        ("scrub", lambda b: shrub(b, "#8a9a6a", "#6a7a50", 10)),
        ("dead_tree", dead_tree),
        ("lichen_rock", lambda b: rocks(b, "#8c8a84", "#5c5a54", 12, n=3, size=0.12, moss="#c8902a")),
        ("tundra_shrub", lambda b: shrub(b, "#5a6a3a", "#8a3a2a", 13)),
        ("snow_rock", lambda b: (rocks(b, "#7a7c80", "#4e5054", 14, n=3, size=0.12),
                                 b.blob(mat("snowcap", c("#f4f6fa"), 0.6), (0, 0, 0.1), (0.1, 0.09, 0.03), seg=12))),
        ("reeds", lambda b: reeds(b, 15)),
        ("log", log),
    ]
    os.makedirs(out, exist_ok=True)
    for i, (name, fn) in enumerate(DOODADS):
        lib.reset_scene()
        b = Builder(name)
        fn(b)
        sc = lib.setup_render(res=CELL, samples=48, ortho=SPAN, elev=52.0, target_z=0.08, shift=0.0)
        b.finish()
        lib.shadow_catcher(radius=0.6, z=0.0)
        sc.render.filepath = os.path.join(out, "doodad_%02d.png" % i)
        bpy.ops.render.render(write_still=True)
        print("RENDERED", i, name, flush=True)


def pack(raw, out):
    from PIL import Image
    files = sorted(f for f in os.listdir(raw) if f.startswith("doodad_") and f.endswith(".png"))
    rows = (len(files) + COLS - 1) // COLS
    sheet = Image.new("RGBA", (COLS * CELL, rows * CELL), (0, 0, 0, 0))
    for i, f in enumerate(files):
        im = Image.open(os.path.join(raw, f)).convert("RGBA").resize((CELL, CELL), Image.LANCZOS)
        sheet.paste(im, ((i % COLS) * CELL, (i // COLS) * CELL))
    sheet.save(out, optimize=True)
    print("wrote", out, sheet.size)


if __name__ == "__main__":
    if "--pack" in sys.argv:
        k = sys.argv.index("--pack")
        pack(sys.argv[k + 1], sys.argv[k + 2])
    else:
        blender_main(sys.argv[sys.argv.index("--") + 1])
