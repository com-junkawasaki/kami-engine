//! VRM dance, fully clj/edn-driven, via the shared `common/vrm.rs` renderer.
//! Reads `:dance/avatar` (vrm path / spring-bones / scale) from scene.edn and
//! renders the real VRM with skinning + textures + MToon + render-IR multi-light
//! + expression morph + spring bones. Change the EDN → change the render.
//! `cargo run -p kami-live --example vrm_edn --target aarch64-apple-darwin`

#[path = "common/vrm.rs"]
mod vrm;

use glam::{Mat4, Vec3};
use kami_live::scene::DanceScene;
use vrm::{Globals, GpuBox, GpuLight, GpuParticle, GpuRenderer, MAX_LIGHTS, VrmDance};

const SCENE: &str = include_str!("../../kami-clj-play3d/games/dance/scene.edn");

/// A live effect particle (CPU sim): spawned by a `:fx` trigger, drawn additively.
struct P {
    pos: Vec3,
    vel: Vec3,
    color: [f32; 3],
    size: f32,
    age: f32,
    life: f32,
    grav: f32,
}

/// Per-fx burst signature tuned to the VRM's scale: (colour, count, speed, life,
/// gravity[+down], size). Mirrors the kami-live `:fx` vocabulary for the demo.
fn fx_params(fx: &str) -> Option<([f32; 3], usize, f32, f32, f32, f32)> {
    Some(match fx {
        "confetti" => ([1.0, 0.6, 0.2], 44, 2.2, 2.5, 1.2, 0.05),
        "fireworks" | "firework" => ([0.7, 0.85, 1.0], 80, 3.4, 2.2, 1.0, 0.06),
        "pyro" | "fire" | "flame" => ([1.0, 0.45, 0.12], 40, 2.6, 1.6, -0.8, 0.08),
        "sparkle" | "sparkles" | "sparkle-blast" | "glitter" => {
            ([1.0, 1.0, 0.7], 56, 2.0, 1.4, 0.2, 0.04)
        }
        "laser" | "laser-burst" => ([0.4, 1.0, 0.6], 30, 6.0, 0.9, 0.0, 0.03),
        "smoke" | "haze" => ([0.65, 0.65, 0.72], 22, 0.7, 3.0, -0.5, 0.16),
        "bubbles" => ([0.6, 0.85, 1.0], 28, 1.0, 2.6, -0.6, 0.07),
        "hearts" => ([1.0, 0.4, 0.6], 20, 1.0, 2.4, -0.4, 0.08),
        "stars" | "star-shower" => ([1.0, 0.95, 0.7], 32, 1.6, 2.2, 0.8, 0.05),
        "snow" => ([0.95, 0.97, 1.0], 44, 0.5, 4.0, 0.3, 0.05),
        "petals" | "sakura" => ([1.0, 0.7, 0.8], 32, 0.6, 3.5, 0.35, 0.06),
        "embers" => ([1.0, 0.5, 0.2], 28, 1.4, 2.8, -0.5, 0.04),
        _ => return None,
    })
}

/// Deterministic upward-biased unit direction (no RNG — varies by index).
fn hash_dir(i: usize) -> Vec3 {
    let h = |k: f32| {
        let x = (i as f32 * k).sin() * 43758.5453;
        x - x.floor()
    };
    let theta = h(12.9898) * std::f32::consts::TAU;
    let zz = h(78.233);
    let r = (1.0 - zz * zz).sqrt();
    Vec3::new(r * theta.cos(), zz * 0.9 + 0.2, r * theta.sin()).normalize_or_zero()
}

fn main() {
    pollster::block_on(run());
}

