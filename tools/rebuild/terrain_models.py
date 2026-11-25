"""Run in Blender: individual catalog terrain meshes, FBX exports and review scene.
Pointy hex radius 1m, Z-up source, Y-up FBX, common edge at z=0.
"""
import bpy, math, json, random, tomllib
from pathlib import Path
from mathutils import Vector
ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'art'
CAT = tomllib.loads((ROOT/'crates/wayciv-data/data/terrain.toml').read_text().split('# Everyday features.')[0])
PALETTE = {'GRASS':(.19,.36,.10), 'PLAINS':(.48,.40,.17), 'DESERT':(.70,.44,.20), 'TUNDRA':(.29,.32,.26), 'SNOW':(.76,.86,.92), 'COAST':(.08,.46,.48), 'OCEAN':(.035,.22,.34), 'DEEP_OCEAN':(.025,.09,.20)}

def material(name, color, rough=.9):
    m=bpy.data.materials.new(name); m.diffuse_color=(*color,1); m.use_nodes=True
    p=m.node_tree.nodes.get('Principled BSDF'); p.inputs['Base Color'].default_value=(*color,1); p.inputs['Roughness'].default_value=rough
    return m

def mesh_for(entry, variant):
    ident=entry['id']; biome=ident.removeprefix('TERRAIN_').replace('_HILLS','').replace('_MOUNTAIN','')
    water=entry.get('water',False); mountain=entry.get('mountain',False); hills=entry.get('hills',False)
    seed=sum(map(ord,ident))+variant*193; rng=random.Random(seed)
    peaks=[(rng.uniform(-.35,.35),rng.uniform(-.3,.3),rng.uniform(.65,1.1)) for _ in range(3)]
    def height(x,y,r):
        edge=max(0,1-r*r)**1.6
        if water: return 0.0
        if mountain:
            h=max(a*max(0,1-math.hypot(x-px,y-py)/.85)**1.3 for px,py,a in peaks)
            h*=.86+.14*math.sin(15*x+4*math.sin(9*y))
        elif hills: h=.23+.13*math.sin(x*6+variant)*math.cos(y*5-variant)
        elif biome=='DESERT': h=.035+.035*math.sin(12*x+3*y+variant)
        else: h=.014+.012*math.sin(8*x+variant)*math.cos(9*y)
        return h*edge
    n=72; rings=16; verts=[(0,0,height(0,0,0))]; faces=[]
    for ring in range(1,rings+1):
        r=ring/rings
        for j in range(n):
            side=j//12; t=(j%12)/12
            a=math.pi/6+side*math.pi/3; b=a+math.pi/3
            x=r*((1-t)*math.cos(a)+t*math.cos(b)); y=r*((1-t)*math.sin(a)+t*math.sin(b))
            verts.append((x,y,height(x,y,r)))
    for j in range(n): faces.append((0,1+j,1+(j+1)%n))
    for ring in range(1,rings):
        lo=1+(ring-1)*n; hi=1+ring*n
        for j in range(n):
            k=(j+1)%n; faces.extend([(lo+j,hi+j,hi+k),(lo+j,hi+k,lo+k)])
    top_count=len(faces); edge_start=1+(rings-1)*n; bottom=len(verts)
    for j in range(n):
        x,y,_=verts[edge_start+j]; verts.append((x,y,-.16))
    for j in range(n):
        k=(j+1)%n; faces.append((edge_start+j,bottom+j,bottom+k,edge_start+k))
    faces.append(tuple(reversed(range(bottom,bottom+n))))
    name=f'{ident}_v{variant:02d}'; mesh=bpy.data.meshes.new(name); mesh.from_pydata(verts,[],faces); mesh.update()
    obj=bpy.data.objects.new(name,mesh)
    base=PALETTE[biome]
    for i in range(7): mesh.materials.append(material(f'{name}_earth_{i}',tuple(min(1,c*(.82+.06*i)) for c in base),.24 if water else .92))
    mesh.materials.append(material(name+'_bedrock',(.20,.17,.13)))
    mesh.materials.append(material(name+'_rock',(.35,.34,.31)))
    mesh.materials.append(material(name+'_snow',(.88,.93,.97)))
    for p in mesh.polygons:
        z=sum(verts[i][2] for i in p.vertices)/len(p.vertices)
        p.material_index=7 if p.index>=top_count else rng.randrange(7)
        if mountain and p.index<top_count:
            if z>.42 and biome!='DESERT': p.material_index=9
            elif z>.14: p.material_index=8
    uv=mesh.uv_layers.new(name='UVMap')
    for p in mesh.polygons:
        for li in p.loop_indices:
            co=mesh.vertices[mesh.loops[li].vertex_index].co; uv.data[li].uv=((co.x+1)/2,(co.y+1)/2)
    obj['catalog_id']=ident; obj['variant']=variant; obj['hex_radius_m']=1.0
    obj['placement']='Pointy hex, shared z=0 edge; terrain surface at local origin'
    obj.asset_mark()
    return obj

