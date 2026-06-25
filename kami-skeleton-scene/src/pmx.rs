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

/// A parsed PMX mesh: interleaved vertices + a triangle index list + model name.
#[derive(Debug, Clone, PartialEq)]
pub struct PmxModel {
    pub name: String,
    pub vertices: Vec<PmxVertex>,
    pub indices: Vec<u32>,
}

struct Cur<'a> {
    b: &'a [u8],
    p: usize,
    enc_utf16: bool,
    vidx: usize, // vertex index size
    bidx: usize, // bone index size
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
    let vidx = *globals.get(2)? as usize; // vertex index size
    let bidx = *globals.get(4)? as usize; // bone index size

    let mut c = Cur { b: bytes, p: 9 + gcount, enc_utf16, vidx, bidx, add_uv };
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

    Some(PmxModel { name, vertices, indices })
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
}
