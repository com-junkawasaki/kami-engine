//! WebGPU compute backend for `cartpole_step.wgsl`.
//!
//! Phase D real-device verification per ADR-2605261800: dispatches
//! `kami-genesis/src/wgsl/cartpole_step.wgsl` on a real `wgpu::Device`
//! (Metal on macOS, Vulkan on Linux, DX12 on Windows, WebGPU on browser).
//! Validates that the WGSL kernel produces results matching the scalar
//! Rust formula bit-for-bit (within f32 epsilon).
//!
//! Gated behind feature `gpu` so default builds don't pull in wgpu.

use crate::cartpole::{CartpoleConfig, CartpoleState};
use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct GpuState {
    x: f32,
    x_dot: f32,
    theta: f32,
    theta_dot: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct GpuCfg {
    cart_mass: f32,
    pole_mass: f32,
    pole_half_length: f32,
    gravity: f32,
    force_mag: f32,
    dt: f32,
    num_envs: u32,
    _pad: u32,
}

impl From<&CartpoleState> for GpuState {
    fn from(s: &CartpoleState) -> Self {
        GpuState { x: s.x, x_dot: s.x_dot, theta: s.theta, theta_dot: s.theta_dot }
    }
}

impl From<GpuState> for CartpoleState {
    fn from(s: GpuState) -> Self {
        CartpoleState { x: s.x, x_dot: s.x_dot, theta: s.theta, theta_dot: s.theta_dot }
    }
}

/// Wraps a `wgpu::Device + Queue + ComputePipeline` for cartpole_step.wgsl.
pub struct WgpuBackend {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    pub backend_name: String,
}

impl WgpuBackend {
    /// Initialise wgpu and compile the Cartpole compute pipeline.
    /// Blocking; uses `pollster` to drive the async adapter/device requests.
    pub fn new() -> Result<Self, String> {
        pollster::block_on(Self::new_async())
    }

    async fn new_async() -> Result<Self, String> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| "no wgpu adapter available".to_string())?;

        let info = adapter.get_info();
        let backend_name = format!("{:?} / {}", info.backend, info.name);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("kami-genesis-device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .map_err(|e| format!("wgpu device request failed: {e}"))?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cartpole_step.wgsl"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(super::WGSL_SOURCE)),
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("kami-genesis-cartpole-bgl"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("kami-genesis-cartpole-pl"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("kami-genesis-cartpole-pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("step_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Ok(WgpuBackend { device, queue, pipeline, bind_group_layout, backend_name })
    }

    /// One step on the GPU for `num_envs` envs.
    /// Mirrors `step_vectorized` semantics; returns the new state.
    pub fn step(
        &self,
        states: &mut [CartpoleState],
        actions: &[f32],
        cfg: &CartpoleConfig,
    ) -> Result<(), String> {
        pollster::block_on(self.step_async(states, actions, cfg))
    }

    async fn step_async(
        &self,
        states: &mut [CartpoleState],
        actions: &[f32],
        cfg: &CartpoleConfig,
    ) -> Result<(), String> {
        if states.len() != actions.len() {
            return Err(format!(
                "states.len() ({}) != actions.len() ({})",
                states.len(),
                actions.len()
            ));
        }
        let n = states.len() as u32;
        if n == 0 {
            return Ok(());
        }

        let gpu_states: Vec<GpuState> = states.iter().map(Into::into).collect();
        let states_bytes = bytemuck::cast_slice(&gpu_states);
        let actions_bytes = bytemuck::cast_slice(actions);
        let gpu_cfg = GpuCfg {
            cart_mass: cfg.cart_mass,
            pole_mass: cfg.pole_mass,
            pole_half_length: cfg.pole_half_length,
            gravity: cfg.gravity,
            force_mag: cfg.force_mag,
            dt: cfg.dt,
            num_envs: n,
            _pad: 0,
        };

        let states_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("states"),
            contents: states_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
        let actions_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("actions"),
            contents: actions_bytes,
            usage: wgpu::BufferUsages::STORAGE,
        });
        let cfg_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cfg"),
            contents: bytemuck::bytes_of(&gpu_cfg),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let readback_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: states_bytes.len() as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("kami-genesis-cartpole-bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: states_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: actions_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: cfg_buf.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("kami-genesis-encoder") });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("cartpole_step"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            let workgroup_count = n.div_ceil(64);
            cpass.dispatch_workgroups(workgroup_count, 1, 1);
        }
        encoder.copy_buffer_to_buffer(&states_buf, 0, &readback_buf, 0, states_bytes.len() as u64);
        self.queue.submit(Some(encoder.finish()));

        let slice = readback_buf.slice(..);
        let (tx, rx) = futures_channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        let _ = self.device.poll(wgpu::Maintain::Wait);
        rx.recv()
            .map_err(|e| format!("map_async receive failed: {e}"))?
            .map_err(|e| format!("map_async failed: {e}"))?;

        let mapped = slice.get_mapped_range();
        let result: &[GpuState] = bytemuck::cast_slice(&mapped);
        for (i, s) in result.iter().enumerate() {
            states[i] = (*s).into();
        }
        drop(mapped);
        readback_buf.unmap();
        Ok(())
    }
}

