using System;
using System.Runtime.InteropServices;
using System.Text;
using UnityEngine;

namespace WayCiv
{
    public static class NativeGame
    {
        [DllImport("wayciv_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr wc_request(byte[] input);
        [DllImport("wayciv_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern void wc_free(IntPtr response);
        public static T Request<T>(Command command)
        {
            var ptr = wc_request(Encoding.UTF8.GetBytes(JsonUtility.ToJson(command) + "\0"));
            if (ptr == IntPtr.Zero) throw new InvalidOperationException("The game engine returned no response.");
            try
            {
                var text = Marshal.PtrToStringUTF8(ptr);
                var result = JsonUtility.FromJson<Response<T>>(text);
                if (!result.ok) throw new InvalidOperationException(result.error);
                return result.data;
            }
            finally { wc_free(ptr); }
        }
    }
    [Serializable] public class Response<T> { public bool ok; public string error; public T data; }
    [Serializable] public class Command
    {
        public string command, action, tree, kind, leader, map = "random", size = "small", length = "500";
        public int seed = 42, opponents = 3, cityStates = 4, unit, city, index, x, y, player;
        public Command(string name) { command = name; }
    }
    [Serializable] public class Snapshot
    {
        public int version, turn, turnLimit, human, width, height, gold, faith, netGold, tourism, envoysAvailable;
        public int eraScore, darkThreshold, goldThreshold;
        public float compute;
        public bool aiUnlocked;
        public string year, era, age, projectedAge, government, pantheon, religion, winner;
        public float[] income;
        public TileData[] tiles;
        public UnitData[] units;
        public CityData[] cities;
        public ResearchData[] research;
        public PlayerData[] players;
        public CityStateData[] cityStates;
        public GovernmentData[] governments;
        public PolicySlotData[] policySlots;
        public PolicyCardData[] policyDraft;
        public PolicyCardData[] adoptedPolicies;
        public BeliefData[] pantheons;
        public DedicationData[] dedications;
        public GreatPersonData[] greatPeople;
        public VictoryData[] victory;
        public BlockerData[] blockerItems;
        public string[] moments, notifications, blockers;
    }
    [Serializable] public class HexPos { public int x, y; }
    [Serializable] public class TileData : HexPos
    {
        public bool known, visible, river, road, worked;
        public int owner, riverMask, shoreMask;
        public string terrain, feature, resource, improvement, district, wonder;
        public int[] yields;
    }
    [Serializable] public class UnitData : HexPos
    {
        public int id, owner, hp, moves, charges;
        public bool fortified, sleeping;
        public string asset, name, domain;
        public OrderData[] actions;
        public HexPos[] rebaseTargets;
    }
    [Serializable] public class CityData : HexPos
    {
        public int id, owner, era, hp, wallHp, maxWalls, population, housing, amenities, amenitiesNeeded, turnsToGrow;
        public bool capital;
        public string name, production;
        public string[] buildings, buildingNames;
        public float[] yields;
        public ProductionData[] choices;
        public FaithUnitData[] faithUnits;
        public HexPos[] bombardTargets;
    }
    [Serializable] public class FaithUnitData { public int index, cost; public string name; }
    [Serializable] public class OrderData { public string id, name; }
    [Serializable] public class ProductionData
    {
        public int index, cost, purchase;
        public string kind, name;
        public HexPos[] sites;
    }
    [Serializable] public class ResearchData
    {
        public string tree, name, era, description, eureka;
        public int index, cost, progress;
        public bool known, available, current, boosted;
    }
    [Serializable] public class PlayerData
    {
        public int id, score, opinion, grievances, military, cities;
        public int[] color;
        public string name, civilization, agendaNote;
        public bool alive, war, friendship, alliance, denounced, canWar, canPeace, canAlly;
    }
    [Serializable] public class CityStateData
    {
        public int id, envoys;
        public int[] color;
        public string name, category, suzerain, bonus;
    }
    [Serializable] public class GovernmentData
    {
        public int index;
        public string name, description;
        public bool current;
    }
    [Serializable] public class PolicySlotData { public string slot; public int count; }
    [Serializable] public class PolicyCardData
    {
        public int index;
        public string name, slot, description;
        public bool slotted;
    }
    [Serializable] public class BeliefData { public int index; public string name, description; }
    [Serializable] public class DedicationData { public int index; public string name, moment, description; }
    [Serializable] public class GreatPersonData
    {
        public int classIndex, personIndex, points, cost, rate;
        public string className, name, era, effect;
    }
    [Serializable] public class VictoryData { public string name, detail; public float frac; }
    [Serializable] public class BlockerData { public string kind, text; public int unit, city; }
    [Serializable] public class LeaderData { public string id, name, civilization; }
    [Serializable] public class LeaderCatalog { public LeaderData[] leaders; }
    [Serializable] public class ReachableData { public HexPos[] positions; }
    [Serializable] public class AssetCatalog { public AssetRecord[] assets; }
    [Serializable] public class AssetRecord { public string id, asset, path, category; public PaletteMaterial[] materials; }
    [Serializable] public class PaletteMaterial { public string name; public float[] rgba; }
}
