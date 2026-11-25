using System;
using System.Collections.Generic;
using System.Linq;
using UnityEngine;
using UnityEngine.Rendering;

namespace WayCiv
{
    public sealed class HexHit : MonoBehaviour { public int X, Y; }

    public sealed class WorldView : MonoBehaviour
    {
        GameController game;
        Camera view;
        AssetRecord[] assets;
        readonly Dictionary<string, GameObject> prefabs = new();
        readonly Dictionary<int, GameObject> tileObjects = new(), unitObjects = new(), cityObjects = new();
        readonly Dictionary<int, string> tileSignatures = new(), citySignatures = new();
        readonly Dictionary<int, Vector3> unitTargets = new();
        readonly HashSet<string> missing = new();
        Transform mapRoot, unitsRoot, citiesRoot, marksRoot;
        Material fogMaterial, lineMaterial;
        Mesh hexMesh;
        Vector3 focus = new(20, 0, -20);
        float distance = 18, yaw = 0;
        int width;
        bool hasWorld;
        public Camera ViewCamera => view;
        public static Vector3 Position(int x, int y) => new(Mathf.Sqrt(3) * (x + (y & 1) * .5f), 0, -1.5f * y);

        static int RotateMask(int mask, int k) => k == 0 ? (mask & 63) : (((mask << k) | (mask >> (6 - k))) & 63);
        static (int canonical, int steps) CanonicalMask(int mask)
        {
            int best = 64, steps = 0;
            for (int k = 0; k < 6; k++)
            {
                int r = RotateMask(mask, k);
                if (r < best) { best = r; steps = (6 - k) % 6; }
            }
            return (best, steps);
        }

