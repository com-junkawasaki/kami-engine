//! Procedural 3D mesh generators for Pokoa (ぽこあ) brainrot creatures.
//! Each function returns `(Vec<f32>, Vec<u32>)` in interleaved format: pos3 + norm3 + uv2 = 8 floats per vertex.
//!
//! Reuses shared primitives from `brainrot_mesh` — no local duplication.

use crate::brainrot_mesh::{
    capsule, character_mesh, cylinder_mesh as cylinder, merge_meshes, offset_mesh, rounded_box,
    scale_mesh, sphere_mesh as sphere,
};
use std::f32::consts::PI;

// =============================================================================
// Pokoa Creature Meshes
// =============================================================================

/// Toilettle: Tiny toilet creature with stubby legs and big eyes.
/// Toilet bowl body + tiny legs + eyeball spheres on top.
pub fn toilettle_mesh() -> (Vec<f32>, Vec<u32>) {
    let segs = 12;
    // Bowl body (squashed sphere)
    let mut body = sphere(segs, segs, 0.6, 0.0, 0.4, 0.0);
    scale_mesh(&mut body, 1.0, 0.7, 1.0);

    // Tank (small box on back)
    let mut tank = rounded_box(0.5, 0.6, 0.3, 0.05);
    offset_mesh(&mut tank, 0.0, 0.5, -0.5);

    // Left eye (big sphere)
    let left_eye = sphere(8, 8, 0.15, -0.2, 0.7, 0.35);
    // Right eye
    let right_eye = sphere(8, 8, 0.15, 0.2, 0.7, 0.35);
    // Pupils
    let left_pupil = sphere(6, 6, 0.07, -0.2, 0.72, 0.47);
    let right_pupil = sphere(6, 6, 0.07, 0.2, 0.72, 0.47);

    // Stubby legs
    let left_leg = cylinder(8, 0.1, 0.15, -0.25, 0.0, 0.0);
    let right_leg = cylinder(8, 0.1, 0.15, 0.25, 0.0, 0.0);

    // Lid (disc on top)
    let lid = sphere(6, segs, 0.5, 0.0, 0.55, 0.0);

    merge_meshes(&[
        body,
        tank,
        left_eye,
        right_eye,
        left_pupil,
        right_pupil,
        left_leg,
        right_leg,
        lid,
    ])
}

/// Skibidrain: Medium toilet creature with rotating head and arms.
pub fn skibidrain_mesh() -> (Vec<f32>, Vec<u32>) {
    let segs = 14;
    // Bowl body (larger)
    let mut body = sphere(segs, segs, 0.9, 0.0, 0.6, 0.0);
    scale_mesh(&mut body, 1.0, 0.75, 1.0);

    // Tank
    let mut tank = rounded_box(0.8, 0.9, 0.5, 0.08);
    offset_mesh(&mut tank, 0.0, 0.7, -0.7);

    // Head (separate sphere on top, "poking out")
    let head = sphere(segs, segs, 0.45, 0.0, 1.3, 0.1);

    // Eyes (wider, menacing)
    let left_eye = sphere(8, 8, 0.12, -0.2, 1.4, 0.45);
    let right_eye = sphere(8, 8, 0.12, 0.2, 1.4, 0.45);

    // Arms (cylinder)
    let mut left_arm = cylinder(8, 0.08, 0.3, 0.0, 0.0, 0.0);
    offset_mesh(&mut left_arm, -0.9, 0.6, 0.0);
    let mut right_arm = cylinder(8, 0.08, 0.3, 0.0, 0.0, 0.0);
    offset_mesh(&mut right_arm, 0.9, 0.6, 0.0);

    // Legs
    let left_leg = cylinder(8, 0.12, 0.2, -0.35, 0.0, 0.0);
    let right_leg = cylinder(8, 0.12, 0.2, 0.35, 0.0, 0.0);

    merge_meshes(&[
        body, tank, head, left_eye, right_eye, left_arm, right_arm, left_leg, right_leg,
    ])
}

