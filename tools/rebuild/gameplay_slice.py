"""End-to-end 3D gameplay slice driven by the compiled libwayciv_unity.so C ABI."""
import bpy, ctypes, json, math, sys, os
from pathlib import Path
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'art'


def rotate_mask(mask, k):
    return (mask & 63) if k == 0 else (((mask << k) | (mask >> (6 - k))) & 63)


def canonical_mask(mask):
    best, steps = 64, 0
    for k in range(6):
        r = rotate_mask(mask, k)
        if r < best:
            best, steps = r, (6 - k) % 6
    return best, steps


def call_engine(lib, payload):
    raw = (json.dumps(payload) + '\0').encode('utf-8')
    ptr = lib.wc_request(raw)
    try:
        text = ctypes.cast(ptr, ctypes.c_char_p).value.decode('utf-8')
        res = json.loads(text)
        assert res['ok'], res.get('error')
        return res['data']
    finally:
        lib.wc_free(ptr)


def load_asset(blend_path, scene):
    with bpy.data.libraries.load(str(blend_path)) as (src, dst):
        dst.scenes = src.scenes
    s = dst.scenes[0]
    objs = list(s.objects)
    for o in objs:
        scene.collection.objects.link(o)
    bpy.data.scenes.remove(s)
    root = next(o for o in objs if o.parent is None)
    return root, objs


def hex_pos(x, y):
    return Vector((math.sqrt(3) * (x + (y & 1) * 0.5), -1.5 * y, 0.0))


