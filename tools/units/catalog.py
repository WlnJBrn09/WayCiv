"""Unit id -> model recipe.  spec_for() gives framing; build() makes the model.

Unique units reuse the model of the unit they replace with a distinguishing
accessory or colour (see the UNIQUE section)."""
from lib import *
from figures import *
from mounts import *
from vehicles import *
import naval
import air

# (ortho width in world units, anchor: how far down the frame the ground point is)
SIZE = dict(fig=(2.4, 0.80), mount=(3.4, 0.74), big=(4.2, 0.72), huge=(5.2, 0.74), tall=(5.6, 0.86), ship=(6.2, 0.66), bigship=(8.0, 0.66),
            air=(4.6, 0.5), airbig=(6.6, 0.5), small=(2.0, 0.78))
YAW = -38

S = lambda **kw: kw                     # shorthand for a spec
sh = lambda side=NEAR, **kw: ("shield", dict(side=side, **kw))
it = lambda name, **kw: (name, kw)

# ---------------------------------------------------------------- humans
H = {}


def hum(uid, **o):
    H[uid] = o


# civilians
hum("UNIT_BUILDER", body="tunic", top="undyed", legs="brown", helmet="headband", team=("hat", "sash"), hat_c="team",
    items=[it("pick", side=FAR), it("bag", side=NEAR)], skin="tan")
hum("UNIT_MISSIONARY", body="robe", top="white", helmet="hood", hat_c="white", team=("sash",), beard=None,
    items=[it("staff", side=FAR, topper="cross"), it("book", side=NEAR)])
hum("UNIT_APOSTLE", body="robe", top="white", team=("cape", "sash"), helmet="mitre", hat_c="team", hair="hair_grey", beard="hair_grey",
    items=[it("staff", side=FAR, topper="crook", c="gold"), it("book", side=NEAR)])
hum("UNIT_INQUISITOR", body="robe", top="#2a2430", team=("sash",), helmet="hood", hat_c="#2a2430",
    items=[it("torch", side=FAR), it("book", side=NEAR, c="#4a1a1a")])
hum("UNIT_GURU", body="robe", top="orange", team=("sash", "hat"), helmet="turban", hat_c="team", beard="hair_grey", hair="hair_grey", skin="tan",
    items=[it("staff", side=FAR, topper="orb")])
hum("UNIT_WARRIOR_MONK", body="bare", legs="orange", team=("sash", "hat"), helmet="headband", skin="tan", barearms=True,
    items=[it("staff", side=FAR), it("shield", side=NEAR, kind="round", r=0.24, c="orange", rim="gold", boss=False)], hair="hair_black")
hum("UNIT_ARCHAEOLOGIST", body="uniform", top="khaki", legs="#6b5a3a", helmet="pith", team=("scarf", "backpack"),
    items=[it("shovel", side=FAR), it("bag", side=NEAR, c="leather")])
hum("UNIT_NATURALIST", body="uniform", top="#7a8a50", legs="#5d6140", helmet="widehat", hat_c="#8a7a50", team=("scarf", "backpack"),
    items=[it("binoculars")])
hum("UNIT_SPY", body="coat", top="#2a2a34", legs="#2a2a34", helmet="widehat", hat_c="#2a2a34", team=("scarf",),
    items=[it("dagger", side=FAR)], boots="black")
hum("UNIT_MEDIC", body="uniform", top="white", legs="#4a5a40", helmet="modern", hat_c="white", team=("sash",),
    items=[it("medic_bag", side=NEAR), it("flag", side=FAR, c="white")])
hum("UNIT_MILITARY_ENGINEER", body="uniform", top="#8a6a40", legs="#4a4036", helmet="hardhat", team=("tabard",),
    items=[it("hammer", side=FAR), it("bag", side=NEAR, c="#5a5a5a")])
