"""Editable feature overlays; run with Blender's Python. Does not replace terrain."""
import bpy, math, random, json, sys
from pathlib import Path
from mathutils import Vector
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(Path(__file__).parent))
from terrain_models import material
OUT=ROOT/'art'
IDS=['FOREST','TAIGA','JUNGLE','MARSH','FLOODPLAINS','FLOODPLAINS_GRASSLAND','FLOODPLAINS_PLAINS','OASIS','DUNES','ICE','GLACIER']
M={}
def init_materials():
    for name,c in {'bark':(.20,.10,.045),'leaf':(.12,.32,.07),'leaf_light':(.26,.46,.10),'pine':(.055,.20,.12),'water':(.04,.35,.39),'silt':(.36,.25,.11),'reeds':(.36,.42,.10),'sand':(.74,.48,.23),'ice':(.36,.67,.79),'snow':(.87,.94,.98)}.items():
        M[name]=material('Feature_'+name,c,.18 if name=='water' else .85)

def finish(obj,name,mat):
    obj.name=name; obj.data.materials.append(M[mat]); obj.parent=ROOTOBJ
    bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
    return obj

def ellipsoid(name,p,s,mat):
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=2,radius=1,location=p)
    obj=bpy.context.object; obj.scale=s
    return finish(obj,name,mat)

def cone(name,p,r1,r2,depth,mat,vertices=9):
    bpy.ops.mesh.primitive_cone_add(vertices=vertices,radius1=r1,radius2=r2,depth=depth,location=p)
    return finish(bpy.context.object,name,mat)

def branch(name,a,b,r,mat='bark'):
    a,b=Vector(a),Vector(b); obj=cone(name,(a+b)/2,r,r*.65,(b-a).length,mat,7)
    obj.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler()
    return obj

def ribbon(name,points,width,mat):
    vs=[]
    for i,p in enumerate(points):
        tangent=Vector(points[min(i+1,len(points)-1)])-Vector(points[max(i-1,0)])
        side=Vector((-tangent.y,tangent.x,0)).normalized()*width
        vs.extend([tuple(Vector(p)+side),tuple(Vector(p)-side)])
    mesh=bpy.data.meshes.new(name); mesh.from_pydata(vs,[],[(i*2,i*2+1,i*2+3,i*2+2) for i in range(len(points)-1)]); mesh.update()
    obj=bpy.data.objects.new(name,mesh); bpy.context.scene.collection.objects.link(obj); obj.parent=ROOTOBJ; mesh.materials.append(M[mat]); return obj

def tree(x,y,h,kind,rng):
    branch('Trunk',(x,y,.005),(x+.025,y,h*.8),.025)
    if kind=='TAIGA':
        for k in range(3): cone('Pine_whorl',(x,y,h*(.38+k*.20)),h*(.29-k*.065),0,h*.45,'pine')
    else:
        for k in range(4):
            angle=k*math.tau/4+rng.random(); d=h*.18
            tip=(x+math.cos(angle)*d,y+math.sin(angle)*d,h*(.65+rng.random()*.22))
            branch('Bough',(x,y,h*.4),tip,.013)
            ellipsoid('Broadleaf_crown',tip,(h*.24,h*.23,h*.22),'leaf_light' if k%2 else 'leaf')
        if kind=='JUNGLE':
            for k in range(4):
                a=k*math.pi/2; branch('Buttress_root',(x+math.cos(a)*.09,y+math.sin(a)*.09,.01),(x,y,.20),.018)

def palm(x,y):
    branch('Palm_trunk',(x,y,0),(x+.08,y,.52),.025)
    for k in range(7):
        a=k*math.tau/7
        ps=[(x+.08+math.cos(a)*t,y+math.sin(a)*t,.52+.12*math.sin(t*math.pi/.32)-t*.2) for t in [0,.08,.16,.24,.32]]
        ribbon('Palm_frond',ps,.028,'leaf')

