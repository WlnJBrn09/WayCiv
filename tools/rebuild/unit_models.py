"""Individual, editable, rigged and animated unit sources from authored recipes."""
import sys, math, bpy
from pathlib import Path
from mathutils import Matrix, Vector
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).parent))
from architecture_models import main
sys.path.insert(0, str(ROOT / 'tools/units'))
import lib, catalog, naval

HEAD_PARTS = {'head', 'jaw', 'nose', 'ear', 'eyewhite', 'eye', 'brow', 'beard', 'moustache', 'hair', 'fringe', 'sideburn', 'hairback', 'bun', 'cap', 'helm', 'cheek', 'crest', 'crestbase', 'cone', 'comb', 'brim', 'brim2', 'bearskin', 'hood', 'bicorne'}


def _wake_3d(L, B, y=0.0):
    """Slender 3D V-shaped bow-wave ribbons instead of a solid oval sprite plate."""
    for sy in (-1, 1):
        lib.limb((L * 0.42, y + sy * B * 0.18, 0.012), (-L * 0.46, y + sy * B * 0.56, 0.008), 0.022, "#eaf6fa", r1=0.01, seg=6, name="foam")


naval.wake = _wake_3d


def _choose_bone(ob, max_z, kind):
    part = (ob.get('part_name') or ob.name).lower()
    verts = [v.co for v in ob.data.vertices]
    if not verts:
        return 'Body'
    avg = sum(verts, Vector((0, 0, 0))) / len(verts)
    if part in HEAD_PARTS or (kind == 'land' and avg.z > max_z * 0.68 and abs(avg.y) < 0.16):
        return 'Head'
    if kind == 'land' and avg.z < max_z * 0.34 and abs(avg.y) > 0.025 and part in ('limb', 'box', 'cyl', 'torus'):
        return 'LeftLeg' if avg.y < 0 else 'RightLeg'
    if avg.z >= max_z * 0.34 and abs(avg.y) > 0.085 and part in ('limb', 'beam', 'poly', 'shield', 'wing'):
        return 'LeftArm' if avg.y < 0 else 'RightArm'
    return 'Body'


