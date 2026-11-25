"""Promote the existing authored architecture to individual editable 3D assets.
No sprite-camera stretching, shadow catcher, or render-only objects in exports.
"""
import bpy,bmesh,sys,math,json,importlib,re,random
from pathlib import Path
from mathutils import Vector
ROOT=Path(__file__).resolve().parents[2]; OUT=ROOT/'art'
sys.path.insert(0,str(ROOT/'tools/buildings'))
import bl,matlib
MODULES=['districts','districts_infra','districts_special','wonders_ancient','wonders_medieval','wonders_modern','cities','districts_rebuild']

def export_material(source):
    # Record concrete palette values; Blender procedural nodes do not survive FBX.
    if 'game_base_color' in source:
        m=bpy.data.materials.new('Game_'+source.name); m.use_nodes=True; m.diffuse_color=tuple(source['game_base_color']); p=m.node_tree.nodes.get('Principled BSDF'); p.inputs['Base Color'].default_value=m.diffuse_color; p.inputs['Roughness'].default_value=source.get('game_roughness',.8); p.inputs['Metallic'].default_value=source.get('game_metallic',0); return m
    key=next((k for k,v in matlib._mats.items() if v==source),None)
    name=key[0] if key else re.sub(r'\d+$','',source.name)
    if name=='ground': name=key[1]
    if source.name.startswith('ground_'): name=source.name[7:].split('.')[0]
    spec=matlib.PAL.get(name,matlib.PAL['pave']); rgb=spec[0]
    if key and len(key)==7 and key[1] is not None: rgb=key[1]
    color=matlib.lin(matlib.hexc(rgb) if isinstance(rgb,str) else rgb)
    rough=spec[1]; metal=spec[2] if len(spec)>2 else 0
    if key and len(key)==7:
        if key[2] is not None: rough=key[2]
        if key[3] is not None: metal=key[3]
    m=bpy.data.materials.new('Game_'+source.name); m.diffuse_color=(*color,1); m.use_nodes=True
    p=m.node_tree.nodes.get('Principled BSDF'); p.inputs['Base Color'].default_value=(*color,1); p.inputs['Roughness'].default_value=rough; p.inputs['Metallic'].default_value=metal
    if key and len(key)==7 and key[4]:
        ec=key[4]; ec=matlib.hexc(ec) if isinstance(ec,str) else ec; p.inputs['Emission Color'].default_value=(*matlib.lin(ec),1); p.inputs['Emission Strength'].default_value=2
    return m

def model(name,builder):
    random.seed(425+sum(map(ord,name))); bl.CONTENT_SCALE=1.0; matlib.MASK_MODE=False; bl.reset(); builder(); bl.realise()
    for ob in list(bpy.context.scene.objects):
        if ob.is_shadow_catcher: bpy.data.objects.remove(ob,do_unlink=True)
    root=bpy.data.objects.new(name,None); bpy.context.scene.collection.objects.link(root)
    rig=next((o for o in bpy.context.scene.objects if o.type=='ARMATURE'),None)
    if rig is not None: rig.parent=root
    if name.startswith('district_'): ident=re.sub(r'_\d+$','',name).upper()
    elif name.startswith('wonder_'): ident='BUILDING_'+name.removeprefix('wonder_').upper()
    elif name.startswith(('building_','resource_','unit_')): ident=name.upper()
    elif name.startswith('improvement_'):
        ident=name.upper()
        if name.startswith('improvement_farm'): ident='IMPROVEMENT_FARM'
        elif name.startswith('improvement_plantation'): ident='IMPROVEMENT_PLANTATION'
    else: ident='DISTRICT_CITY_CENTER' if not name.startswith('walls_') else ['BUILDING_WALLS','BUILDING_CASTLE','BUILDING_STAR_FORT'][int(name.split('_')[1])]
    root['catalog_id']=ident; root['asset_id']=name; root.asset_mark()
    original={}; game={}; triangles=0
    for i,ob in enumerate([o for o in bpy.context.scene.objects if o.type=='MESH']):
        ob['part_name']=ob.name
        ob.name=f'{name}_{ob.data.materials[0].name}_{i:02d}'
        if ob.parent is None: ob.parent=rig or root
        bm=bmesh.new(); bm.from_mesh(ob.data); bmesh.ops.dissolve_degenerate(bm,dist=1e-7,edges=list(bm.edges)); bm.to_mesh(ob.data); bm.free(); ob.data.update()
        uv=ob.data.uv_layers.new(name='UVMap')
        for p in ob.data.polygons:
            axes=sorted(range(3),key=lambda a:abs(p.normal[a]))[:2]
            for li in p.loop_indices:
                co=ob.data.vertices[ob.data.loops[li].vertex_index].co; uv.data[li].uv=(co[axes[0]],co[axes[1]])
        triangles+=sum(len(p.vertices)-2 for p in ob.data.polygons)
        original[ob]=list(ob.data.materials)
        for m in original[ob]:
            if m not in game: game[m]=export_material(m)
    return root,original,game,triangles

