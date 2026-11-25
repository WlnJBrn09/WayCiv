using System;
using System.IO;
using System.Linq;
using UnityEditor;
using UnityEditor.Build.Reporting;
using UnityEditor.SceneManagement;
using UnityEngine;
using WayCiv;

public static class RebuildProject
{
    [MenuItem("WayCiv/Prepare game scene")]
    public static void Prepare()
    {
        Directory.CreateDirectory("Assets/Scenes");
        var scene=EditorSceneManager.NewScene(NewSceneSetup.EmptyScene,NewSceneMode.Single);
        // GameController boots at runtime; the scene is deliberately free of test objects.
        EditorSceneManager.SaveScene(scene,"Assets/Scenes/WayCiv.unity");
        EditorBuildSettings.scenes=new[]{new EditorBuildSettingsScene("Assets/Scenes/WayCiv.unity",true)};
        PlayerSettings.companyName="WayCiv";PlayerSettings.productName="WayCiv";
        PlayerSettings.colorSpace=ColorSpace.Linear;
        PlayerSettings.defaultScreenWidth=1600;PlayerSettings.defaultScreenHeight=1000;
        PlayerSettings.fullScreenMode=FullScreenMode.Windowed;
        AssetDatabase.SaveAssets();
    }
    [MenuItem("WayCiv/Validate imported models")]
    public static void Validate()
    {
        var catalog=JsonUtility.FromJson<AssetCatalog>(File.ReadAllText("Assets/Resources/AssetCatalog.json"));
        int checkedCount=0;
        foreach(var item in catalog.assets)
        {
            var model=AssetDatabase.LoadAssetAtPath<GameObject>("Assets/Resources/"+item.path+".fbx");
            if(model==null)throw new Exception("Missing import: "+item.path);
            var meshes=model.GetComponentsInChildren<MeshFilter>();
            if(meshes.Length==0||meshes.Any(m=>m.sharedMesh==null||m.sharedMesh.vertexCount==0))throw new Exception("Empty mesh: "+item.path);
            if(model.GetComponentsInChildren<Renderer>().Any(r=>r.sharedMaterials.Any(m=>m==null)))throw new Exception("Missing material: "+item.path);
            checkedCount++;
        }
        var state=NativeGame.Request<Snapshot>(new Command("new"){leader="LEADER_REAGAN"});
        if(state.tiles.Length!=state.width*state.height)throw new Exception("Native snapshot is incomplete");
        Directory.CreateDirectory("Validation");
        File.WriteAllText("Validation/import.json",JsonUtility.ToJson(new ImportReport{models=checkedCount,tiles=state.tiles.Length,unity=Application.unityVersion},true));
        Debug.Log($"WAYCIV_IMPORT_PASS models={checkedCount} tiles={state.tiles.Length}");
    }
    [Serializable] class ImportReport {public int models,tiles;public string unity;}
    public static void BuildLinux()
    {
        Prepare();Validate();Directory.CreateDirectory("Builds/Linux");
        var report=BuildPipeline.BuildPlayer(new BuildPlayerOptions{scenes=new[]{"Assets/Scenes/WayCiv.unity"},locationPathName="Builds/Linux/WayCiv.x86_64",target=BuildTarget.StandaloneLinux64,options=BuildOptions.Development});
        if(report.summary.result!=BuildResult.Succeeded)throw new Exception("Build failed: "+report.summary.result);
    }
}

public sealed class BlenderModelImporter : AssetPostprocessor
{
    void OnPreprocessModel()
    {
        if(!assetPath.StartsWith("Assets/Resources/Models/"))return;
        var importer=(ModelImporter)assetImporter;
        bool isUnit=assetPath.Contains("/units/");
        importer.materialImportMode=ModelImporterMaterialImportMode.ImportStandard;
        importer.importCameras=false;importer.importLights=false;
        importer.importAnimation=isUnit;
        importer.animationType=isUnit?ModelImporterAnimationType.Generic:ModelImporterAnimationType.None;
        importer.globalScale=1;
        importer.useFileScale=true;importer.isReadable=false;
        importer.importNormals=ModelImporterNormals.Import;
    }
    void OnPostprocessMaterial(Material material)
    {
        if(!assetPath.StartsWith("Assets/Resources/Models/"))return;
        var oldColor=material.color;material.shader=Shader.Find("Standard");material.color=oldColor;
        material.SetFloat("_Glossiness",.15f);material.enableInstancing=true;
        // The export palette records exact linear colors, independent of FBX importer defaults.
        var catalog=JsonUtility.FromJson<AssetCatalog>(File.ReadAllText("Assets/Resources/AssetCatalog.json"));
        var record=catalog.assets.FirstOrDefault(a=>assetPath=="Assets/Resources/"+a.path+".fbx");
        var palette=record?.materials?.FirstOrDefault(m=>m.name==material.name);
        if(palette?.rgba?.Length==4)material.color=new Color(palette.rgba[0],palette.rgba[1],palette.rgba[2],palette.rgba[3]);
    }
    public override uint GetVersion()=>1;
}
