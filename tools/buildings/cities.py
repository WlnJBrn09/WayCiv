"""City-center sprites.

    city_<era>_<tier>     era 0 ancient, 1 medieval, 2 industrial, 3 modern;
                          tier 1..4 = population 1-3, 4-8, 9-15, 16+
    capital_<era>_<tier>  the same with a palace
    walls_<style>_<back|front>   style 0 palisade, 1 stone, 2 modern;
                          drawn before / after the city when it has walls

Owner-coloured parts use the 'acc' material (banners, flags, some roofs);
the _tint.png layer is painted over the sprite in the owner's colour.
The eras live in town.py (ancient), medieval.py, industrial.py and modern.py.
"""
import industrial
import medieval
import modern
import town
from bl import _hex_pts
from parts import *  # noqa: F401,F403

ERAS = [town.ancient, medieval.build, industrial.build, modern.build]


def walls(style, part):
    def b():
        r = 0.84
        pts = _hex_pts(r)

        def seg(a, c2):
            # split side edges at y = 0 so back/front parts line up
            pieces = [(a, c2)]
            if (a[1] > 0 > c2[1]) or (a[1] < 0 < c2[1]):
                t = a[1] / (a[1] - c2[1])
                m = (a[0] + (c2[0] - a[0]) * t, 0.0)
                pieces = [(a, m), (m, c2)]
            for (p, q) in pieces:
                my = (p[1] + q[1]) / 2
                if (my > 0.0) == (part == 'back'):
                    if style == 0:
                        palisade(p[0], p[1], q[0], q[1], 0.0, h=0.17, mat='woodL')
                    elif style == 1:
                        wall_seg(p[0], p[1], q[0], q[1], 0.0, h=0.17, t=0.06, wall='stone')
                    else:
                        wall_seg(p[0], p[1], q[0], q[1], 0.0, h=0.12, t=0.07, wall='concrete', crenel=False)
                        wall_seg(p[0], p[1], q[0], q[1], 0.12, h=0.02, t=0.05, wall='concreteD', crenel=False)

        for i in range(6):
            seg(pts[i], pts[(i + 1) % 6])
        for p in pts:
            if (p[1] > 0.05) == (part == 'back'):
                if style == 0:
                    cyl(p[0], p[1], 0.0, 0.03, 0.26, M('woodD'), n=6)
                    box(p[0], p[1], 0.26, 0.1, 0.1, 0.05, M('woodL'))
                    cone(p[0], p[1], 0.31, 0.08, 0.08, M('thatch'), n=6)
                elif style == 1:
                    tower(p[0], p[1], 0.0, 0.065, 0.3, 'stone', roof='terra', rh=0.17)
                else:
                    box(p[0], p[1], 0.0, 0.12, 0.12, 0.16, M('concreteD'), rot=45)
                    cyl(p[0], p[1], 0.16, 0.025, 0.1, M('steel'), n=6)
                    box(p[0], p[1], 0.26, 0.05, 0.05, 0.015, M('red'))
    return b


SPRITES = {}
for _e, _fn in enumerate(ERAS):
    for _t in (1, 2, 3, 4):
        SPRITES[f'city_{_e}_{_t}'] = (_fn(_t, False), True)
        SPRITES[f'capital_{_e}_{_t}'] = (_fn(_t, True), True)
for _s in (0, 1, 2):
    for _p in ('back', 'front'):
        SPRITES[f'walls_{_s}_{_p}'] = (walls(_s, _p), False)
