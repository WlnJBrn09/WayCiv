using System;
using System.Collections.Generic;
using System.Linq;
using UnityEngine;
using UnityEngine.UIElements;

namespace WayCiv
{
    public sealed class GameHud : MonoBehaviour
    {
        GameController game;
        VisualElement root;
        readonly List<VisualElement> panels = new();
        string page = "";
        public bool HasDrawer => !string.IsNullOrEmpty(page);
        public void OpenDrawer(string next) { page = next ?? ""; Refresh(); }
        public void ToggleDrawer(string target) { page = page == target ? "" : target; Refresh(); }
        public bool PointerOverPanel
        {
            get
            {
                if (root?.panel == null) return false;
                Vector2 screen = new(Input.mousePosition.x, Screen.height - Input.mousePosition.y);
                var point = RuntimePanelUtils.ScreenToPanel(root.panel, screen);
                return panels.Any(p => p.worldBound.Contains(point));
            }
        }
        public void Initialize(GameController controller)
        {
            game = controller;
            var document = gameObject.AddComponent<UIDocument>();
            var settings = ScriptableObject.CreateInstance<PanelSettings>();
            settings.scaleMode = PanelScaleMode.ScaleWithScreenSize;
            settings.referenceResolution = new Vector2Int(1600, 1000);
            settings.match = .5f;
            document.panelSettings = settings;
            root = document.rootVisualElement;
            root.pickingMode = PickingMode.Ignore;
            root.style.flexGrow = 1;
            root.style.color = new Color(.91f, .92f, .88f);
            root.style.fontSize = 14;
            var style = Resources.Load<StyleSheet>("Interface");
            if (style) root.styleSheets.Add(style);
            Refresh();
        }
        VisualElement Panel(string className)
        {
            var panel = new VisualElement();
            panel.AddToClassList("panel");
            panel.AddToClassList(className);
            root.Add(panel);
            panels.Add(panel);
            return panel;
        }
        static Label Label(VisualElement parent, string text, string cls = "")
        {
            var label = new Label(text);
            if (cls != "") label.AddToClassList(cls);
            parent.Add(label);
            return label;
        }
        static Button Button(VisualElement parent, string text, Action action, string cls = "")
        {
            var button = new Button(action) { text = text };
            if (cls != "") button.AddToClassList(cls);
            parent.Add(button);
            return button;
        }
        public void Refresh()
        {
            if (root == null) return;
            root.Clear();
            panels.Clear();
            if (game.State == null) { Menu(); return; }
            var s = game.State;
            var top = Panel("topbar");
            Label(top, "WAYCIV", "brand");
            Label(top, $"{s.year}   /   {s.era} ({s.age})", "date");
            Label(top, $"GOLD  {s.gold}  ({s.netGold:+0;-0;0})", "stat");
            Label(top, $"SCIENCE  +{s.income[3]:0.#}", "stat");
            Label(top, $"CULTURE  +{s.income[4]:0.#}", "stat");
            Label(top, $"FAITH  {s.faith}  (+{s.income[5]:0.#})", "stat");
            Label(top, $"TOURISM  {s.tourism}", "stat");
            if (s.aiUnlocked || s.compute > 0)
                Label(top, $"COMPUTE  {s.compute:0.#}", "stat");
            Label(top, $"ENVOYS  {s.envoysAvailable}", "stat");

            var nav = Panel("navigation");
            Button(nav, "Research", () => ToggleDrawer("research"));
            if (s.aiUnlocked || s.compute > 0)
                Button(nav, "AI Tree", () => ToggleDrawer("ai"));
            Button(nav, "Gov & Policies [G]", () => ToggleDrawer("government"));
            Button(nav, "Religion [R]", () => ToggleDrawer("religion"));
            Button(nav, "Great People [P]", () => ToggleDrawer("great_people"));
            Button(nav, "City-States", () => ToggleDrawer("city_states"));
            Button(nav, "Leaders [L]", () => ToggleDrawer("leaders"));
            Button(nav, "Eras [T]", () => ToggleDrawer("eras"));
            Button(nav, "Victory [V]", () => ToggleDrawer("victory"));
            Button(nav, "Chronicle [N]", () => ToggleDrawer("chronicle"));
            Button(nav, "Next Unit [.]", game.NextUnit);

            var bottom = Panel("turnbar");
            string limit = s.turnLimit > 0 ? $" / {s.turnLimit}" : "";
            Label(bottom, $"TURN {s.turn}{limit}  •  ERA SCORE {s.eraScore} ({s.projectedAge})", "eyebrow");
            bool hasBlocker = s.blockerItems != null && s.blockerItems.Length > 0;
            string blockerLabel = hasBlocker ? s.blockerItems[0].text : "END TURN  →";
            Label(bottom, hasBlocker ? $"{s.blockerItems.Length} decision(s) needed before next turn" : "Your empire is ready.", "muted");
            Button(bottom, blockerLabel.ToUpperInvariant(), game.AdvanceOrResolveBlocker, "primary");

            var help = Panel("help");
            Label(help, "W A S D pan  •  Scroll zoom  •  Q/E rotate  •  Right-click move/attack  •  Enter next action/turn", "muted");
            if (!string.IsNullOrEmpty(game.Message))
            {
                var message = Panel("message");
                Label(message, game.Message);
            }
            if (page != "") { Drawer(page); return; }
            Selection();
        }
        void Menu()
        {
            var panel = Panel("mainmenu");
            Label(panel, "A WORLD WAITING TO BE WRITTEN", "eyebrow");
            Label(panel, "WAYCIV", "title");
            Label(panel, "From the first settlement to the age of intelligence.", "subtitle");
            LeaderCatalog catalog;
            try { catalog = NativeGame.Request<LeaderCatalog>(new Command("catalog")); }
            catch (Exception e) { Label(panel, "Engine could not start: " + e.Message); return; }
            var leader = new DropdownField("Leader", catalog.leaders.Select(l => l.name).ToList(), 0); panel.Add(leader);
            var world = new DropdownField("World", new List<string> { "Continents", "Earth" }, 1); panel.Add(world);
            var size = new DropdownField("Map size", new List<string> { "Small", "Standard", "Large" }, 0); panel.Add(size);
            var length = new DropdownField("Game length", new List<string> { "100", "250", "500", "750", "1000", "infinite" }, 2); panel.Add(length);
            var seed = new IntegerField("World seed") { value = 42 }; panel.Add(seed);
            Button(panel, "BEGIN YOUR CIVILIZATION", () =>
            {
                page = "";
                game.StartGame(new Command("new")
                {
                    leader = catalog.leaders[leader.index].id,
                    map = world.index == 0 ? "random" : "earth",
                    size = size.value.ToLowerInvariant(),
                    length = length.value,
                    seed = Math.Max(0, seed.value)
                });
            }, "primary");
            Label(panel, "3D Blender + Unity rebuild • Full Rust rules engine", "muted");
            if (!string.IsNullOrEmpty(game.Message)) Label(panel, game.Message, "muted");
        }
        void Selection()
        {
            var s = game.State;
            var unit = s.units.FirstOrDefault(u => u.id == game.SelectedUnit);
            var city = s.cities.FirstOrDefault(c => c.id == game.SelectedCity);
            if (unit == null && city == null && game.SelectedTile == null) return;
            var panel = Panel("selection");
            if (unit != null)
            {
                Label(panel, "UNIT ORDERS", "eyebrow");
                Label(panel, unit.name, "heading");
                string extra = unit.charges > 0 ? $"  •  Charges {unit.charges}" : "";
                Label(panel, $"Health {unit.hp}/100  •  Moves {unit.moves}{extra}", "muted");
                foreach (var order in unit.actions)
                {
                    if (order.id == "Delete") continue;
                    Button(panel, order.name, () => game.Send(new Command("action") { unit = unit.id, action = order.id }));
                }
                if (unit.rebaseTargets != null && unit.rebaseTargets.Length > 0)
                {
                    Button(panel, "Rebase aircraft", () =>
                    {
                        game.RebaseMode = true;
                        game.Message = "Select a highlighted city or airstrip to rebase.";
                        game.World.ShowSites(unit.rebaseTargets);
                        Refresh();
                    });
                }
                Button(panel, "Show movement range", () =>
                {
                    try { game.World.ShowSites(NativeGame.Request<ReachableData>(new Command("reachable") { unit = unit.id }).positions); }
                    catch (Exception e) { game.Message = e.Message; Refresh(); }
                });
            }
            if (city != null)
            {
                Label(panel, city.capital ? "CAPITAL" : "CITY", "eyebrow");
                Label(panel, city.name, "heading");
                string walls = city.maxWalls > 0 ? $"  •  Walls {city.wallHp}/{city.maxWalls}" : "";
                string grow = city.turnsToGrow > 0 ? $"  •  Grows in {city.turnsToGrow}t" : "";
                Label(panel, $"Pop {city.population}{grow}  •  HP {city.hp}{walls}", "muted");
                Label(panel, $"Housing {city.housing}  •  Amenities {city.amenities}/{city.amenitiesNeeded}", "muted");
                Label(panel, string.IsNullOrEmpty(city.production) ? "No production selected" : $"Building: {city.production}");
                if (city.buildingNames != null && city.buildingNames.Length > 0)
                    Label(panel, "Built: " + string.Join(", ", city.buildingNames), "muted");
                if (city.bombardTargets != null && city.bombardTargets.Length > 0)
                {
                    Button(panel, "City ranged strike", () =>
                    {
                        game.BombardMode = true;
                        game.Message = "Select a highlighted enemy target within range.";
                        game.World.ShowSites(city.bombardTargets);
                        Refresh();
                    });
                }
                Button(panel, "Choose production / purchase", () => OpenDrawer("production"), "primary");
            }
            var tile = game.SelectedTile;
            if (tile != null && tile.known)
            {
                Label(panel, Humanize(tile.terrain), "eyebrow");
                if (!string.IsNullOrEmpty(tile.feature)) Label(panel, Humanize(tile.feature), "muted");
                if (!string.IsNullOrEmpty(tile.resource)) Label(panel, "Resource: " + Humanize(tile.resource), "muted");
                if (!string.IsNullOrEmpty(tile.improvement)) Label(panel, "Improvement: " + Humanize(tile.improvement), "muted");
                if (!string.IsNullOrEmpty(tile.district)) Label(panel, "District: " + Humanize(tile.district), "muted");
                if (!string.IsNullOrEmpty(tile.wonder)) Label(panel, "Wonder: " + Humanize(tile.wonder), "muted");
                if (tile.yields != null && tile.yields.Length >= 6)
                    Label(panel, $"Food {tile.yields[0]} • Prod {tile.yields[1]} • Gold {tile.yields[2]} • Sci {tile.yields[3]} • Cul {tile.yields[4]} • Faith {tile.yields[5]}", "muted");
            }
        }
        void Drawer(string title)
        {
            var s = game.State;
            var panel = Panel("drawer");
            Button(panel, "← Back to map [Esc]", () => OpenDrawer(""));
            Label(panel, Humanize(title), "heading");
            var list = new ScrollView();
            list.style.flexGrow = 1;
            panel.Add(list);

            if (title == "research" || title == "ai")
            {
                string tree = title == "ai" ? "ai" : "tech";
                foreach (var tech in s.research.Where(t => t.tree == tree && (t.current || t.available)).OrderByDescending(t => t.current).ThenBy(t => t.cost))
                {
                    var card = new VisualElement(); card.AddToClassList("card"); list.Add(card);
                    Label(card, tech.name, "card-title");
                    Label(card, $"{tech.progress} / {tech.cost} {(tree == "ai" ? "Compute" : "Science")}  •  {Humanize(tech.era)}", "muted");
                    if (!string.IsNullOrEmpty(tech.description)) Label(card, tech.description, "muted");
                    if (!string.IsNullOrEmpty(tech.eureka)) Label(card, (tech.boosted ? "Boosted: " : "Eureka: ") + tech.eureka, "muted");
                    Button(card, tech.current ? "Researching" : "Research", () => { game.Send(new Command("research") { index = tech.index, tree = tech.tree }); OpenDrawer(""); }).SetEnabled(!tech.current);
                }
            }
            else if (title == "production")
            {
                var city = s.cities.FirstOrDefault(c => c.id == game.SelectedCity) ?? s.cities.FirstOrDefault(c => c.owner == s.human);
                if (city == null) return;
                game.SelectedCity = city.id;
                Label(list, city.name, "subtitle");
                if (city.faithUnits != null && city.faithUnits.Length > 0)
                {
                    Label(list, "FAITH PURCHASES", "eyebrow");
                    foreach (var fu in city.faithUnits)
                    {
                        var card = new VisualElement(); card.AddToClassList("card"); list.Add(card);
                        Label(card, fu.name, "card-title");
                        Button(card, $"Recruit · {fu.cost} Faith", () => game.Send(new Command("faith_purchase") { city = city.id, index = fu.index })).SetEnabled(s.faith >= fu.cost);
                    }
                }
                Label(list, "PRODUCTION & GOLD PURCHASES", "eyebrow");
                foreach (var item in city.choices)
                {
                    var card = new VisualElement(); card.AddToClassList("card"); list.Add(card);
                    Label(card, item.name, "card-title");
                    Label(card, $"{Humanize(item.kind)}  •  {item.cost} production", "muted");
                    Button(card, "Build", () => { page = ""; game.ChooseProduction(item); });
                    if (item.purchase >= 0 && item.kind != "district" && item.kind != "wonder")
                        Button(card, $"Buy · {item.purchase} Gold", () => game.Send(new Command("purchase") { city = city.id, index = item.index, kind = item.kind })).SetEnabled(s.gold >= item.purchase);
                }
            }
            else if (title == "government")
            {
                Label(list, $"Current Government: {s.government}", "subtitle");
                if (s.policySlots != null && s.policySlots.Length > 0)
                    Label(list, "Slots: " + string.Join("  •  ", s.policySlots.Select(ps => $"{Humanize(ps.slot)}: {ps.count}")), "muted");
                if (s.policyDraft != null && s.policyDraft.Length > 0)
                {
                    Label(list, "CHOOSE A NEW POLICY CARD", "eyebrow");
                    foreach (var draft in s.policyDraft)
                    {
                        var card = new VisualElement(); card.AddToClassList("card"); list.Add(card);
                        Label(card, $"{draft.name} ({Humanize(draft.slot)})", "card-title");
                        Label(card, draft.description, "muted");
                        Button(card, "Adopt card", () => game.Send(new Command("adopt_policy") { index = draft.index }), "primary");
                    }
                }
                if (s.governments != null && s.governments.Length > 0)
                {
                    Label(list, "GOVERNMENTS", "eyebrow");
                    foreach (var gov in s.governments)
                    {
                        var card = new VisualElement(); card.AddToClassList("card"); list.Add(card);
                        Label(card, gov.name, "card-title");
                        Label(card, gov.description, "muted");
                        Button(card, gov.current ? "Current Government" : "Adopt Government", () => game.Send(new Command("government") { index = gov.index })).SetEnabled(!gov.current);
                    }
                }
                if (s.adoptedPolicies != null && s.adoptedPolicies.Length > 0)
                {
                    Label(list, "ADOPTED POLICY CARDS", "eyebrow");
                    foreach (var pol in s.adoptedPolicies)
                    {
                        var card = new VisualElement(); card.AddToClassList("card"); list.Add(card);
                        Label(card, $"{pol.name} ({Humanize(pol.slot)})", "card-title");
                        Label(card, pol.description, "muted");
                        Button(card, pol.slotted ? "Unslot card" : "Slot card", () => game.Send(new Command("toggle_policy") { index = pol.index }));
                    }
                }
            }
            else if (title == "religion")
            {
                Label(list, string.IsNullOrEmpty(s.pantheon) ? "No Pantheon chosen yet" : $"Pantheon: {s.pantheon}", "subtitle");
                if (!string.IsNullOrEmpty(s.religion))
                    Label(list, $"Religion: {s.religion}", "subtitle");
                if (s.pantheons != null && s.pantheons.Length > 0)
                {
                    Label(list, "CHOOSE A PANTHEON", "eyebrow");
                    foreach (var pan in s.pantheons)
                    {
                        var card = new VisualElement(); card.AddToClassList("card"); list.Add(card);
                        Label(card, pan.name, "card-title");
                        Label(card, pan.description, "muted");
                        Button(card, "Adopt Pantheon", () => { game.Send(new Command("pantheon") { index = pan.index }); OpenDrawer(""); }, "primary");
                    }
                }
                else if (string.IsNullOrEmpty(s.pantheon))
                {
                    Label(list, "Accumulate 25 Faith to found a Pantheon, then earn a Great Prophet for a Religion.", "muted");
                }
            }
            else if (title == "great_people")
            {
                if (s.greatPeople != null)
                    foreach (var gp in s.greatPeople)
                    {
                        var card = new VisualElement(); card.AddToClassList("card"); list.Add(card);
                        Label(card, $"{gp.name} ({gp.className})", "card-title");
                        Label(card, $"{gp.points} / {gp.cost} pts (+{gp.rate}/turn)  •  {Humanize(gp.era)}", "muted");
                        Label(card, gp.effect, "muted");
                    }
            }
            else if (title == "city_states")
            {
                Label(list, $"Envoys available: {s.envoysAvailable}", "subtitle");
                if (s.cityStates != null && s.cityStates.Length > 0)
                    foreach (var cs in s.cityStates)
                    {
                        var card = new VisualElement(); card.AddToClassList("card"); list.Add(card);
                        Label(card, $"{cs.name} ({Humanize(cs.category)})", "card-title");
                        Label(card, $"Your Envoys: {cs.envoys}  •  Suzerain: {cs.suzerain}", "muted");
                        Button(card, "Send Envoy (+1)", () => game.Send(new Command("diplomacy") { player = cs.id, action = "envoy" })).SetEnabled(s.envoysAvailable > 0);
                    }
                else Label(list, "Explore the world to meet city-states.", "muted");
            }
            else if (title == "leaders")
            {
                foreach (var player in s.players.Where(p => p.id != s.human))
                {
                    var card = new VisualElement(); card.AddToClassList("card"); list.Add(card);
                    string status = player.war ? "At war" : player.alliance ? "Allied" : player.friendship ? "Declared friend" : player.denounced ? "Denounced" : "At peace";
                    Label(card, player.name, "card-title");
                    Label(card, $"{player.civilization}  •  {status}  •  Score {player.score}  •  Opinion {player.opinion:+#;-#;0}", "muted");
                    if (!string.IsNullOrEmpty(player.agendaNote)) Label(card, player.agendaNote, "muted");
                    if (player.war)
                    {
                        if (player.canPeace) Button(card, "Propose peace", () => game.Send(new Command("diplomacy") { player = player.id, action = "peace" }));
                    }
                    else
                    {
                        if (!player.friendship) Button(card, "Declare friendship", () => game.Send(new Command("diplomacy") { player = player.id, action = "friendship" }));
                        if (player.canAlly) Button(card, "Form alliance", () => game.Send(new Command("diplomacy") { player = player.id, action = "alliance" }));
                        if (!player.denounced && !player.friendship) Button(card, "Denounce", () => game.Send(new Command("diplomacy") { player = player.id, action = "denounce" }));
                        if (player.canWar) Button(card, "Declare war", () => game.Send(new Command("diplomacy") { player = player.id, action = "war" }));
                    }
                }
            }
            else if (title == "eras")
            {
                Label(list, $"{s.era}  •  {s.age}", "subtitle");
                Label(list, $"Era Score: {s.eraScore} (Dark < {s.darkThreshold}, Golden >= {s.goldThreshold})  •  Next: {s.projectedAge}", "muted");
                if (s.dedications != null && s.dedications.Length > 0)
                {
                    Label(list, "CHOOSE AN ERA DEDICATION", "eyebrow");
                    foreach (var ded in s.dedications)
                    {
                        var card = new VisualElement(); card.AddToClassList("card"); list.Add(card);
                        Label(card, ded.name, "card-title");
                        Label(card, ded.description, "muted");
                        Button(card, "Choose Dedication", () => { game.Send(new Command("dedication") { index = ded.index }); OpenDrawer(""); }, "primary");
                    }
                }
                Label(list, "HISTORIC MOMENTS", "eyebrow");
                if (s.moments != null)
                    foreach (var m in s.moments) Label(list, m, "card");
            }
            else if (title == "victory")
            {
                if (s.victory != null)
                    foreach (var v in s.victory)
                    {
                        var card = new VisualElement(); card.AddToClassList("card"); list.Add(card);
                        Label(card, $"{v.name} Victory — {Mathf.RoundToInt(v.frac * 100)}%", "card-title");
                        Label(card, v.detail, "muted");
                    }
            }
            else if (title == "chronicle")
            {
                foreach (var note in s.notifications) Label(list, note, "card");
            }
        }
        static string Humanize(string id) => System.Globalization.CultureInfo.InvariantCulture.TextInfo.ToTitleCase((id ?? "").Replace("TERRAIN_", "").Replace("FEATURE_", "").Replace("RESOURCE_", "").Replace("IMPROVEMENT_", "").Replace("DISTRICT_", "").Replace("BUILDING_", "").Replace("ERA_", "").Replace("SLOT_", "").Replace('_', ' ').ToLowerInvariant());
    }
}
