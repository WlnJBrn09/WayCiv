"""Twenty separately authored natural-wonder compositions from WayCiv's catalog."""
import bpy, sys, math, json, random, tomllib
from pathlib import Path
from mathutils import Vector
ROOT=Path(__file__).resolve().parents[2]; OUT=ROOT/'art'
sys.path.insert(0,str(Path(__file__).parent))
import feature_models as f
from terrain_models import material
CAT=tomllib.loads('[[natural_wonder]]'+(ROOT/'crates/wayciv-data/data/terrain.toml').read_text().split('[[natural_wonder]]',1)[1])['natural_wonder']

def cube(name,p,s,mat):
    bpy.ops.mesh.primitive_cube_add(size=1,location=p); ob=bpy.context.object; ob.scale=s; return f.finish(ob,name,mat)

def terrain(name,fn,mat,extent=.77,snow=None):
    # Radial mesh keeps each landmark inside a hex tile without a square base.
    segments=96; rings=28; vs=[(0,0,fn(0,0))]; faces=[]
    for ring in range(1,rings+1):
        r=extent*ring/rings
        for j in range(segments):
            a=j*math.tau/segments; x,y=r*math.cos(a),r*math.sin(a)
            fade=min(1,(extent-r)/.09)
            vs.append((x,y,fn(x,y)*max(0,fade)))
    for j in range(segments): faces.append((0,1+j,1+(j+1)%segments))
    for ring in range(1,rings):
        a=1+(ring-1)*segments; b=a+segments
        for j in range(segments):
            k=(j+1)%segments; faces.extend([(a+j,b+j,b+k),(a+j,b+k,a+k)])
    me=bpy.data.meshes.new(name); me.from_pydata(vs,[],faces); me.update(); ob=bpy.data.objects.new(name,me); bpy.context.scene.collection.objects.link(ob); ob.parent=f.ROOTOBJ; me.materials.append(f.M[mat])
    if snow is not None:
        me.materials.append(f.M['snow'])
        for p in me.polygons:
            if p.center.z>snow: p.material_index=1
    return ob

def pool(name='Lake',sx=.72,sy=.62,mat='water',z=.018): return f.ellipsoid(name,(0,0,z),(sx,sy,.025),mat)
def peak(x,y,h,r,mat='granite'):
    return f.cone('Eroded_spire',(x,y,h/2),r,r*.10,h,mat,7)
def plants(rng,count=10,kind='FOREST'):
    for k in range(count):
        a=k*2.399; r=.55+(.12 if k%2 else 0); f.tree(math.cos(a)*r,math.sin(a)*r,rng.uniform(.18,.32),kind,rng)
def mountain(x,y,cx,cy,r,h,p=1.3): return h*max(0,1-math.hypot(x-cx,y-cy)/r)**p

