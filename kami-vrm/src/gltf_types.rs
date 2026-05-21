//! glTF 2.0 JSON schema types (serde roundtrip).

use serde::{Deserialize, Serialize};

/// Top-level glTF 2.0 document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GltfDocument {
    pub asset: Asset,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scenes: Vec<Scene>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<Node>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub meshes: Vec<Mesh>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accessors: Vec<Accessor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub buffer_views: Vec<BufferView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub buffers: Vec<Buffer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub materials: Vec<Material>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub textures: Vec<Texture>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<Image>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub samplers: Vec<Sampler>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skins: Vec<Skin>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub animations: Vec<Animation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extensions_used: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extensions_required: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    #[serde(default)]
    pub nodes: Vec<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mesh: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skin: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translation: Option<[f32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f32; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<[f32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matrix: Option<[f32; 16]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub primitives: Vec<Primitive>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub weights: Vec<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Primitive {
    pub attributes: serde_json::Map<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indices: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<serde_json::Map<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accessor {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buffer_view: Option<usize>,
    pub component_type: u32,
    pub count: usize,
    #[serde(rename = "type")]
    pub accessor_type: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub byte_offset: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<Vec<f32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<Vec<f32>>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub normalized: bool,
}

fn is_zero(v: &usize) -> bool {
    *v == 0
}
fn is_false(v: &bool) -> bool {
    !*v
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BufferView {
    pub buffer: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub byte_offset: usize,
    pub byte_length: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_stride: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Buffer {
    pub byte_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Material {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pbr_metallic_roughness: Option<PbrMetallicRoughness>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub double_sided: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alpha_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alpha_cutoff: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PbrMetallicRoughness {
    #[serde(default = "default_base_color")]
    pub base_color_factor: [f32; 4],
    #[serde(default)]
    pub metallic_factor: f32,
    #[serde(default = "default_roughness")]
    pub roughness_factor: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_color_texture: Option<TextureInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metallic_roughness_texture: Option<TextureInfo>,
}

fn default_base_color() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]
}
fn default_roughness() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureInfo {
    pub index: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub tex_coord: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Texture {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampler: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buffer_view: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sampler {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mag_filter: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_filter: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrap_s: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrap_t: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub joints: Vec<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inverse_bind_matrices: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skeleton: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub channels: Vec<AnimationChannel>,
    pub samplers: Vec<AnimationSampler>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationChannel {
    pub sampler: usize,
    pub target: ChannelTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelTarget {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<usize>,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationSampler {
    pub input: usize,
    pub output: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interpolation: Option<String>,
}

/// glTF component type constants.
pub mod component_type {
    pub const BYTE: u32 = 5120;
    pub const UNSIGNED_BYTE: u32 = 5121;
    pub const SHORT: u32 = 5122;
    pub const UNSIGNED_SHORT: u32 = 5123;
    pub const UNSIGNED_INT: u32 = 5125;
    pub const FLOAT: u32 = 5126;
}

/// glTF buffer view target constants.
pub mod buffer_target {
    pub const ARRAY_BUFFER: u32 = 34962;
    pub const ELEMENT_ARRAY_BUFFER: u32 = 34963;
}
