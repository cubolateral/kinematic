use std::{cell::RefCell, collections::BTreeMap, rc::Rc, sync::Arc};

pub(crate) const SAMPLE_RATE: u32 = 48_000;
pub(crate) const MAX_SAMPLE_FRAMES: usize = 8_000_000;
pub(crate) type SceneAudio = Rc<RefCell<AudioEvents>>;

#[derive(Clone, Debug)]
pub(crate) struct Sound {
    pub waveform: String,
    pub frequency: f32,
    pub end_frequency: f32,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub hold: f32,
    pub release: f32,
    pub noise: f32,
    pub overtone: f32,
    pub cutoff: f32,
    pub seed: u32,
}

impl Default for Sound {
    fn default() -> Self {
        Self {
            waveform: "sine".into(),
            frequency: 440.0,
            end_frequency: 440.0,
            attack: 0.004,
            decay: 0.08,
            sustain: 0.3,
            hold: 0.03,
            release: 0.15,
            noise: 0.0,
            overtone: 0.0,
            cutoff: 12_000.0,
            seed: 0,
        }
    }
}

impl Sound {
    pub fn preset(name: &str) -> Result<Self, String> {
        let mut sound = Self::default();
        match name {
            "boink" => {
                sound.frequency = 450.0;
                sound.end_frequency = 90.0;
                sound.overtone = 0.25;
            }
            "pluck" => {
                sound.waveform = "triangle".into();
                sound.sustain = 0.0;
                sound.decay = 0.22;
                sound.release = 0.05;
            }
            "click" => {
                sound.frequency = 1800.0;
                sound.end_frequency = 600.0;
                sound.noise = 0.5;
                sound.attack = 0.001;
                sound.decay = 0.015;
                sound.sustain = 0.0;
                sound.hold = 0.0;
                sound.release = 0.005;
            }
            "impact" => {
                sound.frequency = 140.0;
                sound.end_frequency = 35.0;
                sound.noise = 0.65;
                sound.cutoff = 1800.0;
                sound.sustain = 0.0;
                sound.decay = 0.18;
            }
            "whoosh" => {
                sound.waveform = "noise".into();
                sound.attack = 0.15;
                sound.decay = 0.1;
                sound.sustain = 0.6;
                sound.release = 0.25;
                sound.cutoff = 2400.0;
            }
            _ => return Err(format!("Unknown sound preset '{name}'.")),
        }
        Ok(sound)
    }

    pub fn sample_count(&self) -> usize {
        ((self.attack + self.decay + self.hold + self.release) * SAMPLE_RATE as f32).ceil() as usize
    }

    pub fn samples(&self) -> Arc<Vec<[f32; 2]>> {
        let duration = self.attack + self.decay + self.hold + self.release;
        let count = self.sample_count();
        let mut phase = 0.0_f64;
        let mut filtered = 0.0;
        let alpha = 1.0 - (-std::f32::consts::TAU * self.cutoff / SAMPLE_RATE as f32).exp();
        Arc::new(
            (0..count)
                .map(|i| {
                    let t = i as f32 / SAMPLE_RATE as f32;
                    let envelope = if t < self.attack {
                        t / self.attack
                    } else if t < self.attack + self.decay {
                        1.0 - (1.0 - self.sustain) * (t - self.attack) / self.decay
                    } else if t < self.attack + self.decay + self.hold {
                        self.sustain
                    } else {
                        self.sustain * (duration - t) / self.release.max(1.0 / SAMPLE_RATE as f32)
                    };
                    let mut hash = (i as u32).wrapping_add(self.seed).wrapping_add(0x9e3779b9);
                    hash = (hash ^ (hash >> 16)).wrapping_mul(0x21f0aaad);
                    hash = (hash ^ (hash >> 15)).wrapping_mul(0x735a2d97);
                    let noise = (hash ^ (hash >> 15)) as f32 / u32::MAX as f32 * 2.0 - 1.0;
                    let tone = match self.waveform.as_str() {
                        "triangle" => 1.0 - 4.0 * (phase.fract() - 0.5).abs() as f32,
                        "noise" => noise,
                        _ => (phase * std::f64::consts::TAU).sin() as f32,
                    };
                    let voice = tone * (1.0 - self.noise)
                        + noise * self.noise
                        + (phase * 2.0 * std::f64::consts::TAU).sin() as f32 * self.overtone;
                    filtered += alpha * (voice - filtered);
                    // Always fade the edges, including envelopes with zero attack or release.
                    let fade = (t / 0.002).min((duration - t) / 0.002).clamp(0.0, 1.0);
                    let sample = filtered * envelope * fade;
                    phase += (self.frequency + (self.end_frequency - self.frequency) * t / duration)
                        as f64
                        / SAMPLE_RATE as f64;
                    [sample; 2]
                })
                .collect(),
        )
    }
}

