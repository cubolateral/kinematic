/// A deterministic xoshiro256++ pseudo-random number generator with stable output.
///
/// Its state is initialized and fork keys are mixed with SplitMix64. Cloning
/// preserves the exact current state. Forks are derived from the original seed,
/// so their output is unaffected by values consumed by the parent generator.
#[derive(Clone, Debug)]
pub struct Random {
    seed: u64,
    state: [u64; 4],
}

impl Random {
    /// Creates a generator from `seed`.
    pub fn new(seed: u64) -> Self {
        let mut mixer = seed;
        let state = std::array::from_fn(|_| split_mix64(&mut mixer));
        Self { seed, state }
    }

    /// Creates a deterministic child generator from the original seed and `key`.
    pub fn fork(&self, key: u64) -> Self {
        let mut key_mixer = key;
        let mixed_key = split_mix64(&mut key_mixer);
        let mut seed_mixer = self.seed ^ mixed_key;
        Self::new(split_mix64(&mut seed_mixer))
    }

    /// Generates a boolean with equal probability for either value.
    pub fn bool(&mut self) -> bool {
        self.u64() & 1 != 0
    }

    /// Generates an unsigned 8-bit integer.
    pub fn u8(&mut self) -> u8 {
        self.u64() as u8
    }

    /// Generates an unsigned 16-bit integer.
    pub fn u16(&mut self) -> u16 {
        self.u64() as u16
    }

    /// Generates an unsigned 32-bit integer.
    pub fn u32(&mut self) -> u32 {
        self.u64() as u32
    }