/// MegaSkibidi: Giant toilet boss with massive head and laser eyes.
pub fn mega_skibidi_mesh() -> (Vec<f32>, Vec<u32>) {
    let segs = 16;
    // Massive bowl
    let mut body = sphere(segs, segs, 1.5, 0.0, 1.0, 0.0);
    scale_mesh(&mut body, 1.0, 0.7, 1.0);

    // Giant tank
    let mut tank = rounded_box(1.5, 1.8, 0.8, 0.1);
    offset_mesh(&mut tank, 0.0, 1.2, -1.3);

    // Massive head
    let head = sphere(segs, segs, 0.8, 0.0, 2.5, 0.2);

    // Laser eyes (elongated cylinders pointing forward)
    let mut left_eye = cylinder(8, 0.08, 0.4, 0.0, 0.0, 0.0);
    offset_mesh(&mut left_eye, -0.35, 2.6, 0.9);
    let mut right_eye = cylinder(8, 0.08, 0.4, 0.0, 0.0, 0.0);
    offset_mesh(&mut right_eye, 0.35, 2.6, 0.9);

    // Crown spikes
    let spike1 = sphere(6, 6, 0.15, 0.0, 3.3, 0.2);
    let spike2 = sphere(6, 6, 0.12, -0.3, 3.1, 0.1);
    let spike3 = sphere(6, 6, 0.12, 0.3, 3.1, 0.1);

    // Arms (thick)
    let mut left_arm = cylinder(10, 0.15, 0.5, 0.0, 0.0, 0.0);
    offset_mesh(&mut left_arm, -1.5, 1.0, 0.0);
    let mut right_arm = cylinder(10, 0.15, 0.5, 0.0, 0.0, 0.0);
    offset_mesh(&mut right_arm, 1.5, 1.0, 0.0);

    // Fist spheres
    let left_fist = sphere(8, 8, 0.2, -1.5, 0.5, 0.0);
    let right_fist = sphere(8, 8, 0.2, 1.5, 0.5, 0.0);

    // Legs
    let left_leg = cylinder(8, 0.18, 0.35, -0.6, 0.0, 0.0);
    let right_leg = cylinder(8, 0.18, 0.35, 0.6, 0.0, 0.0);

    merge_meshes(&[
        body, tank, head, left_eye, right_eye, spike1, spike2, spike3, left_arm, right_arm,
        left_fist, right_fist, left_leg, right_leg,
    ])
}

/// Sigpup: Small electric puppy with spiky fur and determined eyes.
pub fn sigpup_mesh() -> (Vec<f32>, Vec<u32>) {
    let segs = 10;
    // Body (slightly elongated capsule)
    let mut body = capsule(0.25, 0.2, segs);
    offset_mesh(&mut body, 0.0, 0.35, 0.0);

    // Head (sphere)
    let head = sphere(segs, segs, 0.22, 0.0, 0.7, 0.15);

    // Ears (small cones approximated as thin cylinders)
    let mut left_ear = cylinder(6, 0.06, 0.15, 0.0, 0.0, 0.0);
    offset_mesh(&mut left_ear, -0.12, 0.95, 0.1);
    let mut right_ear = cylinder(6, 0.06, 0.15, 0.0, 0.0, 0.0);
    offset_mesh(&mut right_ear, 0.12, 0.95, 0.1);

    // Tail (lightning bolt shape approximated as angled cylinder)
    let mut tail = cylinder(6, 0.04, 0.2, 0.0, 0.0, 0.0);
    offset_mesh(&mut tail, 0.0, 0.4, -0.35);

    // 4 legs
    let fl = cylinder(6, 0.06, 0.12, -0.15, 0.0, 0.15);
    let fr = cylinder(6, 0.06, 0.12, 0.15, 0.0, 0.15);
    let bl = cylinder(6, 0.06, 0.12, -0.15, 0.0, -0.15);
    let br = cylinder(6, 0.06, 0.12, 0.15, 0.0, -0.15);

    // Nose
    let nose = sphere(6, 6, 0.04, 0.0, 0.68, 0.35);

    merge_meshes(&[body, head, left_ear, right_ear, tail, fl, fr, bl, br, nose])
}

