"""Compile against the installed Unity APIs without claiming an Editor/import test."""
import argparse, subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--editor',type=Path,default=Path.home()/'Applications/Unity/6000.3.28f1/Editor');args=p.parse_args()
root=Path(__file__).resolve().parents[2];data=args.editor/'Data';out=root/'unity/Temp/CompileCheck';out.mkdir(parents=True,exist_ok=True)
refs=[data/'NetStandard/ref/2.1.0/netstandard.dll',*sorted((data/'Managed/UnityEngine').glob('*.dll'))]
refs += [r for r in (data/'Managed').glob('Unity*.dll') if r.name not in ('UnityEngine.dll','UnityEditor.dll')]
compiler=[str(data/'NetCoreRuntime/dotnet'),str(data/'DotNetSdkRoslyn/csc.dll')]
for label,folder,extra in [('Runtime','Scripts',[]),('Editor','Editor',[out/'WayCiv.Runtime.dll'])]:
    sources=sorted((root/f'unity/Assets/{folder}').glob('*.cs'))
    command=compiler+['-nologo','-target:library','-langversion:9.0','-nostdlib+',f'-out:{out}/WayCiv.{label}.dll']+[f'-r:{r}' for r in refs+extra]+[str(s) for s in sources]
    result=subprocess.run(command)
    if result.returncode: raise SystemExit(result.returncode)
    print(f'{label}: compiled against Unity {args.editor.parent.name}')
