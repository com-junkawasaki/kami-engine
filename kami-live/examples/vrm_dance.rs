//! Seed-san (real VRM) DANCING — VRM geometry + skin (JOINTS_0/WEIGHTS_0 +
//! skeleton) GPU-skinned by the clj/edn `DancePose` (humanoid bones posed each
//! frame), rendered offscreen to PNG/GIF. The real VRM × skinning × the show.
//! `cargo run -p kami-live --example vrm_dance --target aarch64-apple-darwin`

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};
use kami_live::scene::DanceScene;
use kami_vrm::vrm_types::HumanBoneName;
use std::collections::HashMap;

const SCENE: &str = include_str!("../../kami-clj-play3d/games/dance/scene.edn");

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct V { pos: [f32; 3], normal: [f32; 3], joints: [u32; 4], weights: [f32; 4] }
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct G { vp: [[f32; 4]; 4], light: [f32; 4] }

fn node_local(n: &kami_vrm::gltf_types::Node) -> Mat4 {
    if let Some(m) = n.matrix { return Mat4::from_cols_array(&m); }
    let t = n.translation.map(Vec3::from).unwrap_or(Vec3::ZERO);
    let r = n.rotation.map(Quat::from_array).unwrap_or(Quat::IDENTITY);
    let s = n.scale.map(Vec3::from).unwrap_or(Vec3::ONE);
    Mat4::from_scale_rotation_translation(s, r, t)
}

fn main() { pollster::block_on(run()); }