    /// Generates an unsigned 64-bit integer.
    pub fn u64(&mut self) -> u64 {
        let result = self.state[0]
            .wrapping_add(self.state[3])
            .rotate_left(23)
            .wrapping_add(self.state[0]);
        let temporary = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];
        self.state[2] ^= temporary;
        self.state[3] = self.state[3].rotate_left(45);
        result
    }

    /// Generates an unsigned 128-bit integer.
    pub fn u128(&mut self) -> u128 {
        (u128::from(self.u64()) << 64) | u128::from(self.u64())
    }

    /// Generates an unsigned pointer-sized integer.
    pub fn usize(&mut self) -> usize {
        self.u64() as usize
    }

    /// Generates a signed 8-bit integer.
    pub fn i8(&mut self) -> i8 {
        self.u8() as i8
    }

    /// Generates a signed 16-bit integer.
    pub fn i16(&mut self) -> i16 {
        self.u16() as i16
    }

    /// Generates a signed 32-bit integer.
    pub fn i32(&mut self) -> i32 {
        self.u32() as i32
    }

    /// Generates a signed 64-bit integer.
    pub fn i64(&mut self) -> i64 {
        self.u64() as i64
    }

    /// Generates a signed 128-bit integer.
    pub fn i128(&mut self) -> i128 {
        self.u128() as i128
    }

    /// Generates a signed pointer-sized integer.
    pub fn isize(&mut self) -> isize {
        self.usize() as isize
    }

    /// Generates a floating-point value in `0.0..1.0`.
    pub fn f32(&mut self) -> f32 {
        (self.u64() >> 40) as f32 * (1.0 / (1_u32 << 24) as f32)
    }

    /// Generates a floating-point value in `0.0..1.0`.
    pub fn f64(&mut self) -> f64 {
        (self.u64() >> 11) as f64 * (1.0 / (1_u64 << 53) as f64)
    }

    /// Generates a uniformly distributed value in `range`.
    pub fn range_u32(&mut self, range: std::ops::Range<u32>) -> u32 {
        assert!(range.start < range.end, "Random ranges must not be empty.");
        range.start + self.bounded_u64(u64::from(range.end - range.start)) as u32
    }

    /// Generates a uniformly distributed value in `range`.
    pub fn range_u64(&mut self, range: std::ops::Range<u64>) -> u64 {
        assert!(range.start < range.end, "Random ranges must not be empty.");
        range.start + self.bounded_u64(range.end - range.start)
    }

    /// Generates a uniformly distributed value in `range`.
    pub fn range_usize(&mut self, range: std::ops::Range<usize>) -> usize {
        assert!(range.start < range.end, "Random ranges must not be empty.");
        range.start + self.bounded_u64((range.end - range.start) as u64) as usize
    }

    /// Generates a uniformly distributed value in `range`.
    pub fn range_i32(&mut self, range: std::ops::Range<i32>) -> i32 {
        assert!(range.start < range.end, "Random ranges must not be empty.");
        let width = (i64::from(range.end) - i64::from(range.start)) as u64;
        (i64::from(range.start) + self.bounded_u64(width) as i64) as i32
    }

    /// Generates a uniformly distributed value in `range`.
    pub fn range_i64(&mut self, range: std::ops::Range<i64>) -> i64 {
        assert!(range.start < range.end, "Random ranges must not be empty.");
        let width = (i128::from(range.end) - i128::from(range.start)) as u64;
        (i128::from(range.start) + i128::from(self.bounded_u64(width))) as i64
    }

    /// Generates a uniformly distributed value in `range`.
    pub fn range_f32(&mut self, range: std::ops::Range<f32>) -> f32 {
        let width = range.end - range.start;
        assert!(
            range.start.is_finite()
                && range.end.is_finite()
                && width.is_finite()
                && range.start < range.end,
            "Random ranges must be finite and non-empty."
        );
        range.start + width * self.f32()
    }

    /// Generates a uniformly distributed value in `range`.
    pub fn range_f64(&mut self, range: std::ops::Range<f64>) -> f64 {
        let width = range.end - range.start;
        assert!(
            range.start.is_finite()
                && range.end.is_finite()
                && width.is_finite()
                && range.start < range.end,
            "Random ranges must be finite and non-empty."
        );
        range.start + width * self.f64()
    }

    /// Returns true with the supplied probability in `0.0..=1.0`.
    pub fn chance(&mut self, probability: f64) -> bool {
        assert!(
            probability.is_finite() && (0.0..=1.0).contains(&probability),
            "Chance must be between zero and one."
        );
        probability > 0.0 && (probability == 1.0 || self.f64() < probability)
    }

    /// Chooses one value, or returns `None` when the slice is empty.
    pub fn choose<'a, T>(&mut self, values: &'a [T]) -> Option<&'a T> {
        (!values.is_empty()).then(|| &values[self.range_usize(0..values.len())])
    }

    /// Chooses one mutable value, or returns `None` when the slice is empty.
    pub fn choose_mut<'a, T>(&mut self, values: &'a mut [T]) -> Option<&'a mut T> {
        if values.is_empty() {
            None
        } else {
            let index = self.range_usize(0..values.len());
            Some(&mut values[index])
        }
    }

    /// Chooses one value according to matching non-negative `weights`.
    ///
    /// Returns `None` for empty slices, mismatched lengths, invalid weights, or
    /// when every weight is zero.
    pub fn choose_weighted<'a, T>(&mut self, values: &'a [T], weights: &[f64]) -> Option<&'a T> {
        if values.is_empty() || values.len() != weights.len() {
            return None;
        }
        let total = weights.iter().try_fold(0.0, |total, weight| {
            (weight.is_finite() && *weight >= 0.0).then_some(total + weight)
        })?;
        if !total.is_finite() || total <= 0.0 {
            return None;
        }

        let mut target = self.range_f64(0.0..total);
        let mut last_positive = None;
        for (value, weight) in values.iter().zip(weights) {
            if *weight > 0.0 {
                last_positive = Some(value);
            }
            if target < *weight {
                return Some(value);
            }
            target -= weight;
        }
        last_positive
    }

    /// Chooses up to `amount` distinct values without changing their slice.
    pub fn choose_multiple<'a, T>(&mut self, values: &'a [T], amount: usize) -> Vec<&'a T> {
        let amount = amount.min(values.len());
        let mut indices = (0..values.len()).collect::<Vec<_>>();
        for index in 0..amount {
            let selected = self.range_usize(index..indices.len());
            indices.swap(index, selected);
        }
        indices[..amount]
            .iter()
            .map(|index| &values[*index])
            .collect()
    }

    /// Randomly permutes a slice in place.
    pub fn shuffle<T>(&mut self, values: &mut [T]) {
        for end in (1..values.len()).rev() {
            let selected = self.range_usize(0..end + 1);
            values.swap(end, selected);
        }
    }

    /// Generates a normally distributed value with `mean` and standard deviation.
    pub fn normal(&mut self, mean: f64, standard_deviation: f64) -> f64 {
        assert!(
            mean.is_finite() && standard_deviation.is_finite() && standard_deviation >= 0.0,
            "Normal distribution parameters must be finite and its deviation non-negative."
        );
        if standard_deviation == 0.0 {
            return mean;
        }
        let radius = (-2.0 * (1.0 - self.f64()).ln()).sqrt();
        let angle = std::f64::consts::TAU * self.f64();
        mean + standard_deviation * radius * angle.cos()
    }

    fn bounded_u64(&mut self, bound: u64) -> u64 {
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let value = self.u64();
            if value >= threshold {
                return value % bound;
            }
        }
    }
}

