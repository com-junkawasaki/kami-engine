//! Miata-class roadster — full pipeline: SCAD primitives →
//! VehicleAssembly → JBeam JSON + CycloneDX 1.5 SBOM.
//!
//! ```bash
//! cargo run -p kami-cad-import --example scad_roadster
//! ```

use kami_cad_import::demos::roadster_na;
use kami_cad_import::{jbeam_emit, sbom};

fn main() {
    let asm = roadster_na();
    let jbeam = jbeam_emit::emit(&asm).expect("jbeam emit");
    let cdx = sbom::emit(&asm, &sbom::CycloneDxOptions::default()).expect("sbom emit");

    let jbeam_v: serde_json::Value = serde_json::from_str(&jbeam).unwrap();
    let cdx_v: serde_json::Value = serde_json::from_str(&cdx).unwrap();

    eprintln!(
        "[roadster] {} parts / {:.0} kg / {} hardpoints / {} jbeam-nodes / {} jbeam-beams / {} cdx-components",
        asm.parts.len(),
        asm.total_mass_kg(),
        asm.hardpoints.len(),
        jbeam_v["nodes"].as_array().unwrap().len(),
        jbeam_v["beams"].as_array().unwrap().len(),
        cdx_v["components"].as_array().unwrap().len(),
    );
    println!("==== JBEAM bytes={} ====", jbeam.len());
    println!("==== CYCLONEDX bytes={} ====", cdx.len());
}