for g, kw in dict(
    GREAT_GENERAL=dict(body="plate", metal="steel", helmet="bronze", hat_c="gold", team=("cape",), items=[it("sword", side=FAR), sh(NEAR, r=0.3, kind="kite", rim="gold")]),
    GREAT_ADMIRAL=dict(body="coat", top="navy", legs="white", helmet="bicorne", team=("cape", "hat"), items=[it("spyglass", side=FAR)], boots="black"),
    GREAT_ENGINEER=dict(body="tunic", top="#9a6a40", legs="brown", helmet="cap", hat_c="#5a5a5a", team=("tabard", "cape"), items=[it("hammer", side=FAR), it("gear", side=NEAR)]),
    GREAT_MERCHANT=dict(body="robe", top="team", team=("top", "hat"), helmet="turban", hat_c="team", items=[it("coins", side=FAR), it("scroll", side=NEAR)], skin="tan"),
    GREAT_PROPHET=dict(body="robe", top="white", team=("cape", "sash"), hair="hair_grey", beard="hair_grey", hairstyle="long", halo=True, items=[it("staff", side=FAR, topper="orb"), it("book", side=NEAR)]),
    GREAT_SCIENTIST=dict(body="robe", top="dkblue", team=("cape", "sash"), hair="hair_grey", hairstyle="short", beard="hair_grey", items=[it("flask", side=FAR), it("scroll", side=NEAR)]),
    GREAT_WRITER=dict(body="robe", top="#7a4a3a", team=("cape", "hat"), helmet="beret", hat_c="team", items=[it("quill", side=FAR), it("book", side=NEAR, c="brown")]),
    GREAT_ARTIST=dict(body="tunic", top="#8a4a6a", legs="dkgrey", team=("hat", "sash"), helmet="beret", hat_c="team", items=[it("palette", side=NEAR), it("quill", side=FAR)], hairstyle="long"),
    GREAT_MUSICIAN=dict(body="coat", top="purple", legs="dkgrey", team=("hat", "sash"), helmet="widehat", hat_c="team", items=[it("lute", side=NEAR)]),
).items():
    hum("UNIT_" + g, aura=True, **kw)
hum("UNIT_COMANDANTE_GENERAL", aura=True, body="coat", top="dkblue", legs="white", helmet="bicorne", team=("cape", "hat"),
    items=[it("saber", side=FAR), sh(NEAR, r=0.0001, kind="round")], boots="black")
H["UNIT_COMANDANTE_GENERAL"]["items"] = [it("saber", side=FAR)]

# melee, ranged, anti-cavalry
hum("UNIT_SCOUT", body="tunic", top="#6a7a4a", legs="brown", helmet="hood", hat_c="#6a7a4a", team=("scarf",),
    items=[it("spear", side=FAR, length=1.9), it("bag", side=NEAR)])
hum("UNIT_WARRIOR", body="bare", legs="leather", helmet=None, hair="hair_black", hairstyle="long", skin="tan",
    items=[it("club", side=FAR), sh(NEAR, kind="round", r=0.32)], boots="leather")
hum("UNIT_SLINGER", body="tunic", top="undyed", legs="brown", helmet="headband", hat_c="team", team=("hat",), skin="tan",
    items=[it("sling", side=FAR), it("bag", side=NEAR)])
hum("UNIT_ARCHER", body="tunic", top="#6b7a45", legs="brown", helmet="headband", team=("hat",), skin="tan", quiver=True,
    items=[it("bow", side=FAR)])
hum("UNIT_SPEARMAN", body="armor", under="undyed", metal="bronze", helmet="bronze", team=("tabard",), legs="undyed",
    items=[it("spear", side=FAR), sh(NEAR, kind="round", r=0.34)])
hum("UNIT_SWORDSMAN", body="armor", under="#7a5a3c", metal="iron", helmet="conical", hat_c="iron", team=(), legs="brown",
    items=[it("sword", side=FAR), sh(NEAR, kind="kite", r=0.28, rim="iron")])
hum("UNIT_PIKEMAN", body="armor", under="#7a3a30", metal="iron", helmet="kettle", team=("tabard",), legs="#4a4a50",
    items=[it("pike", side=FAR)])
hum("UNIT_MAN_AT_ARMS", body="plate", metal="steel", helmet="kettle", team=("tabard",), legs="#4a4a50",
    items=[it("sword", side=FAR), sh(NEAR, kind="kite", r=0.28, rim="steel")])
