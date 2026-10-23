#!/bin/bash
# Rebuilds every unit sprite: Blender renders (beauty + team mask), then Pillow
# post-processing into assets/units/ (sprites, masks, portraits, manifest).
#   tools/units/build_all.sh [PYTHON_WITH_PILLOW_AND_NUMPY]
# Needs: blender 4.2+ on PATH.  Cycles on CPU, 4 threads per process, 2 processes.
set -e
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../.." && pwd)
PY=${1:-python3}
RAW=${RAW:-$(mktemp -d)}
mkdir -p "$RAW"
OUT=$ROOT/assets/units
ids=$(blender -b --factory-startup -P /dev/stdin <<PYEOF 2>/dev/null | grep '^IDS ' | cut -c5-
import sys
sys.path.insert(0, "$HERE")
import catalog
print("IDS " + " ".join(catalog.all_ids()))
PYEOF
)
set -- $ids
half=$(( ($# + 1) / 2 ))
A=${@:1:$half}
B=${@:$((half + 1))}
blender -b --factory-startup -t 4 -P "$HERE/render.py" -- --out "$RAW" $A > "$RAW/a.log" 2>&1 &
blender -b --factory-startup -t 4 -P "$HERE/render.py" -- --out "$RAW" $B > "$RAW/b.log" 2>&1 &
wait
mkdir -p "$OUT"
"$PY" "$HERE/post.py" "$RAW" "$OUT"
echo "done: $(ls "$OUT"/*.png | wc -l) sprites in $OUT"
