"""Export all 533 Blender 3D assets into a compact binary 3D mesh pack (assets/models3d.pack)
for real-time 3D rendering in WayCiv."""
import bpy, bmesh, json, math, struct, sys, os
from pathlib import Path
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[2]
ART = ROOT / 'art'
OUT_PACK = ROOT / 'assets/models3d.pack'

MANIFESTS = [
    ('terrain', 99999),
    ('connection', 99999),
    ('feature', 700),
    ('natural_wonder', 900),
    ('architecture', 1100),
    ('building', 900),
    ('resource', 700),
    ('unit', 800),
]


def lin_to_srgb_u8(c):
    c = max(0.0, min(1.0, float(c)))
    s = c * 12.92 if c <= 0.0031308 else 1.055 * (c ** (1.0 / 2.4)) - 0.055
    return int(round(max(0.0, min(1.0, s)) * 255.0))


def mat_color_and_flag(mat, mat_lookup):
    if mat is None:
        return (180, 180, 180, 0)
    flag = 1 if (mat.get('team_tint') or mat.name.lower().startswith(('team', 'game_acc', 'acc'))) else 0
    if 'game_base_color' in mat:
        col = mat['game_base_color']
        return (lin_to_srgb_u8(col[0]), lin_to_srgb_u8(col[1]), lin_to_srgb_u8(col[2]), flag)
    if mat.name in mat_lookup:
        col = mat_lookup[mat.name]
        return (lin_to_srgb_u8(col[0]), lin_to_srgb_u8(col[1]), lin_to_srgb_u8(col[2]), flag)
    if mat.use_nodes and mat.node_tree:
        p = next((n for n in mat.node_tree.nodes if n.type == 'BSDF_PRINCIPLED'), None)
        if p:
            col = p.inputs['Base Color'].default_value
            return (lin_to_srgb_u8(col[0]), lin_to_srgb_u8(col[1]), lin_to_srgb_u8(col[2]), flag)
    col = mat.diffuse_color
    return (lin_to_srgb_u8(col[0]), lin_to_srgb_u8(col[1]), lin_to_srgb_u8(col[2]), flag)


def extract_asset(blend_path, max_tris, mat_lookup):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    with bpy.data.libraries.load(str(blend_path)) as (src, dst):
        dst.scenes = src.scenes
    sc = dst.scenes[0]
    bpy.context.window.scene = sc
    for o in sc.objects:
        if o.type == 'ARMATURE':
            for mod in list(o.modifiers):
                o.modifiers.remove(mod)
    meshes = [o for o in sc.objects if o.type == 'MESH']
    for o in meshes:
        for mod in list(o.modifiers):
            if mod.type == 'ARMATURE':
                o.modifiers.remove(mod)
    deps = bpy.context.evaluated_depsgraph_get()
    total_raw_tris = 0
    eval_meshes = []
    for o in meshes:
        eo = o.evaluated_get(deps)
        me = bpy.data.meshes.new_from_object(eo)
        me.transform(o.matrix_world)
        bm = bmesh.new()
        bm.from_mesh(me)
        bmesh.ops.triangulate(bm, faces=bm.faces)
        total_raw_tris += len(bm.faces)
        eval_meshes.append((o, bm, me))

    ratio = min(1.0, max_tris / max(total_raw_tris, 1))
    verts = []
    tris = []
    for o, bm, me in eval_meshes:
        if ratio < 0.85 and len(bm.faces) > 60:
            bm.to_mesh(me)
            bm.free()
            tmp = bpy.data.objects.new('tmp', me)
            sc.collection.objects.link(tmp)
            dec = tmp.modifiers.new('dec', 'DECIMATE')
            dec.ratio = max(0.08, ratio)
            deps2 = bpy.context.evaluated_depsgraph_get()
            me2 = bpy.data.meshes.new_from_object(tmp.evaluated_get(deps2))
            bm = bmesh.new()
            bm.from_mesh(me2)
            bmesh.ops.triangulate(bm, faces=bm.faces)
            bpy.data.meshes.remove(me2)
            bpy.data.objects.remove(tmp, do_unlink=True)
        mats = list(o.data.materials)
        palette = [mat_color_and_flag(m, mat_lookup) for m in mats] or [(180, 180, 180, 0)]
        base_v = len(verts)
        bm.verts.ensure_lookup_table()
        for v in bm.verts:
            verts.append((v.co.x, v.co.y, v.co.z))
        for f in bm.faces:
            if f.calc_area() < 1e-9:
                continue
            v0, v1, v2 = (base_v + f.verts[0].index, base_v + f.verts[1].index, base_v + f.verts[2].index)
            if v0 >= 65535 or v1 >= 65535 or v2 >= 65535:
                continue
            mi = min(max(f.material_index, 0), len(palette) - 1)
            r, g, b, fl = palette[mi]
            tris.append((v0, v1, v2, r, g, b, fl))
        bm.free()
        bpy.data.meshes.remove(me)
    return verts, tris


def main():
    entries = {}
    for mname, budget in MANIFESTS:
        data = json.loads((ART / f'{mname}_manifest.json').read_text())
        for row in data['assets']:
            mat_lookup = {}
            for m in row.get('materials', []):
                raw = m['name'].removeprefix('Game_')
                mat_lookup[raw] = m['rgba']
                mat_lookup[m['name']] = m['rgba']
            blend = ROOT / row['blend']
            key_primary = f"{row['id']}#{row.get('variant', 0)}"
            asset_id = row.get('asset_id') or blend.stem
            verts, tris = extract_asset(blend, budget, mat_lookup)
            payload = (verts, tris)
            entries[asset_id] = payload
            entries.setdefault(row['id'], payload)
            entries[key_primary] = payload
        print(f"Packed {mname}: {len(data['assets'])} assets (total keys: {len(entries)})", flush=True)

    # Deduplicate mesh payloads by identical (verts, tris) identity to keep file tiny.
    unique_meshes = []
    mesh_index_by_id = {}
    key_to_mesh = {}
    for k, payload in entries.items():
        pid = id(payload)
        if pid not in mesh_index_by_id:
            mesh_index_by_id[pid] = len(unique_meshes)
            unique_meshes.append(payload)
        key_to_mesh[k] = mesh_index_by_id[pid]

    OUT_PACK.parent.mkdir(parents=True, exist_ok=True)
    with open(OUT_PACK, 'wb') as f:
        f.write(b'WC3D')
        f.write(struct.pack('<II', len(unique_meshes), len(key_to_mesh)))
        for verts, tris in unique_meshes:
            f.write(struct.pack('<II', len(verts), len(tris)))
            for x, y, z in verts:
                f.write(struct.pack('<fff', x, y, z))
            for v0, v1, v2, r, g, b, fl in tris:
                f.write(struct.pack('<HHHBBBB', v0, v1, v2, r, g, b, fl))
        for k, midx in key_to_mesh.items():
            kb = k.encode('utf-8')
            f.write(struct.pack('<HI', len(kb), midx))
            f.write(kb)
    print(f"Wrote {OUT_PACK} ({OUT_PACK.stat().st_size / (1024*1024):.2f} MB, {len(unique_meshes)} unique 3D meshes, {len(key_to_mesh)} lookup keys)", flush=True)


if __name__ == '__main__':
    main()
    sys.stdout.flush()
    os._exit(0)
