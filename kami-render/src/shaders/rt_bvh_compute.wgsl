// kami-render — software-BVH path-tracer primary-hit compute pass.
//
// The WGSL "LBVH compute" executor for kami.rt (the portable path that runs on
// stable WebGPU without the ray-query extension). Binds the CPU-built LBVH from
// kami-rt (nodes + Morton-sorted triangles) as storage buffers, generates a
// primary ray per pixel from RtGlobals, traverses the BVH in-shader, and writes
// the nearest hit. Struct layouts match kami_rt::gpu::{GpuNode, GpuTri} exactly.

struct BvhNode {
    min: vec3<f32>,
    left: u32,
    max: vec3<f32>,
    right: u32,
    start: u32,
    count: u32,
    pad0: u32,
    pad1: u32,
};

struct Tri {
    v0: vec3<f32>,
    id: u32,
    v1: vec3<f32>,
    p1: u32,
    v2: vec3<f32>,
    p2: u32,
};

struct RtGlobals {
    inv_view_proj: mat4x4<f32>,
    cam_pos: vec4<f32>,
    dims: vec4<u32>,   // x=width, y=height
};

@group(0) @binding(0) var<storage, read>       nodes:    array<BvhNode>;
@group(0) @binding(1) var<storage, read>       tris:     array<Tri>;
@group(0) @binding(2) var<uniform>             globals:  RtGlobals;
// Per pixel: (t, tri_id, bary_u, bary_v); t < 0 means miss.
@group(0) @binding(3) var<storage, read_write> out_hits: array<vec4<f32>>;

struct Hit { t: f32, id: f32, u: f32, v: f32 };

fn intersect_tri(origin: vec3<f32>, dir: vec3<f32>, tri: Tri) -> vec4<f32> {
    // returns (t, u, v, hit?1:0)
    let e1 = tri.v1 - tri.v0;
    let e2 = tri.v2 - tri.v0;
    let p = cross(dir, e2);
    let det = dot(e1, p);
    if (abs(det) < 1e-8) { return vec4<f32>(0.0, 0.0, 0.0, 0.0); }
    let inv = 1.0 / det;
    let tv = origin - tri.v0;
    let u = dot(tv, p) * inv;
    if (u < 0.0 || u > 1.0) { return vec4<f32>(0.0, 0.0, 0.0, 0.0); }
    let q = cross(tv, e1);
    let v = dot(dir, q) * inv;
    if (v < 0.0 || u + v > 1.0) { return vec4<f32>(0.0, 0.0, 0.0, 0.0); }
    let t = dot(e2, q) * inv;
    if (t <= 1e-4) { return vec4<f32>(0.0, 0.0, 0.0, 0.0); }
    return vec4<f32>(t, u, v, 1.0);
}

fn slab_hit(origin: vec3<f32>, inv_dir: vec3<f32>, lo: vec3<f32>, hi: vec3<f32>, tmax: f32) -> bool {
    let t0 = (lo - origin) * inv_dir;
    let t1 = (hi - origin) * inv_dir;
    let tsmall = min(t0, t1);
    let tbig = max(t0, t1);
    let tmin = max(max(tsmall.x, tsmall.y), tsmall.z);
    let tcap = min(min(tbig.x, tbig.y), min(tbig.z, tmax));
    return tcap >= max(tmin, 0.0);
}

fn primary_ray(px: u32, py: u32) -> array<vec3<f32>, 2> {
    let w = f32(globals.dims.x);
    let h = f32(globals.dims.y);
    let ndc = (vec2<f32>(f32(px) + 0.5, f32(py) + 0.5) / vec2<f32>(w, h)) * 2.0 - 1.0;
    let far = globals.inv_view_proj * vec4<f32>(ndc.x, -ndc.y, 1.0, 1.0);
    let world = far.xyz / far.w;
    let origin = globals.cam_pos.xyz;
    let dir = normalize(world - origin);
    return array<vec3<f32>, 2>(origin, dir);
}

@compute @workgroup_size(8, 8, 1)
fn trace(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= globals.dims.x || gid.y >= globals.dims.y) { return; }
    let idx = gid.y * globals.dims.x + gid.x;

    let ray = primary_ray(gid.x, gid.y);
    let origin = ray[0];
    let dir = ray[1];
    let inv_dir = vec3<f32>(1.0) / dir;

    var best = vec4<f32>(-1.0, 0.0, 0.0, 0.0); // (t, id, u, v)
    var best_t = 1e30;

    // Explicit stack traversal (depth cap 64 is ample for a median-split BVH).
    var stack: array<u32, 64>;
    var sp = 0;
    stack[sp] = 0u;
    sp = sp + 1;

    loop {
        if (sp == 0) { break; }
        sp = sp - 1;
        let ni = stack[sp];
        let node = nodes[ni];
        if (!slab_hit(origin, inv_dir, node.min, node.max, best_t)) { continue; }

        if (node.count > 0u) {
            var k = node.start;
            let end = node.start + node.count;
            loop {
                if (k >= end) { break; }
                let r = intersect_tri(origin, dir, tris[k]);
                if (r.w > 0.5 && r.x < best_t) {
                    best_t = r.x;
                    best = vec4<f32>(r.x, f32(tris[k].id), r.y, r.z);
                }
                k = k + 1u;
            }
        } else {
            if (sp < 63) { stack[sp] = node.left;  sp = sp + 1; }
            if (sp < 63) { stack[sp] = node.right; sp = sp + 1; }
        }
    }

    out_hits[idx] = best;
}
