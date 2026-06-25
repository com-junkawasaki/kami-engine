//! Render an MMD `.pmx` model's mesh offscreen (static, Lambert) → PNG.
//! `cargo run -p kami-live --example pmx_render --target aarch64-apple-darwin`
//!
//! Loads `assets/model.pmx` via `kami_skeleton_scene::pmx_to_model` when present
//! (drop your own *licensed* PMX there — MMD models are not redistributable, so
//! none ships here), otherwise renders a built-in cube so the pmx→pixels path is
//! demonstrable without an asset. Proves the MMD geometry import reaches the GPU,
//! the counterpart of `vrm_real` for VRM.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use kami_skeleton_scene::{pmx_to_model, PmxModel, PmxVertex};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct V { pos: [f32; 3], normal: [f32; 3] }
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct G { vp: [[f32; 4]; 4], light: [f32; 4] }

/// A built-in unit cube as a PmxModel, for when no `.pmx` asset is present.
fn cube_model() -> PmxModel {
    let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
        ([0.0, 0.0, 1.0], [[-0.5, -0.5, 0.5], [0.5, -0.5, 0.5], [0.5, 0.5, 0.5], [-0.5, 0.5, 0.5]]),
        ([0.0, 0.0, -1.0], [[0.5, -0.5, -0.5], [-0.5, -0.5, -0.5], [-0.5, 0.5, -0.5], [0.5, 0.5, -0.5]]),
        ([1.0, 0.0, 0.0], [[0.5, -0.5, 0.5], [0.5, -0.5, -0.5], [0.5, 0.5, -0.5], [0.5, 0.5, 0.5]]),
        ([-1.0, 0.0, 0.0], [[-0.5, -0.5, -0.5], [-0.5, -0.5, 0.5], [-0.5, 0.5, 0.5], [-0.5, 0.5, -0.5]]),
        ([0.0, 1.0, 0.0], [[-0.5, 0.5, 0.5], [0.5, 0.5, 0.5], [0.5, 0.5, -0.5], [-0.5, 0.5, -0.5]]),
        ([0.0, -1.0, 0.0], [[-0.5, -0.5, -0.5], [0.5, -0.5, -0.5], [0.5, -0.5, 0.5], [-0.5, -0.5, 0.5]]),
    ];
    let (mut vertices, mut indices) = (Vec::new(), Vec::new());
    for (n, q) in faces {
        let b = vertices.len() as u32;
        for p in q {
            vertices.push(PmxVertex { pos: p.into(), normal: n.into(), uv: glam::Vec2::ZERO, bones: [-1; 4], weights: [0.0; 4] });
        }
        indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }
    PmxModel { name: "cube".into(), vertices, indices, bones: vec![], morphs: vec![], materials: vec![], textures: vec![] }
}

fn main() { pollster::block_on(run()); }