def build(ident):
    rng=random.Random(sum(map(ord,ident)))
    if ident=='BARRIER_REEF':
        pool('Coral_lagoon',.79,.65)
        for k in range(34):
            a=k*2.399; r=.60*math.sqrt((k+.5)/34); x,y=math.cos(a)*r,math.sin(a)*r
            f.ellipsoid('Coral_bommie',(x,y,.045),(.09,.07,.045),'coral' if k%3 else 'coral_gold')
            for j in range(3): f.branch('Coral_branch',(x,y,.05),(x+.04*math.cos(j*2),y+.04*math.sin(j*2),.13),.012,'coral')
    elif ident=='CLIFFS_DOVER':
        pool('Channel',.80,.62)
        for k in range(12):
            x=-.66+k*.12; y=.08+.06*math.sin(k)
            cube('Chalk_buttress',(x,y,.24),(.135,.44,.47),'chalk'); cube('Turf_cap',(x,y,.48),(.135,.44,.03),'leaf')
    elif ident=='CRATER_LAKE':
        pool('Caldera_lake',.53,.53,'deepwater',.08)
        terrain('Caldera_rim',lambda x,y:.035+.38*math.exp(-((math.hypot(x,y)-.58)/.10)**2)*(1+.13*math.sin(math.atan2(y,x)*9)),'granite')
        peak(-.23,-.10,.18,.13,'pine')
    elif ident=='DEAD_SEA':
        pool('Salt_shore',.74,.49,'chalk'); pool('Brine',.65,.37,'water',.045)
        for side in [-1,1]:
            for k in range(8): f.ellipsoid('Salt_crust',(k*.18-.63,side*.40,.04),(.12,.035,.025),'chalk')
        terrain('Desert_escarpment',lambda x,y:.29*math.exp(-((y-.55)/.16)**2)*(1+.2*math.sin(x*14)),'sand')
    elif ident=='EVEREST':
        terrain('Everest_massif',lambda x,y:max(mountain(x,y,.08,.1,.72,1.28),mountain(x,y,-.38,.0,.4,.61),mountain(x,y,.39,.34,.4,.72))*(.94+.06*math.cos(x*36+y*14)),'granite',snow=.43)
        f.ribbon('Glacier',[(.0,-.05,.27),(-.12,-.24,.17),(-.09,-.48,.07),(-.19,-.70,.02)],.06,'ice')
    elif ident=='GALAPAGOS':
        pool('Pacific',.79,.69,'deepwater')
        for x,y,r,h in [(-.30,.1,.29,.28),(.30,.30,.22,.21),(.28,-.3,.25,.18),(-.35,-.4,.16,.13)]:
            f.ellipsoid('Lava_island',(x,y,.055),(r,r*.7,.07),'basalt'); f.cone('Shield_volcano',(x,y,h/2+.02),r*.7,r*.16,h,'basalt'); f.ellipsoid('Island_vegetation',(x+.05,y,.08),(r*.50,r*.4,.06),'leaf')
        f.ellipsoid('Tortoise_shell',(-.30,.0,.14),(.07,.05,.04),'sand')
    elif ident=='KILIMANJARO':
        terrain('Kibo_shield',lambda x,y:.84*max(0,1-(math.hypot(x,y)/.82))**.65-.12*math.exp(-(math.hypot(x,y)/.13)**2),'granite',snow=.60)
        plants(rng,8)
    elif ident=='PANTANAL':
        pool('Wetland',.78,.68)
        for k in range(15):
            x,y=rng.uniform(-.6,.6),rng.uniform(-.5,.5); f.ellipsoid('Marsh_islet',(x,y,.038),(.13,.08,.028),'reeds')
            for j in range(4): f.branch('Rush',(x+j*.02-.03,y,.04),(x+j*.02-.03,y,.13),.005,'reeds')
        plants(rng,5)
    elif ident in ['PIOPIOTAHI','LYSEFJORDEN']:
        pool('Fjord',.75,.69,'deepwater')
        terrain('Fjord_walls',lambda x,y:(.58+.17*math.sin(y*5))*math.exp(-((abs(x)-.49)/.17)**4),'granite')
        if ident=='PIOPIOTAHI':
            peak(.46,.28,.93,.28,'pine')
            f.ribbon('Waterfall',[(-.34,.2,.65),(-.32,.2,.48),(-.30,.2,.28),(-.27,.2,.04)],.025,'snow')
        else: cube('Pulpit_Rock',(-.30,.0,.62),(.37,.30,.09),'granite')
    elif ident=='TORRES_DEL_PAINE':
        pool('Glacial_lake',.73,.60,'ice')
        for x,y,h,r in [(-.32,.16,.82,.15),(0,.25,1.06,.17),(.31,.15,.91,.14)]: peak(x,y,h,r)
        terrain('Scree_apron',lambda x,y:.18*math.exp(-((y-.17)/.35)**2)*(1+.2*math.sin(x*17)),'granite')
    elif ident=='TSINGY':
        for k in range(55):
            a=k*2.399; r=.7*math.sqrt((k+.5)/55); peak(math.cos(a)*r,math.sin(a)*r,rng.uniform(.19,.57),.075,'chalk')
        plants(rng,6)
    elif ident=='YOSEMITE':
        terrain('Valley_floor',lambda x,y:.03+.03*math.sin(y*4),'leaf')
        cube('El_Capitan',(-.44,.12,.34),(.38,.66,.66),'granite')
        dome=f.ellipsoid('Half_Dome',(.38,.29,.35),(.27,.32,.52),'granite')
        # Compress the west slope to a near-vertical face without collapsing polygons.
        for v in dome.data.vertices:
            if v.co.x<0: v.co.x*=.08
        f.ribbon('Merced_River',[(.08,-.73,.06),(-.10,-.42,.06),(.04,-.1,.06),(.0,.32,.06)],.035,'water')
        plants(rng,7)
    elif ident=='ULURU':
        terrain('Sandstone_monolith',lambda x,y:.46*max(0,1-(abs(x)/.70)**6-(abs(y)/.40)**6)**.22*(.95+.05*math.sin(x*38)),'redrock')
    elif ident=='HA_LONG_BAY':
        pool('Emerald_bay',.79,.70)
        for k in range(11):
            a=k*2.399; r=.64*math.sqrt((k+.5)/11); x,y=math.cos(a)*r,math.sin(a)*r; h=rng.uniform(.20,.55)
            f.cone('Karst_tower',(x,y,h/2),.13,.085,h,'chalk',9); f.ellipsoid('Karst_canopy',(x,y,h),(.10,.09,.04),'leaf')
    elif ident=='EYJAFJALLAJOKULL':
        terrain('Ice_cap',lambda x,y:.62*max(0,1-(x/.75)**2-(y/.63)**2)**.7,'basalt',snow=.20)
        for k in range(6):
            a=k*math.tau/6; f.ribbon('Glacial_tongue',[(math.cos(a)*r,math.sin(a)*r,.53*(1-r/.8)) for r in [.25,.4,.6,.74]],.04,'ice')
    elif ident=='GIANTS_CAUSEWAY':
        pool('Atlantic',.80,.70,'deepwater')
        for i in range(-6,7):
            for j in range(-5,6):
                x=i*.102; y=(j+(i%2)*.5)*.118
                if math.hypot(x,y)>.70: continue
                h=.07+.20*(y+.7)/1.4+.03*rng.random()
                f.cone('Basalt_column',(x,y,h/2),.065,.065,h,'basalt',6)
    elif ident=='BERMUDA_TRIANGLE':
        pool('Atlantic_anomaly',.80,.72,'deepwater')
        for k in range(4):
            ps=[((.12+k*.14)*math.cos(a),(.12+k*.14)*math.sin(a),.05) for a in [j*math.tau/64 for j in range(65)]]
            f.ribbon('Current_ring',ps,.012,'water')
        for a in [math.pi/2,math.pi*7/6,math.pi*11/6]:
            x,y=.61*math.cos(a),.61*math.sin(a); f.ellipsoid('Boundary_islet',(x,y,.05),(.10,.06,.04),'sand')
    elif ident=='FOUNTAIN_OF_YOUTH':
        pool('Limestone_basin',.48,.40,'chalk'); pool('Spring_water',.38,.29,'water',.052)
        for k in range(8):
            a=k*math.tau/8; f.ellipsoid('Spring_rocks',(.44*math.cos(a),.36*math.sin(a),.08),(.09,.08,.08),'chalk')
        f.cone('Bubbling_spring',(0,0,.14),.055,.022,.20,'ice',12); plants(rng,9,'JUNGLE')
    elif ident=='PAITITI':
        for level in range(5): cube('Temple_terrace',(0,.1,.055+level*.09),(.66-level*.11,.59-level*.09,.09),'goldstone')
        for k in range(9): cube('Temple_stair',(0,-.44+k*.047,.025+k*.028),(.15,.055,.05+k*.056),'goldstone')
        for x in [-.48,.47]:
            cube('Ruined_hall',(x,.2,.10),(.22,.35,.20),'goldstone')
        plants(rng,10,'JUNGLE')
    else: raise ValueError(ident)

