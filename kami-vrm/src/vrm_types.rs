//! VRM extension data types (VRM 1.0 primary, 0.x via compat).

use serde::{Deserialize, Serialize};

use crate::gltf_types::GltfDocument;

/// Complete VRM document: glTF base + parsed VRM extension data + raw binary buffer.
#[derive(Debug, Clone)]
pub struct VrmDocument {
    /// glTF 2.0 JSON structure.
    pub gltf: GltfDocument,
    /// Raw BIN chunk data.
    pub bin: Vec<u8>,
    /// VRM spec version.
    pub version: VrmVersion,
    /// Model metadata.
    pub meta: VrmMeta,
    /// Humanoid bone mapping.
    pub humanoid: VrmHumanoid,
    /// Expression definitions (blendshape groups in 0.x).
    pub expressions: Vec<VrmExpression>,
    /// Spring bone chains.
    pub spring_bones: Vec<VrmSpringBoneChain>,
    /// Spring bone colliders.
    pub spring_bone_colliders: Vec<VrmCollider>,
    /// Spring bone collider groups.
    pub spring_bone_collider_groups: Vec<VrmColliderGroup>,
    /// Per-material MToon parameters.
    pub mtoon_materials: Vec<VrmMtoonMaterial>,
    /// LookAt configuration.
    pub look_at: Option<VrmLookAt>,
    /// First person mesh annotations.
    pub first_person: Option<VrmFirstPerson>,
    /// Node constraints (aim, rotation, roll).
    pub node_constraints: Vec<VrmNodeConstraint>,
}

/// VRM specification version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VrmVersion {
    /// VRM 0.x (legacy, converted to 1.0 types internally).
    V0x,
    /// VRM 1.0.
    V1_0,
}

// ── Meta ──

/// Model metadata (name, author, license).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrmMeta {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_redistribution: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail_image: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_permission: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commercial_usage: Option<String>,
}

// ── Humanoid ──

/// VRM humanoid bone mapping.
#[derive(Debug, Clone)]
pub struct VrmHumanoid {
    pub human_bones: Vec<VrmHumanBone>,
}

/// A single humanoid bone → glTF node mapping.
#[derive(Debug, Clone)]
pub struct VrmHumanBone {
    pub bone: HumanBoneName,
    /// glTF node index.
    pub node: usize,
}

/// VRM 1.0 humanoid bone names (required + optional).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HumanBoneName {
    // Required bones
    Hips,
    Spine,
    Chest,
    UpperChest,
    Neck,
    Head,
    // Eyes + jaw
    LeftEye,
    RightEye,
    Jaw,
    // Left arm
    LeftShoulder,
    LeftUpperArm,
    LeftLowerArm,
    LeftHand,
    // Right arm
    RightShoulder,
    RightUpperArm,
    RightLowerArm,
    RightHand,
    // Left leg
    LeftUpperLeg,
    LeftLowerLeg,
    LeftFoot,
    LeftToes,
    // Right leg
    RightUpperLeg,
    RightLowerLeg,
    RightFoot,
    RightToes,
    // Left fingers
    LeftThumbMetacarpal,
    LeftThumbProximal,
    LeftThumbDistal,
    LeftIndexProximal,
    LeftIndexIntermediate,
    LeftIndexDistal,
    LeftMiddleProximal,
    LeftMiddleIntermediate,
    LeftMiddleDistal,
    LeftRingProximal,
    LeftRingIntermediate,
    LeftRingDistal,
    LeftLittleProximal,
    LeftLittleIntermediate,
    LeftLittleDistal,
    // Right fingers
    RightThumbMetacarpal,
    RightThumbProximal,
    RightThumbDistal,
    RightIndexProximal,
    RightIndexIntermediate,
    RightIndexDistal,
    RightMiddleProximal,
    RightMiddleIntermediate,
    RightMiddleDistal,
    RightRingProximal,
    RightRingIntermediate,
    RightRingDistal,
    RightLittleProximal,
    RightLittleIntermediate,
    RightLittleDistal,
}

