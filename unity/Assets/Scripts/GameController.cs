using System;
using System.Linq;
using UnityEngine;

namespace WayCiv
{
    public sealed class GameController : MonoBehaviour
    {
        public Snapshot State { get; private set; }
        public WorldView World { get; private set; }
        public GameHud Hud { get; private set; }
        public int SelectedUnit = -1, SelectedCity = -1;
        public TileData SelectedTile;
        public ProductionData Placement;
        public bool RebaseMode, BombardMode;
        public string Message = "Choose a leader and write your history.";

        [RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.AfterSceneLoad)]
        static void Boot()
        {
            if (FindFirstObjectByType<GameController>() == null)
                new GameObject("WayCiv").AddComponent<GameController>();
        }
        void Awake()
        {
            Application.targetFrameRate = 60;
            QualitySettings.shadows = ShadowQuality.All;
            World = gameObject.AddComponent<WorldView>(); World.Initialize(this);
            Hud = gameObject.AddComponent<GameHud>(); Hud.Initialize(this);
        }
        public void StartGame(Command command)
        {
            SelectedUnit = SelectedCity = -1; Placement = null; RebaseMode = BombardMode = false; SelectedTile = null;
            if (!Send(command)) return;
            var unit = State.units.FirstOrDefault(u => u.owner == State.human);
            if (unit != null) { Select(unit.x, unit.y); World.Focus(unit.x, unit.y); }
        }
        public bool Send(Command command)
        {
            try
            {
                var next = NativeGame.Request<Snapshot>(command);
                if (next.version != 1) throw new InvalidOperationException("Engine and presentation versions do not match.");
                State = next;
                if (!State.units.Any(u => u.id == SelectedUnit)) SelectedUnit = -1;
                if (!State.cities.Any(c => c.id == SelectedCity)) SelectedCity = -1;
                Message = string.IsNullOrEmpty(State.winner) ? "" : State.winner;
                World.Refresh(State); Hud.Refresh(); return true;
            }
            catch (Exception e) { Message = e.Message; Debug.LogWarning(e); Hud.Refresh(); return false; }
        }
        public void Select(int x, int y)
        {
            if (State == null) return;
            SelectedTile = State.tiles.FirstOrDefault(t => t.x == x && t.y == y);
            if (Placement != null)
            {
                if (Placement.sites.Any(p => p.x == x && p.y == y))
                {
                    var pending = Placement; Placement = null;
                    Send(new Command("production") { city = SelectedCity, kind = pending.kind, index = pending.index, x = x, y = y });
                }
                else Message = "Choose one of the highlighted construction sites.";
                Hud.Refresh(); return;
            }
            if (RebaseMode && SelectedUnit >= 0)
            {
                RebaseMode = false;
                Send(new Command("rebase") { unit = SelectedUnit, x = x, y = y });
                return;
            }
            if (BombardMode && SelectedCity >= 0)
            {
                BombardMode = false;
                Send(new Command("city_attack") { city = SelectedCity, x = x, y = y });
                return;
            }
            SelectedUnit = State.units.FirstOrDefault(u => u.x == x && u.y == y && u.owner == State.human)?.id ?? -1;
            SelectedCity = State.cities.FirstOrDefault(c => c.x == x && c.y == y && c.owner == State.human)?.id ?? -1;
            World.Highlight(x, y);
            Hud.Refresh();
        }
        public void Order(int x, int y)
        {
            if (SelectedUnit < 0 || State == null) return;
            bool hostile = State.units.Any(u => u.x == x && u.y == y && u.owner != State.human) || State.cities.Any(c => c.x == x && c.y == y && c.owner != State.human);
            Send(new Command(hostile ? "attack" : "move") { unit = SelectedUnit, x = x, y = y });
        }
        public void ChooseProduction(ProductionData item)
        {
            if (item.kind == "district" || item.kind == "wonder")
            {
                Placement = item; RebaseMode = BombardMode = false;
                Message = "Choose a highlighted site for " + item.name;
                World.ShowSites(item.sites); Hud.Refresh();
            }
            else Send(new Command("production") { city = SelectedCity, index = item.index, kind = item.kind });
        }
        public void AdvanceOrResolveBlocker()
        {
            if (State == null) return;
            if (State.blockerItems == null || State.blockerItems.Length == 0)
            {
                Hud.OpenDrawer("");
                Send(new Command("end_turn"));
                return;
            }
            var b = State.blockerItems[0];
            switch (b.kind)
            {
                case "research": Hud.OpenDrawer("research"); break;
                case "ai": Hud.OpenDrawer("ai"); break;
                case "pantheon": Hud.OpenDrawer("religion"); break;
                case "policy_draft":
                case "government": Hud.OpenDrawer("government"); break;
                case "dedication": Hud.OpenDrawer("eras"); break;
                case "production":
                    var city = State.cities.FirstOrDefault(c => c.id == b.city);
                    if (city != null) { Select(city.x, city.y); World.Focus(city.x, city.y); SelectedCity = city.id; Hud.OpenDrawer("production"); }
                    break;
                case "unit":
                    var unit = State.units.FirstOrDefault(u => u.id == b.unit);
                    if (unit != null) { Hud.OpenDrawer(""); Select(unit.x, unit.y); World.Focus(unit.x, unit.y); SelectedUnit = unit.id; Hud.Refresh(); }
                    break;
                default:
                    Send(new Command("end_turn"));
                    break;
            }
        }
        public void NextUnit()
        {
            if (State == null) return;
            var units = State.units.Where(u => u.owner == State.human && u.moves > 0 && !u.fortified && !u.sleeping).ToArray();
            if (units.Length == 0) units = State.units.Where(u => u.owner == State.human).ToArray();
            if (units.Length == 0) return;
            int current = Array.FindIndex(units, u => u.id == SelectedUnit);
            var next = units[(current + 1) % units.Length];
            Select(next.x, next.y); World.Focus(next.x, next.y);
        }
    }
}
