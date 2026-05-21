//! Shared utilities for kami-game: deterministic PRNG, HP trait.

/// Deterministic xorshift32 PRNG — no_std compatible, zero allocation.
/// Used across NPC AI, battle systems, gacha, and terrain generation.
#[derive(Debug, Clone)]
pub struct SimpleRng {
    pub seed: u32,
}

impl SimpleRng {
    pub fn new(seed: u32) -> Self {
        Self {
            seed: if seed == 0 { 1 } else { seed },
        }
    }

    /// Returns a pseudo-random f32 in [0.0, 1.0).
    pub fn next_f32(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed as f32 / u32::MAX as f32).abs()
    }

    /// Returns a pseudo-random f32 in [min, max).
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }

    /// Returns a pseudo-random u32 in [0, max).
    pub fn next_u32(&mut self, max: u32) -> u32 {
        (self.next_f32() * max as f32) as u32
    }

    /// Returns a pseudo-random bool with given probability (0.0–1.0).
    pub fn chance(&mut self, probability: f32) -> bool {
        self.next_f32() < probability
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_deterministic() {
        let mut a = SimpleRng::new(42);
        let mut b = SimpleRng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_f32().to_bits(), b.next_f32().to_bits());
        }
    }

    #[test]
    fn rng_range_bounded() {
        let mut rng = SimpleRng::new(123);
        for _ in 0..1000 {
            let v = rng.range(5.0, 10.0);
            assert!(v >= 5.0 && v < 10.0, "range({v}) out of bounds");
        }
    }

    #[test]
    fn rng_next_f32_bounded() {
        let mut rng = SimpleRng::new(999);
        for _ in 0..1000 {
            let v = rng.next_f32();
            assert!(v >= 0.0 && v < 1.0, "next_f32({v}) out of bounds");
        }
    }

    #[test]
    fn rng_next_u32_bounded() {
        let mut rng = SimpleRng::new(555);
        for _ in 0..1000 {
            let v = rng.next_u32(10);
            assert!(v < 10, "next_u32({v}) out of bounds");
        }
    }

    #[test]
    fn rng_different_seeds_diverge() {
        let mut a = SimpleRng::new(1);
        let mut b = SimpleRng::new(2);
        let va: Vec<f32> = (0..10).map(|_| a.next_f32()).collect();
        let vb: Vec<f32> = (0..10).map(|_| b.next_f32()).collect();
        assert_ne!(va, vb);
    }

    #[test]
    fn rng_chance_probability() {
        let mut rng = SimpleRng::new(42);
        let hits: usize = (0..10000).filter(|_| rng.chance(0.5)).count();
        // Should be roughly 50% (4000-6000 range)
        assert!(
            hits > 4000 && hits < 6000,
            "chance(0.5) produced {hits}/10000 hits"
        );
    }
}
