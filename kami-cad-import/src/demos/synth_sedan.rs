//! 9-part synthetic sedan — the original Phase 1 reference assembly,
//! kept around as a tiny smoke-test that exercises every CDX field
//! including supplier MPNs and the bidirectional dependency edges.

use crate::part::{
    Hardpoint, HardpointKind, Material, PartKind, ProvenanceSource, Supplier, VehicleAssembly,
    VehiclePart,
};

pub fn synth_sedan() -> VehicleAssembly {
    let scad_sha = "0".repeat(64);
    let prov = |uri: &str| ProvenanceSource {
        uri: uri.into(),
        sha256: scad_sha.clone(),
        license: "MIT".into(),
    };
    let supplier_gftd = || Supplier {
        name: "gftd".into(),
        cpe: String::new(),
        mpn: String::new(),
    };

    let mut a = VehicleAssembly::new("synth-sedan-na", prov("scad://synth-sedan/v1.0.0"));
    a.display_name = "GFTD Synth Sedan NA".into();
    a.revision = "1.0.0".into();

    a.add_part(VehiclePart {
        id: "chassis".into(),
        display_name: "Chassis main rail + floor pan".into(),
        kind: PartKind::Chassis,
        material: Material::SteelHss,
        aabb_min: [-0.85, 0.20, -2.20],
        aabb_max: [0.85, 0.55, 2.20],
        mass_kg: Some(220.0),
        parent: None,
        break_group: None,
        source: prov("scad://synth-sedan/chassis.scad"),
        supplier: supplier_gftd(),
        revision: "1.0.0".into(),
    });
    a.add_part(VehiclePart {
        id: "hood".into(),
        display_name: "Hood (aluminium sheet)".into(),
        kind: PartKind::Body,
        material: Material::AluminiumSheet,
        aabb_min: [-0.80, 0.70, 1.20],
        aabb_max: [0.80, 0.78, 2.10],
        mass_kg: Some(11.0),
        parent: Some("chassis".into()),
        break_group: None,
        source: prov("scad://synth-sedan/hood.scad"),
        supplier: supplier_gftd(),
        revision: "1.0.0".into(),
    });
    a.add_part(VehiclePart {
        id: "trunk".into(),
        display_name: "Trunk lid".into(),
        kind: PartKind::Body,
        material: Material::SteelMild,
        aabb_min: [-0.80, 0.85, -2.10],
        aabb_max: [0.80, 0.93, -1.20],
        mass_kg: Some(14.0),
        parent: Some("chassis".into()),
        break_group: None,
        source: prov("scad://synth-sedan/trunk.scad"),
        supplier: supplier_gftd(),
        revision: "1.0.0".into(),
    });
    a.add_part(VehiclePart {
        id: "windshield".into(),
        display_name: "Windshield (laminated glass)".into(),
        kind: PartKind::Window,
        material: Material::Glass,
        aabb_min: [-0.78, 0.95, 0.80],
        aabb_max: [0.78, 1.45, 1.20],
        mass_kg: Some(15.0),
        parent: Some("chassis".into()),
        break_group: None,
        source: prov("scad://synth-sedan/windshield.scad"),
        supplier: Supplier {
            name: "AGC".into(),
            cpe: String::new(),
            mpn: "AGC-WSH-1989-NA".into(),
        },
        revision: "1.0.0".into(),
    });
    a.add_part(VehiclePart {
        id: "engine".into(),
        display_name: "1.6L NA engine block".into(),
        kind: PartKind::Powertrain,
        material: Material::AluminiumCast,
        aabb_min: [-0.30, 0.45, 1.30],
        aabb_max: [0.30, 0.85, 1.95],
        mass_kg: Some(115.0),
        parent: Some("chassis".into()),
        break_group: None,
        source: prov("scad://synth-sedan/engine.scad"),
        supplier: Supplier {
            name: "Mazda".into(),
            cpe: String::new(),
            mpn: "B6ZE-RS".into(),
        },
        revision: "1.0.0".into(),
    });
    for (i, (id, x, z)) in [
        ("wheel_fl", -0.78, 1.30),
        ("wheel_fr", 0.78, 1.30),
        ("wheel_rl", -0.78, -1.30),
        ("wheel_rr", 0.78, -1.30),
    ]
    .iter()
    .enumerate()
    {
        a.add_part(VehiclePart {
            id: id.to_string(),
            display_name: format!("Wheel #{}", i + 1),
            kind: PartKind::Wheel,
            material: Material::Rubber,
            aabb_min: [x - 0.12, 0.0, z - 0.32],
            aabb_max: [x + 0.12, 0.64, z + 0.32],
            mass_kg: Some(18.0),
            parent: Some("chassis".into()),
            break_group: None,
            source: prov("scad://synth-sedan/wheel.scad"),
            supplier: Supplier {
                name: "Bridgestone".into(),
                cpe: String::new(),
                mpn: "ER300-185-60-R14".into(),
            },
            revision: "1.0.0".into(),
        });
    }
    a.add_hardpoint(Hardpoint {
        id: "hp_hood".into(),
        from_part: "chassis".into(),
        to_part: "hood".into(),
        position: [0.0, 0.70, 1.80],
        kind: HardpointKind::Hinge,
    });
    a.add_hardpoint(Hardpoint {
        id: "hp_trunk".into(),
        from_part: "chassis".into(),
        to_part: "trunk".into(),
        position: [0.0, 0.85, -1.50],
        kind: HardpointKind::Hinge,
    });
    a.add_hardpoint(Hardpoint {
        id: "hp_windshield".into(),
        from_part: "chassis".into(),
        to_part: "windshield".into(),
        position: [0.0, 0.95, 1.00],
        kind: HardpointKind::Adhesive,
    });
    a.add_hardpoint(Hardpoint {
        id: "hp_engine_mount_l".into(),
        from_part: "chassis".into(),
        to_part: "engine".into(),
        position: [-0.30, 0.45, 1.50],
        kind: HardpointKind::Bolt,
    });
    a.add_hardpoint(Hardpoint {
        id: "hp_engine_mount_r".into(),
        from_part: "chassis".into(),
        to_part: "engine".into(),
        position: [0.30, 0.45, 1.50],
        kind: HardpointKind::Bolt,
    });
    for w in ["wheel_fl", "wheel_fr", "wheel_rl", "wheel_rr"] {
        a.add_hardpoint(Hardpoint {
            id: format!("hp_{w}"),
            from_part: "chassis".into(),
            to_part: w.into(),
            position: [0.0, 0.30, 0.0],
            kind: HardpointKind::Bolt,
        });
    }
    a
}
