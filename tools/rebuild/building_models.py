"""Standalone models for every regular and civilization-unique building."""
import sys,math,re,random
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]; sys.path.insert(0,str(ROOT/'tools/buildings')); sys.path.insert(0,str(Path(__file__).parent))
import bl,parts as p,town,medieval,industrial,modern,cities
from bl import box,cyl,cone,dome,at,sphere
from matlib import M
from architecture_models import main
IDS=sorted(set(re.findall(r'id = "(BUILDING_[A-Z_]+)"',''.join((ROOT/'crates/wayciv-data/data'/name).read_text() for name in ['buildings.toml','civilizations.toml']))))

def hall(w=.6,d=.4,h=.30,wall='stone',roof='terra',x=0,y=0):
    p.house(x,y,0,w,d,h,wall,roof); p.door(x,y,0,w,d,dw=.09,dh=.14)
def wheel(x,y,z,r=.17):
    # Axle lies along X; spokes and paddles remain true 3D geometry.
    for j in range(16):
        a=j*math.tau/16; b=(j+1)*math.tau/16
        town.rod((x,y+math.cos(a)*r,z+math.sin(a)*r),(x,y+math.cos(b)*r,z+math.sin(b)*r),.012,M('woodD'))
        town.rod((x,y,z),(x,y+math.cos(a)*r,z+math.sin(a)*r),.007,M('wood'))
        box(x,y+math.cos(a)*r,z+math.sin(a)*r,.10,.025,.025,M('woodL'))
def arena(ball=False,modern_style=False):
    for j in range(64):
        a=j*math.tau/64; b=(j+1)*math.tau/64
        for k in range(4):
            rx=.40+k*.045; ry=.24+k*.04; z=.035+k*.04
            bl._face(M('concrete' if modern_style else 'stone'),[(rx*math.cos(a),ry*math.sin(a),z),(rx*math.cos(b),ry*math.sin(b),z),((rx+.04)*math.cos(b),(ry+.035)*math.sin(b),z),((rx+.04)*math.cos(a),(ry+.035)*math.sin(a),z)])
    box(0,0,0,.70,.36,.015,M('grass' if modern_style else 'sand'))
    if ball:
        for x in [-.30,.30]: box(x,0,.015,.015,.30,.005,M('white'))
    if modern_style:
        for x in [-.5,.5]:
            for y in [-.35,.35]: cyl(x,y,0,.013,.45,M('steel')); box(x,y,.45,.12,.04,.055,M('white'))

