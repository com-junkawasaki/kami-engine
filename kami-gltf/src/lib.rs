//! Output layer: LoadedMesh → binary glTF 2.0 (.glb).
//! Minimal GLB writer — no external dependencies.

use kami_render::mesh::LoadedMesh;

/// Export LoadedMesh to binary glTF 2.0 (.glb) bytes.
pub fn export_glb(mesh: &LoadedMesh, color: [f32; 4]) -> Vec<u8> {
    // Vertex buffer: interleaved [pos3, norm3, uv2] × N
    let vertex_bytes: Vec<u8> = mesh.vertices.iter().flat_map(|f| f.to_le_bytes()).collect();

    // Index buffer: u32 × N
    let index_bytes: Vec<u8> = mesh.indices.iter().flat_map(|i| i.to_le_bytes()).collect();

    let vertex_byte_len = vertex_bytes.len();
    let index_byte_len = index_bytes.len();
    let total_buffer_len = vertex_byte_len + index_byte_len;

    // Compute bounds for positions
    let (mut min_pos, mut max_pos) = ([f32::MAX; 3], [f32::MIN; 3]);
    for i in 0..mesh.vertex_count as usize {
        let base = i * 8;
        for j in 0..3 {
            min_pos[j] = min_pos[j].min(mesh.vertices[base + j]);
            max_pos[j] = max_pos[j].max(mesh.vertices[base + j]);
        }
    }

    let json = serde_json::json!({
        "asset": { "version": "2.0", "generator": "kami-scad" },
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": [{ "mesh": 0 }],
        "meshes": [{
            "primitives": [{
                "attributes": {
                    "POSITION": 0,
                    "NORMAL": 1,
                    "TEXCOORD_0": 2
                },
                "indices": 3,
                "material": 0
            }]
        }],
        "materials": [{
            "pbrMetallicRoughness": {
                "baseColorFactor": color,
                "metallicFactor": 0.0,
                "roughnessFactor": 0.5
            }
        }],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126, // FLOAT
                "count": mesh.vertex_count,
                "type": "VEC3",
                "byteOffset": 0,
                "min": min_pos,
                "max": max_pos
            },
            {
                "bufferView": 0,
                "componentType": 5126,
                "count": mesh.vertex_count,
                "type": "VEC3",
                "byteOffset": 12 // after position (3 floats)
            },
            {
                "bufferView": 0,
                "componentType": 5126,
                "count": mesh.vertex_count,
                "type": "VEC2",
                "byteOffset": 24 // after position + normal (6 floats)
            },
            {
                "bufferView": 1,
                "componentType": 5125, // UNSIGNED_INT
                "count": mesh.index_count,
                "type": "SCALAR"
            }
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": 0,
                "byteLength": vertex_byte_len,
                "byteStride": 32, // 8 floats × 4 bytes
                "target": 34962 // ARRAY_BUFFER
            },
            {
                "buffer": 0,
                "byteOffset": vertex_byte_len,
                "byteLength": index_byte_len,
                "target": 34963 // ELEMENT_ARRAY_BUFFER
            }
        ],
        "buffers": [{
            "byteLength": total_buffer_len
        }]
    });

    let json_str = serde_json::to_string(&json).unwrap();
    let json_bytes = json_str.as_bytes();

    // Pad JSON to 4-byte alignment
    let json_pad = (4 - (json_bytes.len() % 4)) % 4;
    let json_chunk_len = json_bytes.len() + json_pad;

    // Pad binary to 4-byte alignment
    let bin_pad = (4 - (total_buffer_len % 4)) % 4;
    let bin_chunk_len = total_buffer_len + bin_pad;

    // GLB header: magic(4) + version(4) + length(4) = 12 bytes
    // JSON chunk: length(4) + type(4) + data(json_chunk_len)
    // BIN chunk: length(4) + type(4) + data(bin_chunk_len)
    let total_len = 12 + 8 + json_chunk_len + 8 + bin_chunk_len;

    let mut glb = Vec::with_capacity(total_len);

    // Header
    glb.extend_from_slice(&0x46546C67u32.to_le_bytes()); // magic: "glTF"
    glb.extend_from_slice(&2u32.to_le_bytes()); // version: 2
    glb.extend_from_slice(&(total_len as u32).to_le_bytes());

    // JSON chunk
    glb.extend_from_slice(&(json_chunk_len as u32).to_le_bytes());
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // type: "JSON"
    glb.extend_from_slice(json_bytes);
    glb.extend(std::iter::repeat(b' ').take(json_pad));

    // BIN chunk
    glb.extend_from_slice(&(bin_chunk_len as u32).to_le_bytes());
    glb.extend_from_slice(&0x004E4942u32.to_le_bytes()); // type: "BIN\0"
    glb.extend_from_slice(&vertex_bytes);
    glb.extend_from_slice(&index_bytes);
    glb.extend(std::iter::repeat(0u8).take(bin_pad));

    glb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glb_header_valid() {
        let mesh = LoadedMesh {
            vertices: vec![0.0; 24], // 3 vertices × 8 floats
            indices: vec![0, 1, 2],
            vertex_count: 3,
            index_count: 3,
        };
        let glb = export_glb(&mesh, [1.0, 0.0, 0.0, 1.0]);

        // Check magic
        assert_eq!(&glb[0..4], &0x46546C67u32.to_le_bytes()); // "glTF"
        assert_eq!(&glb[4..8], &2u32.to_le_bytes()); // version 2
        let total_len = u32::from_le_bytes(glb[8..12].try_into().unwrap());
        assert_eq!(total_len as usize, glb.len());
    }
}