def run():
    f.init_materials()
    for key,color in {'granite':(.40,.43,.45),'chalk':(.78,.79,.68),'basalt':(.11,.14,.16),'redrock':(.58,.20,.08),'deepwater':(.025,.12,.25),'coral':(.66,.20,.16),'coral_gold':(.66,.48,.12),'goldstone':(.58,.43,.18)}.items(): f.M[key]=material('Wonder_'+key,color)
    gallery=bpy.data.scenes.new('WayCiv_Natural_Wonders_Review'); records=[]
    for path in ['blender/natural_wonders','exports/natural_wonders']: (OUT/path).mkdir(parents=True,exist_ok=True)
    for idx,entry in enumerate(CAT):
        ident=entry['id']; scene=bpy.data.scenes.new(ident); bpy.context.window.scene=scene; scene.unit_settings.system='METRIC'
        root=bpy.data.objects.new(ident,None); scene.collection.objects.link(root); root['catalog_id']=ident; root['display_name']=entry['name']; root['description']=entry['description']; root.asset_mark(); f.ROOTOBJ=root
        build(ident.removeprefix('FEATURE_'))
        objects=list(scene.objects)
        for ob in objects: ob.select_set(True)
        bpy.context.view_layer.objects.active=root
        blend=OUT/'blender/natural_wonders'/f'{ident}.blend'; fbx=OUT/'exports/natural_wonders'/f'{ident}.fbx'
        bpy.data.libraries.write(str(blend),{scene},fake_user=True,compress=True)
        bpy.ops.export_scene.fbx(filepath=str(fbx),use_selection=True,object_types={'EMPTY','MESH'},axis_forward='-Z',axis_up='Y',bake_anim=False,use_custom_props=True)
        records.append({'id':ident,'name':entry['name'],'variant':0,'blend':str(blend.relative_to(ROOT)),'fbx':str(fbx.relative_to(ROOT)),'meshes':sum(o.type=='MESH' for o in objects)})
        for ob in objects: gallery.collection.objects.link(ob)
        root.location=((idx%5)*2.6,(idx//5)*3.0,0)
        bpy.context.window.scene=gallery; bpy.data.scenes.remove(scene)
        curve=bpy.data.curves.new(ident+'_label','FONT'); curve.body=entry['name']; curve.size=.14; curve.align_x='CENTER'; ob=bpy.data.objects.new(curve.name,curve); gallery.collection.objects.link(ob); ob.location=(root.location.x,root.location.y-1.0,0); curve.materials.append(f.M['snow'])
    cam=bpy.data.cameras.new('Wonders_Camera'); ob=bpy.data.objects.new(cam.name,cam); gallery.collection.objects.link(ob); target=Vector((5.2,4.2,0)); ob.location=target+Vector((0,-12,18)); ob.rotation_euler=(target-ob.location).to_track_quat('-Z','Y').to_euler(); cam.type='ORTHO'; cam.ortho_scale=15; gallery.camera=ob
    sun=bpy.data.lights.new('Wonders_Sun','SUN'); sun.energy=3; ob=bpy.data.objects.new(sun.name,sun); gallery.collection.objects.link(ob); ob.rotation_euler=(.4,-.5,-.4)
    world=bpy.data.worlds.new('Wonders_World'); world.use_nodes=True; world.node_tree.nodes['Background'].inputs[0].default_value=(.12,.17,.23,1); world.node_tree.nodes['Background'].inputs[1].default_value=.6; gallery.world=world
    gallery.render.engine='CYCLES'; gallery.cycles.samples=24; gallery.cycles.use_denoising=True; gallery.render.resolution_x=1800; gallery.render.resolution_y=1500; gallery.render.resolution_percentage=100; gallery.render.filepath=str(OUT/'previews/natural_wonders_review.png')
    (OUT/'natural_wonder_manifest.json').write_text(json.dumps({'assets':records},indent=2)+'\n')
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'blender/natural_wonders_review.blend'))
    return {'assets':len(records),'meshes':sum(r['meshes'] for r in records)}
if __name__=='__main__': result=run()
