"""Blender background validation of actual saved scenes, not gallery transforms."""
import bpy, json, math, re, sys, os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
errors=[]; checked=[]
text=(ROOT/'crates/wayciv-data/data/terrain.toml').read_text()
expected={'terrain':set(re.findall(r'id = "(TERRAIN_[A-Z_]+)"',text.split('# Everyday features.')[0])), 'feature':set(re.findall(r'id = "(FEATURE_[A-Z_]+)"',text.split('# Everyday features.')[1].split('# Natural wonders:')[0]))}
expected['natural_wonder']=set(re.findall(r'id = "(FEATURE_[A-Z_]+)"',text.split('# Natural wonders:')[1]))
connection_masks={min(int((format(m,'06b')*2)[k:k+6],2) for k in range(6)) for m in range(1,64)}
expected['connection']={f'{kind}_{mask:02d}' for kind in ['RIVER','SHORE'] for mask in connection_masks}
for category in ['terrain','feature','natural_wonder','connection']:
    manifest=json.loads((ROOT/f'art/{category}_manifest.json').read_text())
    assert {r['id'] for r in manifest['assets']}==expected[category],f'Catalog mismatch: {category}'
    for row in manifest['assets']:
        for ext in ['blend','fbx']: assert (ROOT/row[ext]).stat().st_size>1000,row[ext]
        with bpy.data.libraries.load(str(ROOT/row['blend'])) as (source,target): target.scenes=source.scenes
        scene=target.scenes[0]; meshes=[o for o in scene.objects if o.type=='MESH']; roots=[o for o in scene.objects if o.parent is None]
        assert len(roots)==1 and roots[0].location.length<1e-6,row['blend']
        assert roots[0].get('catalog_id')==row['id'],row['id']
        for ob in meshes:
            assert len(ob.data.vertices)>0 and len(ob.data.polygons)>0,ob.name
            assert all(math.isfinite(c) for v in ob.data.vertices for c in v.co),ob.name
            assert all(p.area>1e-10 for p in ob.data.polygons),ob.name
            assert len(ob.data.materials)>0,ob.name
        if category=='terrain':
            ob=meshes[0]
            assert all(abs(ob.data.vertices[i].co.z)<1e-6 for i in range(1081,1153)),ob.name
        checked.append({'id':row['id'],'variant':row['variant'],'meshes':len(meshes)})
        objects=list(scene.objects); bpy.data.scenes.remove(scene)
        for ob in objects: bpy.data.objects.remove(ob,do_unlink=True)
report={'checked':len(checked),'catalog_coverage':{k:len(v) for k,v in expected.items()},'checks':['saved file existence','catalog ID coverage','origin placement','finite vertices','nonzero face area','material assignment','terrain boundary elevation'],'assets':checked}
(ROOT/'art/validation.json').write_text(json.dumps(report,indent=2)+'\n')
print('Validated',len(checked),'saved Blender assets',flush=True)
sys.stdout.flush(); os._exit(0)