hum("UNIT_CROSSBOWMAN", body="armor", under="#3a4a6a", metal="iron", helmet="kettle", team=("tabard",), legs="#4a3a30",
    items=[it("crossbow")], quiver=True)
hum("UNIT_MUSKETMAN", body="coat", top="team", team=("top",), legs="white", helmet="tricorn", hat_c="black",
    items=[it("musket")], boots="black")
hum("UNIT_LINE_INFANTRY", body="coat", top="team", team=("top",), legs="white", helmet="shako", hat_c="black", belt="white",
    items=[it("musket")], boots="black")
hum("UNIT_RANGER", body="coat", top="#5a6a3a", team=("scarf",), legs="#4a4036", helmet="widehat", hat_c="#6a5a3a",
    items=[it("musket", rifle=True, bayonet=False)], boots="leather")
hum("UNIT_FIELD_CANNON_CREW", body="coat", top="team", team=("top",), legs="white", helmet="shako", hat_c="black", items=[], boots="black")
hum("UNIT_INFANTRY", body="uniform", top="#6a6a40", legs="#5a5a3a", helmet="modern", hat_c="olive", team=("scarf", "backpack"),
    items=[it("musket", rifle=True, bayonet=False)], boots="black")
hum("UNIT_MODERN_AT", body="uniform", top="#4a5a45", legs="#3a4a35", helmet="modern", hat_c="#3a4a35", team=("scarf", "backpack"),
    items=[it("launcher")], boots="black")
hum("UNIT_MACHINE_GUN", body="uniform", top="#6a6a40", legs="#5a5a3a", helmet="modern", hat_c="olive", team=("scarf",),
    items=[], boots="black", stride=0.0, hands={NEAR: Vector((0.7, -0.15, 0.85)), FAR: Vector((0.7, 0.15, 0.85))})

# unique infantry
hum("UNIT_GREEK_HOPLITE", **dict(H["UNIT_SPEARMAN"], helmet="bronze", team=("hat",), hat_c="bronze", hair="hair_black",
    items=[it("spear", side=FAR), sh(NEAR, kind="round", r=0.4, rim="bronze")]))
hum("UNIT_ROMAN_LEGION", body="armor", under="#a8382c", metal="iron", helmet="bronze", hat_c="iron", team=("tabard",), legs="#a8382c",
    items=[it("sword", side=FAR, raised=False, length=0.7), sh(NEAR, kind="rect", r=0.3, c="team", rim="gold")])
hum("UNIT_KONGO_SHIELD_BEARER", body="bare", legs="#c8a050", helmet="feather", skin="dark_skin", hair="hair_black", team=("hat",),
    items=[it("spear", side=FAR, length=1.7), sh(NEAR, kind="oval", r=0.34)], barelegs=True, boots="leather")
hum("UNIT_JAPANESE_SAMURAI", body="armor", under="#2a3a5a", metal="dkiron", helmet="kabuto", team=("tabard", "hat"), legs="#2a3a5a",
    items=[it("sword", side=FAR, length=0.95)], skin="tan", hair="hair_black")
hum("UNIT_NORWEGIAN_BERSERKER", body="bare", legs="#5a4a3a", helmet="horned", beard="hair_red", hair="hair_red", hairstyle="long", team=(),
    items=[it("axe", side=FAR, big=True), sh(NEAR, kind="round", r=0.36, c="team", rim="iron")], boots="leather")
hum("UNIT_SPANISH_CONQUISTADOR", body="plate", metal="steel", helmet="morion", team=("tabard",), legs="#8a3a30", beard="hair_black",
    items=[it("sword", side=FAR), sh(NEAR, kind="round", r=0.3, rim="gold")], boots="black")
hum("UNIT_ENGLISH_REDCOAT", **dict(H["UNIT_LINE_INFANTRY"], top="redcoat", team=("hat", "sash"), hat_c="black", helmet="shako"))
hum("UNIT_FRENCH_GARDE_IMPERIALE", **dict(H["UNIT_LINE_INFANTRY"], top="dkblue", helmet="bearskin", team=("hat", "sash")))
hum("UNIT_AZTEC_EAGLE_WARRIOR", body="bare", legs="#c8a050", helmet="feather", skin="brown_skin", hair="hair_black", team=("hat",),
    items=[it("club", side=FAR), sh(NEAR, kind="round", r=0.32, rim="gold")], boots="leather")