        public void Initialize(GameController controller)
        {
            game = controller;
            var file = Resources.Load<TextAsset>("AssetCatalog");
            assets = file != null ? JsonUtility.FromJson<AssetCatalog>(file.text).assets : Array.Empty<AssetRecord>();
            mapRoot = new GameObject("Hex world").transform;
            unitsRoot = new GameObject("Units").transform;
            citiesRoot = new GameObject("Cities").transform;
            marksRoot = new GameObject("Selection").transform;
            var cam = new GameObject("Strategy camera"); view = cam.AddComponent<Camera>(); cam.tag = "MainCamera";
            cam.AddComponent<AudioListener>();
            view.fieldOfView = 38; view.nearClipPlane = .2f; view.farClipPlane = 450;
            view.clearFlags = CameraClearFlags.SolidColor; view.backgroundColor = new Color(.026f, .045f, .07f);
            var sun = new GameObject("Morning sun").AddComponent<Light>(); sun.type = LightType.Directional; sun.intensity = 1.6f; sun.color = new Color(1, .91f, .76f); sun.transform.rotation = Quaternion.Euler(48, -30, 0); sun.shadows = LightShadows.Soft;
            RenderSettings.ambientMode = AmbientMode.Trilight;
            RenderSettings.ambientSkyColor = new Color(.54f, .64f, .74f);
            RenderSettings.ambientEquatorColor = new Color(.30f, .37f, .40f);
            RenderSettings.ambientGroundColor = new Color(.15f, .20f, .18f);
            fogMaterial = new Material(Shader.Find("Standard")) { color = new Color(.045f, .074f, .098f) }; fogMaterial.SetFloat("_Glossiness", .1f);
            lineMaterial = new Material(Shader.Find("Sprites/Default"));
            hexMesh = MakeHex();
        }
        static Mesh MakeHex()
        {
            var vertices = new Vector3[7]; var triangles = new int[18];
            for (int i = 0; i < 6; i++)
            {
                float a = (30 + i * 60) * Mathf.Deg2Rad; vertices[i + 1] = new Vector3(Mathf.Cos(a), -.025f, Mathf.Sin(a));
                triangles[i * 3] = 0; triangles[i * 3 + 1] = (i + 1) % 6 + 1; triangles[i * 3 + 2] = i + 1;
            }
            var mesh = new Mesh { name = "Pointy hex" }; mesh.vertices = vertices; mesh.triangles = triangles; mesh.RecalculateNormals(); mesh.RecalculateBounds(); return mesh;
        }
        GameObject Spawn(string id, Transform parent, int variant = 0, bool exact = false)
        {
            if (string.IsNullOrEmpty(id)) return null;
            var candidates = assets.Where(a => exact ? a.asset == id : a.id == id).ToArray();
            if (candidates.Length == 0) { if (missing.Add(id)) Debug.LogWarning("Missing Blender catalog asset: " + id); return null; }
            var record = candidates[Math.Abs(variant) % candidates.Length];
            if (!prefabs.TryGetValue(record.path, out var prefab)) { prefab = Resources.Load<GameObject>(record.path); prefabs[record.path] = prefab; }
            if (prefab == null) return null;
            return Instantiate(prefab, parent, false);
        }
        void SpawnConnection(string prefix, int mask, Transform parent, float lift)
        {
            if (mask <= 0) return;
            var (canonical, steps) = CanonicalMask(mask & 63);
            var conn = Spawn($"{prefix}_{canonical:00}", parent);
            if (conn != null)
            {
                conn.transform.localPosition = Vector3.up * lift;
                conn.transform.localRotation = Quaternion.Euler(0, -steps * 60f, 0);
            }
        }
        void ClearWorld()
        {
            foreach (Transform t in mapRoot) Destroy(t.gameObject);
            foreach (Transform t in unitsRoot) Destroy(t.gameObject);
            foreach (Transform t in citiesRoot) Destroy(t.gameObject);
            tileObjects.Clear(); tileSignatures.Clear(); unitObjects.Clear(); unitTargets.Clear(); cityObjects.Clear(); citySignatures.Clear();
        }
        public void Refresh(Snapshot state)
        {
            if (!hasWorld || width != state.width || state.turn == 0) { ClearWorld(); width = state.width; hasWorld = true; }
            foreach (var tile in state.tiles)
            {
                int key = tile.y * width + tile.x;
                string signature = $"{tile.known}/{tile.visible}/{tile.terrain}/{tile.feature}/{tile.resource}/{tile.improvement}/{tile.district}/{tile.wonder}/{tile.owner}/{tile.riverMask}/{tile.shoreMask}/{tile.road}/{tile.worked}";
                if (tileSignatures.TryGetValue(key, out var old) && old == signature) continue;
                if (tileObjects.TryGetValue(key, out var previous)) Destroy(previous);
                var root = new GameObject($"Hex {tile.x},{tile.y}"); root.transform.SetParent(mapRoot, false); root.transform.position = Position(tile.x, tile.y);
                var hit = root.AddComponent<HexHit>(); hit.X = tile.x; hit.Y = tile.y;
                root.AddComponent<MeshCollider>().sharedMesh = hexMesh;
                if (!tile.known) { root.AddComponent<MeshFilter>().sharedMesh = hexMesh; root.AddComponent<MeshRenderer>().sharedMaterial = fogMaterial; }
                else
                {
                    int variant = (tile.x * 17 + tile.y * 31) % 3;
                    bool hills = tile.terrain != null && tile.terrain.Contains("HILLS");
                    bool mountain = tile.terrain != null && tile.terrain.Contains("MOUNTAIN");
                    bool floodplains = tile.feature != null && tile.feature.StartsWith("FEATURE_FLOODPLAINS");
                    float surface = hills ? .16f : 0f;
                    Spawn(tile.terrain, root.transform, variant);
                    if (tile.riverMask > 0 && !hills && !mountain && !floodplains)
                        SpawnConnection("RIVER", tile.riverMask, root.transform, 0f);
                    if (tile.shoreMask > 0 && !hills && !mountain)
                        SpawnConnection("SHORE", tile.shoreMask, root.transform, .003f);
                    if (string.IsNullOrEmpty(tile.district) && string.IsNullOrEmpty(tile.wonder))
                    {
                        var feature = Spawn(tile.feature, root.transform, variant); if (feature) feature.transform.localPosition = Vector3.up * surface;
                        var prop = Spawn(string.IsNullOrEmpty(tile.improvement) ? tile.resource : tile.improvement, root.transform); if (prop) prop.transform.localPosition = Vector3.up * (surface + .025f);
                    }
                    else
                    {
                        var arch = Spawn(!string.IsNullOrEmpty(tile.wonder) ? tile.wonder : tile.district, root.transform);
                        if (arch) arch.transform.localPosition = Vector3.up * surface;
                    }
                    if (tile.road)
                        Ring(root.transform, .22f, surface + .02f, new Color(.66f, .56f, .40f, .8f), .035f);
                    if (tile.worked)
                        Ring(root.transform, .82f, surface + .022f, new Color(.35f, .92f, .45f, .45f), .018f);
                    if (tile.owner >= 0)
                    {
                        var player = state.players.FirstOrDefault(p => p.id == tile.owner);
                        var color = player != null ? PlayerColor(player) : new Color(.45f, .45f, .45f);
                        color.a = .65f; Ring(root.transform, 1.0f, .025f, color, .024f);
                    }
                    if (!tile.visible)
                    {
                        foreach (var renderer in root.GetComponentsInChildren<Renderer>())
                        {
                            if (renderer is LineRenderer) continue;
                            var block = new MaterialPropertyBlock(); var color = renderer.sharedMaterial ? renderer.sharedMaterial.color : Color.white;
                            block.SetColor("_Color", color * .48f); renderer.SetPropertyBlock(block);
                        }
                    }
                }
                tileObjects[key] = root; tileSignatures[key] = signature;
            }
            foreach (var id in unitObjects.Keys.Where(id => !state.units.Any(u => u.id == id)).ToArray()) { Destroy(unitObjects[id]); unitObjects.Remove(id); unitTargets.Remove(id); }
            foreach (var unit in state.units)
            {
                var tile = state.tiles[unit.y * width + unit.x];
                float lift = unit.domain == "DOMAIN_AIR" ? .58f : (tile.terrain != null && tile.terrain.Contains("HILLS") ? .2f : .04f);
                Vector3 target = Position(unit.x, unit.y) + Vector3.up * lift;
                if (!unitObjects.TryGetValue(unit.id, out var obj))
                {
                    obj = new GameObject(unit.name); obj.transform.SetParent(unitsRoot, false); obj.transform.position = target;
                    Spawn(unit.asset, obj.transform);
                    var owner = state.players.FirstOrDefault(p => p.id == unit.owner);
                    var color = owner != null ? PlayerColor(owner) : Color.red;
                    Ring(obj.transform, .48f, .01f, color, .035f);
                    foreach (var renderer in obj.GetComponentsInChildren<MeshRenderer>())
                        for (int j = 0; j < renderer.sharedMaterials.Length; j++) if (renderer.sharedMaterials[j] != null && renderer.sharedMaterials[j].name.IndexOf("team", StringComparison.OrdinalIgnoreCase) >= 0) { var block = new MaterialPropertyBlock(); block.SetColor("_Color", color); renderer.SetPropertyBlock(block, j); }
                    unitObjects[unit.id] = obj;
                }
                unitTargets[unit.id] = target;
            }
            foreach (var id in cityObjects.Keys.Where(id => !state.cities.Any(c => c.id == id)).ToArray()) { Destroy(cityObjects[id]); cityObjects.Remove(id); citySignatures.Remove(id); }
            foreach (var city in state.cities)
            {
                int era = Mathf.Clamp(city.era, 0, 3);
                int tier = Mathf.Clamp(1 + city.population / 4, 1, 4);
                string model = $"{(city.capital ? "capital" : "city")}_{era}_{tier}";
                string bSig = city.buildings != null ? string.Join(",", city.buildings) : "";
                string sig = $"{model}:{bSig}";
                if (citySignatures.TryGetValue(city.id, out var prevSig) && prevSig == sig) continue;
                if (cityObjects.TryGetValue(city.id, out var previous)) Destroy(previous);
                var root = new GameObject(model); root.transform.SetParent(citiesRoot, false); root.transform.position = Position(city.x, city.y) + Vector3.up * .04f;
                Spawn(model, root.transform, 0, true);
                if (city.buildings != null)
                {
                    string wallId = city.buildings.Contains("BUILDING_STAR_FORT") ? "BUILDING_STAR_FORT"
                        : city.buildings.Contains("BUILDING_CASTLE") ? "BUILDING_CASTLE"
                        : city.buildings.Contains("BUILDING_WALLS") ? "BUILDING_WALLS" : null;
                    if (wallId != null)
                        Spawn(wallId, root.transform);
                    var centerBuildings = city.buildings.Where(b => b != "BUILDING_WALLS" && b != "BUILDING_CASTLE" && b != "BUILDING_STAR_FORT" && b != "BUILDING_PALACE").Take(6).ToArray();
                    for (int i = 0; i < centerBuildings.Length; i++)
                    {
                        var bObj = Spawn(centerBuildings[i], root.transform);
                        if (bObj != null)
                        {
                            float ang = i * Mathf.PI / 3f;
                            bObj.transform.localPosition = new Vector3(Mathf.Cos(ang) * .52f, 0, Mathf.Sin(ang) * .52f);
                            bObj.transform.localScale = Vector3.one * .38f;
                        }
                    }
                }
                cityObjects[city.id] = root; citySignatures[city.id] = sig;
            }
        }
        public static Color PlayerColor(PlayerData player) => new(player.color[0] / 255f, player.color[1] / 255f, player.color[2] / 255f);
        void Ring(Transform parent, float radius, float height, Color color, float thickness)
        {
            var line = new GameObject("Hex outline").AddComponent<LineRenderer>(); line.transform.SetParent(parent, false); line.useWorldSpace = false; line.sharedMaterial = lineMaterial; line.startColor = line.endColor = color; line.startWidth = line.endWidth = thickness; line.loop = true; line.positionCount = 6;
            for (int i = 0; i < 6; i++) { float a = (30 + i * 60) * Mathf.Deg2Rad; line.SetPosition(i, new Vector3(Mathf.Cos(a) * radius, height, Mathf.Sin(a) * radius)); }
        }
        public void Highlight(int x, int y) { foreach (Transform t in marksRoot) Destroy(t.gameObject); var marker = new GameObject("Selected tile"); marker.transform.SetParent(marksRoot, false); marker.transform.position = Position(x, y); Ring(marker.transform, .98f, .08f, new Color(1, .82f, .35f), .055f); }
        public void ShowSites(HexPos[] sites) { foreach (Transform t in marksRoot) Destroy(t.gameObject); if (sites == null) return; foreach (var p in sites) { var root = new GameObject("Construction site"); root.transform.SetParent(marksRoot, false); root.transform.position = Position(p.x, p.y); Ring(root.transform, .9f, .08f, new Color(.4f, 1, .8f), .05f); } }
        public void Focus(int x, int y) { focus = Position(x, y); }
        void Update()
        {
            bool ui = game.Hud != null && game.Hud.PointerOverPanel;
            if (!ui)
            {
                distance = Mathf.Clamp(distance - Input.mouseScrollDelta.y * 1.3f, 5, 65);
                if (Input.GetKey(KeyCode.Q)) yaw -= Time.deltaTime * 55;
                if (Input.GetKey(KeyCode.E)) yaw += Time.deltaTime * 55;
                var pan = new Vector3((Input.GetKey(KeyCode.D) || Input.GetKey(KeyCode.RightArrow) ? 1 : 0) - (Input.GetKey(KeyCode.A) || Input.GetKey(KeyCode.LeftArrow) ? 1 : 0), 0, (Input.GetKey(KeyCode.W) || Input.GetKey(KeyCode.UpArrow) ? 1 : 0) - (Input.GetKey(KeyCode.S) && !Input.GetKey(KeyCode.LeftControl) || Input.GetKey(KeyCode.DownArrow) ? 1 : 0));
                focus += Quaternion.Euler(0, yaw, 0) * pan * (distance * .5f * Time.deltaTime);
                if (Input.GetMouseButtonDown(0) || Input.GetMouseButtonDown(1))
                    if (Physics.Raycast(view.ScreenPointToRay(Input.mousePosition), out var hit, 500) && hit.collider.TryGetComponent<HexHit>(out var hex))
                    { if (Input.GetMouseButtonDown(0)) game.Select(hex.X, hex.Y); else game.Order(hex.X, hex.Y); }
            }
            if (game.State != null)
            {
                if (Input.GetKeyDown(KeyCode.Tab) || Input.GetKeyDown(KeyCode.Period)) game.NextUnit();
                if (Input.GetKeyDown(KeyCode.Return) || Input.GetKeyDown(KeyCode.KeypadEnter)) game.AdvanceOrResolveBlocker();
                if (Input.GetKeyDown(KeyCode.Space) && game.SelectedUnit >= 0) game.Send(new Command("action") { unit = game.SelectedUnit, action = "Skip" });
                if (Input.GetKeyDown(KeyCode.F) && game.SelectedUnit >= 0) game.Send(new Command("action") { unit = game.SelectedUnit, action = "Fortify" });
                if (Input.GetKeyDown(KeyCode.Z) && game.SelectedUnit >= 0) game.Send(new Command("action") { unit = game.SelectedUnit, action = "Sleep" });
                if (Input.GetKeyDown(KeyCode.B) && game.SelectedUnit >= 0) game.Send(new Command("action") { unit = game.SelectedUnit, action = "FoundCity" });
                if (Input.GetKeyDown(KeyCode.L)) game.Hud.ToggleDrawer("leaders");
                if (Input.GetKeyDown(KeyCode.G)) game.Hud.ToggleDrawer("government");
                if (Input.GetKeyDown(KeyCode.R)) game.Hud.ToggleDrawer("religion");
                if (Input.GetKeyDown(KeyCode.P)) game.Hud.ToggleDrawer("great_people");
                if (Input.GetKeyDown(KeyCode.E) && Input.GetKey(KeyCode.LeftShift)) game.Hud.ToggleDrawer("city_states");
                if (Input.GetKeyDown(KeyCode.T)) game.Hud.ToggleDrawer("eras");
                if (Input.GetKeyDown(KeyCode.V)) game.Hud.ToggleDrawer("victory");
                if (Input.GetKeyDown(KeyCode.N)) game.Hud.ToggleDrawer("chronicle");
            }
            if (Input.GetKeyDown(KeyCode.Escape))
            {
                if (game.Placement != null || game.RebaseMode || game.BombardMode) { game.Placement = null; game.RebaseMode = game.BombardMode = false; game.Hud.Refresh(); }
                else if (game.Hud.HasDrawer) game.Hud.OpenDrawer("");
                else { game.SelectedUnit = game.SelectedCity = -1; game.SelectedTile = null; game.Hud.Refresh(); }
            }
            view.transform.position = focus + Quaternion.Euler(0, yaw, 0) * new Vector3(0, distance * .80f, -distance * .60f);
            view.transform.LookAt(focus);
            foreach (var pair in unitObjects)
            {
                var target = unitTargets[pair.Key];
                var delta = target - pair.Value.transform.position;
                delta.y = 0;
                if (delta.sqrMagnitude > .001f)
                    pair.Value.transform.rotation = Quaternion.Slerp(pair.Value.transform.rotation, Quaternion.LookRotation(delta.normalized, Vector3.up), 1 - Mathf.Exp(-Time.deltaTime * 12));
                float bob = Mathf.Sin(Time.time * 2.6f + pair.Key) * .012f;
                pair.Value.transform.position = Vector3.Lerp(pair.Value.transform.position, target + Vector3.up * bob, 1 - Mathf.Exp(-Time.deltaTime * 8));
            }
        }
    }
}
