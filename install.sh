#!/usr/bin/env bash
# Builds WayCiv and installs it for this user, so it shows up in the
# Omarchy menu (and any other app launcher):
#   ~/.local/bin/wayciv
#   ~/.local/share/wayciv/assets/leaders/*.png   (leader portraits)
#   ~/.local/share/wayciv/assets/icons/          (unit, resource, tree icons)
#   ~/.local/share/wayciv/assets/{terrain,units,buildings,resources}/ (map art)
#   ~/.local/share/applications/wayciv.desktop
#   ~/.local/share/icons/hicolor/scalable/apps/wayciv.svg
# Everything WayCiv needs is in this repository; no other game is required.
# Run it again after pulling changes to update the installed copy.
# ./install.sh --uninstall removes those files.
set -euo pipefail

cd "$(dirname "$0")"
bin="$HOME/.local/bin/wayciv"
data="$HOME/.local/share/wayciv"
desktop="$HOME/.local/share/applications/wayciv.desktop"
icon="$HOME/.local/share/icons/hicolor/scalable/apps/wayciv.svg"

if [[ ${1:-} == "--uninstall" ]]; then
  rm -f "$bin" "$desktop" "$icon"
  rm -rf "$data"
  echo "WayCiv removed."
  exit 0
fi

cargo build --release -p wayciv
install -Dm755 target/release/wayciv "$bin"
mkdir -p "$data/assets/leaders"
rm -f "$data/assets/leaders/"*.png
install -m644 assets/leaders/leader_*.png "$data/assets/leaders/"
# Art (icons, terrain, units, buildings, resources): each folder is
# replaced as a whole.
for dir in icons terrain units buildings resources; do
  rm -rf "$data/assets/$dir"
  if [[ -d assets/$dir ]]; then
    mkdir -p "$data/assets/$dir"
    cp -r "assets/$dir/." "$data/assets/$dir/"
  fi
done
install -Dm644 assets/wayciv.svg "$icon"
# The session PATH may not include ~/.local/bin, so the entry uses the full path.
mkdir -p "$(dirname "$desktop")"
sed "s|@BIN@|$bin|" wayciv.desktop > "$desktop.tmp"
mv "$desktop.tmp" "$desktop"
if command -v update-desktop-database >/dev/null; then
  update-desktop-database -q "$(dirname "$desktop")" || true
fi
if command -v gtk-update-icon-cache >/dev/null; then
  gtk-update-icon-cache -q -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi
echo "Installed WayCiv: $bin"
echo "It is in the app menu as \"WayCiv\"."