hum("UNIT_DIGGER", **dict(H["UNIT_INFANTRY"], helmet="slouch", hat_c="khaki", top="#8a7a4a", legs="#7a6a40", team=("scarf", "backpack")))
hum("UNIT_MACEDONIAN_HYPASPIST", body="armor", under="linen", metal="bronze", helmet="bronze", team=("hat", "tabard"), legs="linen",
    items=[it("pike", side=FAR), sh(NEAR, kind="round", r=0.3, rim="bronze")])
hum("UNIT_PERSIAN_IMMORTAL", body="coat", top="team", team=("top",), legs="#d8b050", helmet="turban", hat_c="white", beard="hair_black", skin="tan",
    items=[it("spear", side=FAR, length=2.0), sh(NEAR, kind="wicker", r=0.3, c="ltwood")], quiver=True, boots="leather")
hum("UNIT_NUBIAN_PITATI", **dict(H["UNIT_ARCHER"], skin="dark_skin", helmet="feather", top="#c8a050", team=("hat",)))
hum("UNIT_MAYAN_HULCHE", **dict(H["UNIT_ARCHER"], skin="brown_skin", helmet="feather", top="#3a8a6a", team=("hat",)))
hum("UNIT_GAUL_GAESATAE", body="bare", legs="team", team=("legs",), helmet=None, skin="skin", hair="hair_red", hairstyle="long", beard="hair_red",
    items=[it("spear", side=FAR), sh(NEAR, kind="oval", r=0.3, c="#7a5a3c", rim="iron")], boots="leather")
hum("UNIT_BABYLONIAN_SABUM_KIBITTUM", body="armor", under="team", metal="bronze", helmet="conical", hat_c="bronze", team=("under",), legs="team",
    beard="hair_black", skin="tan", items=[it("spear", side=FAR), sh(NEAR, kind="wicker", r=0.3, c="ltwood")])
hum("UNIT_LAHORE_NIHANG", body="robe", top="dkblue", team=("hat", "sash"), helmet="dumala", hat_c="team", beard="hair_black", skin="tan",
    items=[it("spear", side=FAR), sh(NEAR, kind="round", r=0.28, rim="silver", c="team")])
hum("UNIT_CHINESE_CROUCHING_TIGER", **dict(H["UNIT_CROSSBOWMAN"], under="orange", team=("sash", "hat"), helmet="hood", hat_c="orange", skin="tan", hair="hair_black"))
hum("UNIT_VIETNAMESE_VOI_CHIEN", **H["UNIT_CROSSBOWMAN"])  # (replaced by elephant below)

TEAMED = {}


# --------------------------------------------------------------- mounted
M = {}


def mnt(uid, rider, **horse_kw):
    M[uid] = (rider, horse_kw)


mnt("UNIT_BARBARIAN_HORSEMAN", dict(body="bare", legs="leather", helmet=None, hair="hair_black", hairstyle="long", skin="tan",
    items=[it("spear", side=FAR, length=2.0), sh(NEAR, kind="round", r=0.28, c="#6a4a30")]), coat="#6a4a30", blanket="team")
mnt("UNIT_BARBARIAN_HORSE_ARCHER", dict(body="bare", legs="leather", helmet="fur", hair="hair_black", skin="tan", quiver=True,
    items=[it("bow", side=FAR)]), coat="#4a3a2a", blanket="team")
mnt("UNIT_SCYTHIAN_HORSE_ARCHER", dict(body="coat", top="#7a4a30", legs="team", team=("legs", "hat"), helmet="cap", hat_c="team", skin="tan", quiver=True,
    items=[it("bow", side=FAR)]), coat="#b8905a", blanket="team")
mnt("UNIT_HORSEMAN", dict(body="tunic", top="team", team=("top",), legs="undyed", helmet="conical", hat_c="bronze",
    items=[it("spear", side=FAR, length=2.0), sh(NEAR, kind="round", r=0.28)]), coat="#a07a50")
