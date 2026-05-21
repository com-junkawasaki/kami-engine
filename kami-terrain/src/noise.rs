//! Value noise + FBM (Fractal Brownian Motion) for terrain generation.
//!
//! Uses value noise (not Perlin) to avoid US10232272B2 patent risk.

/// Hash-based value noise at integer coordinates.
fn hash2d(x: i32, y: i32) -> f32 {
    let n = x.wrapping_mul(1619).wrapping_add(y.wrapping_mul(31337));
    let n = n.wrapping_mul(n).wrapping_mul(n);
    let n = (n >> 13) ^ n;
    let n = n
        .wrapping_mul(n.wrapping_mul(n.wrapping_mul(60493)).wrapping_add(19990303))
        .wrapping_add(1376312589);
    // Map to [0, 1]
    (n & 0x7fffffff) as f32 / 0x7fffffff as f32
}

/// Cosine interpolation for smooth transitions.
fn cosine_interp(a: f32, b: f32, t: f32) -> f32 {
    let ft = t * std::f32::consts::PI;
    let f = (1.0 - ft.cos()) * 0.5;
    a * (1.0 - f) + b * f
}

/// 2D value noise with cosine interpolation.
pub fn value_noise(x: f32, y: f32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let xf = x - x.floor();
    let yf = y - y.floor();

    let v00 = hash2d(xi, yi);
    let v10 = hash2d(xi + 1, yi);
    let v01 = hash2d(xi, yi + 1);
    let v11 = hash2d(xi + 1, yi + 1);

    let ix0 = cosine_interp(v00, v10, xf);
    let ix1 = cosine_interp(v01, v11, xf);
    cosine_interp(ix0, ix1, yf)
}

/// Fractal Brownian Motion: layered value noise for terrain height.
///
/// `octaves`: number of noise layers (4-8 typical)
/// `lacunarity`: frequency multiplier per octave (typically 2.0)
/// `persistence`: amplitude multiplier per octave (typically 0.5)
pub fn fbm_noise(x: f32, y: f32, octaves: u32, lacunarity: f32, persistence: f32) -> f32 {
    let mut value = 0.0f32;
    let mut amplitude = 1.0f32;
    let mut frequency = 1.0f32;
    let mut max_amplitude = 0.0f32;

    for _ in 0..octaves {
        value += amplitude * value_noise(x * frequency, y * frequency);
        max_amplitude += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    value / max_amplitude
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fbm_range() {
        for x in 0..100 {
            for y in 0..100 {
                let v = fbm_noise(x as f32 * 0.1, y as f32 * 0.1, 6, 2.0, 0.5);
                assert!(v >= 0.0 && v <= 1.0, "fbm out of range: {v}");
            }
        }
    }

    #[test]
    fn deterministic() {
        let a = fbm_noise(1.5, 2.5, 4, 2.0, 0.5);
        let b = fbm_noise(1.5, 2.5, 4, 2.0, 0.5);
        assert_eq!(a, b);
    }
}
