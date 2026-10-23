#!/bin/bash
# Regenerate every sprite in assets/buildings with headless Blender (Cycles on 4 CPU threads).
#   tools/buildings/build.sh [samples] [name-prefix,...]
# e.g. tools/buildings/build.sh 48            all sprites
#      tools/buildings/build.sh 32 wonder_    only the wonders
# A contact sheet (needs Pillow) is written by sheet.py, see its docstring.
set -e
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
samples=${1:-48}
only=${2:+--only $2}
export TMPDIR=${TMPDIR:-$(mktemp -d)}
blender -b --factory-startup -t 4 -P "$here/render.py" -- --out "$root/assets/buildings" --samples "$samples" $only