def run():
    gallery=bpy.data.scenes.new('WayCiv_Terrain_Review'); bpy.context.window.scene=gallery
    gallery.unit_settings.system='METRIC'
    records=[]
    for idx,entry in enumerate(CAT['terrain']):
        for variant in range(3):
            obj=mesh_for(entry,variant)
            scene=bpy.data.scenes.new(obj.name); scene.collection.objects.link(obj); scene.unit_settings.system='METRIC'
            bpy.context.window.scene=scene; bpy.context.view_layer.objects.active=obj; obj.select_set(True)
            blend=OUT/'blender/terrain'/f'{obj.name}.blend'; fbx=OUT/'exports/terrain'/f'{obj.name}.fbx'
            bpy.data.libraries.write(str(blend),{scene},fake_user=True,compress=True)
            bpy.ops.export_scene.fbx(filepath=str(fbx),use_selection=True,object_types={'MESH'},axis_forward='-Z',axis_up='Y',apply_unit_scale=True,bake_anim=False)
            records.append({'id':entry['id'],'variant':variant,'blend':str(blend.relative_to(ROOT)),'fbx':str(fbx.relative_to(ROOT)),'vertices':len(obj.data.vertices),'triangles':sum(len(p.vertices)-2 for p in obj.data.polygons)})
            gallery.collection.objects.link(obj); obj.location=((idx%6)*2.5,(idx//6)*7+variant*2.2,0)
            bpy.context.window.scene=gallery; bpy.data.scenes.remove(scene)
    cam=bpy.data.cameras.new('Terrain_Review_Camera'); co=bpy.data.objects.new(cam.name,cam); gallery.collection.objects.link(co)
    target=Vector((6.25,9,0)); co.location=target+Vector((11,-23,30)); co.rotation_euler=(target-co.location).to_track_quat('-Z','Y').to_euler(); cam.type='ORTHO'; cam.ortho_scale=28; gallery.camera=co
    light=bpy.data.lights.new('Terrain_Sun','SUN'); light.energy=2.5; ob=bpy.data.objects.new(light.name,light); gallery.collection.objects.link(ob); ob.rotation_euler=(.4,-.5,-.4)
    world=bpy.data.worlds.new('Terrain_Studio'); world.use_nodes=True; world.node_tree.nodes['Background'].inputs[0].default_value=(.15,.19,.25,1); world.node_tree.nodes['Background'].inputs[1].default_value=.5; gallery.world=world
    gallery.render.engine='CYCLES'; gallery.cycles.samples=16; gallery.cycles.use_denoising=True
    gallery.render.resolution_x=1500; gallery.render.resolution_y=1500; gallery.render.resolution_percentage=100
    gallery.render.filepath=str(OUT/'previews/terrain_review.png')
    (OUT/'terrain_manifest.json').write_text(json.dumps({'source':'crates/wayciv-data/data/terrain.toml','hex_radius_m':1,'assets':records},indent=2)+'\n')
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'blender/terrain_review.blend'))
    return {'terrain_types':len(CAT['terrain']),'individual_assets':len(records),'review':gallery.render.filepath}

if __name__=='__main__':
    result=run()
    print(result)