/// Sigmachu: Electric fighting mouse with buff arms and sunglasses.
pub fn sigmachu_mesh() -> (Vec<f32>, Vec<u32>) {
    let segs = 12;
    // Muscular body
    let mut body = capsule(0.3, 0.3, segs);
    offset_mesh(&mut body, 0.0, 0.5, 0.0);

    // Head
    let head = sphere(segs, segs, 0.25, 0.0, 1.0, 0.1);

    // Big ears (lightning bolt ears)
    let mut left_ear = cylinder(6, 0.05, 0.25, 0.0, 0.0, 0.0);
    offset_mesh(&mut left_ear, -0.15, 1.3, 0.0);
    let mut right_ear = cylinder(6, 0.05, 0.25, 0.0, 0.0, 0.0);
    offset_mesh(&mut right_ear, 0.15, 1.3, 0.0);

    // Sunglasses (thin box across eyes)
    let mut glasses = rounded_box(0.4, 0.08, 0.05, 0.01);
    offset_mesh(&mut glasses, 0.0, 1.05, 0.3);

    // Buff arms
    let mut left_arm = capsule(0.1, 0.2, 8);
    offset_mesh(&mut left_arm, -0.45, 0.6, 0.0);
    let mut right_arm = capsule(0.1, 0.2, 8);
    offset_mesh(&mut right_arm, 0.45, 0.6, 0.0);

    // Fists
    let left_fist = sphere(8, 8, 0.1, -0.45, 0.35, 0.0);
    let right_fist = sphere(8, 8, 0.1, 0.45, 0.35, 0.0);

    // Legs
    let left_leg = cylinder(8, 0.1, 0.18, -0.15, 0.0, 0.0);
    let right_leg = cylinder(8, 0.1, 0.18, 0.15, 0.0, 0.0);

    // Tail (zigzag approximated)
    let mut tail = cylinder(6, 0.04, 0.3, 0.0, 0.0, 0.0);
    offset_mesh(&mut tail, 0.0, 0.5, -0.4);

    merge_meshes(&[
        body, head, left_ear, right_ear, glasses, left_arm, right_arm, left_fist, right_fist,
        left_leg, right_leg, tail,
    ])
}

/// Gigachad: Ultimate sigma evolution — massive jaw, huge muscles, golden aura.
pub fn gigachad_mesh() -> (Vec<f32>, Vec<u32>) {
    let segs = 14;
    // Massive torso
    let mut body = capsule(0.5, 0.4, segs);
    offset_mesh(&mut body, 0.0, 0.8, 0.0);

    // Head with prominent jaw
    let head = sphere(segs, segs, 0.3, 0.0, 1.5, 0.1);
    // Jaw (box protruding forward)
    let mut jaw = rounded_box(0.25, 0.12, 0.2, 0.03);
    offset_mesh(&mut jaw, 0.0, 1.3, 0.3);

    // Crown (golden orb on head)
    let crown = sphere(8, 8, 0.1, 0.0, 1.85, 0.0);

    // Massive arms
    let mut left_arm = capsule(0.15, 0.3, 10);
    offset_mesh(&mut left_arm, -0.7, 0.9, 0.0);
    let mut right_arm = capsule(0.15, 0.3, 10);
    offset_mesh(&mut right_arm, 0.7, 0.9, 0.0);

    // Fists
    let left_fist = sphere(10, 10, 0.15, -0.7, 0.5, 0.0);
    let right_fist = sphere(10, 10, 0.15, 0.7, 0.5, 0.0);

    // Legs (thick)
    let left_leg = cylinder(10, 0.15, 0.3, -0.25, 0.0, 0.0);
    let right_leg = cylinder(10, 0.15, 0.3, 0.25, 0.0, 0.0);

    // Aura ring (torus approximated as a ring of small spheres)
    let mut aura_parts: Vec<(Vec<f32>, Vec<u32>)> = Vec::new();
    for i in 0..8 {
        let angle = (i as f32 / 8.0) * 2.0 * PI;
        let x = angle.cos() * 0.8;
        let z = angle.sin() * 0.8;
        aura_parts.push(sphere(4, 4, 0.06, x, 1.0, z));
    }

    let mut all = vec![
        body, head, jaw, crown, left_arm, right_arm, left_fist, right_fist, left_leg, right_leg,
    ];
    all.extend(aura_parts);
    merge_meshes(&all)
}

