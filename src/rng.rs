//! splitmix64: a tiny seeded generator that behaves identically everywhere,
//! so `-s SEED` always reproduces the same maze.

#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng {
            state: seed
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(0x1234_5678_90AB_CDEF),
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// 0 .. n-1; returns 0 when n <= 1.
    pub fn below(&mut self, n: u32) -> u32 {
        if n <= 1 {
            return 0;
        }
        (self.next_u64() % n as u64) as u32
    }

    /// Convenience for indices: 0 .. n-1.
    pub fn below_usize(&mut self, n: usize) -> usize {
        self.below(n as u32) as usize
    }

    /// [0, 1)
    pub fn double(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Rng;

    /// Golden values produced by the original C implementation.
    #[test]
    fn matches_c_sequence() {
        let mut r = Rng::new(42);
        let got: Vec<u64> = (0..4).map(|_| r.next_u64()).collect();
        assert_eq!(
            got,
            [
                15473599319004437942u64,
                16207244988956626443u64,
                16753953004171473518u64,
                12868333742600082700u64,
            ]
        );
        let mut r = Rng::new(7);
        let got: Vec<u32> = (0..4).map(|_| r.below(10)).collect();
        assert_eq!(got, [4, 9, 6, 5]);
    }
}