impl HumanBoneName {
    /// All 55 bone names in specification order.
    pub const ALL: &[HumanBoneName] = &[
        Self::Hips,
        Self::Spine,
        Self::Chest,
        Self::UpperChest,
        Self::Neck,
        Self::Head,
        Self::LeftEye,
        Self::RightEye,
        Self::Jaw,
        Self::LeftShoulder,
        Self::LeftUpperArm,
        Self::LeftLowerArm,
        Self::LeftHand,
        Self::RightShoulder,
        Self::RightUpperArm,
        Self::RightLowerArm,
        Self::RightHand,
        Self::LeftUpperLeg,
        Self::LeftLowerLeg,
        Self::LeftFoot,
        Self::LeftToes,
        Self::RightUpperLeg,
        Self::RightLowerLeg,
        Self::RightFoot,
        Self::RightToes,
        Self::LeftThumbMetacarpal,
        Self::LeftThumbProximal,
        Self::LeftThumbDistal,
        Self::LeftIndexProximal,
        Self::LeftIndexIntermediate,
        Self::LeftIndexDistal,
        Self::LeftMiddleProximal,
        Self::LeftMiddleIntermediate,
        Self::LeftMiddleDistal,
        Self::LeftRingProximal,
        Self::LeftRingIntermediate,
        Self::LeftRingDistal,
        Self::LeftLittleProximal,
        Self::LeftLittleIntermediate,
        Self::LeftLittleDistal,
        Self::RightThumbMetacarpal,
        Self::RightThumbProximal,
        Self::RightThumbDistal,
        Self::RightIndexProximal,
        Self::RightIndexIntermediate,
        Self::RightIndexDistal,
        Self::RightMiddleProximal,
        Self::RightMiddleIntermediate,
        Self::RightMiddleDistal,
        Self::RightRingProximal,
        Self::RightRingIntermediate,
        Self::RightRingDistal,
        Self::RightLittleProximal,
        Self::RightLittleIntermediate,
        Self::RightLittleDistal,
    ];