async fn run() {
    let bytes = std::fs::read("assets/Seed-san.vrm").expect("assets/Seed-san.vrm");
    let doc = kami_vrm::parse_vrm(&bytes).expect("parse VRM");
    let nodes = &doc.gltf.nodes;
    let nn = nodes.len();

    // node parent map
    let mut parent = vec![-1i32; nn];
    for (i, n) in nodes.iter().enumerate() { for &c in &n.children { parent[c] = i as i32; } }
    // process order: parents before children
    let mut order = Vec::new();
    let mut seen = vec![false; nn];
    fn visit(i: usize, nodes: &[kami_vrm::gltf_types::Node], seen: &mut [bool], order: &mut Vec<usize>) {
        if seen[i] { return; } seen[i] = true; order.push(i);
        for &c in &nodes[i].children { visit(c, nodes, seen, order); }
    }
    for i in 0..nn { if parent[i] < 0 { visit(i, nodes, &mut seen, &mut order); } }

    // inverse-bind per node (from each skin)
    let mut inv_bind = vec![Mat4::IDENTITY; nn];
    for skin in &doc.gltf.skins {
        if let Some(ibm) = skin.inverse_bind_matrices {
            if let Ok(flat) = kami_vrm::convert::read_accessor_f32(&doc, ibm) {
                for (j, &node) in skin.joints.iter().enumerate() {
                    if (j + 1) * 16 <= flat.len() {
                        let mut m = [0.0f32; 16];
                        m.copy_from_slice(&flat[j * 16..j * 16 + 16]);
                        inv_bind[node] = Mat4::from_cols_array(&m);
                    }
                }
            }
        }
    }

    // humanoid bone -> node
    let hb: HashMap<HumanBoneName, usize> = doc.humanoid.human_bones.iter().map(|b| (b.bone, b.node)).collect();

    // geometry with skin weights, joints remapped to NODE indices
    let mut verts: Vec<V> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    for (ni, node) in nodes.iter().enumerate() {
        let (Some(mi), Some(si)) = (node.mesh, node.skin) else { continue };
        let _ = ni;
        let skin = &doc.gltf.skins[si];
        let mesh = &doc.gltf.meshes[mi];
        for pi in 0..mesh.primitives.len() {
            let Ok((inter, idx)) = kami_vrm::convert::extract_primitive_mesh(&doc, mi, pi) else { continue };
            let prim = &mesh.primitives[pi];
            let jacc = prim.attributes.get("JOINTS_0").and_then(|v| v.as_u64());
            let wacc = prim.attributes.get("WEIGHTS_0").and_then(|v| v.as_u64());
            let jdata = jacc.and_then(|a| kami_vrm::convert::read_accessor_f32(&doc, a as usize).ok());
            let wdata = wacc.and_then(|a| kami_vrm::convert::read_accessor_f32(&doc, a as usize).ok());
            let base = verts.len() as u32;
            let vc = inter.len() / 8;
            for v in 0..vc {
                let p = [inter[v*8], inter[v*8+1], inter[v*8+2]];
                let n = [inter[v*8+3], inter[v*8+4], inter[v*8+5]];
                let mut j = [0u32; 4];
                let mut w = [0.0f32; 4];
                if let (Some(jd), Some(wd)) = (&jdata, &wdata) {
                    for k in 0..4 {
                        let local = jd[v*4+k] as usize;
                        j[k] = *skin.joints.get(local).unwrap_or(&0) as u32;
                        w[k] = wd[v*4+k];
                    }
                    let sum: f32 = w.iter().sum();
                    if sum > 0.0 { for x in &mut w { *x /= sum; } } else { w[0] = 1.0; }
                } else { w[0] = 1.0; }
                verts.push(V { pos: p, normal: n, joints: j, weights: w });
            }
            indices.extend(idx.iter().map(|i| i + base));
        }
    }
    println!("Seed-san: {} verts, {} tris, {} nodes", verts.len(), indices.len()/3, nn);

    // bounds for framing
    let (mut lo, mut hi) = ([f32::MAX;3],[f32::MIN;3]);
    for v in &verts { for k in 0..3 { lo[k]=lo[k].min(v.pos[k]); hi[k]=hi[k].max(v.pos[k]); } }
    let center = Vec3::new((lo[0]+hi[0])/2.0,(lo[1]+hi[1])/2.0,(lo[2]+hi[2])/2.0);
    let height = hi[1]-lo[1];

    // ---- GPU ----
    let (w,h)=(420u32,620u32);
    let inst = wgpu::Instance::default();
    let adapter = inst.request_adapter(&wgpu::RequestAdapterOptions::default()).await.unwrap();
    let (device,queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await.unwrap();
    let vbuf = device.create_buffer(&wgpu::BufferDescriptor{label:None,size:std::mem::size_of_val(&verts[..]) as u64,usage:wgpu::BufferUsages::VERTEX|wgpu::BufferUsages::COPY_DST,mapped_at_creation:false});
    queue.write_buffer(&vbuf,0,bytemuck::cast_slice(&verts));
    let ibuf = device.create_buffer(&wgpu::BufferDescriptor{label:None,size:std::mem::size_of_val(&indices[..]) as u64,usage:wgpu::BufferUsages::INDEX|wgpu::BufferUsages::COPY_DST,mapped_at_creation:false});
    queue.write_buffer(&ibuf,0,bytemuck::cast_slice(&indices));
    let gbuf = device.create_buffer(&wgpu::BufferDescriptor{label:None,size:std::mem::size_of::<G>() as u64,usage:wgpu::BufferUsages::UNIFORM|wgpu::BufferUsages::COPY_DST,mapped_at_creation:false});
    let pbuf = device.create_buffer(&wgpu::BufferDescriptor{label:None,size:(nn*64) as u64,usage:wgpu::BufferUsages::STORAGE|wgpu::BufferUsages::COPY_DST,mapped_at_creation:false});

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{label:None,entries:&[
        wgpu::BindGroupLayoutEntry{binding:0,visibility:wgpu::ShaderStages::VERTEX_FRAGMENT,ty:wgpu::BindingType::Buffer{ty:wgpu::BufferBindingType::Uniform,has_dynamic_offset:false,min_binding_size:None},count:None},
        wgpu::BindGroupLayoutEntry{binding:1,visibility:wgpu::ShaderStages::VERTEX,ty:wgpu::BindingType::Buffer{ty:wgpu::BufferBindingType::Storage{read_only:true},has_dynamic_offset:false,min_binding_size:None},count:None},
    ]});
    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor{label:None,layout:&bgl,entries:&[
        wgpu::BindGroupEntry{binding:0,resource:gbuf.as_entire_binding()},
        wgpu::BindGroupEntry{binding:1,resource:pbuf.as_entire_binding()},
    ]});
    let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{label:None,bind_group_layouts:&[&bgl],push_constant_ranges:&[]});
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor{label:None,source:wgpu::ShaderSource::Wgsl(r#"
        struct G { vp: mat4x4<f32>, light: vec4<f32> };
        @group(0) @binding(0) var<uniform> g: G;
        @group(0) @binding(1) var<storage, read> palette: array<mat4x4<f32>>;
        struct VO { @builtin(position) clip: vec4<f32>, @location(0) n: vec3<f32> };
        @vertex fn vs(@location(0) p: vec3<f32>, @location(1) nor: vec3<f32>, @location(2) j: vec4<u32>, @location(3) wt: vec4<f32>) -> VO {
          let skin = palette[j.x]*wt.x + palette[j.y]*wt.y + palette[j.z]*wt.z + palette[j.w]*wt.w;
          var o: VO; o.clip = g.vp * (skin * vec4<f32>(p,1.0)); o.n = normalize((skin*vec4<f32>(nor,0.0)).xyz); return o;
        }
        @fragment fn fs(i: VO) -> @location(0) vec4<f32> {
          let d = max(dot(normalize(i.n), -normalize(g.light.xyz)), 0.0);
          let c = vec3<f32>(0.80, 0.70, 0.66) * (0.30 + 0.62*d);
          return vec4<f32>(pow(c, vec3<f32>(1.0/2.2)), 1.0);
        }
    "#.into())});
    let fmt = wgpu::TextureFormat::Rgba8UnormSrgb;
    let vbl = wgpu::VertexBufferLayout{array_stride:std::mem::size_of::<V>() as u64,step_mode:wgpu::VertexStepMode::Vertex,attributes:&wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Uint32x4,3=>Float32x4]};
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{label:None,layout:Some(&pl),
        vertex:wgpu::VertexState{module:&shader,entry_point:Some("vs"),buffers:&[vbl],compilation_options:Default::default()},
        fragment:Some(wgpu::FragmentState{module:&shader,entry_point:Some("fs"),targets:&[Some(fmt.into())],compilation_options:Default::default()}),
        primitive:wgpu::PrimitiveState{cull_mode:None,..Default::default()},
        depth_stencil:Some(wgpu::DepthStencilState{format:wgpu::TextureFormat::Depth32Float,depth_write_enabled:true,depth_compare:wgpu::CompareFunction::Less,stencil:Default::default(),bias:Default::default()}),
        multisample:Default::default(),multiview:None,cache:None});
    let color = device.create_texture(&wgpu::TextureDescriptor{label:None,size:wgpu::Extent3d{width:w,height:h,depth_or_array_layers:1},mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format:fmt,usage:wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::COPY_SRC,view_formats:&[]});
    let cview = color.create_view(&Default::default());
    let dtex = device.create_texture(&wgpu::TextureDescriptor{label:None,size:wgpu::Extent3d{width:w,height:h,depth_or_array_layers:1},mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Depth32Float,usage:wgpu::TextureUsages::RENDER_ATTACHMENT,view_formats:&[]});
    let dview = dtex.create_view(&Default::default());
    let dist = height*1.5;
    let eye = center + Vec3::new(0.0, height*0.02, dist);
    let vp = Mat4::perspective_rh(0.7, w as f32/h as f32, 0.05, 100.0) * Mat4::look_at_rh(eye, center, Vec3::Y);
    queue.write_buffer(&gbuf,0,bytemuck::bytes_of(&G{vp:vp.to_cols_array_2d(),light:[-0.3,-0.5,-0.75,0.0]}));

    let mut scene = DanceScene::from_edn(SCENE).unwrap();
    scene.show.start();
    for _ in 0..(61.0*60.0) as i32 { scene.frame(1.0/60.0); } // into the wota (arms up)

    let base_local: Vec<Mat4> = nodes.iter().map(node_local).collect();
    let bpr=(w*4).div_ceil(256)*256;
    let rbuf = device.create_buffer(&wgpu::BufferDescriptor{label:None,size:(bpr*h) as u64,usage:wgpu::BufferUsages::COPY_DST|wgpu::BufferUsages::MAP_READ,mapped_at_creation:false});
    let mut gif=Vec::new();
    for frame in 0..32 {
        for _ in 0..3 { scene.frame(1.0/60.0); }
        let pose = scene.show.snapshot().performer_pose;
        // pose humanoid bones
        let mut local = base_local.clone();
        let mut apply = |bone: HumanBoneName, q: Quat| { if let Some(&nd) = hb.get(&bone) { local[nd] = local[nd] * Mat4::from_quat(q); } };
        apply(HumanBoneName::Spine, Quat::from_rotation_z(pose.spine_sway * 0.5));
        apply(HumanBoneName::Chest, Quat::from_rotation_z(pose.spine_sway * 0.5));
        apply(HumanBoneName::LeftUpperArm, Quat::from_rotation_z(-pose.arms_up * 1.1));
        apply(HumanBoneName::RightUpperArm, Quat::from_rotation_z(pose.arms_up * 1.1));
        apply(HumanBoneName::LeftUpperLeg, Quat::from_rotation_x(pose.vertical_bob * 2.0));
        apply(HumanBoneName::RightUpperLeg, Quat::from_rotation_x(-pose.vertical_bob * 2.0));
        if let Some(&hips) = hb.get(&HumanBoneName::Hips) {
            local[hips] = Mat4::from_translation(Vec3::new(pose.root_translation.x, pose.vertical_bob, 0.0))
                * Mat4::from_rotation_y(pose.root_yaw) * local[hips];
        }
        // FK
        let mut world = vec![Mat4::IDENTITY; nn];
        for &i in &order { world[i] = if parent[i] < 0 { local[i] } else { world[parent[i] as usize] * local[i] }; }
        let palette: Vec<[[f32;4];4]> = (0..nn).map(|i| (world[i] * inv_bind[i]).to_cols_array_2d()).collect();
        queue.write_buffer(&pbuf, 0, bytemuck::cast_slice(&palette));

        let mut enc = device.create_command_encoder(&Default::default());
        {
            let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor{label:None,
                color_attachments:&[Some(wgpu::RenderPassColorAttachment{view:&cview,resolve_target:None,ops:wgpu::Operations{load:wgpu::LoadOp::Clear(wgpu::Color{r:0.55,g:0.6,b:0.7,a:1.0}),store:wgpu::StoreOp::Store}})],
                depth_stencil_attachment:Some(wgpu::RenderPassDepthStencilAttachment{view:&dview,depth_ops:Some(wgpu::Operations{load:wgpu::LoadOp::Clear(1.0),store:wgpu::StoreOp::Store}),stencil_ops:None}),
                timestamp_writes:None,occlusion_query_set:None});
            rp.set_pipeline(&pipeline); rp.set_bind_group(0,&bg,&[]);
            rp.set_vertex_buffer(0,vbuf.slice(..)); rp.set_index_buffer(ibuf.slice(..),wgpu::IndexFormat::Uint32);
            rp.draw_indexed(0..indices.len() as u32,0,0..1);
        }
        enc.copy_texture_to_buffer(wgpu::ImageCopyTexture{texture:&color,mip_level:0,origin:wgpu::Origin3d::ZERO,aspect:wgpu::TextureAspect::All},wgpu::ImageCopyBuffer{buffer:&rbuf,layout:wgpu::ImageDataLayout{offset:0,bytes_per_row:Some(bpr),rows_per_image:Some(h)}},wgpu::Extent3d{width:w,height:h,depth_or_array_layers:1});
        queue.submit([enc.finish()]);
        let sl=rbuf.slice(..); sl.map_async(wgpu::MapMode::Read,|_|{}); device.poll(wgpu::Maintain::Wait);
        let data=sl.get_mapped_range();
        let mut px=vec![0u8;(w*h*4) as usize];
        for y in 0..h { let s=(y*bpr) as usize; let d=(y*w*4) as usize; px[d..d+(w*4) as usize].copy_from_slice(&data[s..s+(w*4) as usize]); }
        drop(data); rbuf.unmap();
        if frame%8==0 { image::save_buffer(format!("seed_{frame:02}.png"),&px,w,h,image::ExtendedColorType::Rgba8).unwrap(); }
        gif.push(image::Frame::from_parts(image::RgbaImage::from_raw(w,h,px).unwrap(),0,0,image::Delay::from_numer_denom_ms(70,1)));
    }
    let fl=std::fs::File::create("seed_dance.gif").unwrap();
    let mut e=image::codecs::gif::GifEncoder::new(fl);
    e.set_repeat(image::codecs::gif::Repeat::Infinite).unwrap();
    e.encode_frames(gif.into_iter()).unwrap();
    println!("wrote seed_dance.gif + seed_*.png — the real Seed-san VRM dancing the show");
}
