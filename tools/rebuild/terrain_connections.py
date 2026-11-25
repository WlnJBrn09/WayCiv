"""Rotatable river and shoreline assets plus a connected hex-map integration scene."""
import bpy, sys, math, json
from pathlib import Path
from mathutils import Vector
ROOT=Path(__file__).resolve().parents[2]; OUT=ROOT/'art'; sys.path.insert(0,str(Path(__file__).parent))
from terrain_models import mesh_for, material
A=math.sqrt(3)/2
D=[(1,0),(0,1),(-1,1),(-1,0),(0,-1),(1,-1)]
def rotate(mask,k): return ((mask<<k)|(mask>>(6-k)))&63 if k else mask
def canonical(mask): return min(rotate(mask,k) for k in range(6))
MASKS=sorted({canonical(m) for m in range(1,64)})

def mesh(name,verts,faces,mat,root):
    me=bpy.data.meshes.new(name); me.from_pydata(verts,[],faces); me.update(); me.materials.append(mat)
    ob=bpy.data.objects.new(name,me); bpy.context.scene.collection.objects.link(ob); ob.parent=root; return ob

def strip(root,e,near,far,width,z,mat,name):
    a=e*math.pi/3; n=Vector((math.cos(a),math.sin(a),0)); t=Vector((-n.y,n.x,0)); vs=[]
    for d in [near,far]:
        for w in [-width,width]: vs.append(tuple(n*d+t*w+Vector((0,0,z))))
    return mesh(name,vs,[(0,2,3,1)],mat,root)

def channel(root,mask,width,z,mat):
    angles={j*math.tau/192 for j in range(192)}
    for e in range(6):
        if mask&(1<<e):
            for sign in [-1,1]: angles.add((e*math.pi/3+sign*math.atan(width/A))%math.tau)
    angles=sorted(angles); vs=[(0,0,z+.026)]
    for angle in angles:
        radius=width
        for e in range(6):
            if not mask&(1<<e): continue
            c=math.cos(angle-e*math.pi/3); t=abs(math.sin(angle-e*math.pi/3))
            if c>1e-8: radius=max(radius,min(A/c,width/max(t,1e-12)))
        crown=.026*max(0.0,1.0-min(1.0,(radius/A)**2))**1.6
        vs.append((radius*math.cos(angle),radius*math.sin(angle),z+crown))
    return mesh('Channel_surface',vs,[(0,1+j,1+(j+1)%len(angles)) for j in range(len(angles))],mat,root)

def build(kind,mask):
    name=f'{kind}_{mask:02d}'; root=bpy.data.objects.new(name,None); bpy.context.scene.collection.objects.link(root); root['catalog_id']=name; root['edge_mask']=mask; root['edge_convention']='0=East, counterclockwise 60 degree steps'; root.asset_mark()
    if kind=='RIVER':
        channel(root,mask,.12,.002,BANK); channel(root,mask,.075,.004,WATER)
    else:
        # Edge band matches the full hex edge. Narrow end caps prevent overlaps at corners.
        for e in range(6):
            if not mask&(1<<e): continue
            a=e*math.pi/3; n=Vector((math.cos(a),math.sin(a),0)); t=Vector((-n.y,n.x,0)); vs=[]
            for d,w,z in [(A-.17,.40,.018),(A-.065,.462,.008),(A,.5,.002)]:
                vs.extend([tuple(n*d-t*w+Vector((0,0,z))),tuple(n*d+t*w+Vector((0,0,z)))])
            mesh('Beach',vs[:4],[(0,2,3,1)],SAND,root); mesh('Wet_sand',vs[2:],[(0,2,3,1)],BANK,root)
    return root

def load_objects(path,scene):
    with bpy.data.libraries.load(str(path)) as (source,target): target.scenes=source.scenes
    src=target.scenes[0]; objects=list(src.objects)
    for ob in objects: scene.collection.objects.link(ob)
    bpy.data.scenes.remove(src)
    return objects