def main(sources_override=None, manifest_name="architecture"):
    sources={}
    for mod in ([] if sources_override is not None else MODULES):
        for name,(builder,tint) in importlib.import_module(mod).SPRITES.items(): sources[name]=(builder,mod)
    if sources_override is not None: sources=sources_override
    records=[]; pages={}; galleries=[]
    for kind in ['districts','wonders','cities','buildings','resources','improvements','units']:
        for prefix in ['blender','exports']: (OUT/prefix/kind).mkdir(parents=True,exist_ok=True)
    for name,(builder,module) in sources.items():
        kind='units' if name.startswith('unit_') else 'resources' if name.startswith('resource_') else 'improvements' if name.startswith('improvement_') else 'buildings' if name.startswith('building_') else 'districts' if name.startswith('district_') else 'wonders' if name.startswith('wonder_') else 'cities'
        scene=bpy.data.scenes.new(name); bpy.context.window.scene=scene; scene.unit_settings.system='METRIC'
        root,original,game,tris=model(name,builder)
        blend=OUT/'blender'/kind/f'{name}.blend'; fbx=OUT/'exports'/kind/f'{name}.fbx'
        bpy.data.libraries.write(str(blend),{scene},fake_user=True,compress=True)
        for ob,mats in original.items():
            ob.data.materials.clear()
            for m in mats: ob.data.materials.append(game[m])
        for ob in scene.objects: ob.select_set(True)
        bpy.context.view_layer.objects.active=root
        has_rig=any(o.type=='ARMATURE' for o in scene.objects)
        bpy.ops.export_scene.fbx(filepath=str(fbx),use_selection=True,object_types={'ARMATURE','EMPTY','MESH'} if has_rig else {'EMPTY','MESH'},axis_forward='-Z',axis_up='Y',bake_anim=has_rig,bake_anim_use_all_actions=False,bake_anim_use_nla_strips=has_rig,add_leaf_bones=False,use_custom_props=True)
        rec={'id':root['catalog_id'],'asset_id':name,'variant':name,'category':kind,'source_builder':f'tools/buildings/{module}.py','blend':str(blend.relative_to(ROOT)),'fbx':str(fbx.relative_to(ROOT)),'triangles':tris,'materials':[{'name':m.name,'rgba':list(m.diffuse_color)} for m in game.values()]}
        if has_rig: rec['rigged']=True; rec['animations']=['Idle','Move','Attack']
        records.append(rec)
        # Page galleries use game-compatible materials to show the actual export palette.
        count=pages.get(kind,0); page=count//20; key=f'{kind}_{page:02d}'
        gallery=next((g for g in galleries if g.name==key),None)
        if gallery is None: gallery=bpy.data.scenes.new(key); galleries.append(gallery)
        for ob in scene.objects: gallery.collection.objects.link(ob)
        root.location=((count%5)*2.7,((count%20)//5)*2.8,0)
        bpy.context.window.scene=gallery; bpy.data.scenes.remove(scene); pages[kind]=count+1
        label=bpy.data.curves.new(name+'_label','FONT'); label.body=name.replace('district_','').replace('wonder_','').replace('_',' '); label.align_x='CENTER'; label.size=.12; ob=bpy.data.objects.new(label.name,label); gallery.collection.objects.link(ob); ob.location=(root.location.x,root.location.y-1.12,.01)
        print('ASSET',name,tris,flush=True)
    for gallery in galleries:
        bpy.context.window.scene=gallery
        cam=bpy.data.cameras.new('Review_camera'); ob=bpy.data.objects.new(cam.name,cam); gallery.collection.objects.link(ob); target=Vector((5.4,4,0)); ob.location=target+Vector((0,-13,16)); ob.rotation_euler=(target-ob.location).to_track_quat('-Z','Y').to_euler(); cam.type='ORTHO'; cam.ortho_scale=15.5; gallery.camera=ob
        sun=bpy.data.lights.new('Review_sun','SUN'); sun.energy=3; ob=bpy.data.objects.new(sun.name,sun); gallery.collection.objects.link(ob); ob.rotation_euler=(.4,-.5,-.4)
        world=bpy.data.worlds.new('Review_world'); world.use_nodes=True; world.node_tree.nodes['Background'].inputs[0].default_value=(.16,.20,.27,1); world.node_tree.nodes['Background'].inputs[1].default_value=.7; gallery.world=world
        gallery.render.engine='CYCLES'; gallery.cycles.samples=16; gallery.cycles.use_denoising=True; gallery.render.resolution_x=1800; gallery.render.resolution_y=1500; gallery.render.resolution_percentage=100; gallery.render.filepath=str(OUT/'previews'/f'{gallery.name}.png')
    (OUT/f'{manifest_name}_manifest.json').write_text(json.dumps({'assets':records,'counts':pages,'review_pages':[g.name for g in galleries]},indent=2)+'\n')
    bpy.context.window.scene=galleries[0]
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT/f'blender/{manifest_name}_review.blend'))
    return {'assets':len(records),'counts':pages}
if __name__=='__main__': result=main()
