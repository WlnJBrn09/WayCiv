"""Save authored resource and improvement meshes individually, at original tile scale."""
import sys, importlib, math, random
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).parent))
from architecture_models import main
sys.path.insert(0, str(ROOT / 'tools/resources'))
import lib


def _water_rings_3d(b, r=0.7, n=2):
    """3D concentric water ripple rings (replaces the 2D sprite alpha halo platter)."""
    fm = lib.mat("foam_ring", (0.72, 0.88, 0.96), 0.4)
    for k in range(max(n, 2)):
        b.ring(fm, (0, 0, 0.008), r * (0.56 + 0.24 * k), 0.016, seg=36)


def _soft_ground_3d(b, col, col2, rx, ry=1.0, z=-0.01, scale=10):
    R = rx * 0.95
    gm = lib.soft_mat("sg" + col, lib.c(col), col2=lib.c(col2), scale=scale, r_in=R * 0.5, r_out=R, ry=1.0)
    lib.soft_disc(b, gm, (0, 0, z), R, 1.0, h=0.008)


def _ground_patch_3d(b, col="#8a6a44", col2="#6a4e30", r=0.5, z=0.0, seed=1, ry=1.0, grass=True):
    R = r * 0.98
    gm_r = lib.soft_mat("gpatch" + col + "r", lib.c(col), col2=lib.c(col2), scale=7, r_in=R * 0.52, r_out=R, ry=1.0)
    lib.soft_disc(b, gm_r, (0, 0, z - 0.004), R, 1.0, h=0.008)
    if grass:
        rng = random.Random(seed)
        tm = lib.leaf_mat("#6a9a3a")
        for _ in range(14):
            a = rng.random() * math.tau
            d = R * (0.62 + 0.28 * rng.random())
            b.tuft(tm, (math.cos(a) * d, math.sin(a) * d, z), h=0.09 + 0.05 * rng.random(), n=4, spread=0.035)
    return gm_r


_orig_soft_disc = lib.soft_disc
def _soft_disc_3d(b, m, pos, r, ry=1.0, h=0.012, seg=48):
    if m.name.startswith(('wake', 'foam')):
        return
    return _orig_soft_disc(b, m, pos, r, ry=ry, h=h, seg=seg)

lib.soft_disc = _soft_disc_3d
lib.water_rings = _water_rings_3d
lib.soft_ground = _soft_ground_3d
lib.ground_patch = _ground_patch_3d


def wrap(name, fn):
    def build():
        lib.MATS.clear()
        b = lib.Builder(name)
        fn(b)
        objects = b.finish()
        for ob in objects:
            for m in ob.data.materials:
                p = next((n for n in m.node_tree.nodes if n.type == 'BSDF_PRINCIPLED'), None)
                if p:
                    m['game_base_color'] = list(p.inputs['Base Color'].default_value)
                    m['game_roughness'] = p.inputs['Roughness'].default_value
                    m['game_metallic'] = p.inputs['Metallic'].default_value
    return build


def run():
    sources = {}
    for module in ['res_bonus', 'res_lux', 'res_strat', 'improvements', 'fields']:
        mod = importlib.import_module(module)
        mod.soft_disc = _soft_disc_3d
        mod.water_rings = _water_rings_3d
        mod.soft_ground = _soft_ground_3d
        mod.ground_patch = _ground_patch_3d
        for name, fn in mod.MODELS.items():
            ident = 'improvement_barbarian_camp' if name == 'barbarian_camp' else name.replace('imp_', 'improvement_', 1)
            sources[ident] = (wrap(ident, fn), '../resources/' + module)
    return main(sources, 'resource')


if __name__ == '__main__':
    result = run()