async fn run() {
    let model = match std::fs::read("assets/model.pmx").ok().and_then(|b| pmx_to_model(&b)) {
        Some(m) => { println!("loaded PMX '{}': {} verts, {} tris, {} bones, {} morphs", m.name, m.vertices.len(), m.indices.len()/3, m.bones.len(), m.morphs.len()); m }
        None => { println!("no assets/model.pmx — rendering the built-in cube (drop a licensed .pmx to render it)"); cube_model() }
    };
    let verts: Vec<V> = model.vertices.iter().map(|v| V { pos: v.pos.into(), normal: v.normal.into() }).collect();
    let indices = &model.indices;
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for v in &verts { for k in 0..3 { lo[k] = lo[k].min(v.pos[k]); hi[k] = hi[k].max(v.pos[k]); } }
    let center = Vec3::new((lo[0]+hi[0])/2.0, (lo[1]+hi[1])/2.0, (lo[2]+hi[2])/2.0);
    let height = (hi[1]-lo[1]).max(0.5);

    let (w, h) = (420u32, 560u32);
    let inst = wgpu::Instance::default();
    let adapter = inst.request_adapter(&wgpu::RequestAdapterOptions::default()).await.unwrap();
    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await.unwrap();
    let vbuf = device.create_buffer(&wgpu::BufferDescriptor{label:None,size:std::mem::size_of_val(&verts[..]) as u64,usage:wgpu::BufferUsages::VERTEX|wgpu::BufferUsages::COPY_DST,mapped_at_creation:false});
    queue.write_buffer(&vbuf,0,bytemuck::cast_slice(&verts));
    let ibuf = device.create_buffer(&wgpu::BufferDescriptor{label:None,size:std::mem::size_of_val(&indices[..]) as u64,usage:wgpu::BufferUsages::INDEX|wgpu::BufferUsages::COPY_DST,mapped_at_creation:false});
    queue.write_buffer(&ibuf,0,bytemuck::cast_slice(indices));
    let gbuf = device.create_buffer(&wgpu::BufferDescriptor{label:None,size:std::mem::size_of::<G>() as u64,usage:wgpu::BufferUsages::UNIFORM|wgpu::BufferUsages::COPY_DST,mapped_at_creation:false});
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{label:None,entries:&[wgpu::BindGroupLayoutEntry{binding:0,visibility:wgpu::ShaderStages::VERTEX_FRAGMENT,ty:wgpu::BindingType::Buffer{ty:wgpu::BufferBindingType::Uniform,has_dynamic_offset:false,min_binding_size:None},count:None}]});
    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor{label:None,layout:&bgl,entries:&[wgpu::BindGroupEntry{binding:0,resource:gbuf.as_entire_binding()}]});
    let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{label:None,bind_group_layouts:&[&bgl],push_constant_ranges:&[]});
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor{label:None,source:wgpu::ShaderSource::Wgsl(r#"
        struct G { vp: mat4x4<f32>, light: vec4<f32> };
        @group(0) @binding(0) var<uniform> g: G;
        struct VO { @builtin(position) clip: vec4<f32>, @location(0) n: vec3<f32> };
        @vertex fn vs(@location(0) p: vec3<f32>, @location(1) n: vec3<f32>) -> VO { var o: VO; o.clip = g.vp*vec4<f32>(p,1.0); o.n = n; return o; }
        @fragment fn fs(i: VO) -> @location(0) vec4<f32> { let d = max(dot(normalize(i.n), -normalize(g.light.xyz)), 0.0); let c = vec3<f32>(0.8,0.7,0.66)*(0.3+0.65*d); return vec4<f32>(pow(c, vec3<f32>(1.0/2.2)), 1.0); }
    "#.into())});
    let fmt = wgpu::TextureFormat::Rgba8UnormSrgb;
    let vbl = wgpu::VertexBufferLayout{array_stride:std::mem::size_of::<V>() as u64,step_mode:wgpu::VertexStepMode::Vertex,attributes:&wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3]};
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{label:None,layout:Some(&pl),
        vertex:wgpu::VertexState{module:&shader,entry_point:Some("vs"),buffers:&[vbl],compilation_options:Default::default()},
        fragment:Some(wgpu::FragmentState{module:&shader,entry_point:Some("fs"),targets:&[Some(fmt.into())],compilation_options:Default::default()}),
        primitive:wgpu::PrimitiveState{cull_mode:None,..Default::default()},
        depth_stencil:Some(wgpu::DepthStencilState{format:wgpu::TextureFormat::Depth32Float,depth_write_enabled:true,depth_compare:wgpu::CompareFunction::Less,stencil:Default::default(),bias:Default::default()}),
        multisample:Default::default(),multiview:None,cache:None});
    let color = device.create_texture(&wgpu::TextureDescriptor{label:None,size:wgpu::Extent3d{width:w,height:h,depth_or_array_layers:1},mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format:fmt,usage:wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::COPY_SRC,view_formats:&[]});
    let cview=color.create_view(&Default::default());
    let dtex=device.create_texture(&wgpu::TextureDescriptor{label:None,size:wgpu::Extent3d{width:w,height:h,depth_or_array_layers:1},mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Depth32Float,usage:wgpu::TextureUsages::RENDER_ATTACHMENT,view_formats:&[]});
    let dview=dtex.create_view(&Default::default());
    let dist = height * 3.5;
    let eye = center + Vec3::new(dist * 0.45, height * 0.5, dist);
    let vp = (Mat4::perspective_rh(0.7, w as f32/h as f32, 0.05, 100.0)*Mat4::look_at_rh(eye, center, Vec3::Y)).to_cols_array_2d();
    queue.write_buffer(&gbuf,0,bytemuck::bytes_of(&G{vp,light:[-0.3,-0.5,-0.75,0.0]}));
    let mut enc=device.create_command_encoder(&Default::default());
    {
        let mut rp=enc.begin_render_pass(&wgpu::RenderPassDescriptor{label:None,
            color_attachments:&[Some(wgpu::RenderPassColorAttachment{view:&cview,resolve_target:None,ops:wgpu::Operations{load:wgpu::LoadOp::Clear(wgpu::Color{r:0.55,g:0.6,b:0.7,a:1.0}),store:wgpu::StoreOp::Store}})],
            depth_stencil_attachment:Some(wgpu::RenderPassDepthStencilAttachment{view:&dview,depth_ops:Some(wgpu::Operations{load:wgpu::LoadOp::Clear(1.0),store:wgpu::StoreOp::Store}),stencil_ops:None}),timestamp_writes:None,occlusion_query_set:None});
        rp.set_pipeline(&pipeline); rp.set_bind_group(0,&bg,&[]); rp.set_vertex_buffer(0,vbuf.slice(..)); rp.set_index_buffer(ibuf.slice(..),wgpu::IndexFormat::Uint32); rp.draw_indexed(0..indices.len() as u32,0,0..1);
    }
    let bpr=(w*4).div_ceil(256)*256;
    let rbuf=device.create_buffer(&wgpu::BufferDescriptor{label:None,size:(bpr*h) as u64,usage:wgpu::BufferUsages::COPY_DST|wgpu::BufferUsages::MAP_READ,mapped_at_creation:false});
    enc.copy_texture_to_buffer(wgpu::ImageCopyTexture{texture:&color,mip_level:0,origin:wgpu::Origin3d::ZERO,aspect:wgpu::TextureAspect::All},wgpu::ImageCopyBuffer{buffer:&rbuf,layout:wgpu::ImageDataLayout{offset:0,bytes_per_row:Some(bpr),rows_per_image:Some(h)}},wgpu::Extent3d{width:w,height:h,depth_or_array_layers:1});
    queue.submit([enc.finish()]);
    let sl=rbuf.slice(..); sl.map_async(wgpu::MapMode::Read,|_|{}); device.poll(wgpu::Maintain::Wait);
    let data=sl.get_mapped_range(); let mut px=vec![0u8;(w*h*4) as usize];
    for y in 0..h { let s=(y*bpr) as usize; let d=(y*w*4) as usize; px[d..d+(w*4) as usize].copy_from_slice(&data[s..s+(w*4) as usize]); }
    image::save_buffer("pmx_render.png",&px,w,h,image::ExtendedColorType::Rgba8).unwrap();
    println!("wrote pmx_render.png — MMD .pmx mesh import → GPU");
}
