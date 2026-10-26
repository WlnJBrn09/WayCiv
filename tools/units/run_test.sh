#!/bin/bash
# run_test.sh OUTDIR UNIT_ID... : quick render + contact sheet (dev helper).
# Needs PYTHON (with Pillow + numpy); COLS/CELL tune the sheet.
S=${1:?outdir}
shift
T=$(cd "$(dirname "$0")" && pwd)
PY=${PYTHON:-python3}
blender -b --factory-startup -t ${THREADS:-8} -P "$T/render.py" -- --out "$S/raw" "$@" 2>&1 | grep -E "UNIT|Error|Traceback|error|File"
"$PY" "$T/post.py" "$S/raw" "$S/spr" "$@" >/dev/null
COLS=${COLS:-4} "$PY" "$T/preview.py" "$S/prev.png" "$S/spr" "$@"