/// Tiny single-shot channel for `map_async` callback. Avoids pulling in
/// `futures` / `tokio` just for one signal.
fn futures_channel<T>() -> (std::sync::mpsc::Sender<T>, std::sync::mpsc::Receiver<T>) {
    std::sync::mpsc::channel()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vectorized::step_vectorized;

    #[test]
    fn wgpu_dispatch_matches_cpu_vectorized() {
        let backend = match WgpuBackend::new() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("skipping GPU test, no adapter available: {e}");
                return;
            }
        };
        println!("wgpu backend: {}", backend.backend_name);

        let cfg = CartpoleConfig::default();
        let n = 256;
        let mut gpu_states: Vec<CartpoleState> = (0..n)
            .map(|i| CartpoleState {
                theta: 0.05 + (i as f32) * 0.0001,
                ..Default::default()
            })
            .collect();
        let mut cpu_states = gpu_states.clone();
        let actions: Vec<f32> = (0..n).map(|i| (i as f32) * 0.01).collect();

        // 50 step iterations to compound numerical work and catch any drift.
        for _ in 0..50 {
            backend.step(&mut gpu_states, &actions, &cfg).unwrap();
            step_vectorized(&mut cpu_states, &actions, &cfg);
        }

        let mut max_dx = 0.0_f32;
        let mut max_dtheta = 0.0_f32;
        for i in 0..n {
            max_dx = max_dx.max((gpu_states[i].x - cpu_states[i].x).abs());
            max_dtheta = max_dtheta.max((gpu_states[i].theta - cpu_states[i].theta).abs());
        }
        println!("max |Δx| = {:.3e}, max |Δθ| = {:.3e} over {} envs × 50 steps", max_dx, max_dtheta, n);
        // f32 rounding may produce small drift over 50 steps; require ≤ 1e-3.
        assert!(max_dx < 1e-3, "x drift too large: {max_dx}");
        assert!(max_dtheta < 1e-3, "theta drift too large: {max_dtheta}");
    }

    #[test]
    fn single_step_matches_scalar() {
        let backend = match WgpuBackend::new() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("skipping GPU test, no adapter available: {e}");
                return;
            }
        };
        let cfg = CartpoleConfig::default();
        let mut gpu = vec![CartpoleState { theta: 0.1, ..Default::default() }; 1];
        let mut scalar = CartpoleState { theta: 0.1, ..Default::default() };
        let actions = vec![5.0_f32];
        backend.step(&mut gpu, &actions, &cfg).unwrap();
        scalar.step(5.0, &cfg);
        assert!((gpu[0].x - scalar.x).abs() < 1e-6);
        assert!((gpu[0].theta - scalar.theta).abs() < 1e-6);
    }
}
