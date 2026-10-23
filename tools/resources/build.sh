#!/bin/bash
# Regenerates every sprite in assets/resources/ (needs blender, python3).
#   tools/resources/build.sh [model-name ...]      (default: all models)
# Renders with Cycles on CPU (4 threads), then post.py turns the raw renders
# into sprites, class-coloured round icons and soft-edged field decals.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
raw=${WAYCIV_RAW:-${TMPDIR:-/tmp}/wayciv-resources-raw}
venv="$here/.venv"
if [ ! -x "$venv/bin/python" ]; then
  python3 -m venv "$venv"
  "$venv/bin/pip" install -q pillow numpy
fi
mkdir -p "$raw"
blender -b --factory-startup -t 4 -P "$here/render.py" -- "$raw" "$@" | grep -E "RENDERED|Error|Traceback" || true
"$venv/bin/python" "$here/post.py" "$raw" "$root/assets/resources"
"$venv/bin/python" "$here/sheet.py" "$root/assets/resources" "${WAYCIV_SHEET:-$raw/contact-sheet.png}"
