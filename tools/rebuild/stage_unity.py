"""Stage Blender FBX assets and the compiled native engine in the Unity project."""
import json, shutil
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
RES = ROOT / 'unity/Assets/Resources'
RES.mkdir(parents=True, exist_ok=True)
records = []
for name in ['terrain','feature','natural_wonder','connection','architecture','building','resource','unit']:
    for row in json.loads((ROOT / f'art/{name}_manifest.json').read_text())['assets']:
        src = ROOT / row['fbx']
        rel = Path('Models') / src.parent.name / src.name
        dst = RES / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        if not dst.exists() or src.stat().st_mtime_ns > dst.stat().st_mtime_ns:
            shutil.copy2(src, dst)
        records.append({'id':row['id'], 'asset':src.stem, 'path':rel.with_suffix('').as_posix(), 'category':src.parent.name, 'materials':row.get('materials',[])})
(RES/'AssetCatalog.json').write_text(json.dumps({'assets':records},indent=2)+'\n')
plugin=ROOT/'target/release/libwayciv_unity.so'
if plugin.exists():
    dst=ROOT/'unity/Assets/Plugins/x86_64';dst.mkdir(parents=True,exist_ok=True);shutil.copy2(plugin,dst/plugin.name)
print(f'Staged {len(records)} Blender models; native library present: {plugin.exists()}')
