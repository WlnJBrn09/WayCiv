"""Additional catalog districts authored for the editable 3D rebuild."""
import math
import bl
from bl import box,cyl,dome,cone,at
from matlib import M
from town import banner,tree

def observatory():
    g=bl.plot('grass')
    for i in range(5):
        box(0,.12,g+i*.055,.76-i*.09,.65-i*.075,.055,M('stone2'))
    cyl(0,.12,g+.275,.21,.22,M('stone'),n=24)
    dome(0,.12,g+.495,.23,M('stone2'),h=.15,n=24)
    for a in range(0,360,60):
        with at(0,.12,g+.32,rot=a):
            box(0,-.211,0,.055,.008,.095,M('dark'))
            box(0,-.219,.10,.08,.014,.02,M('stoneD'))
    # Wide processional stair and carved plinths.
    for i in range(10): box(0,-.49+i*.039,g+i*.025,.18,.043,.025,M('stone'))
    for x in [-.48,.48]:
        box(x,-.25,g,.11,.11,.12,M('stoneD')); box(x,-.25,g+.12,.14,.14,.025,M('stone'))
        tree(x,.35,.75,'round',g)

def hippodrome():
    g=bl.plot('pave')
    # Oval track, terraced seating, and a central spina with obelisks.
    for j in range(80):
        a=j*math.tau/80; b=(j+1)*math.tau/80
        pts=[(.76*math.cos(a),.49*math.sin(a),g+.01),(.76*math.cos(b),.49*math.sin(b),g+.01),(.53*math.cos(b),.28*math.sin(b),g+.01),(.53*math.cos(a),.28*math.sin(a),g+.01)]
        bl._face(M('sand'),pts)
        for level in range(3):
            rx=.77+level*.035; ry=.50+level*.035
            bl._face(M('stone'),[(rx*math.cos(a),ry*math.sin(a),g+.05+level*.045),(rx*math.cos(b),ry*math.sin(b),g+.05+level*.045),((rx+.035)*math.cos(b),(ry+.035)*math.sin(b),g+.05+level*.045),((rx+.035)*math.cos(a),(ry+.035)*math.sin(a),g+.05+level*.045)])
    box(0,0,g,.90,.095,.045,M('stoneD'))
    for x in [-.29,.29]:
        box(x,0,g+.045,.08,.08,.15,M('marble')); cone(x,0,g+.195,.057,.085,M('marble'),n=4)
    for x in [-.62,0,.62]: banner(x,.56,g,h=.26,w=.06,mat=M('royal'))
    box(0,.54,g,.34,.15,.20,M('stone'))
    for x in [-.12,-.04,.04,.12]: cyl(x,.45,g+.20,.012,.12,M('marble'),n=8)
    box(0,.51,g+.32,.37,.21,.035,M('terra'))

SPRITES={'district_observatory_1':(observatory,False),'district_hippodrome_1':(hippodrome,False)}