def _rig_and_animate(uid, objects, max_z, kind):
    name = f"{uid.lower()}_rig"
    arm_data = bpy.data.armatures.new(name)
    arm_obj = bpy.data.objects.new(name, arm_data)
    bpy.context.scene.collection.objects.link(arm_obj)
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.object.mode_set(mode='EDIT')
    hz = max(max_z, 0.35)
    eb = arm_data.edit_bones
    b_root = eb.new('Root'); b_root.head = (0, 0, 0); b_root.tail = (0, 0, hz * 0.2)
    b_body = eb.new('Body'); b_body.head = (0, 0, hz * 0.2); b_body.tail = (0, 0, hz * 0.65); b_body.parent = b_root
    b_head = eb.new('Head'); b_head.head = (0, 0, hz * 0.65); b_head.tail = (0, 0, hz); b_head.parent = b_body
    for side, sgn in (('Left', -1), ('Right', 1)):
        leg = eb.new(f'{side}Leg')
        leg.head = (0, sgn * 0.08, hz * 0.34)
        leg.tail = (0, sgn * 0.08, 0.02)
        leg.parent = b_body
        arm = eb.new(f'{side}Arm')
        arm.head = (0, sgn * 0.14, hz * 0.56)
        arm.tail = (0.14, sgn * 0.16, hz * 0.42)
        arm.parent = b_body
    bpy.ops.object.mode_set(mode='OBJECT')

    for ob in objects:
        ob.parent = arm_obj
        bone_name = _choose_bone(ob, hz, kind)
        vg = ob.vertex_groups.new(name=bone_name)
        vg.add(list(range(len(ob.data.vertices))), 1.0, 'REPLACE')
        mod = ob.modifiers.new(name='Armature', type='ARMATURE')
        mod.object = arm_obj

    arm_obj.animation_data_create()
    pb = arm_obj.pose.bones
    for b in pb:
        b.rotation_mode = 'XYZ'

    def reset_pose():
        for b in pb:
            b.location = (0, 0, 0)
            b.rotation_euler = (0, 0, 0)

    def key_all(frame):
        for bname in ('Body', 'Head', 'LeftLeg', 'RightLeg', 'LeftArm', 'RightArm'):
            pb[bname].keyframe_insert(data_path='location', frame=frame)
            pb[bname].keyframe_insert(data_path='rotation_euler', frame=frame)

    # 1. Idle (breathing / gentle sway or float)
    act_idle = bpy.data.actions.new(f"{uid.lower()}_Idle")
    arm_obj.animation_data.action = act_idle
    reset_pose(); key_all(1)
    pb['Body'].location = (0, 0, 0.018 if kind != 'air' else 0.035)
    pb['Body'].rotation_euler = (0.03 if kind == 'sea' else 0.0, -0.02, 0.0)
    pb['Head'].rotation_euler = (0.0, 0.0, 0.06)
    pb['LeftArm'].rotation_euler = (0.0, 0.04, 0.0)
    pb['RightArm'].rotation_euler = (0.0, -0.04, 0.0)
    key_all(12)
    reset_pose(); key_all(24)

    # 2. Move (walk/gallop stride or vehicle/naval advance)
    act_move = bpy.data.actions.new(f"{uid.lower()}_Move")
    arm_obj.animation_data.action = act_move
    reset_pose(); key_all(1)
    swing = 0.32 if kind == 'land' else 0.08
    pb['Body'].location = (0.02, 0, 0.025)
    pb['Body'].rotation_euler = (0.0, 0.06, 0.0)
    pb['LeftLeg'].rotation_euler = (0.0, swing, 0.0)
    pb['RightLeg'].rotation_euler = (0.0, -swing, 0.0)
    pb['LeftArm'].rotation_euler = (0.0, -swing * 0.7, 0.0)
    pb['RightArm'].rotation_euler = (0.0, swing * 0.7, 0.0)
    key_all(7)
    reset_pose(); pb['Body'].location = (0, 0, -0.01); key_all(13)
    pb['Body'].location = (0.02, 0, 0.025)
    pb['Body'].rotation_euler = (0.0, 0.06, 0.0)
    pb['LeftLeg'].rotation_euler = (0.0, -swing, 0.0)
    pb['RightLeg'].rotation_euler = (0.0, swing, 0.0)
    pb['LeftArm'].rotation_euler = (0.0, swing * 0.7, 0.0)
    pb['RightArm'].rotation_euler = (0.0, -swing * 0.7, 0.0)
    key_all(19)
    reset_pose(); key_all(24)

    # 3. Attack (wind-up, strike/fire lunge, recovery)
    act_atk = bpy.data.actions.new(f"{uid.lower()}_Attack")
    arm_obj.animation_data.action = act_atk
    reset_pose(); key_all(1)
    pb['Body'].location = (-0.04, 0, 0.01)
    pb['Body'].rotation_euler = (0.0, -0.10, -0.08)
    pb['RightArm'].rotation_euler = (0.0, -0.55, 0.15)
    pb['LeftArm'].rotation_euler = (0.0, -0.25, -0.10)
    key_all(7)
    pb['Body'].location = (0.09, 0, 0.02)
    pb['Body'].rotation_euler = (0.0, 0.16, 0.10)
    pb['RightArm'].rotation_euler = (0.0, 0.48, -0.12)
    pb['LeftArm'].rotation_euler = (0.0, 0.22, 0.08)
    pb['Head'].rotation_euler = (0.0, -0.08, 0.0)
    key_all(13)
    reset_pose(); key_all(24)

    for clip, act in (('Idle', act_idle), ('Move', act_move), ('Attack', act_atk)):
        track = arm_obj.animation_data.nla_tracks.new()
        track.name = clip
        strip = track.strips.new(clip, 1, act)
        strip.name = clip
    reset_pose()
    arm_obj.animation_data.action = None


def wrap(uid):
    def build():
        lib.SC = bpy.context.scene
        lib._MATS.clear()
        lib._STACK[:] = [Matrix.Identity(4)]
        catalog.build(uid)
        objects = [o for o in bpy.context.scene.objects if o.type == 'MESH']
        bpy.context.view_layer.update()
        points = [o.matrix_world @ v.co for o in objects for v in o.data.vertices]
        mn = Vector(tuple(min(p[a] for p in points) for a in range(3)))
        mx = Vector(tuple(max(p[a] for p in points) for a in range(3)))
        size = mx - mn
        scale = min(1.40 / max(size.x, size.y, .001), 1.0 / max(size.z, .001))
        if uid in catalog.H:
            scale = min(scale, .74 / max(size.z, .001))
        center = Vector(((mn.x + mx.x) / 2, (mn.y + mx.y) / 2, mn.z))
        kind = catalog.kind_for(uid)
        for ob in objects:
            ob['part_name'] = ob.name
            for v in ob.data.vertices:
                v.co = (v.co - center) * scale
            for mod in ob.modifiers:
                if mod.type == 'BEVEL':
                    mod.width *= scale
                elif mod.type == 'SOLIDIFY':
                    mod.thickness *= scale
            ob['unit_kind'] = kind
            ob['source_scale_to_tile'] = scale
            for m in ob.data.materials:
                m['game_base_color'] = list(m.diffuse_color)
                p = next((n for n in m.node_tree.nodes if n.type == 'BSDF_PRINCIPLED'), None)
                if p:
                    m['game_roughness'] = p.inputs['Roughness'].default_value
                    m['game_metallic'] = p.inputs['Metallic'].default_value
                m['team_tint'] = m.name.startswith('team')
        _rig_and_animate(uid, objects, size.z * scale, kind)
    return build


def run():
    return main({uid.lower(): (wrap(uid), '../units/catalog') for uid in catalog.all_ids()}, 'unit')


if __name__ == '__main__':
    result = run()