mnt("UNIT_KNIGHT", dict(body="plate", metal="steel", helmet="kettle", team=("tabard",), legs="#4a4a50",
    items=[it("lance", side=NEAR)]), coat="#d8d8d8", barding="cloth")
mnt("UNIT_ARABIAN_MAMLUK", dict(body="armor", under="team", metal="steel", helmet="turban", hat_c="white", team=("under",), skin="tan", beard="hair_black",
    items=[it("saber", side=FAR), sh(NEAR, kind="round", r=0.28, rim="gold")]), coat="#e8e0d0", blanket="team")
mnt("UNIT_BYZANTINE_TAGMA", dict(body="armor", under="#a89060", metal="bronze", helmet="conical", hat_c="bronze", team=("tabard",), skin="tan",
    items=[it("lance", side=NEAR)]), coat="#6a4a30", barding="cloth")
mnt("UNIT_MACEDONIAN_HETAIROI", dict(body="armor", under="linen", metal="bronze", helmet="bronze", team=("cape",), legs="undyed",
    items=[it("spear", side=FAR, length=2.1)]), coat="#c8a878", blanket="team")
mnt("UNIT_POLISH_HUSSAR", dict(body="plate", metal="steel", helmet="kettle", team=("tabard",), legs="#8a3a30", wings=True,
    items=[it("lance", side=NEAR, length=3.2)]), coat="#d8d8d8", blanket="team", plume="team")
mnt("UNIT_CAVALRY", dict(body="coat", top="team", team=("top",), legs="white", helmet="shako", hat_c="black", boots="black",
    items=[it("saber", side=FAR)]), coat="#6a4630", blanket="team")
mnt("UNIT_RUSSIAN_COSSACK", dict(body="coat", top="#3a5a3a", team=("sash", "hat"), legs="#2a2a34", helmet="fur", boots="black", beard="hair_black",
    items=[it("saber", side=FAR)]), coat="#4a3a2a", blanket="team")
mnt("UNIT_AMERICAN_ROUGH_RIDER", dict(body="uniform", top="#8a7a4a", legs="#6a5a3a", helmet="slouch", hat_c="khaki", team=("scarf",), boots="leather",
    items=[it("saber", side=FAR)]), coat="#8a5a38", blanket="team")
mnt("UNIT_COLOMBIAN_LLANERO", dict(body="coat", top="team", team=("top",), legs="#d8d0c0", helmet="widehat", hat_c="#5a4a30", boots="leather", skin="tan",
    items=[it("lance", side=NEAR, length=2.8)]), coat="#7a5a3a", blanket="#d8d0c0")
mnt("UNIT_ETHIOPIAN_OROMO_CAVALRY", dict(body="bare", legs="team", team=("legs",), helmet="headband", hat_c="team", skin="dark_skin", hair="hair_black",
    items=[it("spear", side=FAR, length=2.0), sh(NEAR, kind="round", r=0.28, rim="gold")]), coat="#5a3a28", blanket="team")


# ----------------------------------------------------- vehicles & machines
V_ = {}


def veh(uid, fn, size="big", **kw):
    V_[uid] = (fn, size, kw)


for _u, _f, _s in (
    ("GALLEY", naval.galley, "ship"), ("NORWEGIAN_LONGSHIP", naval.longship, "ship"), ("QUADRIREME", naval.quadrireme, "ship"),
    ("BYZANTINE_DROMON", naval.dromon, "ship"), ("CARAVEL", naval.caravel, "ship"), ("PORTUGUESE_NAU", naval.nau, "ship"),
    ("FRIGATE", naval.frigate, "ship"), ("PRIVATEER", naval.privateer, "ship"), ("ENGLISH_SEADOG", naval.sea_dog, "ship"),
    ("INDONESIAN_JONG", naval.jong, "ship"), ("IRONCLAD", naval.ironclad, "ship"), ("BATTLESHIP", naval.battleship, "bigship"),
    ("BRAZILIAN_MINAS_GERAES", naval.minas_geraes, "bigship"), ("DESTROYER", naval.destroyer, "ship"),
    ("MISSILE_CRUISER", naval.missile_cruiser, "bigship"), ("AIRCRAFT_CARRIER", naval.aircraft_carrier, "bigship"),
    ("SUBMARINE", naval.submarine, "ship"), ("GERMAN_UBOAT", naval.uboat, "ship"), ("NUCLEAR_SUBMARINE", naval.nuclear_sub, "ship"),
):
    veh("UNIT_" + _u, (lambda f=_f: f()), _s)