def main():
    lib = ctypes.CDLL(str(ROOT / 'target/release/libwayciv_unity.so'))
    lib.wc_request.argtypes = [ctypes.c_char_p]
    lib.wc_request.restype = ctypes.c_void_p
    lib.wc_free.argtypes = [ctypes.c_void_p]
    lib.wc_free.restype = None

    state = call_engine(lib, {'command': 'new', 'seed': 42, 'map': 'random', 'size': 'small'})
    human = state['human']
    settler = next(u for u in state['units'] if u['owner'] == human and any(a['id'] == 'FoundCity' for a in u['actions']))
    cx, cy = settler['x'], settler['y']
    state = call_engine(lib, {'command': 'action', 'unit': settler['id'], 'action': 'FoundCity'})
    tech = next(t for t in state['research'] if t['tree'] == 'tech' and t['available'])
    state = call_engine(lib, {'command': 'research', 'tree': 'tech', 'index': tech['index']})
    city = next(c for c in state['cities'] if c['owner'] == human)
    prod = next(ch for ch in city['choices'] if ch['kind'] == 'unit')
    state = call_engine(lib, {'command': 'production', 'city': city['id'], 'kind': prod['kind'], 'index': prod['index']})
    state = call_engine(lib, {'command': 'end_turn'})

    catalog = {}
    for mname in ['terrain', 'feature', 'natural_wonder', 'connection', 'architecture', 'building', 'resource', 'unit']:
        for row in json.loads((OUT / f'{mname}_manifest.json').read_text())['assets']:
            catalog.setdefault(row['id'], []).append(row)
            if 'asset_id' in row:
                catalog.setdefault(row['asset_id'], []).append(row)

    bpy.ops.wm.read_factory_settings(use_empty=True)
    sc = bpy.context.scene
    sc.name = 'WayCiv_Gameplay_Slice'
    sc.unit_settings.system = 'METRIC'

    width = state['width']
    tiles_by_pos = {(t['x'], t['y']): t for t in state['tiles']}
    spawned_tiles = 0
    for dy in range(-4, 5):
        for dx in range(-4, 5):
            tx = (cx + dx) % width
            ty = cy + dy
            t = tiles_by_pos.get((tx, ty))
            if not t or not t.get('known'):
                continue
            pos = hex_pos(cx + dx, ty)
            variant = (tx * 17 + ty * 31) % 3
            terrain_id = t['terrain']
            hills = 'HILLS' in terrain_id
            mountain = 'MOUNTAIN' in terrain_id
            surf = 0.16 if hills else 0.0

            rows = catalog[terrain_id]
            r_root, _ = load_asset(ROOT / rows[variant % len(rows)]['blend'], sc)
            r_root.location = pos
            spawned_tiles += 1

            if t.get('shoreMask', 0) > 0 and not hills and not mountain:
                can, steps = canonical_mask(t['shoreMask'])
                s_root, _ = load_asset(ROOT / catalog[f'SHORE_{can:02d}'][0]['blend'], sc)
                s_root.location = pos + Vector((0, 0, 0.003))
                s_root.rotation_euler.z = steps * math.pi / 3

            has_floodplains = (t.get('feature') or '').startswith('FEATURE_FLOODPLAINS')
            if t.get('riverMask', 0) > 0 and not hills and not mountain and not has_floodplains:
                can, steps = canonical_mask(t['riverMask'])
                rv_root, _ = load_asset(ROOT / catalog[f'RIVER_{can:02d}'][0]['blend'], sc)
                rv_root.location = pos
                rv_root.rotation_euler.z = steps * math.pi / 3

            if t.get('feature') and t['feature'] in catalog:
                f_rows = catalog[t['feature']]
                f_root, _ = load_asset(ROOT / f_rows[variant % len(f_rows)]['blend'], sc)
                f_root.location = pos + Vector((0, 0, surf))

            prop_id = t.get('improvement') or t.get('resource')
            if prop_id and prop_id in catalog:
                p_root, _ = load_asset(ROOT / catalog[prop_id][0]['blend'], sc)
                p_root.location = pos + Vector((0, 0, surf + 0.025))

    for c in state['cities']:
        dx = ((c['x'] - cx + width // 2) % width) - width // 2
        dy = c['y'] - cy
        if abs(dx) <= 4 and abs(dy) <= 4:
            pos = hex_pos(cx + dx, c['y'])
            era = min(max(c.get('era', 0), 0), 3)
            tier = min(max(1 + c['population'] // 4, 1), 4)
            model = f"{'capital' if c['capital'] else 'city'}_{era}_{tier}"
            c_root, _ = load_asset(ROOT / catalog[model][0]['blend'], sc)
            c_root.location = pos + Vector((0, 0, 0.04))

    for u in state['units']:
        dx = ((u['x'] - cx + width // 2) % width) - width // 2
        dy = u['y'] - cy
        if abs(dx) <= 4 and abs(dy) <= 4 and u['asset'] in catalog:
            t = tiles_by_pos.get((u['x'], u['y']), {})
            surf = 0.20 if 'HILLS' in (t.get('terrain') or '') else 0.04
            pos = hex_pos(cx + dx, u['y']) + Vector((0, 0, surf))
            u_root, _ = load_asset(ROOT / catalog[u['asset']][0]['blend'], sc)
            u_root.location = pos

    center = hex_pos(cx, cy)
    cam = bpy.data.cameras.new('Slice_Camera')
    cam.type = 'ORTHO'
    cam.ortho_scale = 14.5
    co = bpy.data.objects.new('Slice_Camera', cam)
    sc.collection.objects.link(co)
    co.location = center + Vector((0, -13, 14))
    co.rotation_euler = (center - co.location).to_track_quat('-Z', 'Y').to_euler()
    sc.camera = co

    sun = bpy.data.lights.new('Slice_Sun', 'SUN')
    sun.energy = 3.2
    so = bpy.data.objects.new('Slice_Sun', sun)
    sc.collection.objects.link(so)
    so.rotation_euler = (0.45, -0.45, -0.35)

    world = bpy.data.worlds.new('Slice_World')
    world.use_nodes = True
    world.node_tree.nodes['Background'].inputs[0].default_value = (0.10, 0.15, 0.22, 1)
    world.node_tree.nodes['Background'].inputs[1].default_value = 0.65
    sc.world = world

    sc.render.engine = 'CYCLES'
    sc.cycles.samples = 24
    sc.cycles.use_denoising = True
    sc.render.resolution_x = 1600
    sc.render.resolution_y = 1200
    sc.render.resolution_percentage = 100
    sc.render.filepath = str(OUT / 'previews/gameplay_slice.png')
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT / 'blender/gameplay_slice.blend'))
    bpy.ops.render.render(write_still=True)
    report = {'turn': state['turn'], 'year': state['year'], 'spawned_tiles': spawned_tiles, 'cities': len(state['cities']), 'units': len(state['units'])}
    (OUT / 'gameplay_slice_validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('GAMEPLAY_SLICE', report, flush=True)


if __name__ == '__main__':
    main()
    sys.stdout.flush()
    os._exit(0)