async fn run() {
    // clj/edn drives the render: VRM path + spring toggle + scale from :dance/avatar.
    let cfg = DanceScene::from_edn(SCENE).expect("scene");
    let av = &cfg.avatar;
    let edn_path = format!("kami-clj-play3d/games/dance/{}", av.vrm);
    let vrm_path = if std::path::Path::new(&edn_path).exists() {
        edn_path
    } else {
        "assets/Seed-san.vrm".to_string()
    };
    let spring_enabled = av.spring_bones;
    let avatar_scale = av.scale;
    println!(
        "EDN-driven :dance/avatar → vrm={:?} (→ {}), spring-bones={}, scale={}",
        av.vrm, vrm_path, spring_enabled, avatar_scale
    );

    let bytes = std::fs::read(&vrm_path)
        .expect("vrm asset (set :dance/avatar :vrm, or download assets/Seed-san.vrm)");
    let mut model = VrmDance::load(&bytes);
    println!(
        "loaded: {} verts, {} tris, {} morph-prims, {} spring chains",
        model.verts.len(),
        model.indices.len() / 3,
        model.morph_prims.len(),
        model.spring.chain_count()
    );

    let (w, h) = (420u32, 620u32);
    let r = GpuRenderer::new(&model, w, h).await;
    let proj = Mat4::perspective_rh(0.8, w as f32 / h as f32, 0.05, 100.0);
    // map the dance-world camera offsets (sized for a ~1.8 m performer) to the VRM.
    let ms = (model.height / 1.8 / avatar_scale.max(0.1)).max(0.1);

    let mut scene = DanceScene::from_edn(SCENE).unwrap();
    scene.show.start();

    // Render across the whole show so the :dance/camera :shots choreography
    // (wide → dolly-in → side → pull-back) plays out over the set.
    let spawn_base = Vec3::new(model.center.x, model.height * 0.62, model.center.z);
    let mut parts: Vec<P> = Vec::new();
    let mut gif = Vec::new();
    let (mut saved, mut tick) = (0usize, 0usize);
    while saved < 72 {
        let fr = scene.frame(1.0 / 60.0);
        tick += 1;
        let dt = 1.0 / 60.0;
        // spawn particles for every :fx that fired this tick (effect bursts).
        for a in &fr.actions {
            if let Some(fx) = a.action("fx") {
                if let Some((col, count, speed, life, grav, size)) = fx_params(&fx) {
                    for k in 0..count {
                        let dir = hash_dir(parts.len() + k);
                        parts.push(P {
                            pos: spawn_base,
                            vel: dir * (speed * ms),
                            color: col,
                            size: size * ms * 3.0,
                            age: 0.0,
                            life,
                            grav: grav * ms,
                        });
                    }
                }
            }
        }
        // advance + cull every tick so bursts animate smoothly between samples.
        for p in &mut parts {
            p.vel.y -= p.grav * dt;
            p.pos += p.vel * dt;
            p.age += dt;
        }
        parts.retain(|p| p.age < p.life);
        if parts.len() > 8000 {
            let cut = parts.len() - 8000;
            parts.drain(0..cut);
        }
        if tick % 75 != 0 {
            continue;
        } // sample ~0.67 bar of show time per frame
        let ir = kami_webgpu_rs::parse_render_ir(&fr.render_ir_edn());
        let mut lights = [GpuLight {
            dir: [0.0; 4],
            color: [0.0; 4],
        }; MAX_LIGHTS];
        let nl = ir.lights.len().min(MAX_LIGHTS);
        for (k, l) in ir.lights.iter().take(MAX_LIGHTS).enumerate() {
            lights[k] = GpuLight {
                dir: [l.dir[0], l.dir[1], l.dir[2], 0.0],
                color: [l.color[0], l.color[1], l.color[2], l.intensity.max(0.3)],
            };
        }
        let n_used = if nl == 0 {
            lights[0] = GpuLight {
                dir: [-0.3, -0.5, -0.75, 0.0],
                color: [1.0, 0.96, 0.85, 1.0],
            };
            1
        } else {
            nl
        };
        let amb = ir.env.ambient;
        let snap = scene.show.snapshot();
        let pose = snap.performer_pose;
        // camera: EDN :dance/camera :shots, dollied by the current bar, framing the VRM.
        let barf = snap.phase.bar as f32 + snap.phase.bar_frac;
        let (off, lk) = cfg.camera.framing_at(barf);
        let eye = Vec3::new(
            model.center.x + off.x * ms,
            off.y * ms,
            model.center.z + off.z * ms,
        );
        let target = Vec3::new(
            model.center.x + lk.x * ms,
            lk.y * ms,
            model.center.z + lk.z * ms,
        );
        let vp = (proj * Mat4::look_at_rh(eye, target, Vec3::Y)).to_cols_array_2d();
        // camera right/up for screen-facing particle billboards.
        let fwd = (target - eye).normalize_or_zero();
        let cr = fwd.cross(Vec3::Y).normalize_or_zero();
        let cu = cr.cross(fwd);
        // expression weights are authored in clj/edn (:dance/avatar :expressions).
        let mut expr = cfg.avatar.expression_weights(
            snap.cheer_loudness,
            snap.phase.beat_frac,
            snap.phase.time,
        );
        // :dance/avatar :voice vowel timeline drives the mouth (a-i-u-e-o).
        if let Some(voice) = &cfg.avatar.voice {
            let beat = snap.phase.beat as f32 + snap.phase.beat_frac;
            match voice.vowel_weight(beat) {
                Some((name, w)) => { expr.insert(name.to_string(), w); }
                None => { expr.insert("aa".to_string(), 0.0); }
            }
        }
        let (mv, palette) = model.frame(&pose, &expr, spring_enabled);
        let g = Globals {
            vp,
            ambient: [amb[0] * 0.45, amb[1] * 0.45, amb[2] * 0.5, 1.0],
            n_lights: [n_used as u32, 0, 0, 0],
            lights,
            cam_right: [cr.x, cr.y, cr.z, 0.0],
            cam_up: [cu.x, cu.y, cu.z, 0.0],
        };
        // build the GPU billboard list from live particles (fade by remaining life).
        let gparts: Vec<GpuParticle> = parts
            .iter()
            .map(|p| {
                let fade = (1.0 - p.age / p.life).clamp(0.0, 1.0);
                GpuParticle {
                    pos: [p.pos.x, p.pos.y, p.pos.z],
                    size: p.size,
                    color: [p.color[0] * fade, p.color[1] * fade, p.color[2] * fade],
                    _pad: 0.0,
                }
            })
            .collect();
        // render-IR :instances (crowd fans + stage set) as lit boxes around the VRM,
        // skipping [0] (the placeholder performer — the VRM mesh replaces it).
        let boxes: Vec<GpuBox> = ir
            .instances
            .iter()
            .skip(1)
            .map(|inst| GpuBox {
                pos: [
                    model.center.x + inst.pos[0] * ms,
                    inst.pos[1] * ms,
                    model.center.z + inst.pos[2] * ms,
                ],
                h: inst.size[1] * ms,
                color: [
                    inst.color[0] + inst.emissive,
                    inst.color[1] + inst.emissive,
                    inst.color[2] + inst.emissive,
                ],
                w: inst.size[0] * ms,
            })
            .collect();
        let px = r.render(&mv, &palette, g, &gparts, &boxes);
        if saved % 18 == 0 {
            image::save_buffer(
                format!("seededn_{saved:02}.png"),
                &px,
                w,
                h,
                image::ExtendedColorType::Rgba8,
            )
            .unwrap();
        }
        gif.push(image::Frame::from_parts(
            image::RgbaImage::from_raw(w, h, px).unwrap(),
            0,
            0,
            image::Delay::from_numer_denom_ms(60, 1),
        ));
        saved += 1;
    }
    let fl = std::fs::File::create("seed_edn.gif").unwrap();
    let mut e = image::codecs::gif::GifEncoder::new(fl);
    e.set_repeat(image::codecs::gif::Repeat::Infinite).unwrap();
    e.encode_frames(gif.into_iter()).unwrap();
    println!("wrote seed_edn.gif + seededn_*.png — clj/edn-driven VRM dance (via common/vrm.rs)");
}
