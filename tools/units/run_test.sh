#!/bin/bash
# run_test.sh OUTDIR ids... : quick render + contact sheet (dev helper)
S=/tmp/claude-1000/-home-waylonjbrown-Projects-civ6-decompile/5a757930-95c0-4800-856f-365466ec8800/scratchpad/gfx/units
T=/home/waylonjbrown/Projects/WayCiv-gfx/units/tools/units
blender -b --factory-startup -t 4 -P $T/render.py -- --out $S/raw "$@" 2>&1 | grep -E "UNIT|Error|Traceback|error|File" 
$S/venv/bin/python $T/post.py $S/raw $S/spr "$@" >/dev/null
COLS=${COLS:-4} $S/venv/bin/python $T/preview.py $S/prev.png $S/spr "$@"
