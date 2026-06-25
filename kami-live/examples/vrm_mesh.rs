//! Offscreen GPU validation: render a single lit triangle to PNG (wgpu 24).
//! Foundation for the skinned humanoid mesh that replaces the box performer.
//! `cargo run -p kami-live --example vrm_mesh --target aarch64-apple-darwin`

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let (w, h) = (320u32, 240u32);
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .expect("adapter");
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .expect("device");

    let fmt = wgpu::TextureFormat::Rgba8UnormSrgb;
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: fmt,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(
            r#"
            @vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
              var p = array<vec2<f32>,3>(vec2(0.0,0.6), vec2(-0.6,-0.5), vec2(0.6,-0.5));
              return vec4<f32>(p[i], 0.0, 1.0);
            }
            @fragment fn fs() -> @location(0) vec4<f32> { return vec4<f32>(0.4,0.7,1.0,1.0); }
            "#
            .into(),
        ),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: None,
        vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs"), buffers: &[], compilation_options: Default::default() },
        fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs"), targets: &[Some(fmt.into())], compilation_options: Default::default() }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.15, a: 1.0 }), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        rp.set_pipeline(&pipeline);
        rp.draw(0..3, 0..1);
    }
    // readback
    let bpr = (w * 4).div_ceil(256) * 256;
    let buf = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (bpr * h) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    enc.copy_texture_to_buffer(
        wgpu::ImageCopyTexture { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::ImageCopyBuffer { buffer: &buf, layout: wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(bpr), rows_per_image: Some(h) } },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    queue.submit([enc.finish()]);
    let slice = buf.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::Maintain::Wait);
    let data = slice.get_mapped_range();
    let mut px = vec![0u8; (w * h * 4) as usize];
    for y in 0..h { let s = (y * bpr) as usize; let d = (y * w * 4) as usize; px[d..d + (w*4) as usize].copy_from_slice(&data[s..s + (w*4) as usize]); }
    image::save_buffer("vrm_mesh_test.png", &px, w, h, image::ExtendedColorType::Rgba8).unwrap();
    println!("wrote vrm_mesh_test.png — offscreen wgpu pipeline validated");
}