def build(ident,variant):
    rng=random.Random(137+variant*11+sum(map(ord,ident)))
    if ident in ['FOREST','TAIGA','JUNGLE']:
        for i in range(12 if ident=='TAIGA' else 9):
            a=i*2.399+variant; r=.63*math.sqrt((i+.5)/12)
            tree(math.cos(a)*r,math.sin(a)*r,rng.uniform(.45,.70) if ident!='JUNGLE' else rng.uniform(.65,.9),ident,rng)
    elif ident=='MARSH':
        for i in range(6):
            x,y=rng.uniform(-.5,.5),rng.uniform(-.45,.45)
            ellipsoid('Marsh_pool',(x,y,.012),(.22,.14,.018),'water')
            for j in range(8):
                a=j*math.tau/8; px=x+.21*math.cos(a); py=y+.13*math.sin(a); h=rng.uniform(.10,.22)
                branch('Reed',(px,py,.01),(px+.015,py,h),.006,'reeds')
                cone('Cattail',(px+.015,py,h),.012,.01,.045,'bark',6)
    elif ident.startswith('FLOODPLAINS'):
        ps=[(-.78+i*.052,.22*math.sin(i*.19+variant),.025) for i in range(31)]
        ribbon('Alluvial_banks',[(x,y,z-.008) for x,y,z in ps],.16,'silt')
        ribbon('River_channel',ps,.095,'water')
        for k in range(20):
            x=rng.uniform(-.65,.65); y=.22*math.sin((x+.78)/.052*.19+variant)+rng.choice([-1,1])*.22
            cone('Riparian_grass',(x,y,.05),.025,0,.10,'reeds',5)
    elif ident=='OASIS':
        ellipsoid('Sand_rim',(0,0,.005),(.55,.38,.035),'sand')
        ellipsoid('Spring',(0,0,.025),(.43,.27,.02),'water')
        for x,y in [(-.4,.18),(.35,.25),(.08,.39)]: palm(x,y)
    elif ident=='DUNES':
        for k in range(3):
            verts=[]; faces=[]
            for j in range(25):
                x=-.67+j*.055
                for i in range(13):
                    y=-.18+i*.03; ridge=.08*math.sin(x*4+variant)
                    z=.19*max(0,1-abs(y)/.19)**(.6 if y<0 else 1.8)*max(0,1-(x/.7)**2)
                    verts.append((x,y+(k-1)*.36+ridge,z+.004))
            for j in range(24):
                for i in range(12):
                    n=j*13+i; faces.append((n,n+13,n+14,n+1))
            mesh=bpy.data.meshes.new('Dune'); mesh.from_pydata(verts,[],faces); mesh.update(); ob=bpy.data.objects.new('Wind_sculpted_dune',mesh); bpy.context.scene.collection.objects.link(ob); ob.parent=ROOTOBJ; mesh.materials.append(M['sand'])
    else:
        for i in range(10 if ident=='ICE' else 18):
            a=i*2.399; r=.65*math.sqrt((i+.5)/(10 if ident=='ICE' else 18)); x,y=math.cos(a)*r,math.sin(a)*r
            h=rng.uniform(.04,.10) if ident=='ICE' else rng.uniform(.12,.40)
            ob=cone('Ice_floe' if ident=='ICE' else 'Glacial_serac',(x,y,h/2),rng.uniform(.14,.23),rng.uniform(.10,.18),h,'ice',6)
            cone('Snow_cap',(x,y,h+.004),.12,.10,.016,'snow',6)

def run():
    global ROOTOBJ
    init_materials(); gallery=bpy.data.scenes.new('WayCiv_Features_Review'); records=[]
    for folder in ['blender/features','exports/features']: (OUT/folder).mkdir(parents=True,exist_ok=True)
    for idx,ident in enumerate(IDS):
        for v in range(3):
            name=f'FEATURE_{ident}_v{v:02d}'; scene=bpy.data.scenes.new(name); bpy.context.window.scene=scene; scene.unit_settings.system='METRIC'
            ROOTOBJ=bpy.data.objects.new(name,None); scene.collection.objects.link(ROOTOBJ); ROOTOBJ['catalog_id']='FEATURE_'+ident; ROOTOBJ['variant']=v
            ROOTOBJ['placement']='Overlay at terrain surface; scatter trees to terrain height on hills'; ROOTOBJ.asset_mark()
            build(ident,v)
            objs=list(scene.objects)
            for ob in objs: ob.select_set(True)
            bpy.context.view_layer.objects.active=ROOTOBJ
            blend=OUT/'blender/features'/f'{name}.blend'; fbx=OUT/'exports/features'/f'{name}.fbx'
            bpy.data.libraries.write(str(blend),{scene},fake_user=True,compress=True)
            bpy.ops.export_scene.fbx(filepath=str(fbx),use_selection=True,object_types={'EMPTY','MESH'},axis_forward='-Z',axis_up='Y',bake_anim=False,use_custom_props=True)
            records.append({'id':'FEATURE_'+ident,'variant':v,'blend':str(blend.relative_to(ROOT)),'fbx':str(fbx.relative_to(ROOT)),'meshes':sum(o.type=='MESH' for o in objs)})
            for ob in objs: gallery.collection.objects.link(ob)
            ROOTOBJ.location=((idx%4)*2.4,(idx//4)*7+v*2.1,0)
            bpy.context.window.scene=gallery; bpy.data.scenes.remove(scene)
    cam=bpy.data.cameras.new('Features_Camera'); ob=bpy.data.objects.new(cam.name,cam); gallery.collection.objects.link(ob); target=Vector((3.6,8.7,0)); ob.location=target+Vector((4,-15,21)); ob.rotation_euler=(target-ob.location).to_track_quat('-Z','Y').to_euler(); cam.type='ORTHO'; cam.ortho_scale=23; gallery.camera=ob
    sun=bpy.data.lights.new('Features_Sun','SUN'); sun.energy=3; ob=bpy.data.objects.new(sun.name,sun); gallery.collection.objects.link(ob); ob.rotation_euler=(.4,-.5,-.4)
    world=bpy.data.worlds.new('Features_World'); world.use_nodes=True; world.node_tree.nodes['Background'].inputs[0].default_value=(.12,.17,.23,1); world.node_tree.nodes['Background'].inputs[1].default_value=.7; gallery.world=world
    gallery.render.engine='CYCLES'; gallery.cycles.samples=24; gallery.cycles.use_denoising=True; gallery.render.resolution_x=1400; gallery.render.resolution_y=1600; gallery.render.resolution_percentage=100; gallery.render.filepath=str(OUT/'previews/features_review.png')
    (OUT/'feature_manifest.json').write_text(json.dumps({'assets':records},indent=2)+'\n')
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'blender/features_review.blend'))
    return {'feature_types':len(IDS),'assets':len(records),'mesh_objects':sum(r['meshes'] for r in records)}
if __name__=='__main__': result=run()
