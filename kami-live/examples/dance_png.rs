//! Render the reference dance scene to a PNG sequence with the native executor.
//! `cargo run -p kami-live --example dance_png --target aarch64-apple-darwin`
//!
//! Uses a FIXED stage camera (overriding the performer-tracking one) so the
//! performer's **shuffle side-step** is visible as it slides left↔right within
//! the frame. Each frame: `:dance/*` EDN → render-IR → `kami_webgpu_rs::render`
//! (offscreen PBR + shadows). The performer is a lit placeholder cuboid; the
//! skinned VRM mesh replaces it once the GPU executor consumes `:meshes`.

use kami_live::scene::DanceScene;

const SCENE: &str = include_str!("../../kami-clj-play3d/games/dance/scene.edn");

fn main() {
    let mut scene = DanceScene::from_edn(SCENE).expect("reference scene loads");
    scene.show.start();
    let (w, h) = (640u32, 400u32);
    let dt = 1.0 / 60.0;

    // Advance ~31s to the Verse (shuffle): the performer side-steps ±0.4 units.
    for _ in 0..(31.0 / dt) as i32 {
        scene.frame(dt);
    }
    for i in 0..12 {
        for _ in 0..7 {
            scene.frame(dt);
        }
        let f = scene.frame(dt);
        let (mut g, insts) = kami_webgpu_rs::parse_ir(&f.render_ir_edn());
        // FIXED stage camera so the side-step shows (don't track the performer).
        g.eye = Some([0.0, 2.5, 9.0]);
        g.target = Some([0.0, 1.0, 0.0]);
        let px = kami_webgpu_rs::render(&g, &insts, w, h);
        let name = format!("dance_{i:02}.png");
        image::save_buffer(&name, &px, w, h, image::ExtendedColorType::Rgba8).unwrap();
        let ph = scene.show.grid().phase();
        println!("wrote {name} — bar-frac {:.2}", ph.bar_frac);
    }
    println!("done — fixed camera; performer slides left↔right across the bar");
}
