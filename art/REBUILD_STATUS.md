# Blender → Unity rebuild

The full goal remains: build individual Blender assets in order (terrain, buildings/districts, resources/improvements, units), then rebuild WayCiv in Unity with major changes.

## Current evidence

- **Terrain & connections (133 assets)**:
  - 18 catalog base terrains × 3 variants = 54 individual `.blend` files and FBX exports.
  - 11 everyday terrain features × 3 variants = 33 individual `.blend` files and FBX exports.
  - 20 natural wonders individually modeled and exported (`radial` hex-bounded meshes, catalog IDs, editable components).
  - 26 connection assets (13 biome-independent river channel/bank overlays + 13 shoreline beach/wet-sand overlays) covering all 63 nonempty six-edge masks through rotation.
  - `art/validation.json`: all 133 saved source scenes reopened and checked for catalog coverage, origin placement, finite geometry, nonzero face areas, materials, and common terrain boundary elevation.
- **Districts, constructed wonders & cities (134 assets)**:
  - 54 district layouts (covering every playable catalog district ID, including bespoke Maya Observatory and Byzantine Hippodrome models), 42 constructed-wonder models (covering all 41 catalog wonders), and 38 city/capital/wall layouts.
  - `art/architecture_validation.json`: all 134 saved scenes reopened and verified with zero missing playable districts or wonders and zero geometry errors.
- **Standalone buildings (47 assets)**:
  - All 47 regular and civilization-unique `BUILDING_*` catalog entries individually modeled and exported in `art/blender/buildings/*.blend` and `art/exports/buildings/*.fbx`, including complete wall assemblies (`BUILDING_WALLS`, `BUILDING_CASTLE`, `BUILDING_STAR_FORT`).
  - `art/building_validation.json`: all 47 saved building scenes reopened and verified with zero missing building IDs and zero geometry errors.
- **Resources & improvements (101 assets)**:
  - 51 resource models (covering all 45 catalog `RESOURCE_*` IDs) and 50 improvement models (covering all 29 catalog `IMPROVEMENT_*` IDs plus era/field variants).
  - 2D sprite alpha-halo platters (`water_rings`, `wake`) replaced with 3D concentric ripple rings and circular ground pads.
  - `art/resource_validation.json`: all 101 saved scenes reopened and verified with zero missing resource or improvement IDs and zero geometry errors.
- **Units with rigs & animations (118 assets)**:
  - All 118 unit models (covering all 107 catalog `UNIT_*` IDs plus era/unique variants) built at consistent tile scale with 3D bow-wave wakes on naval units, a 7-bone `ARMATURE` (`Root`, `Body`, `Head`, `LeftLeg`, `RightLeg`, `LeftArm`, `RightArm`), weighted vertex groups, `ARMATURE` modifiers, and 3 baked NLA animation clips (`Idle`, `Move`, `Attack`).
  - `art/unit_validation.json`: all 118 saved unit scenes reopened and verified for catalog coverage, armature presence, vertex groups, and 3 NLA tracks.
  - `art/fbx_roundtrip_validation.json`: architecture and rigged unit FBX exports reimported into Blender to verify triangle counts, materials, armatures, and `Attack`/`Idle`/`Move` animation clips.
- **Rust ↔ Unity engine bridge & Unity 6 project (533 staged FBX assets)**:
  - `crates/wayciv-unity` (`libwayciv_unity.so`): C ABI (`wc_request` / `wc_free`) exposing the full `wayciv-core` rules engine (world generation, movement, combat, city founding, district/wonder site placement, gold & faith purchases, ranged city strikes, air unit rebasing, tech & AI trees, governments & policy card drafting/slotting, pantheons & religion, era score & dedications, great people, city-states & envoys, diplomacy/alliances/wars, victory progress, and turn blockers). Verified by `cargo test -p wayciv-unity --offline`.
  - `unity/`: Unity `6000.3.28f1` project with all 533 FBX models staged under `unity/Assets/Resources/Models/`, `AssetCatalog.json`, `libwayciv_unity.so` under `unity/Assets/Plugins/x86_64/`, runtime C# presentation (`NativeGame.cs`, `GameController.cs`, `WorldView.cs`, `GameHud.cs`, `Interface.uss`), and editor importer/builder (`RebuildProject.cs`).
  - Both `WayCiv.Runtime.dll` and `WayCiv.Editor.dll` compile cleanly with zero warnings and zero errors against Unity `6000.3.28f1` (`python tools/rebuild/check_unity_compile.py`).
  - `art/previews/gameplay_slice.png` (`tools/rebuild/gameplay_slice.py`): end-to-end 3D gameplay slice driven by `libwayciv_unity.so` via `ctypes`, assembling terrain, coastal shorelines, river channels, features, resources, capital city, and rigged units from a live Turn 2 game state.

## Remaining external blocker

- Unity `6000.3.28f1` and Unity Hub are installed under `/home/waylonjbrown/Applications/Unity/`, but batchmode Editor import/build (`RebuildProject.BuildLinux`) currently exits with `No valid Unity Editor license found. Please activate your license.` Once a Personal/Pro license is activated in Unity Hub, run:
  ```bash
  /home/waylonjbrown/Applications/Unity/6000.3.28f1/Editor/Unity \
    -batchmode -nographics -quit \
    -projectPath /home/waylonjbrown/Projects/WayCiv/unity \
    -executeMethod RebuildProject.BuildLinux \
    -logFile /tmp/wayciv-unity-build.log
  ```

## Reproduction

- `tools/rebuild/terrain_models.py`, `feature_models.py`, `natural_wonders.py`, `terrain_connections.py`
- `tools/rebuild/architecture_models.py`, `building_models.py`, `resource_models.py`, `unit_models.py`
- `tools/rebuild/validate_assets.py`, `validate_architecture.py`, `check_architecture_fbx.py`
- `tools/rebuild/gameplay_slice.py`
- `tools/rebuild/stage_unity.py`, `check_unity_compile.py`
