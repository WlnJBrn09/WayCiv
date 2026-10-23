#!/usr/bin/env bash
# Generates WayCiv's icons with Codex image generation.
#   cargo run --release -p wayciv-core --example icon_list > icons.tsv
#   tools/icons/generate.sh icons.tsv [parallel jobs]
# Each line of the list is: kind <TAB> id <TAB> subject. Finished icons are
# skipped, so the script can be re-run to fill in failures.
set -u
list=$1
jobs=${2:-4}
root=$(cd "$(dirname "$0")/../.." && pwd)
export ROOT=$root RAW=${WAYCIV_ICON_RAW:-$HOME/.cache/wayciv-icons/raw}
mkdir -p "$RAW"

one() {
    kind=$1 id=$2 subject=$3
    name=$(echo "$id" | tr '[:upper:]' '[:lower:]')
    out="$ROOT/assets/icons/$kind/$name.png"
    raw="$RAW/$kind-$name.png"
    [ -s "$out" ] && return 0
    mkdir -p "$ROOT/assets/icons/$kind"
    if [ ! -s "$raw" ]; then
        prompt="Use your image generation tool to create one square game icon depicting $subject. Style: polished painterly strategy-game icon like the icons in Civilization VI, bold readable silhouette that stays clear when shrunk to 48 pixels, the subject centered and filling about 70% of the frame, warm rim lighting, plain flat dark charcoal background (#202020). No text, no letters, no numbers, no border, no frame. Then copy the generated PNG file to $raw and reply with just the word done."
        codex exec --skip-git-repo-check -s workspace-write --add-dir "$RAW" "$prompt" < /dev/null > "$RAW/$kind-$name.log" 2>&1
    fi
    if [ -s "$raw" ]; then
        magick "$raw" -resize 128x128 \( -size 128x128 xc:none -fill white -draw "circle 63.5,63.5 63.5,1" \) \
            -compose CopyOpacity -composite "$out" && echo "ok $kind $id" || echo "FAIL convert $kind $id"
    else
        echo "FAIL $kind $id"
    fi
}
export -f one
line() { one "$(cut -f1 <<< "$1")" "$(cut -f2 <<< "$1")" "$(cut -f3- <<< "$1")"; }
export -f line
xargs -d '\n' -P "$jobs" -n 1 bash -c 'line "$1"' _ < "$list"
