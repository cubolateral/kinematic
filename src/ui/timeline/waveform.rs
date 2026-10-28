use std::sync::{Arc, Weak};

const BLOCK: usize = 256;

pub(super) struct Waveform {
    pub samples: Weak<Vec<[f32; 2]>>,
    peaks: Vec<[f32; 2]>,
}

fn bounds(samples: &[[f32; 2]]) -> [f32; 2] {
    samples
        .iter()
        .flatten()
        .fold([0.0_f32; 2], |[min, max], &value| {
            [min.min(value), max.max(value)]
        })
}

impl Waveform {
    pub fn new(samples: &Arc<Vec<[f32; 2]>>) -> Self {
        Self {
            samples: Arc::downgrade(samples),
            peaks: samples.chunks(BLOCK).map(bounds).collect(),
        }
    }

    pub fn range(&self, samples: &[[f32; 2]], from: usize, to: usize) -> [f32; 2] {
        let from = from.min(samples.len());
        let to = to.min(samples.len()).max(from);
        let first = from.div_ceil(BLOCK);
        let last = to / BLOCK;
        if first >= last {
            return bounds(&samples[from..to]);
        }
        let a = bounds(&samples[from..first * BLOCK]);
        let b = bounds(&samples[last * BLOCK..to]);
        self.peaks[first..last]
            .iter()
            .fold([a[0].min(b[0]), a[1].max(b[1])], |[min, max], peak| {
                [min.min(peak[0]), max.max(peak[1])]
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn peaks_keep_impulses_and_exclude_trimmed_samples() {
        let mut samples = vec![[0.0; 2]; 2048];
        samples[333] = [-0.9, 0.8];
        samples[700] = [-0.4, 0.2];
        let samples = Arc::new(samples);
        let wave = Waveform::new(&samples);
        assert_eq!(wave.range(&samples, 0, 2048), [-0.9, 0.8]);
        assert_eq!(wave.range(&samples, 334, 2048), [-0.4, 0.2]);
        assert_eq!(wave.range(&samples, 333, 334), [-0.9, 0.8]);
        assert_eq!(wave.range(&samples, 334, 700), [0.0, 0.0]);
    }
}
