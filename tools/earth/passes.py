#!/usr/bin/env python3
"""Carves mountain passes into the Earth maps so land routes that exist in
real life (roads through the Sierra Madre, for example) exist in the game.

For each route it finds the land path that crosses the fewest mountain
tiles and turns those mountains into hills of the same biome. Run it after
build_earth.py:

    python tools/earth/passes.py            # report and fix every map
    python tools/earth/passes.py --check    # report only

Map files: assets/maps/earth_<w>x<h>.txt (format in crates/wayciv-core/src/earth.rs).
"""
import heapq
import math
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
MAPS = sorted((ROOT / "assets/maps").glob("earth_*.txt"))

# (name, from lat/lon, to lat/lon)
ROUTES = [
    ("United States to Mexico", (38.9, -77.0), (19.4, -99.1)),
    ("Mexico to Central America", (19.4, -99.1), (14.6, -90.5)),
    ("Central America to South America", (14.6, -90.5), (4.7, -74.1)),
]

FLAT = set("gpdtsGPDTS")
MOUNTAIN_TO_HILLS = {"1": "G", "2": "P", "3": "D", "4": "T", "5": "S"}


def miller(lat):
    return 1.25 * math.log(math.tan(math.pi / 4 + 0.4 * math.radians(lat)))


class EarthMap:
    def __init__(self, path):
        self.path = path
        lines = path.read_text().split("\n")
        self.header = lines[0]
        h = self.header.split()
        self.w, self.h = int(h[1]), int(h[2])
        self.lat_top, self.lon_west = float(h[3]), float(h[5])
        self.rows = [list(r) for r in lines[1:1 + 3 * self.h]]
        self.rest = lines[1 + 3 * self.h:]

    def tile_of(self, lat, lon):
        """Same math as Projection::tile_of in earth.rs."""
        hex_w = 2 * math.pi / self.w
        size = hex_w / math.sqrt(3)
        row_h = 1.5 * size
        px = math.radians((lon - self.lon_west) % 360) - hex_w / 2
        py = miller(self.lat_top) - miller(lat) - row_h / 2
        q = (math.sqrt(3) / 3 * px - py / 3) / size
        r = (2 / 3 * py) / size
        x, z = q, r
        y = -x - z
        rx, ry, rz = round(x), round(y), round(z)
        dx, dy, dz = abs(rx - x), abs(ry - y), abs(rz - z)
        if dx > dy and dx > dz:
            rx = -ry - rz
        elif dy <= dz:
            rz = -rx - ry
        return ((rx + (rz - (rz & 1)) // 2) % self.w, rz)

    def terrain(self, x, y):
        return self.rows[y][x]

    def neighbors(self, x, y):
        odd = y & 1
        for dx, dy in [(1, 0), (-1, 0), (odd, -1), (odd - 1, -1), (odd, 1), (odd - 1, 1)]:
            nx, ny = (x + dx) % self.w, y + dy
            if 0 <= ny < self.h:
                yield nx, ny

    def cheapest_route(self, start, goal):
        """Land path crossing the fewest mountains; None if water cuts it."""
        def cost(t):
            if t in FLAT:
                return 1
            if t in MOUNTAIN_TO_HILLS:
                return 1000
            return None

        dist = {start: 0}
        prev = {}
        queue = [(0, start)]
        while queue:
            d, at = heapq.heappop(queue)
            if at == goal:
                break
            if d > dist[at]:
                continue
            for n in self.neighbors(*at):
                c = cost(self.terrain(*n))
                if c is None:
                    continue
                if d + c < dist.get(n, math.inf):
                    dist[n] = d + c
                    prev[n] = at
                    heapq.heappush(queue, (d + c, n))
        if goal not in dist:
            return None
        path = [goal]
        while path[-1] != start:
            path.append(prev[path[-1]])
        return path[::-1]

    def save(self):
        lines = [self.header] + ["".join(r) for r in self.rows] + self.rest
        self.path.write_text("\n".join(lines))


def main():
    check = "--check" in sys.argv
    for path in MAPS:
        m = EarthMap(path)
        changed = False
        for name, a, b in ROUTES:
            start, goal = m.tile_of(*a), m.tile_of(*b)
            # The endpoints themselves may be peaks (Mexico City sits in a
            # high valley that the grid rounds to a mountain).
            for p in (start, goal):
                if m.terrain(*p) in MOUNTAIN_TO_HILLS and not check:
                    m.rows[p[1]][p[0]] = MOUNTAIN_TO_HILLS[m.terrain(*p)]
                    changed = True
            path_ = m.cheapest_route(start, goal)
            if path_ is None:
                print(f"{path.name}: {name}: no land route (water)")
                continue
            peaks = [p for p in path_ if m.terrain(*p) in MOUNTAIN_TO_HILLS]
            if not peaks:
                print(f"{path.name}: {name}: open")
                continue
            print(f"{path.name}: {name}: blocked by {len(peaks)} mountain tiles at {peaks}")
            if not check:
                for x, y in peaks:
                    m.rows[y][x] = MOUNTAIN_TO_HILLS[m.terrain(x, y)]
                changed = True
        if changed and not check:
            m.save()
            print(f"{path.name}: passes carved")


if __name__ == "__main__":
    main()