#[derive(Clone)]
pub(crate) struct SoundEvent {
    pub time: f64,
    pub samples: Arc<Vec<[f32; 2]>>,
    pub offset: usize,
    pub frames: usize,
    pub gain: f32,
    pub pan: f32,
}

#[derive(Clone, Default)]
pub(crate) struct AudioEvents {
    pub scheduled: Vec<SoundEvent>,
    pub clips: Vec<SoundEvent>,
    pub steps: BTreeMap<(hecs::Entity, u64, usize), Vec<SoundEvent>>,
    buffers: Vec<std::sync::Weak<Vec<[f32; 2]>>>,
}

impl AudioEvents {
    pub fn register_samples(&mut self, samples: &Arc<Vec<[f32; 2]>>) -> Result<(), String> {
        self.buffers.retain(|buffer| buffer.strong_count() > 0);
        if self
            .buffers
            .iter()
            .any(|buffer| buffer.as_ptr() == Arc::as_ptr(samples))
        {
            return Ok(());
        }
        let stored = self
            .buffers
            .iter()
            .filter_map(|buffer| buffer.upgrade())
            .map(|buffer| buffer.len())
            .sum::<usize>();
        if stored + samples.len() > MAX_SAMPLE_FRAMES {
            return Err("Scene audio resources exceed the 64 MB sample budget.".into());
        }
        self.buffers.push(Arc::downgrade(samples));
        Ok(())
    }

    pub fn check_capacity(
        &self,
        replacing: Option<(hecs::Entity, u64, usize)>,
        pending: usize,
    ) -> Result<(), String> {
        let stored = self.scheduled.len()
            + self.clips.len()
            + self
                .steps
                .iter()
                .filter(|(key, _)| Some(**key) != replacing)
                .map(|(_, events)| events.len())
                .sum::<usize>();
        if stored + pending >= 100_000 {
            return Err("Scene exceeds 100000 audio occurrences.".into());
        }
        Ok(())
    }

    pub fn mix(&self, offset: f64, start: u64, output: &mut [[f32; 2]]) {
        for event in self
            .scheduled
            .iter()
            .chain(&self.clips)
            .chain(self.steps.values().flatten())
        {
            let at = ((offset + event.time) * SAMPLE_RATE as f64).round() as i64;
            let from = (at - start as i64).max(0) as usize;
            let to = (at
                + event
                    .frames
                    .min(event.samples.len().saturating_sub(event.offset)) as i64
                - start as i64)
                .clamp(0, output.len() as i64) as usize;
            let balance = [
                (event.pan.max(0.0) * std::f32::consts::FRAC_PI_2)
                    .cos()
                    .max(0.0)
                    * event.gain,
                (event.pan.min(0.0) * std::f32::consts::FRAC_PI_2)
                    .cos()
                    .max(0.0)
                    * event.gain,
            ];
            for (index, frame) in output.iter_mut().enumerate().take(to).skip(from) {
                let source =
                    event.samples[event.offset + (start as i64 + index as i64 - at) as usize];
                frame[0] += source[0] * balance[0];
                frame[1] += source[1] * balance[1];
            }
        }
    }