/// Ohiolet: Glitchy ghost creature — distorted body with floating cube fragments.
pub fn ohiolet_mesh(glitch_phase: f32) -> (Vec<f32>, Vec<u32>) {
    let segs = 10;
    let phase = glitch_phase * 2.0 * PI;

    // Main body (wobbly sphere)
    let mut body_verts = Vec::new();
    let mut body_idxs = Vec::new();
    for i in 0..=segs {
        let phi = PI * i as f32 / segs as f32;
        let y = phi.cos();
        let r = phi.sin();
        for j in 0..=segs {
            let theta = 2.0 * PI * j as f32 / segs as f32;
            let nx = r * theta.cos();
            let nz = r * theta.sin();
            let ny = y;
            let glitch = 0.1 * ((3.0 * phi + phase).sin() * (2.0 * theta + phase * 1.7).cos());
            let radius = 0.4 + glitch;
            body_verts.extend_from_slice(&[
                nx * radius,
                ny * radius + 0.5,
                nz * radius,
                nx,
                ny,
                nz,
                j as f32 / segs as f32,
                i as f32 / segs as f32,
            ]);
        }
    }
    let ring = segs + 1;
    for i in 0..segs {
        for j in 0..segs {
            let a = i * ring + j;
            let b = a + ring;
            body_idxs.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    let body = (body_verts, body_idxs);

    // Floating eye
    let eye = sphere(8, 8, 0.12, 0.0, 0.7, 0.35);

    // Glitch fragments (small cubes floating around)
    let frag1 = rounded_box(0.12, 0.12, 0.12, 0.02);
    let mut frag1 = frag1;
    offset_mesh(&mut frag1, 0.5 + phase.sin() * 0.1, 0.6, 0.3);

    let frag2 = rounded_box(0.08, 0.08, 0.08, 0.01);
    let mut frag2 = frag2;
    offset_mesh(&mut frag2, -0.4 + phase.cos() * 0.1, 0.8, -0.2);

    let frag3 = rounded_box(0.1, 0.1, 0.1, 0.02);
    let mut frag3 = frag3;
    offset_mesh(&mut frag3, 0.0, 1.0 + phase.sin() * 0.15, 0.0);

    merge_meshes(&[body, eye, frag1, frag2, frag3])
}

/// Ohiodon: The Ohio Final Boss — massive distorted creature with reality-warping aura.
pub fn ohiodon_mesh(warp_phase: f32) -> (Vec<f32>, Vec<u32>) {
    let segs = 14;
    let phase = warp_phase * 2.0 * PI;

    // Massive warped body
    let mut body_verts = Vec::new();
    let mut body_idxs = Vec::new();
    for i in 0..=segs {
        let phi = PI * i as f32 / segs as f32;
        let y = phi.cos();
        let r = phi.sin();
        for j in 0..=segs {
            let theta = 2.0 * PI * j as f32 / segs as f32;
            let nx = r * theta.cos();
            let nz = r * theta.sin();
            let ny = y;
            let warp = 0.2
                * ((4.0 * phi + phase).sin() * 0.5
                    + (3.0 * theta + phase * 0.8).sin() * 0.3
                    + (5.0 * phi + 2.0 * theta).cos() * 0.2);
            let radius = 0.8 + warp;
            body_verts.extend_from_slice(&[
                nx * radius,
                ny * radius + 1.0,
                nz * radius,
                nx,
                ny,
                nz,
                j as f32 / segs as f32,
                i as f32 / segs as f32,
            ]);
        }
    }
    let ring = segs + 1;
    for i in 0..segs {
        for j in 0..segs {
            let a = i * ring + j;
            let b = a + ring;
            body_idxs.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    let body = (body_verts, body_idxs);

    // Three floating eyes
    let eye1 = sphere(8, 8, 0.15, -0.3, 1.3, 0.6);
    let eye2 = sphere(8, 8, 0.15, 0.3, 1.3, 0.6);
    let eye3 = sphere(8, 8, 0.1, 0.0, 1.6, 0.5);

    // Reality fragments (larger, orbiting)
    let mut fragments: Vec<(Vec<f32>, Vec<u32>)> = Vec::new();
    for i in 0..6 {
        let angle = (i as f32 / 6.0) * 2.0 * PI + phase;
        let r = 1.2;
        let x = angle.cos() * r;
        let z = angle.sin() * r;
        let y = 1.0 + (angle * 2.0 + phase).sin() * 0.3;
        let size = 0.08 + (i as f32 * 0.02);
        let mut frag = rounded_box(size, size, size, 0.01);
        offset_mesh(&mut frag, x, y, z);
        fragments.push(frag);
    }

    // Horns
    let mut left_horn = cylinder(6, 0.06, 0.3, 0.0, 0.0, 0.0);
    offset_mesh(&mut left_horn, -0.4, 1.8, 0.0);
    let mut right_horn = cylinder(6, 0.06, 0.3, 0.0, 0.0, 0.0);
    offset_mesh(&mut right_horn, 0.4, 1.8, 0.0);

    let mut all = vec![body, eye1, eye2, eye3, left_horn, right_horn];
    all.extend(fragments);
    merge_meshes(&all)
}

/// Grimini: Cute baby blob — small purple ball with a smile.
pub fn grimini_mesh(wobble: f32) -> (Vec<f32>, Vec<u32>) {
    let stacks = 14;
    let slices = 14;
    let phase = wobble * 2.0 * PI;

    // Wobbly body
    let mut verts = Vec::new();
    let mut idxs = Vec::new();
    for i in 0..=stacks {
        let phi = PI * i as f32 / stacks as f32;
        let y = phi.cos();
        let r = phi.sin();
        for j in 0..=slices {
            let theta = 2.0 * PI * j as f32 / slices as f32;
            let nx = r * theta.cos();
            let nz = r * theta.sin();
            let ny = y;
            let wobble_disp = 0.08 * (3.0 * phi + phase).sin() * (2.0 * theta).cos();
            let radius = 0.5 + wobble_disp;
            verts.extend_from_slice(&[
                nx * radius,
                ny * radius + 0.5,
                nz * radius,
                nx,
                ny,
                nz,
                j as f32 / slices as f32,
                i as f32 / stacks as f32,
            ]);
        }
    }
    let ring = slices + 1;
    for i in 0..stacks {
        for j in 0..slices {
            let a = i * ring + j;
            let b = a + ring;
            idxs.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    let body = (verts, idxs);

    // Cute eyes
    let left_eye = sphere(6, 6, 0.08, -0.15, 0.65, 0.4);
    let right_eye = sphere(6, 6, 0.08, 0.15, 0.65, 0.4);

    // Smile (tiny sphere below eyes)
    let smile = sphere(4, 4, 0.04, 0.0, 0.5, 0.45);

    merge_meshes(&[body, left_eye, right_eye, smile])
}

/// Grimaceon: Full Grimace evolution — massive purple blob with toxic puddle base.
pub fn grimaceon_mesh(wobble: f32) -> (Vec<f32>, Vec<u32>) {
    let stacks = 18;
    let slices = 18;
    let phase = wobble * 2.0 * PI;

    // Large wobbly body
    let mut verts = Vec::new();
    let mut idxs = Vec::new();
    for i in 0..=stacks {
        let phi = PI * i as f32 / stacks as f32;
        let y = phi.cos();
        let r = phi.sin();
        for j in 0..=slices {
            let theta = 2.0 * PI * j as f32 / slices as f32;
            let nx = r * theta.cos();
            let nz = r * theta.sin();
            let ny = y;
            let wobble_disp = 0.15
                * ((3.0 * phi + phase).sin() * 0.5
                    + (2.0 * theta + phase * 1.3).sin() * 0.3
                    + (4.0 * phi + 3.0 * theta).sin() * 0.2);
            let radius = 1.0 + wobble_disp;
            verts.extend_from_slice(&[
                nx * radius,
                ny * radius + 1.0,
                nz * radius,
                nx,
                ny,
                nz,
                j as f32 / slices as f32,
                i as f32 / stacks as f32,
            ]);
        }
    }
    let ring = slices + 1;
    for i in 0..stacks {
        for j in 0..slices {
            let a = i * ring + j;
            let b = a + ring;
            idxs.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    let body = (verts, idxs);

    // Eyes (wide, happy)
    let left_eye = sphere(8, 8, 0.15, -0.35, 1.3, 0.75);
    let right_eye = sphere(8, 8, 0.15, 0.35, 1.3, 0.75);

    // Big smile
    let smile = sphere(6, 6, 0.08, 0.0, 1.0, 0.9);

    // Puddle base (flat disc)
    let mut puddle_verts = Vec::new();
    let mut puddle_idxs = Vec::new();
    let puddle_segs = 16u32;
    puddle_verts.extend_from_slice(&[0.0, 0.02, 0.0, 0.0, 1.0, 0.0, 0.5, 0.5]);
    for j in 0..=puddle_segs {
        let theta = 2.0 * PI * j as f32 / puddle_segs as f32;
        let x = theta.cos() * 1.5;
        let z = theta.sin() * 1.5;
        puddle_verts.extend_from_slice(&[
            x,
            0.02,
            z,
            0.0,
            1.0,
            0.0,
            0.5 + 0.5 * theta.cos(),
            0.5 + 0.5 * theta.sin(),
        ]);
    }
    for j in 0..puddle_segs {
        puddle_idxs.extend_from_slice(&[0, 1 + j, 2 + j]);
    }
    let puddle = (puddle_verts, puddle_idxs);

    merge_meshes(&[body, left_eye, right_eye, smile, puddle])
}

/// Rizzlord: Legendary fire/psychic — elegant humanoid with flame crown and charm aura.
pub fn rizzlord_mesh() -> (Vec<f32>, Vec<u32>) {
    let segs = 14;
    // Elegant body
    let body = character_mesh("slim", 1.2);

    // Flame crown (5 flame-like spheres on head)
    let flame1 = sphere(8, 8, 0.12, 0.0, 1.7, 0.0);
    let flame2 = sphere(6, 6, 0.1, -0.15, 1.6, 0.05);
    let flame3 = sphere(6, 6, 0.1, 0.15, 1.6, 0.05);
    let flame4 = sphere(6, 6, 0.08, -0.08, 1.75, -0.05);
    let flame5 = sphere(6, 6, 0.08, 0.08, 1.75, -0.05);

    // Charm hearts (small orbs floating around)
    let heart1 = sphere(6, 6, 0.06, 0.6, 1.0, 0.3);
    let heart2 = sphere(6, 6, 0.06, -0.6, 1.0, 0.3);
    let heart3 = sphere(6, 6, 0.06, 0.0, 1.2, -0.5);

    // Cape (large flat box behind body)
    let mut cape = rounded_box(0.8, 1.0, 0.05, 0.02);
    offset_mesh(&mut cape, 0.0, 0.5, -0.4);

    let _ = segs;
    merge_meshes(&[
        body, flame1, flame2, flame3, flame4, flame5, heart1, heart2, heart3, cape,
    ])
}

/// Fanumoth: Legendary steel/normal — bulky collector with metal armor plates.
pub fn fanumoth_mesh() -> (Vec<f32>, Vec<u32>) {
    // Heavy armored body
    let body = character_mesh("stocky", 1.15);

    // Armor plates (boxes attached to body)
    let mut chest_plate = rounded_box(0.5, 0.4, 0.15, 0.04);
    offset_mesh(&mut chest_plate, 0.0, 0.3, 0.3);

    let mut left_shoulder = rounded_box(0.25, 0.15, 0.2, 0.03);
    offset_mesh(&mut left_shoulder, -0.5, 0.5, 0.0);
    let mut right_shoulder = rounded_box(0.25, 0.15, 0.2, 0.03);
    offset_mesh(&mut right_shoulder, 0.5, 0.5, 0.0);

    // Tax collector bag (rounded box)
    let mut bag = rounded_box(0.35, 0.4, 0.25, 0.05);
    offset_mesh(&mut bag, 0.55, 0.1, -0.1);

    // Crown (tax authority)
    let crown = sphere(8, 8, 0.12, 0.0, 1.2, 0.0);

    // Grabber claws (on each hand)
    let mut left_claw = rounded_box(0.1, 0.2, 0.05, 0.01);
    offset_mesh(&mut left_claw, -0.55, -0.1, 0.1);
    let mut right_claw = rounded_box(0.1, 0.2, 0.05, 0.01);
    offset_mesh(&mut right_claw, 0.55, -0.1, 0.1);

    merge_meshes(&[
        body,
        chest_plate,
        left_shoulder,
        right_shoulder,
        bag,
        crown,
        left_claw,
        right_claw,
    ])
}

/// Pokoa Ball: Catch ball item mesh — sphere with horizontal line.
pub fn pokoa_ball_mesh() -> (Vec<f32>, Vec<u32>) {
    let ball = sphere(12, 12, 0.3, 0.0, 0.3, 0.0);
    // Button on front
    let button = sphere(6, 6, 0.06, 0.0, 0.3, 0.28);
    // Dividing line (thin disc)
    let mut line_verts = Vec::new();
    let mut line_idxs = Vec::new();
    let line_segs = 16u32;
    line_verts.extend_from_slice(&[0.0, 0.3, 0.0, 0.0, 1.0, 0.0, 0.5, 0.5]);
    for j in 0..=line_segs {
        let theta = 2.0 * PI * j as f32 / line_segs as f32;
        let x = theta.cos() * 0.31;
        let z = theta.sin() * 0.31;
        line_verts.extend_from_slice(&[
            x,
            0.3,
            z,
            0.0,
            1.0,
            0.0,
            0.5 + theta.cos() * 0.5,
            0.5 + theta.sin() * 0.5,
        ]);
    }
    for j in 0..line_segs {
        line_idxs.extend_from_slice(&[0, 1 + j, 2 + j]);
    }
    let line = (line_verts, line_idxs);

    merge_meshes(&[ball, button, line])
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn validate_mesh(name: &str, verts: &[f32], idxs: &[u32]) {
        assert!(!verts.is_empty(), "{name}: vertices empty");
        assert!(!idxs.is_empty(), "{name}: indices empty");
        assert_eq!(
            verts.len() % 8,
            0,
            "{name}: vertex count not divisible by 8 (got {})",
            verts.len()
        );
        let max_vertex = verts.len() as u32 / 8;
        for (i, &idx) in idxs.iter().enumerate() {
            assert!(
                idx < max_vertex,
                "{name}: index {i} = {idx} out of range (max {max_vertex})"
            );
        }
    }

    #[test]
    fn toilettle_mesh_valid() {
        let (v, i) = toilettle_mesh();
        validate_mesh("toilettle", &v, &i);
    }

    #[test]
    fn skibidrain_mesh_valid() {
        let (v, i) = skibidrain_mesh();
        validate_mesh("skibidrain", &v, &i);
    }

    #[test]
    fn mega_skibidi_mesh_valid() {
        let (v, i) = mega_skibidi_mesh();
        validate_mesh("mega_skibidi", &v, &i);
    }

    #[test]
    fn sigpup_mesh_valid() {
        let (v, i) = sigpup_mesh();
        validate_mesh("sigpup", &v, &i);
    }

    #[test]
    fn sigmachu_mesh_valid() {
        let (v, i) = sigmachu_mesh();
        validate_mesh("sigmachu", &v, &i);
    }

    #[test]
    fn gigachad_mesh_valid() {
        let (v, i) = gigachad_mesh();
        validate_mesh("gigachad", &v, &i);
    }

    #[test]
    fn ohiolet_mesh_valid() {
        for phase in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let (v, i) = ohiolet_mesh(phase);
            validate_mesh(&format!("ohiolet({phase})"), &v, &i);
        }
    }

    #[test]
    fn ohiodon_mesh_valid() {
        for phase in [0.0, 0.5, 1.0] {
            let (v, i) = ohiodon_mesh(phase);
            validate_mesh(&format!("ohiodon({phase})"), &v, &i);
        }
    }

    #[test]
    fn grimini_mesh_valid() {
        for wobble in [0.0, 0.5, 1.0] {
            let (v, i) = grimini_mesh(wobble);
            validate_mesh(&format!("grimini({wobble})"), &v, &i);
        }
    }

    #[test]
    fn grimaceon_mesh_valid() {
        let (v, i) = grimaceon_mesh(0.0);
        validate_mesh("grimaceon", &v, &i);
    }

    #[test]
    fn rizzlord_mesh_valid() {
        let (v, i) = rizzlord_mesh();
        validate_mesh("rizzlord", &v, &i);
    }

    #[test]
    fn fanumoth_mesh_valid() {
        let (v, i) = fanumoth_mesh();
        validate_mesh("fanumoth", &v, &i);
    }

    #[test]
    fn pokoa_ball_mesh_valid() {
        let (v, i) = pokoa_ball_mesh();
        validate_mesh("pokoa_ball", &v, &i);
    }

    #[test]
    fn all_pokoa_meshes_have_substantial_geometry() {
        let meshes: Vec<(&str, (Vec<f32>, Vec<u32>))> = vec![
            ("toilettle", toilettle_mesh()),
            ("skibidrain", skibidrain_mesh()),
            ("mega_skibidi", mega_skibidi_mesh()),
            ("sigpup", sigpup_mesh()),
            ("sigmachu", sigmachu_mesh()),
            ("gigachad", gigachad_mesh()),
            ("ohiolet", ohiolet_mesh(0.0)),
            ("ohiodon", ohiodon_mesh(0.0)),
            ("grimini", grimini_mesh(0.0)),
            ("grimaceon", grimaceon_mesh(0.0)),
            ("rizzlord", rizzlord_mesh()),
            ("fanumoth", fanumoth_mesh()),
        ];
        for (name, (v, i)) in &meshes {
            let vert_count = v.len() / 8;
            let tri_count = i.len() / 3;
            assert!(vert_count > 50, "{name}: too few vertices ({vert_count})");
            assert!(tri_count > 30, "{name}: too few triangles ({tri_count})");
        }
    }
}