    /// Parse from camelCase string (VRM 1.0 spec naming).
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "hips" => Some(Self::Hips),
            "spine" => Some(Self::Spine),
            "chest" => Some(Self::Chest),
            "upperChest" => Some(Self::UpperChest),
            "neck" => Some(Self::Neck),
            "head" => Some(Self::Head),
            "leftEye" => Some(Self::LeftEye),
            "rightEye" => Some(Self::RightEye),
            "jaw" => Some(Self::Jaw),
            "leftShoulder" => Some(Self::LeftShoulder),
            "leftUpperArm" => Some(Self::LeftUpperArm),
            "leftLowerArm" => Some(Self::LeftLowerArm),
            "leftHand" => Some(Self::LeftHand),
            "rightShoulder" => Some(Self::RightShoulder),
            "rightUpperArm" => Some(Self::RightUpperArm),
            "rightLowerArm" => Some(Self::RightLowerArm),
            "rightHand" => Some(Self::RightHand),
            "leftUpperLeg" => Some(Self::LeftUpperLeg),
            "leftLowerLeg" => Some(Self::LeftLowerLeg),
            "leftFoot" => Some(Self::LeftFoot),
            "leftToes" => Some(Self::LeftToes),
            "rightUpperLeg" => Some(Self::RightUpperLeg),
            "rightLowerLeg" => Some(Self::RightLowerLeg),
            "rightFoot" => Some(Self::RightFoot),
            "rightToes" => Some(Self::RightToes),
            "leftThumbMetacarpal" => Some(Self::LeftThumbMetacarpal),
            "leftThumbProximal" => Some(Self::LeftThumbProximal),
            "leftThumbDistal" => Some(Self::LeftThumbDistal),
            "leftIndexProximal" => Some(Self::LeftIndexProximal),
            "leftIndexIntermediate" => Some(Self::LeftIndexIntermediate),
            "leftIndexDistal" => Some(Self::LeftIndexDistal),
            "leftMiddleProximal" => Some(Self::LeftMiddleProximal),
            "leftMiddleIntermediate" => Some(Self::LeftMiddleIntermediate),
            "leftMiddleDistal" => Some(Self::LeftMiddleDistal),
            "leftRingProximal" => Some(Self::LeftRingProximal),
            "leftRingIntermediate" => Some(Self::LeftRingIntermediate),
            "leftRingDistal" => Some(Self::LeftRingDistal),
            "leftLittleProximal" => Some(Self::LeftLittleProximal),
            "leftLittleIntermediate" => Some(Self::LeftLittleIntermediate),
            "leftLittleDistal" => Some(Self::LeftLittleDistal),
            "rightThumbMetacarpal" => Some(Self::RightThumbMetacarpal),
            "rightThumbProximal" => Some(Self::RightThumbProximal),
            "rightThumbDistal" => Some(Self::RightThumbDistal),
            "rightIndexProximal" => Some(Self::RightIndexProximal),
            "rightIndexIntermediate" => Some(Self::RightIndexIntermediate),
            "rightIndexDistal" => Some(Self::RightIndexDistal),
            "rightMiddleProximal" => Some(Self::RightMiddleProximal),
            "rightMiddleIntermediate" => Some(Self::RightMiddleIntermediate),
            "rightMiddleDistal" => Some(Self::RightMiddleDistal),
            "rightRingProximal" => Some(Self::RightRingProximal),
            "rightRingIntermediate" => Some(Self::RightRingIntermediate),
            "rightRingDistal" => Some(Self::RightRingDistal),
            "rightLittleProximal" => Some(Self::RightLittleProximal),
            "rightLittleIntermediate" => Some(Self::RightLittleIntermediate),
            "rightLittleDistal" => Some(Self::RightLittleDistal),
            _ => None,
        }
    }

    /// Convert to camelCase string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hips => "hips",
            Self::Spine => "spine",
            Self::Chest => "chest",
            Self::UpperChest => "upperChest",
            Self::Neck => "neck",
            Self::Head => "head",
            Self::LeftEye => "leftEye",
            Self::RightEye => "rightEye",
            Self::Jaw => "jaw",
            Self::LeftShoulder => "leftShoulder",
            Self::LeftUpperArm => "leftUpperArm",
            Self::LeftLowerArm => "leftLowerArm",
            Self::LeftHand => "leftHand",
            Self::RightShoulder => "rightShoulder",
            Self::RightUpperArm => "rightUpperArm",
            Self::RightLowerArm => "rightLowerArm",
            Self::RightHand => "rightHand",
            Self::LeftUpperLeg => "leftUpperLeg",
            Self::LeftLowerLeg => "leftLowerLeg",
            Self::LeftFoot => "leftFoot",
            Self::LeftToes => "leftToes",
            Self::RightUpperLeg => "rightUpperLeg",
            Self::RightLowerLeg => "rightLowerLeg",
            Self::RightFoot => "rightFoot",
            Self::RightToes => "rightToes",
            Self::LeftThumbMetacarpal => "leftThumbMetacarpal",
            Self::LeftThumbProximal => "leftThumbProximal",
            Self::LeftThumbDistal => "leftThumbDistal",
            Self::LeftIndexProximal => "leftIndexProximal",
            Self::LeftIndexIntermediate => "leftIndexIntermediate",
            Self::LeftIndexDistal => "leftIndexDistal",
            Self::LeftMiddleProximal => "leftMiddleProximal",
            Self::LeftMiddleIntermediate => "leftMiddleIntermediate",
            Self::LeftMiddleDistal => "leftMiddleDistal",
            Self::LeftRingProximal => "leftRingProximal",
            Self::LeftRingIntermediate => "leftRingIntermediate",
            Self::LeftRingDistal => "leftRingDistal",
            Self::LeftLittleProximal => "leftLittleProximal",
            Self::LeftLittleIntermediate => "leftLittleIntermediate",
            Self::LeftLittleDistal => "leftLittleDistal",
            Self::RightThumbMetacarpal => "rightThumbMetacarpal",
            Self::RightThumbProximal => "rightThumbProximal",
            Self::RightThumbDistal => "rightThumbDistal",
            Self::RightIndexProximal => "rightIndexProximal",
            Self::RightIndexIntermediate => "rightIndexIntermediate",
            Self::RightIndexDistal => "rightIndexDistal",
            Self::RightMiddleProximal => "rightMiddleProximal",
            Self::RightMiddleIntermediate => "rightMiddleIntermediate",
            Self::RightMiddleDistal => "rightMiddleDistal",
            Self::RightRingProximal => "rightRingProximal",
            Self::RightRingIntermediate => "rightRingIntermediate",
            Self::RightRingDistal => "rightRingDistal",
            Self::RightLittleProximal => "rightLittleProximal",
            Self::RightLittleIntermediate => "rightLittleIntermediate",
            Self::RightLittleDistal => "rightLittleDistal",
        }
    }
}