for _u, _f in (("BIPLANE", air.biplane), ("FIGHTER", air.fighter), ("AMERICAN_P51", air.p51), ("BOMBER", air.bomber),
               ("JET_FIGHTER", air.jet_fighter), ("JET_BOMBER", air.jet_bomber)):
    veh("UNIT_" + _u, (lambda f=_f: f()), "airbig" if _u in ("BOMBER", "JET_BOMBER") else "air", pitch=14)
veh("UNIT_SETTLER", wagon, "big")
veh("UNIT_TRADER", camel, "big")
veh("UNIT_CATAPULT", catapult, "big")
veh("UNIT_TREBUCHET", trebuchet, "big")
veh("UNIT_BATTERING_RAM", battering_ram, "big")
veh("UNIT_SIEGE_TOWER", siege_tower, "huge")
veh("UNIT_BOMBARD", bombard, "big")
veh("UNIT_FIELD_CANNON", field_cannon, "big")
veh("UNIT_ARTILLERY", artillery, "big")
veh("UNIT_ROCKET_ARTILLERY", rocket_artillery, "big")
veh("UNIT_MOBILE_SAM", mobile_sam, "big")
veh("UNIT_ANTIAIR_GUN", antiair_gun, "big")
veh("UNIT_AT_CREW", at_gun, "big")
veh("UNIT_TANK", tank, "big")
veh("UNIT_MODERN_ARMOR", modern_armor, "big")
veh("UNIT_MECHANIZED_INFANTRY", ifv, "big")
veh("UNIT_HELICOPTER", helicopter, "huge")
veh("UNIT_OBSERVATION_BALLOON", balloon, "tall")
veh("UNIT_ANDURIL_STRIKE_DRONE", strike_drones, "big")
veh("UNIT_SUMERIAN_WAR_CART", lambda: chariot(dict(horses=1, horse=dict(coat="#8a7a60", mane="#3a2a20"), crew=[(dict(body="tunic", top="#8a5a3a", helmet="conical", hat_c="bronze", items=[("spear", {"side": FAR, "length": 2.0}), ("shield", {"side": NEAR, "kind": "round", "r": 0.26})]), -0.65, 0, 0.8)])), "big")
veh("UNIT_HEAVY_CHARIOT", lambda: chariot(dict(horses=2, horse=dict(coat="#8a5a38", blanket="team"), crew=[
    (dict(body="tunic", top="undyed", helmet="conical", hat_c="bronze", items=[("spear", {"side": FAR, "length": 2.0})]), -0.55, 0.2, 0.75),
    (dict(body="tunic", top="brown", helmet="cap", items=[("staff", {"side": FAR})]), -0.85, -0.2, 0.7)])), "big")
veh("UNIT_EGYPTIAN_CHARIOT_ARCHER", lambda: chariot(dict(horses=2, body_c="#d8a830", horse=dict(coat="#e8e0d0", blanket="team", plume="team"), crew=[
    (dict(body="tunic", top="linen", legs="linen", helmet="headband", hat_c="team", team=("hat",), skin="tan", items=[("bow", {"side": FAR})], quiver=True), -0.55, 0.0, 0.8),
    (dict(body="tunic", top="team", team=("top",), helmet="headband", skin="tan", items=[("staff", {"side": FAR})]), -0.88, 0.0, 0.7)])), "big")
veh("UNIT_INDIAN_VARU", lambda: elephant(dict(crew=[
    (dict(body="armor", under="#a89060", metal="bronze", helmet="conical", hat_c="bronze", team=("tabard",), items=[("spear", {"side": FAR}), ("shield", {"side": NEAR, "kind": "round", "r": 0.3})]), -0.1, 0.0, 0.7)],
    mahout=dict(body="tunic", top="orange", helmet="turban", hat_c="white", skin="tan", items=[("staff", {"side": FAR})]))), "huge")