fn split_mix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_seeds_produce_equal_values() {
        let mut first = Random::new(42);
        let mut second = Random::new(42);
        assert_eq!((first.u64(), first.u128()), (second.u64(), second.u128()));
        assert_eq!((first.f32(), first.f64()), (second.f32(), second.f64()));
    }

    #[test]
    fn algorithm_sequence_is_stable() {
        let mut random = Random::new(42);
        assert_eq!(
            [random.u64(), random.u64(), random.u64()],
            [
                15_021_278_609_987_233_951,
                5_881_210_131_331_364_753,
                18_149_643_915_985_481_100,
            ]
        );
    }

    #[test]
    fn forks_ignore_consumed_parent_state() {
        let parent = Random::new(12);
        let mut consumed = parent.clone();
        for _ in 0..100 {
            consumed.u64();
        }
        assert_eq!(parent.fork(7).u64(), consumed.fork(7).u64());
        assert_ne!(parent.fork(7).u64(), parent.fork(8).u64());
        assert_eq!(parent.fork(7).u64(), 12_454_396_449_278_183_096);
    }

    #[test]
    fn ranges_stay_within_their_bounds() {
        let mut random = Random::new(9);
        for _ in 0..1_000 {
            assert!((10..20).contains(&random.range_u32(10..20)));
            assert!((-20..-10).contains(&random.range_i64(-20..-10)));
            assert!((0.25..0.75).contains(&random.range_f64(0.25..0.75)));
        }
    }

    #[test]
    fn chooses_and_mutates_values() {
        let mut random = Random::new(4);
        let values = [10, 20, 30, 40];
        assert!(values.contains(random.choose(&values).unwrap()));
        let chosen = random.choose_multiple(&values, 3);
        assert_eq!(chosen.len(), 3);
        assert_ne!(chosen[0], chosen[1]);

        let mut mutable = values;
        *random.choose_mut(&mut mutable).unwrap() = 99;
        assert!(mutable.contains(&99));
        assert_eq!(
            random.choose_weighted(&values, &[0.0, 0.0, 1.0, 0.0]),
            Some(&30)
        );
    }

    #[test]
    fn shuffle_is_deterministic_and_preserves_values() {
        let mut first = [1, 2, 3, 4, 5, 6];
        let mut second = first;
        Random::new(81).shuffle(&mut first);
        Random::new(81).shuffle(&mut second);
        assert_eq!(first, second);
        first.sort();
        assert_eq!(first, [1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn normal_has_the_requested_distribution() {
        let mut random = Random::new(123);
        let samples = (0..50_000)
            .map(|_| random.normal(4.0, 2.0))
            .collect::<Vec<_>>();
        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        let variance = samples
            .iter()
            .map(|sample| (sample - mean).powi(2))
            .sum::<f64>()
            / samples.len() as f64;
        assert!((mean - 4.0).abs() < 0.05);
        assert!((variance.sqrt() - 2.0).abs() < 0.05);
    }
}

/// Creates a deterministic generator with seed zero.
pub fn random() -> Random {
    Random::new(0)
}