// ── Expressions ──

/// Expression definition (VRM 1.0 expressions / VRM 0.x blendShapeGroups).
#[derive(Debug, Clone)]
pub struct VrmExpression {
    pub name: String,
    pub preset: Option<ExpressionPreset>,
    pub is_binary: bool,
    pub morph_target_binds: Vec<MorphTargetBind>,
    pub material_color_binds: Vec<MaterialColorBind>,
    pub texture_transform_binds: Vec<TextureTransformBind>,
    pub override_blink: Option<OverrideType>,
    pub override_look_at: Option<OverrideType>,
    pub override_mouth: Option<OverrideType>,
}

/// Standard expression presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExpressionPreset {
    Happy,
    Angry,
    Sad,
    Relaxed,
    Surprised,
    Aa,
    Ih,
    Ou,
    Ee,
    Oh,
    Blink,
    BlinkLeft,
    BlinkRight,
    LookUp,
    LookDown,
    LookLeft,
    LookRight,
    Neutral,
}

impl ExpressionPreset {
    /// Parse from string (case-insensitive first char lowercase).
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "happy" => Some(Self::Happy),
            "angry" => Some(Self::Angry),
            "sad" => Some(Self::Sad),
            "relaxed" => Some(Self::Relaxed),
            "surprised" => Some(Self::Surprised),
            "aa" => Some(Self::Aa),
            "ih" => Some(Self::Ih),
            "ou" => Some(Self::Ou),
            "ee" => Some(Self::Ee),
            "oh" => Some(Self::Oh),
            "blink" => Some(Self::Blink),
            "blinkLeft" => Some(Self::BlinkLeft),
            "blinkRight" => Some(Self::BlinkRight),
            "lookUp" => Some(Self::LookUp),
            "lookDown" => Some(Self::LookDown),
            "lookLeft" => Some(Self::LookLeft),
            "lookRight" => Some(Self::LookRight),
            "neutral" => Some(Self::Neutral),
            _ => None,
        }
    }
}

/// Override behavior for expression conflicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverrideType {
    None,
    Block,
    Blend,
}

/// Morph target binding within an expression.
#[derive(Debug, Clone)]
pub struct MorphTargetBind {
    /// glTF mesh index.
    pub mesh_index: usize,
    /// Morph target index within the mesh.
    pub morph_index: usize,
    /// Target weight when expression is at 1.0.
    pub weight: f32,
}

/// Material color binding within an expression.
#[derive(Debug, Clone)]
pub struct MaterialColorBind {
    /// glTF material index.
    pub material_index: usize,
    /// Property name (e.g. "color", "shadeColor", "emissionColor").
    pub property: String,
    /// Target RGBA value.
    pub target_value: [f32; 4],
}

/// Texture transform binding within an expression.
#[derive(Debug, Clone)]
pub struct TextureTransformBind {
    /// glTF material index.
    pub material_index: usize,
    /// UV offset.
    pub offset: [f32; 2],
    /// UV scale.
    pub scale: [f32; 2],
}

// ── Spring Bone ──

/// A spring bone chain (group of joints that simulate physics).
#[derive(Debug, Clone)]
pub struct VrmSpringBoneChain {
    pub name: Option<String>,
    /// Joints in the chain (root → tip order).
    pub joints: Vec<SpringJoint>,
    /// Indices into `VrmDocument.spring_bone_collider_groups`.
    pub collider_groups: Vec<usize>,
    /// Center node for relative space simulation.
    pub center: Option<usize>,
}