veh("UNIT_VIETNAMESE_VOI_CHIEN", lambda: elephant(dict(crew=[
    (dict(body="armor", under="#3a4a6a", metal="iron", helmet="conical", hat_c="iron", team=("tabard",), items=[("crossbow", {})]), -0.3, 0.0, 0.7),
    (dict(body="armor", under="#3a4a6a", metal="iron", helmet="conical", hat_c="iron", team=("tabard",), items=[("crossbow", {})]), 0.1, 0.0, 0.7)],
    mahout=dict(body="tunic", top="#4a8a4a", helmet="conical", hat_c="#c8a050", skin="tan", items=[("staff", {"side": FAR})]))), "huge")


def domrey():
    elephant(dict(howdah=False, blanket="team", mahout=dict(body="tunic", top="#4a8a4a", helmet="conical", hat_c="#c8a050", skin="tan",
                                                           items=[("staff", {"side": FAR})])))
    box((-0.1, 0, 2.3), (1.1, 0.95, 0.1), "dkwood", bevel=0.03)
    for sy in (-1, 1):
        box((-0.1, sy * 0.48, 2.45), (1.1, 0.05, 0.22), "ltwood", bevel=0.02)
    # ballista: stock, bow arms, bolt
    beam((-0.4, 0, 2.55), (0.8, 0, 2.7), 0.12, 0.12, "ltwood")
    limb((0.5, -0.65, 2.7), (0.5, 0.65, 2.7), 0.05, "dkwood", seg=6)
    limb((0.5, -0.65, 2.7), (0.2, 0.0, 2.7), 0.01, "white", seg=4)
    limb((0.5, 0.65, 2.7), (0.2, 0.0, 2.7), 0.01, "white", seg=4)
    limb((0.2, 0, 2.72), (1.3, 0, 2.85), 0.035, "ltwood", seg=5)
    limb((1.3, 0, 2.85), (1.5, 0, 2.88), 0.07, metal("steel"), r1=0.0, seg=5)
    box((0.1, 0, 2.55), (0.3, 0.22, 0.2), "team", bevel=0.03)
    with xf((-0.55, 0.2, 2.35), (0, 0, 0), 0.65):
        human(dict(body="tunic", top="team", team=("top",), helmet="conical", hat_c="bronze", skin="tan", items=[("spear", {"side": FAR, "length": 1.8})]))


veh("UNIT_KHMER_DOMREY", domrey, "huge")


def build_human(uid):
    if uid == "UNIT_MACHINE_GUN":
        with xf((-0.2, 0, 0), (0, 0, 0), 0.9):
            human(H[uid])
        with xf((0.3, 0, 0), (0, 0, 0), 0.9):
            item_mg({})
        return
    human(H[uid])


# --------------------------------------------------------------- entry points
def spec_for(uid):
    if uid in V_:
        k = V_[uid][1]
        o, a = SIZE[k]
        return dict(ortho=o, anchor=a, yaw=YAW, lift=V_[uid][2].get("lift", 0.0), pitch=V_[uid][2].get("pitch", 0))
    if uid in H:
        k = "fig"
    elif uid in M:
        k = "mount"
    else:
        k = "fig"
    o, a = SIZE[k]
    return dict(ortho=o, anchor=a, yaw=YAW)


def build(uid):
    if uid in V_:
        V_[uid][0]()
    elif uid in H:
        build_human(uid)
    elif uid in M:
        rider, kw = M[uid]
        horse(dict(kw, rider=rider))
    else:
        raise KeyError(uid)


def kind_for(uid):
    """Portrait backdrop: sea, air or land."""
    if uid in V_:
        sz = V_[uid][1]
        if sz in ("ship", "bigship"):
            return "sea"
        if sz in ("air", "airbig") or uid in ("UNIT_HELICOPTER", "UNIT_OBSERVATION_BALLOON", "UNIT_ANDURIL_STRIKE_DRONE"):
            return "air"
    return "land"


def all_ids():
    ids = list(H) + list(M) + list(V_)
    return [u for u in dict.fromkeys(ids) if u != "UNIT_FIELD_CANNON_CREW"]
