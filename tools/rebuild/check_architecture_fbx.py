import bpy,json,sys,os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
manifest=json.loads((ROOT/'art/architecture_manifest.json').read_text()); results=[]
for name in ['district_observatory_1','district_hippodrome_1','wonder_colosseum','capital_3_4']:
    row=next(r for r in manifest['assets'] if r['asset_id']==name)
    scene=bpy.data.scenes.new(name); bpy.context.window.scene=scene
    bpy.ops.import_scene.fbx(filepath=str(ROOT/row['fbx']))
    meshes=[o for o in scene.objects if o.type=='MESH']; bpy.context.view_layer.update()
    tris=sum(len(p.vertices)-2 for o in meshes for p in o.data.polygons)
    assert tris==row['triangles'],name
    assert max(max(o.dimensions) for o in meshes)<5,name
    assert all(o.data.materials for o in meshes),name
    results.append({'asset_id':name,'triangles':tris,'mesh_count':len(meshes),'max_object_dimension_m':max(max(o.dimensions) for o in meshes)})
unit_manifest=json.loads((ROOT/'art/unit_manifest.json').read_text())
for name in ['unit_warrior','unit_knight','unit_battleship','unit_jet_fighter']:
    row=next(r for r in unit_manifest['assets'] if r['asset_id']==name)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.fbx(filepath=str(ROOT/row['fbx']))
    arms=[o for o in bpy.context.scene.objects if o.type=='ARMATURE']
    acts=sorted({a.name.split('|')[-1] for a in bpy.data.actions})
    assert len(arms)==1 and {'Attack','Idle','Move'}.issubset(acts),(name,acts)
    results.append({'asset_id':name,'rigged':True,'bones':len(arms[0].data.bones),'actions':acts})
(ROOT/'art/fbx_roundtrip_validation.json').write_text(json.dumps({'checked':results,'scope':'Blender FBX reimport of architecture + rigged/animated units; Unity import pending license activation'},indent=2)+'\n')
print(results,flush=True);sys.stdout.flush();os._exit(0)