    pub fn copy_steps(&mut self, previous: &Self, entity: hecs::Entity) -> Result<(), String> {
        for (key, events) in &previous.steps {
            if key.0 == entity {
                for event in events {
                    self.register_samples(&event.samples)?;
                }
                self.check_capacity(Some(*key), events.len().saturating_sub(1))?;
                self.steps.insert(*key, events.clone());
            }
        }
        Ok(())
    }
}

pub(crate) fn limit(samples: Vec<[f32; 2]>) -> Vec<f32> {
    samples
        .into_iter()
        .flatten()
        .map(|sample| sample.tanh())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthesis_is_deterministic_faded_and_chunk_independent() {
        for name in ["boink", "pluck", "click", "impact", "whoosh"] {
            let sound = Sound::preset(name).unwrap();
            let samples = sound.samples();
            assert_eq!(samples, sound.samples());
            assert_eq!(samples[0], [0.0; 2]);
            assert!(samples.last().unwrap().iter().all(|v| v.abs() < 0.01));
            assert!(samples.iter().flatten().all(|v| v.is_finite()));
            let events = AudioEvents {
                scheduled: vec![SoundEvent {
                    time: 0.01,
                    offset: 0,
                    frames: samples.len(),
                    samples,
                    gain: 0.2,
                    pan: 0.0,
                }],
                ..Default::default()
            };
            let mut whole = vec![[0.0; 2]; 20_000];
            events.mix(0.0, 0, &mut whole);
            let mut split = vec![[0.0; 2]; 20_000];
            events.mix(0.0, 0, &mut split[..777]);
            events.mix(0.0, 777, &mut split[777..]);
            assert_eq!(whole, split);
            assert!(limit(whole).iter().all(|v| v.abs() <= 1.0));
        }
    }
}

#[cfg(test)]
mod resource_tests {
    use super::*;

    #[test]
    fn shared_samples_count_once_and_unused_buffers_release_the_budget() {
        let samples = Arc::new(vec![[0.0; 2]; MAX_SAMPLE_FRAMES]);
        let mut audio = AudioEvents::default();
        audio.register_samples(&samples).unwrap();
        audio.register_samples(&samples).unwrap();
        let extra = Arc::new(vec![[0.0; 2]]);
        assert!(audio.register_samples(&extra).is_err());
        drop(samples);
        audio.register_samples(&extra).unwrap();
    }
}

#[cfg(test)]
mod clip_tests {
    use super::*;

    #[test]
    fn trimmed_stereo_clips_mix_identically_across_chunks_and_scene_offsets() {
        let samples = Arc::new(
            (0..32)
                .map(|i| [i as f32 / 32.0, -(i as f32) / 32.0])
                .collect::<Vec<_>>(),
        );
        let mut audio = AudioEvents::default();
        audio.register_samples(&samples).unwrap();
        audio.clips.push(SoundEvent {
            time: 4.0 / SAMPLE_RATE as f64,
            samples,
            offset: 7,
            frames: 9,
            gain: 0.5,
            pan: -1.0,
        });
        let mut whole = vec![[0.0; 2]; 32];
        let scene_offset = 3.0 / SAMPLE_RATE as f64;
        audio.mix(scene_offset, 0, &mut whole);
        assert!(whole[..7].iter().all(|frame| *frame == [0.0; 2]));
        assert_eq!(whole[7], [7.0 / 64.0, 0.0]);
        assert!(whole[16..].iter().all(|frame| *frame == [0.0; 2]));
        let mut chunks = vec![[0.0; 2]; 32];
        audio.mix(scene_offset, 0, &mut chunks[..10]);
        audio.mix(scene_offset, 10, &mut chunks[10..]);
        assert_eq!(whole, chunks);
    }
}
