//! 9-part synth sedan PoC — emits both JBeam and CycloneDX 1.5 SBOM.
//!
//! ```bash
//! cargo run -p kami-cad-import --example synth_sedan
//! ```

use kami_cad_import::demos::synth_sedan;
use kami_cad_import::{jbeam_emit, sbom};

fn main() {
    let asm = synth_sedan();
    let jbeam = jbeam_emit::emit(&asm).expect("jbeam emit");
    let cdx = sbom::emit(&asm, &sbom::CycloneDxOptions::default()).expect("sbom emit");

    println!("==== JBEAM ====");
    println!("{jbeam}");
    println!("\n==== CYCLONEDX 1.5 ====");
    println!("{cdx}");

    eprintln!(
        "\n[ok] {} parts / {:.0} kg total / {} hardpoints",
        asm.parts.len(),
        asm.total_mass_kg(),
        asm.hardpoints.len()
    );
}
