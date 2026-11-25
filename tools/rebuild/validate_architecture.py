import bpy,json,re,math,sys,os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]; data=ROOT/'crates/wayciv-data/data'; manifest_name='unit' if '--unit' in sys.argv else 'resource' if '--resource' in sys.argv else 'building' if '--building' in sys.argv else 'architecture'; manifest=json.loads((ROOT/f'art/{manifest_name}_manifest.json').read_text()); assets=manifest['assets']; checked=[]
expected_districts=set(re.findall(r'id = "(DISTRICT_[A-Z_]+)"',(data/'districts.toml').read_text()+(data/'civilizations.toml').read_text()))-{'DISTRICT_WONDER'}
expected_wonders=set(re.findall(r'id = "(BUILDING_[A-Z_]+)"',(data/'wonders.toml').read_text()))
expected_buildings=set(re.findall(r'id = "(BUILDING_[A-Z_]+)"',(data/'buildings.toml').read_text()+(data/'civilizations.toml').read_text()))
actual={r['id'] for r in assets}; errors=[]
expected_resources=set(re.findall(r'id = "(RESOURCE_[A-Z_]+)"',(data/'resources.toml').read_text()))
expected_improvements=set(re.findall(r'id = "(IMPROVEMENT_[A-Z_]+)"',(data/'resources.toml').read_text()+(data/'civilizations.toml').read_text()))
for row in assets:
    for ext in ['blend','fbx']: assert (ROOT/row[ext]).stat().st_size>1000,row[ext]
    with bpy.data.libraries.load(str(ROOT/row['blend'])) as (source,target): target.scenes=source.scenes
    scene=target.scenes[0]; objects=list(scene.objects); roots=[o for o in objects if not o.parent]; meshes=[o for o in objects if o.type=='MESH']; armatures=[o for o in objects if o.type=='ARMATURE']
    assert len(roots)==1 and roots[0].location.length<1e-6,row['asset_id']
    assert roots[0].get('catalog_id')==row['id'],row['asset_id']
    if manifest_name=='unit':
        assert len(armatures)==1 and len(armatures[0].data.bones)>=6,row['asset_id']
        assert armatures[0].animation_data and len(armatures[0].animation_data.nla_tracks)>=3,row['asset_id']
    for ob in meshes:
        assert len(ob.data.vertices) and len(ob.data.polygons),ob.name
        assert ob.data.uv_layers and ob.data.materials,ob.name
        if manifest_name=='unit':
            assert ob.vertex_groups and any(m.type=='ARMATURE' for m in ob.modifiers),ob.name
        assert all(math.isfinite(c) for v in ob.data.vertices for c in v.co),ob.name
        bad=sum(p.area<1e-12 for p in ob.data.polygons)
        if bad: errors.append({'asset':row['asset_id'],'object':ob.name,'degenerate_faces':bad})
    checked.append(row['asset_id']); bpy.data.scenes.remove(scene)
    for ob in objects: bpy.data.objects.remove(ob,do_unlink=True)
report={'checked':len(checked),'errors':errors,'missing_districts':sorted(expected_districts-actual),'missing_wonders':sorted(expected_wonders-actual),'buildings_requiring_individual_models':sorted(expected_buildings-actual),'counts':manifest['counts']}
if manifest_name=='unit':
    expected_units=set(re.findall(r'id = "(UNIT_[A-Z_]+)"',(data/'units.toml').read_text()+(data/'civilizations.toml').read_text()))
    report={'checked':len(checked),'errors':errors,'missing_units':sorted(expected_units-actual),'counts':manifest['counts']}
elif manifest_name=='resource': report={'checked':len(checked),'errors':errors,'missing_resources':sorted(expected_resources-actual),'missing_improvements':sorted(expected_improvements-actual),'counts':manifest['counts']}
elif manifest_name=='building': report={'checked':len(checked),'errors':errors,'missing_buildings':sorted(expected_buildings-actual),'counts':manifest['counts']}
(ROOT/f'art/{manifest_name}_validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report),flush=True); sys.stdout.flush(); os._exit(0)