/// A single spring joint in a chain.
#[derive(Debug, Clone)]
pub struct SpringJoint {
    /// glTF node index.
    pub node: usize,
    pub hit_radius: f32,
    pub stiffness: f32,
    pub gravity_power: f32,
    pub gravity_dir: [f32; 3],
    pub drag_force: f32,
}

/// Spring bone collider.
#[derive(Debug, Clone)]
pub struct VrmCollider {
    /// glTF node index.
    pub node: usize,
    pub shape: ColliderShape,
}

/// Collider shape.
#[derive(Debug, Clone)]
pub enum ColliderShape {
    Sphere {
        offset: [f32; 3],
        radius: f32,
    },
    Capsule {
        offset: [f32; 3],
        tail: [f32; 3],
        radius: f32,
    },
}

/// Named group of colliders.
#[derive(Debug, Clone)]
pub struct VrmColliderGroup {
    pub name: Option<String>,
    /// Indices into `VrmDocument.spring_bone_colliders`.
    pub colliders: Vec<usize>,
}

// ── MToon Material ──

/// MToon toon shader parameters for a glTF material.
#[derive(Debug, Clone)]
pub struct VrmMtoonMaterial {
    /// glTF material index.
    pub material_index: usize,
    pub shade_color_factor: [f32; 3],
    pub shade_multiply_texture: Option<usize>,
    pub shading_shift_factor: f32,
    pub shading_toony_factor: f32,
    pub gi_equalization_factor: f32,
    pub rim_color_factor: [f32; 3],
    pub rim_lighting_mix_factor: f32,
    pub rim_fresnel_power_factor: f32,
    pub rim_lift_factor: f32,
    pub rim_multiply_texture: Option<usize>,
    pub outline_width_mode: OutlineWidthMode,
    pub outline_width_factor: f32,
    pub outline_color_factor: [f32; 3],
    pub outline_lighting_mix_factor: f32,
    pub matcap_texture: Option<usize>,
    pub parametric_rim_color_factor: [f32; 3],
    pub uv_animation_scroll_x: f32,
    pub uv_animation_scroll_y: f32,
    pub uv_animation_rotation: f32,
    pub render_queue_offset: i32,
    pub transparent_with_z_write: bool,
}

/// MToon outline width mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutlineWidthMode {
    None,
    WorldCoordinates,
    ScreenCoordinates,
}

// ── LookAt ──

/// Eye gaze tracking configuration.
#[derive(Debug, Clone)]
pub struct VrmLookAt {
    pub look_at_type: LookAtType,
    pub offset_from_head_bone: [f32; 3],
    pub range_map_horizontal_inner: RangeMap,
    pub range_map_horizontal_outer: RangeMap,
    pub range_map_vertical_down: RangeMap,
    pub range_map_vertical_up: RangeMap,
}

/// LookAt implementation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookAtType {
    Bone,
    Expression,
}

/// Degree-to-output mapping for LookAt.
#[derive(Debug, Clone)]
pub struct RangeMap {
    pub input_max_value: f32,
    pub output_scale: f32,
}

// ── FirstPerson ──

/// First-person camera mesh visibility annotations.
#[derive(Debug, Clone)]
pub struct VrmFirstPerson {
    pub mesh_annotations: Vec<MeshAnnotation>,
}

/// Per-mesh first-person visibility.
#[derive(Debug, Clone)]
pub struct MeshAnnotation {
    /// glTF node index.
    pub node: usize,
    pub annotation_type: FirstPersonFlag,
}

/// First-person visibility flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstPersonFlag {
    Auto,
    Both,
    ThirdPersonOnly,
    FirstPersonOnly,
}

// ── Node Constraint ──

/// Node constraint (VRMC_node_constraint).
#[derive(Debug, Clone)]
pub struct VrmNodeConstraint {
    /// glTF node index this constraint applies to.
    pub node: usize,
    pub constraint: ConstraintType,
}

/// Constraint type.
#[derive(Debug, Clone)]
pub enum ConstraintType {
    Aim {
        source: usize,
        aim_axis: [f32; 3],
        weight: f32,
    },
    Rotation {
        source: usize,
        weight: f32,
    },
    Roll {
        source: usize,
        roll_axis: [f32; 3],
        weight: f32,
    },
}
