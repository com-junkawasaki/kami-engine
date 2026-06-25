//! MMD `.pmx` (Polygon Model eXtended) model import — **mesh v1**.
//!
//! Parses the header + vertex + face sections into a renderable [`PmxModel`]
//! (positions / normals / UVs + per-vertex bone indices & weights, ready for GPU
//! skinning the same way a VRM mesh is). Textures / materials / bones / morphs /
//! physics live after the face block and are a v2 concern — this gives the
//! geometry, the MMD counterpart of the VRM mesh load.
//!
//! Layout (PMX 2.0/2.1): `"PMX " + f32 version + globals[u8 count + bytes]`, then
//! 4 text fields (model name ×2, comment ×2), then `i32 vertex count` + vertices,
//! then `i32 face count` + faces (3 indices each). Globals carry the text encoding
//! (0 = UTF-16LE, 1 = UTF-8) and the per-section index byte sizes.

use glam::{Vec2, Vec3};

/// One PMX vertex with up to 4 bone influences (BDEF/SDEF/QDEF flattened).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PmxVertex {
    pub pos: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub bones: [i32; 4],
    pub weights: [f32; 4],
}

/// A PMX bone: name + rest position + parent index (-1 = root). The skeleton a
/// `.vmd` motion retargets onto.
#[derive(Debug, Clone, PartialEq)]
pub struct PmxBone {
    pub name: String,
    pub pos: Vec3,
    pub parent: i32,
}

/// A PMX vertex morph (expression): per-vertex position offsets, keyed by name.
#[derive(Debug, Clone, PartialEq)]
pub struct PmxMorph {
    pub name: String,
    /// `(vertex index, position offset)` pairs (only vertex morphs; others skipped).
    pub offsets: Vec<(u32, Vec3)>,
}

/// A PMX material: a contiguous run of `surface_count` indices drawn with a base
/// colour + optional texture (index into [`PmxModel::textures`], -1 = none).
#[derive(Debug, Clone, PartialEq)]
pub struct PmxMaterial {
    pub name: String,
    pub diffuse: [f32; 4],
    pub texture: i32,
    /// Number of vertex indices (3 × triangles) this material covers, in order.
    pub surface_count: usize,
}

/// A parsed PMX model: mesh (vertices + faces) + skeleton (bones) + expression
/// morphs + texture paths. The MMD counterpart of a loaded VRM.
#[derive(Debug, Clone, PartialEq)]
pub struct PmxModel {
    pub name: String,
    pub vertices: Vec<PmxVertex>,
    pub indices: Vec<u32>,
    pub bones: Vec<PmxBone>,
    pub morphs: Vec<PmxMorph>,
    pub materials: Vec<PmxMaterial>,
    pub textures: Vec<String>,
}

struct Cur<'a> {
    b: &'a [u8],
    p: usize,
    enc_utf16: bool,
    vidx: usize, // vertex index size
    tidx: usize, // texture index size
    midx: usize, // material index size
    bidx: usize, // bone index size
    moidx: usize, // morph index size
    rbidx: usize, // rigidbody index size
    add_uv: usize,
}

