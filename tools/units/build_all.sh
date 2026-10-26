#!/bin/bash
# Rebuilds every unit sprite: Blender renders (beauty + team mask), then Pillow
# post-processing into assets/units/ (sprites, masks, portraits, manifest).
#   tools/units/build_all.sh [PYTHON_WITH_PILLOW_AND_NUMPY]
# Needs: blender 4.2+ on PATH.  Cycles on CPU: PROCS (3) processes x THREADS (5) threads; ~25 min for all units.
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
# round-robin the ids over $PROCS blender processes (long models are spread evenly)
PROCS=${PROCS:-3}
THREADS=${THREADS:-5}
for ((p = 0; p < PROCS; p++)); do
  chunk=()
  i=0
  for id in "$@"; do
    (( i % PROCS == p )) && chunk+=("$id")
    i=$((i + 1))
  done
  blender -b --factory-startup -t "$THREADS" -P "$HERE/render.py" -- --out "$RAW" "${chunk[@]}" > "$RAW/render$p.log" 2>&1 &
done
wait
mkdir -p "$OUT"
"$PY" "$HERE/post.py" "$RAW" "$OUT"
echo "done: $(ls "$OUT"/*.png | wc -l) sprites in $OUT"