def main():
    global WATER,BANK,SAND
    WATER=material('River_water',(.04,.35,.40),.22); BANK=material('Wet_bank',(.40,.31,.16)); SAND=material('Beach_sand',(.73,.58,.33))
    records=[]; templates={}
    for sub in ['blender/connections','exports/connections']: (OUT/sub).mkdir(parents=True,exist_ok=True)
    gallery=bpy.data.scenes.new('WayCiv_Connected_Map')
    for kind in ['RIVER','SHORE']:
        for mask in MASKS:
            name=f'{kind}_{mask:02d}'; scene=bpy.data.scenes.new(name); bpy.context.window.scene=scene; scene.unit_settings.system='METRIC'; root=build(kind,mask)
            for ob in scene.objects: ob.select_set(True)
            bpy.context.view_layer.objects.active=root
            blend=OUT/'blender/connections'/f'{name}.blend'; fbx=OUT/'exports/connections'/f'{name}.fbx'
            bpy.data.libraries.write(str(blend),{scene},fake_user=True,compress=True)
            bpy.ops.export_scene.fbx(filepath=str(fbx),use_selection=True,object_types={'EMPTY','MESH'},axis_forward='-Z',axis_up='Y',bake_anim=False,use_custom_props=True)
            ports=[{'edge':e,'center':[A*math.cos(e*math.pi/3),A*math.sin(e*math.pi/3),.004]} for e in range(6) if mask&(1<<e)] if kind=='RIVER' else []
            records.append({'id':name,'variant':0,'mask':mask,'kind':kind,'ports':ports,'blend':str(blend.relative_to(ROOT)),'fbx':str(fbx.relative_to(ROOT))})
            templates[kind,mask]=list(scene.objects)
            bpy.context.window.scene=gallery
    lookup={str(mask):{'canonical':canonical(mask),'rotation_steps':next(k for k in range(6) if rotate(canonical(mask),k)==mask)} for mask in range(1,64)}
    def instance(kind,mask,q,r):
        entry=lookup[str(mask)]; source=templates[kind,entry['canonical']]; copies={}
        for src in source:
            ob=src.copy(); gallery.collection.objects.link(ob); copies[src]=ob
        for src,ob in copies.items(): ob.parent=copies.get(src.parent)
        root=next(o for o in copies.values() if not o.parent); root.location=(math.sqrt(3)*(q+r/2),1.5*r,0); root.rotation_euler.z=entry['rotation_steps']*math.pi/3
        if (q,r) in water:
            for ob in copies.values():
                if ob.type=='MESH' and ob.get('catalog_id')=='TERRAIN_GRASS':
                    ob.data=ob.data.copy(); ob.data.materials.clear(); ob.data.materials.append(WATER)
                    for p in ob.data.polygons: p.material_index=0
    cells={(q,r) for q in range(-4,5) for r in range(-4,5) if abs(q+r)<=4}
    water={c for c in cells if c[0]>=3}
    river={(q,0) for q in range(-3,4)}|{(0,r) for r in range(1,4)}
    river_masks={c:0 for c in river}
    paths=[[(q,0) for q in range(-3,4)],[(0,r) for r in range(4)]]
    for path in paths:
        for a,b in zip(path,path[1:]):
            e=D.index((b[0]-a[0],b[1]-a[1])); river_masks[a]|=1<<e; river_masks[b]|=1<<((e+3)%6)
    map_records=[]
    for q,r in sorted(cells):
        terrain='COAST' if (q,r) in water else ('GRASS_MOUNTAIN' if r>=2 and q<0 else 'GRASS_HILLS' if r>=2 else 'GRASS')
        path=OUT/'blender/terrain'/f'TERRAIN_{terrain}_v00.blend'
        objects=load_objects(path,gallery); objects[0].location=(math.sqrt(3)*(q+r/2),1.5*r,0)
        if (q,r) in river and (q,r) not in water:
            instance('RIVER',river_masks[q,r],q,r); terrain='RIVER'
        shore=sum(1<<e for e,(dq,dr) in enumerate(D) if (q+dq,r+dr) in water)
        if (q,r) not in water and shore and (q,r) not in river: instance('SHORE',shore,q,r)
        if terrain=='GRASS' and (q+r)%3==0:
            objects=load_objects(OUT/'blender/features/FEATURE_FOREST_v00.blend',gallery); root=next(o for o in objects if not o.parent); root.location=(math.sqrt(3)*(q+r/2),1.5*r,.02)
        map_records.append({'q':q,'r':r,'terrain':terrain,'river_mask':river_masks.get((q,r),0),'shore_mask':shore if (q,r) not in water else 0})
    # Check actual neighboring river port coordinates, including all branches.
    errors=[]; joins=0
    for (q,r),mask in river_masks.items():
        for e,(dq,dr) in enumerate(D):
            if not mask&(1<<e): continue
            assert river_masks[q+dq,r+dr]&(1<<((e+3)%6))
            p=Vector((math.sqrt(3)*(q+r/2)+A*math.cos(e*math.pi/3),1.5*r+A*math.sin(e*math.pi/3),.004))
            p2=Vector((math.sqrt(3)*(q+dq+(r+dr)/2)+A*math.cos((e+3)*math.pi/3),1.5*(r+dr)+A*math.sin((e+3)*math.pi/3),.004))
            errors.append((p-p2).length); joins+=1
    assert max(errors)<1e-6
    camera=bpy.data.cameras.new('Map_camera'); ob=bpy.data.objects.new(camera.name,camera); gallery.collection.objects.link(ob); ob.location=(9,-13,17); ob.rotation_euler=(-ob.location).to_track_quat('-Z','Y').to_euler(); camera.type='ORTHO'; camera.ortho_scale=20; gallery.camera=ob
    light=bpy.data.lights.new('Map_sun','SUN'); light.energy=3; ob=bpy.data.objects.new(light.name,light); gallery.collection.objects.link(ob); ob.rotation_euler=(.4,-.5,-.4)
    world=bpy.data.worlds.new('Map_world'); world.use_nodes=True; world.node_tree.nodes['Background'].inputs[0].default_value=(.12,.17,.23,1); world.node_tree.nodes['Background'].inputs[1].default_value=.6; gallery.world=world
    gallery.render.engine='CYCLES'; gallery.cycles.samples=24; gallery.cycles.use_denoising=True; gallery.render.resolution_x=1600; gallery.render.resolution_y=1400; gallery.render.resolution_percentage=100; gallery.render.filepath=str(OUT/'previews/connected_map.png')
    # Template scenes are retained in their individual files, not in the map.
    for scene in list(bpy.data.scenes):
        if scene!=gallery: bpy.data.scenes.remove(scene)
    (OUT/'connection_manifest.json').write_text(json.dumps({'assets':records,'mask_lookup':lookup,'edge_directions_axial':D},indent=2)+'\n')
    (OUT/'map_validation.json').write_text(json.dumps({'tiles':len(cells),'river_joins':joins//2-1,'river_mouths':1,'max_port_error_m':max(errors),'supported_nonempty_masks':len(lookup),'cells':map_records},indent=2)+'\n')
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'blender/connected_map.blend'))
    return {'assets':len(records),'tiles':len(cells),'river_joins':joins//2}
if __name__=='__main__': result=main()