impl<'a> Cur<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let s = self.b.get(self.p..self.p + n)?;
        self.p += n;
        Some(s)
    }
    fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }
    fn i32(&mut self) -> Option<i32> {
        Some(i32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    fn f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    fn vec3(&mut self) -> Option<Vec3> {
        Some(Vec3::new(self.f32()?, self.f32()?, self.f32()?))
    }
    /// A signed N-byte index (bone refs; -1 = none).
    fn sidx(&mut self, n: usize) -> Option<i32> {
        Some(match n {
            1 => self.take(1)?[0] as i8 as i32,
            2 => i16::from_le_bytes(self.take(2)?.try_into().ok()?) as i32,
            _ => i32::from_le_bytes(self.take(4)?.try_into().ok()?),
        })
    }
    /// An unsigned N-byte index (face vertex refs).
    fn uidx(&mut self, n: usize) -> Option<u32> {
        Some(match n {
            1 => self.take(1)?[0] as u32,
            2 => u16::from_le_bytes(self.take(2)?.try_into().ok()?) as u32,
            _ => u32::from_le_bytes(self.take(4)?.try_into().ok()?),
        })
    }
    fn text(&mut self) -> Option<String> {
        let len = self.i32()? as usize;
        let bytes = self.take(len)?;
        Some(if self.enc_utf16 {
            encoding_rs::UTF_16LE.decode(bytes).0.into_owned()
        } else {
            String::from_utf8_lossy(bytes).into_owned()
        })
    }
}

/// Parse the PMX header + vertex + face sections into a [`PmxModel`].
pub fn pmx_to_model(bytes: &[u8]) -> Option<PmxModel> {
    if bytes.len() < 9 || &bytes[0..4] != b"PMX " {
        return None;
    }
    // version f32 at [4..8]; globals: u8 count then that many config bytes.
    let gcount = *bytes.get(8)? as usize;
    let globals = bytes.get(9..9 + gcount)?;
    let enc_utf16 = *globals.first()? == 0;
    let add_uv = *globals.get(1)? as usize;
    let g = |i: usize| *globals.get(i).unwrap_or(&4) as usize;
    let (vidx, tidx, midx, bidx, moidx, rbidx) = (g(2), g(3), g(5), g(4), g(6), g(7));

    let mut c = Cur { b: bytes, p: 9 + gcount, enc_utf16, vidx, tidx, midx, bidx, moidx, rbidx, add_uv };
    // 4 text fields: model name (local/universal), comment (local/universal).
    let name = c.text()?;
    c.text()?;
    c.text()?;
    c.text()?;

    // ── vertices ────────────────────────────────────────────────────────────
    let vcount = c.i32()? as usize;
    let mut vertices = Vec::with_capacity(vcount.min(1 << 20));
    for _ in 0..vcount {
        let pos = c.vec3()?;
        let normal = c.vec3()?;
        let uv = Vec2::new(c.f32()?, c.f32()?);
        for _ in 0..c.add_uv {
            c.take(16)?; // additional UV vec4s
        }
        let mut bones = [-1i32; 4];
        let mut weights = [0.0f32; 4];
        match c.u8()? {
            0 => {
                bones[0] = c.sidx(c.bidx)?;
                weights[0] = 1.0;
            }
            1 => {
                bones[0] = c.sidx(c.bidx)?;
                bones[1] = c.sidx(c.bidx)?;
                weights[0] = c.f32()?;
                weights[1] = 1.0 - weights[0];
            }
            2 | 4 => {
                for b in &mut bones {
                    *b = c.sidx(c.bidx)?;
                }
                for w in &mut weights {
                    *w = c.f32()?;
                }
            }
            3 => {
                // SDEF: 2 bones + weight + C/R0/R1 (3 vec3).
                bones[0] = c.sidx(c.bidx)?;
                bones[1] = c.sidx(c.bidx)?;
                weights[0] = c.f32()?;
                weights[1] = 1.0 - weights[0];
                c.take(36)?;
            }
            _ => return None,
        }
        c.f32()?; // edge scale
        vertices.push(PmxVertex { pos, normal, uv, bones, weights });
    }

    // ── faces (triangles; 3 vertex indices each) ────────────────────────────
    let icount = c.i32()? as usize;
    let mut indices = Vec::with_capacity(icount.min(1 << 22));
    for _ in 0..icount {
        indices.push(c.uidx(c.vidx)?);
    }

    // ── v2 sections (best-effort): textures → materials → bones → morphs ────
    // A truncated / mesh-only PMX still returns the mesh with empty rig.
    let mut model = PmxModel { name, vertices, indices, bones: vec![], morphs: vec![], materials: vec![], textures: vec![] };
    let _ = parse_rest(&mut c, &mut model);
    Some(model)
}

/// Parse the post-face sections (textures / materials / bones / morphs) into
/// `model`. Returns `None` on a truncated stream — the caller keeps the mesh.
fn parse_rest(c: &mut Cur, model: &mut PmxModel) -> Option<()> {
    // textures: count + paths.
    let tex_n = c.i32()? as usize;
    for _ in 0..tex_n {
        model.textures.push(c.text()?);
    }
    // materials: extract name + diffuse + texture ref + surface run length.
    let mat_n = c.i32()? as usize;
    for _ in 0..mat_n {
        let name = c.text()?; // name local
        c.text()?; // name universal
        let diffuse = [c.f32()?, c.f32()?, c.f32()?, c.f32()?];
        c.take(12 + 4 + 12)?; // specular(3) + spec-strength + ambient(3)
        c.u8()?; // draw flags
        c.take(16 + 4)?; // edge colour(4) + edge size
        let texture = c.sidx(c.tidx)?; // texture index (into model.textures)
        c.sidx(c.tidx)?; // sphere texture index
        c.u8()?; // sphere mode
        if c.u8()? == 0 {
            c.sidx(c.tidx)?; // toon = texture reference
        } else {
            c.u8()?; // toon = shared-toon value
        }
        c.text()?; // memo
        let surface_count = c.i32()? as usize; // index run for this material
        model.materials.push(PmxMaterial { name, diffuse, texture, surface_count });
    }
    // bones: extract name + position + parent; skip the flag-dependent tail.
    let bone_n = c.i32()? as usize;
    for _ in 0..bone_n {
        let name = c.text()?;
        c.text()?; // universal
        let pos = c.vec3()?;
        let parent = c.sidx(c.bidx)?;
        c.i32()?; // transform layer
        let flags = u16::from_le_bytes(c.take(2)?.try_into().ok()?);
        if flags & 0x0001 == 0 {
            c.take(12)?; // tail = offset vec3
        } else {
            c.sidx(c.bidx)?; // tail = bone index
        }
        if flags & (0x0100 | 0x0200) != 0 {
            c.sidx(c.bidx)?;
            c.f32()?; // inherit parent + weight
        }
        if flags & 0x0400 != 0 {
            c.take(12)?; // fixed axis
        }
        if flags & 0x0800 != 0 {
            c.take(24)?; // local coordinate (x,z axes)
        }
        if flags & 0x2000 != 0 {
            c.i32()?; // external parent
        }
        if flags & 0x0020 != 0 {
            // IK: target + loop + angle + link list.
            c.sidx(c.bidx)?;
            c.i32()?;
            c.f32()?;
            let links = c.i32()? as usize;
            for _ in 0..links {
                c.sidx(c.bidx)?;
                if c.u8()? == 1 {
                    c.take(24)?; // lower + upper limit vec3
                }
            }
        }
        model.bones.push(PmxBone { name, pos, parent });
    }
    // morphs: keep vertex morphs (expressions); read past the rest.
    let morph_n = c.i32()? as usize;
    for _ in 0..morph_n {
        let name = c.text()?;
        c.text()?; // universal
        c.u8()?; // panel
        let ty = c.u8()?;
        let n = c.i32()? as usize;
        if ty == 1 {
            let mut offsets = Vec::with_capacity(n);
            for _ in 0..n {
                let vi = c.uidx(c.vidx)?;
                offsets.push((vi, c.vec3()?));
            }
            model.morphs.push(PmxMorph { name, offsets });
        } else {
            // skip: per-offset byte size by morph type.
            let each = match ty {
                0 | 9 => c.moidx + 4,            // group / flip: morph idx + f32
                2 => c.bidx + 28,                // bone: idx + pos(3) + rot(4)
                3..=7 => c.vidx + 16,            // uv 0-4: vidx + vec4
                8 => c.midx + 1 + 28 * 4,        // material: midx + op + 28 f32
                10 => c.rbidx + 1 + 24,          // impulse: rb idx + flag + 2 vec3
                _ => 0,
            };
            c.take(each * n)?;
        }
    }
    Some(())
}

/// Realise a PMX model's bones into a [`kami_skeleton::Skeleton`] — the rig a
/// `.vmd` motion (via [`crate::vmd_to_clip`]) plays on. PMX bone positions are
/// world-space at rest (identity rotation); local position = world − parent
/// world, and the inverse-bind is the inverse of the rest world translation.
pub fn pmx_to_skeleton(model: &PmxModel) -> kami_skeleton::Skeleton {
    let bones = model
        .bones
        .iter()
        .map(|b| {
            let parent = (b.parent >= 0).then_some(b.parent as usize);
            let local = match parent.and_then(|pi| model.bones.get(pi)) {
                Some(p) => b.pos - p.pos,
                None => b.pos,
            };
            kami_skeleton::Bone {
                name: b.name.clone(),
                parent,
                local_position: local.into(),
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0, 1.0, 1.0],
                inverse_bind: glam::Mat4::from_translation(-b.pos).to_cols_array_2d(),
            }
        })
        .collect();
    kami_skeleton::Skeleton { bones }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal 3-vertex, 1-triangle PMX (UTF-8 text, 1-byte indices, BDEF1).
    fn synthetic_pmx() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(b"PMX ");
        v.extend_from_slice(&2.0f32.to_le_bytes());
        // globals: count=8, [enc=1(UTF8), addUV=0, vidx=1, tex=1, mat=1, bone=1, morph=1, rb=1]
        v.push(8);
        v.extend_from_slice(&[1, 0, 1, 1, 1, 1, 1, 1]);
        // 4 text fields (i32 len + bytes)
        for t in [b"Tri".as_slice(), b"", b"", b""] {
            v.extend_from_slice(&(t.len() as i32).to_le_bytes());
            v.extend_from_slice(t);
        }
        // 3 vertices
        v.extend_from_slice(&3i32.to_le_bytes());
        for (i, p) in [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]].iter().enumerate() {
            for f in p {
                v.extend_from_slice(&f.to_le_bytes());
            }
            for f in [0.0f32, 0.0, 1.0] {
                v.extend_from_slice(&f.to_le_bytes()); // normal
            }
            for f in [i as f32 * 0.5, 0.0] {
                v.extend_from_slice(&f.to_le_bytes()); // uv
            }
            v.push(0); // BDEF1
            v.push(0); // bone index (1 byte)
            v.extend_from_slice(&1.0f32.to_le_bytes()); // edge scale
        }
        // 3 indices (1 triangle), 1-byte each
        v.extend_from_slice(&3i32.to_le_bytes());
        v.extend_from_slice(&[0u8, 1, 2]);
        v
    }

    #[test]
    fn parses_pmx_mesh() {
        let m = pmx_to_model(&synthetic_pmx()).expect("pmx");
        assert_eq!(m.name, "Tri");
        assert_eq!(m.vertices.len(), 3);
        assert_eq!(m.indices, vec![0, 1, 2]);
        assert_eq!(m.vertices[1].pos, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(m.vertices[0].weights[0], 1.0);
    }

    #[test]
    fn rejects_non_pmx() {
        assert!(pmx_to_model(b"NOPE....").is_none());
    }

    /// The v1 mesh bytes + v2 sections: 0 textures, 0 materials, 1 bone (named
    /// `bone`), 1 vertex morph.
    fn synthetic_pmx_full(bone: &[u8]) -> Vec<u8> {
        let mut v = synthetic_pmx();
        let text = |v: &mut Vec<u8>, s: &[u8]| {
            v.extend_from_slice(&(s.len() as i32).to_le_bytes());
            v.extend_from_slice(s);
        };
        v.extend_from_slice(&0i32.to_le_bytes()); // 0 textures
        // 1 material covering the single triangle
        v.extend_from_slice(&1i32.to_le_bytes());
        text(&mut v, b"mat");
        text(&mut v, b"");
        for f in [0.8f32, 0.7, 0.6, 1.0] {
            v.extend_from_slice(&f.to_le_bytes()); // diffuse
        }
        v.extend_from_slice(&[0u8; 28]); // specular(3) + spec-strength + ambient(3)
        v.push(0); // draw flags
        v.extend_from_slice(&[0u8; 20]); // edge colour(4) + edge size
        v.push(0xFF); // texture index = -1
        v.push(0xFF); // sphere texture index = -1
        v.push(0); // sphere mode
        v.push(1); // toon flag = 1 (shared-toon value follows)
        v.push(0); // toon value
        text(&mut v, b""); // memo
        v.extend_from_slice(&3i32.to_le_bytes()); // surface count
        // 1 bone
        v.extend_from_slice(&1i32.to_le_bytes());
        text(&mut v, bone);
        text(&mut v, b"");
        for f in [0.0f32, 1.0, 0.0] {
            v.extend_from_slice(&f.to_le_bytes()); // position
        }
        v.push(0xFF); // parent = -1 (1-byte bone index)
        v.extend_from_slice(&0i32.to_le_bytes()); // layer
        v.extend_from_slice(&0u16.to_le_bytes()); // flags = 0 (tail = offset)
        for f in [0.0f32, 0.0, 0.0] {
            v.extend_from_slice(&f.to_le_bytes()); // tail offset vec3
        }
        // 1 vertex morph touching vertex 2
        v.extend_from_slice(&1i32.to_le_bytes());
        text(&mut v, b"smile");
        text(&mut v, b"");
        v.push(0); // panel
        v.push(1); // type = vertex morph
        v.extend_from_slice(&1i32.to_le_bytes()); // 1 offset
        v.push(2); // vertex index (1 byte)
        for f in [0.0f32, 0.1, 0.0] {
            v.extend_from_slice(&f.to_le_bytes()); // position offset
        }
        v
    }

    #[test]
    fn parses_pmx_rig_and_morph() {
        let m = pmx_to_model(&synthetic_pmx_full(b"root")).expect("pmx");
        assert_eq!(m.vertices.len(), 3, "mesh still parsed");
        assert_eq!(m.bones.len(), 1, "one bone");
        assert_eq!(m.bones[0].name, "root");
        assert_eq!(m.bones[0].parent, -1);
        assert_eq!(m.morphs.len(), 1, "one vertex morph");
        assert_eq!(m.morphs[0].name, "smile");
        assert_eq!(m.morphs[0].offsets, vec![(2u32, Vec3::new(0.0, 0.1, 0.0))]);
        assert_eq!(m.materials.len(), 1, "one material");
        assert_eq!(m.materials[0].name, "mat");
        assert_eq!(m.materials[0].diffuse, [0.8, 0.7, 0.6, 1.0]);
        assert_eq!(m.materials[0].surface_count, 3);
        assert_eq!(m.materials[0].texture, -1, "no texture bound");
    }

    /// Minimal 1-keyframe `.vmd` for a (Shift-JIS) bone name.
    fn vmd_for(bone: &str) -> Vec<u8> {
        let mut v = vec![0u8; 50];
        v.extend_from_slice(&1u32.to_le_bytes()); // 1 keyframe
        let sjis = encoding_rs::SHIFT_JIS.encode(bone).0.into_owned();
        let mut name = [0u8; 15];
        let n = sjis.len().min(15);
        name[..n].copy_from_slice(&sjis[..n]);
        v.extend_from_slice(&name);
        v.extend_from_slice(&0u32.to_le_bytes()); // frame 0
        for f in [0.0f32, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0] {
            v.extend_from_slice(&f.to_le_bytes()); // pos(3) + quat(4)
        }
        v.extend_from_slice(&[0u8; 64]); // interpolation
        v
    }

    #[test]
    fn pmx_skeleton_plays_a_vmd_motion() {
        // the full MMD path: a .pmx model's skeleton + a .vmd motion on the same
        // bone — the motion retargets onto the model rig by name.
        let model = pmx_to_model(&synthetic_pmx_full("センター".as_bytes())).expect("pmx");
        let skel = pmx_to_skeleton(&model);
        assert_eq!(skel.bones.len(), 1);
        assert_eq!(skel.bones[0].name, "センター");
        // index the model's bones by name for the motion retarget.
        let idx = |n: &str| skel.bones.iter().position(|b| b.name == n);
        let clip = crate::vmd_to_clip(&vmd_for("センター"), 30.0, idx).expect("clip");
        assert_eq!(clip.tracks.len(), 1, "the センター track retargets onto the .pmx bone");
        assert_eq!(clip.tracks[0].bone_index, 0);
    }
}
