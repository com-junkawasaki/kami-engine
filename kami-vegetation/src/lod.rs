//! LOD: distance-based tier for vegetation instances.

use crate::species::SpeciesId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LodTier {
    Detail,
    Billboard,
    Culled,
}

pub fn classify_lod(distance: f32, species: SpeciesId) -> LodTier {
    let (detail, billboard) = match species {
        SpeciesId::Grass => (25.0, 60.0),
        SpeciesId::Fern => (40.0, 90.0),
        SpeciesId::Bush => (60.0, 150.0),
        SpeciesId::PalmTree => (200.0, 600.0),
        SpeciesId::Conifer => (180.0, 500.0),
    };
    if distance < detail {
        LodTier::Detail
    } else if distance < billboard {
        LodTier::Billboard
    } else {
        LodTier::Culled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grass_culled_far() {
        assert_eq!(classify_lod(100.0, SpeciesId::Grass), LodTier::Culled);
        assert_eq!(classify_lod(10.0, SpeciesId::Grass), LodTier::Detail);
    }

    #[test]
    fn tree_visible_far() {
        assert_eq!(classify_lod(100.0, SpeciesId::PalmTree), LodTier::Detail);
        assert_eq!(classify_lod(400.0, SpeciesId::PalmTree), LodTier::Billboard);
    }
}