def build(key):
    if key in ['WALLS','CASTLE','STAR_FORT']:
        style={'WALLS':1,'CASTLE':1,'STAR_FORT':2}[key]
        cities.walls(style,'front')(); cities.walls(style,'back')()
        if key=='CASTLE': medieval.keep(0,.10,0,w=.45,d=.36,h=.45)
        return
    if key=='PALACE':
        p.temple(0,0,0,.72,.48,ncols=6,ch=.28); dome(0,.03,.40,.17,M('gold'),h=.16)
        for x in [-.46,.46]: p.tower(x,.12,0,.09,.44,roof='slate')
    elif key=='MONUMENT':
        bl.steps(0,0,0,.36,.36,3,.035,.035,M('stone')); box(0,0,.105,.10,.10,.34,M('marble'),taper=.65); bl.pyr(0,0,.445,.065,.065,.065,M('marble'))
    elif key=='GRANARY':
        hall(.42,.32,.25,'adobe','thatch',x=-.15)
        for x,y in [(.22,-.12),(.22,.12)]: cyl(x,y,0,.095,.30,M('sand')); cone(x,y,.30,.11,.12,M('thatch'))
        for x in [-.3,-.15,0]: p.barrel(x,-.26,0,r=.04,h=.07)
    elif key in ['WATER_MILL','PALGUM']:
        hall(.45,.35,.27,'brick' if key=='PALGUM' else 'stone','terra'); wheel(.28,0,.20)
        box(.28,0,-.025,.18,.80,.025,M('water'))
        if key=='PALGUM':
            for y in [-.35,.35]: p.wall_seg(.18,y,.65,y,0,h=.055,wall='adobe',crenel=False)
    elif key=='SEWER':
        bl.arch_wall(0,0,0,.60,.30,.20,M('brickD'),iw=.35,ih=.23); box(0,-.28,-.015,.32,.55,.015,M('waterD'))
        for x in [-.12,-.06,0,.06,.12]: box(x,-.11,0,.012,.014,.20,M('iron'))
    elif key=='SOLAR_FARM':
        for x in [-.30,0,.30]:
            for y in [-.20,.10]: p.solar_panel(x,y,.04,w=.25,d=.18)
        box(.40,.33,0,.13,.15,.13,M('concrete'))
    elif key in ['LIBRARY','UNIVERSITY','MADRASA','NAVIGATION_SCHOOL']:
        if key=='LIBRARY':
            p.temple(0,0,0,.60,.40,ncols=4,ch=.23)
            for x in [-.18,0,.18]: box(x,-.23,.12,.10,.035,.02,M('woodL'))
        elif key=='MADRASA':
            for x in [-.27,.27]: hall(.20,.62,.22,'sand','turq',x=x)
            hall(.74,.20,.24,'sand','turq',y=.26); p.pool(0,0,.01,.25,.32); dome(0,.26,.34,.13,M('turq'))
        else:
            hall(.62,.30,.33,'brick','slate'); p.tower(-.36,.05,0,.10,.55,roof='slate')
            if key=='NAVIGATION_SCHOOL': p.dish(.24,.02,.47,r=.12); box(.34,-.32,0,.18,.22,.015,M('water')); p.ship(.34,-.32,.02,L=.16,W=.05)
    elif key in ['RESEARCH_LAB','GPU_CLUSTER','BROADCAST_CENTER','FILM_STUDIO']:
        hall(.60,.40,.26,'concrete','slate')
        if key=='RESEARCH_LAB':
            for x in [-.18,.05]: dome(x,0,.39,.10,M('white')); p.dish(.33,0,.12,r=.12)
        elif key=='GPU_CLUSTER':
            for x in [-.2,0,.2]:
                for y in [-.07,.10]: p.server_rack(x,y,.38,w=.10,d=.12,h=.16)
            for x in [-.18,.18]: p.fan(x,-.30,.06,r=.08)
        elif key=='BROADCAST_CENTER':
            town.rod((.36,0,0),(.36,0,.85),.012,M('steel')); p.dish(.36,0,.48,r=.11); box(0,-.22,.18,.30,.01,.09,M('blue'))
        else:
            for x in [-.35,.35]:
                town.rod((x,-.33,0),(x,-.33,.33),.01,M('steel')); sphere(x,-.33,.34,.05,M('dark'))
            box(0,-.30,.03,.28,.02,.13,M('dark')); box(0,-.30,.16,.28,.03,.035,M('white'),rot=10)
    elif key in ['SHRINE','TEMPLE','PRASAT','STAVE_CHURCH']:
        if key=='SHRINE': p.temple(0,0,0,.32,.30,ncols=2,ch=.16)
        elif key=='TEMPLE': p.temple(0,0,0,.64,.42,ncols=6,ch=.30)
        elif key=='STAVE_CHURCH':
            hall(.32,.52,.28,'woodD','woodD')
            for k in range(3): bl.hip(0,.08,.32+k*.12,.42-k*.09,.40-k*.08,.13,M('woodD'))
            p.tower(0,.06,.58,.055,.18,wall='woodD',roof='woodD',rh=.14)
        else:
            bl.steps(0,0,0,.55,.50,4,.04,.04,M('stoneD'))
            for k in range(5): box(0,0,.16+k*.075,.27-k*.035,.27-k*.035,.075,M('sand'),taper=.8)
            cone(0,0,.535,.045,.17,M('gold'))
    elif key in ['BARRACKS','BASILIKOI_PAIDES','ARMORY','MILITARY_ACADEMY','STABLE']:
        if key=='STABLE':
            hall(.65,.26,.20,'wood','thatch',y=.15)
            for x in [-.3,0,.3]: p.fence(x,-.3,x,.02,0,h=.10)
            p.fence(-.3,-.3,.3,-.3,0,h=.10)
        else:
            hall(.62,.38,.25 if key=='BARRACKS' else .37,'stone','terra')
            for x in [-.20,0,.20]:
                if key=='ARMORY': town.rod((x,-.3,.02),(x,-.3,.23),.012,M('iron')); sphere(x,-.3,.10,.035,M('steel'))
                else: town.banner(x,-.3,0,h=.25,w=.07,mat=M('royal' if key=='BASILIKOI_PAIDES' else 'red'))
            if key=='MILITARY_ACADEMY': p.tower(.37,.06,0,.09,.50,crenel=True)
    elif key in ['MARKET','SUKIENNICE','BANK','STOCK_EXCHANGE']:
        if key=='MARKET':
            for x in [-.28,0,.28]: p.stall(x,0,0,w=.22,d=.18,awning='red' if x else 'cream')
            for x in [-.28,.28]: p.barrel(x,.2,0,r=.045,h=.08)
        elif key=='SUKIENNICE':
            hall(.80,.30,.30,'brick','terra')
            for x in [-.30,-.15,0,.15,.30]: bl.arch_wall(x,-.20,0,.14,.19,.06,M('stone'),iw=.09,ih=.14)
        else:
            p.temple(0,0,0,.66,.40,ncols=6,ch=.28)
            if key=='STOCK_EXCHANGE': box(0,0,.39,.45,.30,.24,M('glassD')); p.windows(0,0,.39,.45,.3,.24,5,3)
            else: sphere(0,-.24,.28,.04,M('gold'))
    elif key in ['AMPHITHEATER','ARENA','STADIUM','TLACHTLI']:
        arena(key in ['STADIUM','TLACHTLI'],key=='STADIUM')
        if key=='AMPHITHEATER': box(0,.26,0,.45,.13,.16,M('stone')); bl.gable(0,.26,.16,.5,.18,.08,M('terra'))
        if key=='TLACHTLI':
            for x in [-.36,.36]: p.wall_seg(x,-.25,x,.25,.02,h=.23,wall='sand',crenel=False)
    elif key in ['MUSEUM_ART','MUSEUM_ARTIFACT']:
        p.temple(0,0,0,.68,.43,ncols=6,ch=.26)
        for x in [-.24,0,.24]:
            if key=='MUSEUM_ART': box(x,-.32,.06,.10,.012,.15,M('royal' if x else 'red')); box(x,-.32,0,.11,.06,.06,M('stone'))
            else: cyl(x,-.32,0,.045,.05,M('stone')); bl.lathe(x,-.32,.05,[(.02,0),(.04,.05),(.015,.09),(.025,.10)],M('terra'),n=12)
    elif key in ['LIGHTHOUSE','SEAPORT','SHIPYARD']:
        if key=='LIGHTHOUSE':
            cyl(0,0,0,.12,.50,M('white'),r2=.075); cyl(0,0,.50,.105,.08,M('glass')); cone(0,0,.58,.13,.10,M('red'))
            p.house(.25,.10,0,.25,.25,.17)
        else:
            box(0,0,-.05,.70,.55,.05,M('water')); box(0,.28,0,.78,.15,.10,M('stone'))
            for x in [-.30,.30]: box(x,-.10,.015,.12,.65,.05,M('wood'))
            p.ship(0,0,.01,L=.50,W=.16,sail=key=='SEAPORT')
            if key=='SHIPYARD':
                for x in [-.30,.30]: box(x,.10,.05,.035,.04,.48,M('steel'))
                box(0,.10,.53,.66,.04,.04,M('steel'))
    elif key in ['FACTORY','ELECTRONICS_FACTORY','WORKSHOP','POWER_PLANT']:
        industrial.factory(0,0,0,big=key!='WORKSHOP')
        if key=='POWER_PLANT':
            for x in [-.35,.35]: bl.lathe(x,.18,0,[(.13,0),(.08,.18),(.10,.32)],M('concrete'),n=20)
        elif key=='ELECTRONICS_FACTORY':
            for x in [-.24,0,.24]: p.solar_panel(x,.0,.42,w=.17,d=.12)
            p.server_rack(.36,-.25,0,w=.10,d=.13,h=.20)
        elif key=='WORKSHOP':
            for x in [-.22,0,.22]: p.crate(x,-.30,0,s=.09)
    elif key in ['HANGAR','AIRPORT']:
        from districts_special import hangar
        hangar(0,.12,0,w=.65,d=.42,h=.27)
        p.plane(0,-.25,.04,s=.65)
        if key=='AIRPORT':
            box(.39,.15,0,.12,.12,.44,M('concrete')); box(.39,.15,.44,.20,.18,.10,M('glassD')); p.dish(.39,.15,.55,r=.06)
    elif key=='ZOO':
        for x in [-.25,.25]:
            for y in [-.22,.22]: p.fence(x-.19,y-.16,x+.19,y-.16,0,h=.12); p.fence(x-.19,y-.16,x-.19,y+.16,0,h=.12)
            p.pool(x,0,.01,.20,.15,rim='rock'); bl.tree(x,.19,.75,'round')
            sphere(x,-.04,.08,.07,M('sandD')); sphere(x+.06,-.04,.13,.035,M('sandD'))
    else: raise ValueError('Missing bespoke building: '+key)

def run():
    sources={ident.lower():(lambda key=ident.removeprefix('BUILDING_'):build(key),'../rebuild/building_models') for ident in IDS}
    return main(sources,'building')
if __name__=='__main__': result=run()
