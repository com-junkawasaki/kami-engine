//! Render a royale-style city with the native executor and save a PNG.
//! `cargo run -p kami-webgpu-rs --example render_png --target aarch64-apple-darwin`
//! Proves the native Rust/wgpu path renders the same kind of scene (PBR + shadows) the
//! web does — a viewable golden frame, no window.

use kami_webgpu_rs::{render, Globals, Instance};

fn main() {
    let mut insts: Vec<Instance> = Vec::new();
    // ground
    insts.push(Instance { pos: [0.0, -0.5, 0.0], color: [0.34, 0.52, 0.30], size: [400.0, 1.0], yaw: 0.0, metallic: 0.0, roughness: 0.95, emissive: 0.0 });

    // deterministic xorshift scatter (mirrors the web's CLJ scatter)
    let mut seed: u32 = 2654435769;
    let mut rnd = || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed & 0x7fffffff) as f32 / 2147483647.0
    };
    let spread = 90.0;
    for _ in 0..170 {
        let x = (rnd() * 2.0 - 1.0) * spread;
        let z = (rnd() * 2.0 - 1.0) * spread;
        if (x * x + z * z).sqrt() < 8.0 { continue; }
        if rnd() < 0.4 {
            // tree: trunk + foliage
            insts.push(Instance { pos: [x, 0.0, z], color: [0.45, 0.32, 0.2], size: [0.33, 1.3], yaw: 0.0, metallic: 0.0, roughness: 0.95, emissive: 0.0 });
            insts.push(Instance { pos: [x, 1.3, z], color: [0.28, 0.55, 0.30], size: [1.1, 1.6], yaw: 0.0, metallic: 0.0, roughness: 0.95, emissive: 0.0 });
        } else {
            let h = 2.0 + rnd() * 5.0;
            let (color, metallic, roughness) = if rnd() < 0.5 {
                ([0.62, 0.60, 0.66], 0.8, 0.25) // glassy tower
            } else {
                ([0.70, 0.66, 0.55], 0.05, 0.85) // matte concrete
            };
            insts.push(Instance { pos: [x, 0.0, z], color, size: [2.0, h], yaw: 0.0, metallic, roughness, emissive: 0.0 });
        }
    }
    // glowing player at the centre
    insts.push(Instance { pos: [0.0, 0.0, 0.0], color: [0.30, 0.62, 1.0], size: [0.9, 1.9], yaw: 0.0, metallic: 0.2, roughness: 0.35, emissive: 0.5 });

    let g = Globals {
        horizon: [0.74, 0.84, 0.95],
        sun_dir: [-0.4, -0.85, -0.35],
        sun: [1.0, 0.96, 0.85],
        eye: Some([45.0, 40.0, 45.0]),
        target: Some([0.0, 0.0, 0.0]),
    };
    let (w, h) = (900u32, 560u32);
    let px = render(&g, &insts, w, h);
    image::save_buffer("native-royale.png", &px, w, h, image::ExtendedColorType::Rgba8).unwrap();
    println!("wrote native-royale.png — {} instances", insts.len());
}
