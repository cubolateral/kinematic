use super::runtime::Runtime;
use crate::core::audio::{SceneAudio, Sound, SoundEvent};
use mlua::{Result, Table, UserData, UserDataMethods};
use std::rc::{Rc, Weak};

pub(super) struct AudioStep {
    pub entity: hecs::Entity,
    pub frame: u64,
    pub update_index: usize,
    pub time: f32,
    pub events: Vec<SoundEvent>,
}

#[derive(Clone)]
pub(super) struct SoundHandle {
    pub runtime: Weak<Runtime>,
    pub sound: Sound,
    audio: SceneAudio,
    samples: Rc<std::cell::OnceCell<std::sync::Arc<Vec<[f32; 2]>>>>,
}

fn configure(mut sound: Sound, options: Table) -> Result<Sound> {
    if let Some(preset) = options.get::<Option<String>>("preset")? {
        sound = Sound::preset(&preset).map_err(mlua::Error::runtime)?;
    }
    let frequency = options.get::<Option<f32>>("frequency")?;
    if let Some(frequency) = frequency {
        if sound.end_frequency == sound.frequency {
            sound.end_frequency = frequency;
        }
        sound.frequency = frequency;
    }
    for pair in options.pairs::<String, mlua::Value>() {
        let (name, value) = pair?;
        match name.as_str() {
            "preset" | "frequency" => (),
            "waveform" => {
                sound.waveform = match value {
                    mlua::Value::String(v) => v.to_str()?.to_owned(),
                    _ => return Err(mlua::Error::runtime("Sound waveform must be a string.")),
                }
            }
            "seed" => {
                sound.seed = match value {
                    mlua::Value::Integer(v) if (0..=u32::MAX as i64).contains(&v) => v as u32,
                    _ => {
                        return Err(mlua::Error::runtime(
                            "Sound seed must be an unsigned 32-bit integer.",
                        ));
                    }
                }
            }
            _ => {
                let field = match name.as_str() {
                    "end_frequency" => &mut sound.end_frequency,
                    "gain" => &mut sound.gain,
                    "pan" => &mut sound.pan,
                    "attack" => &mut sound.attack,
                    "decay" => &mut sound.decay,
                    "sustain" => &mut sound.sustain,
                    "hold" => &mut sound.hold,
                    "release" => &mut sound.release,
                    "noise" => &mut sound.noise,
                    "overtone" => &mut sound.overtone,
                    "cutoff" => &mut sound.cutoff,
                    _ => {
                        return Err(mlua::Error::runtime(format!(
                            "Unknown sound option '{name}'."
                        )));
                    }
                };
                *field = match value {
                    mlua::Value::Number(v) => v as f32,
                    mlua::Value::Integer(v) => v as f32,
                    _ => {
                        return Err(mlua::Error::runtime(format!(
                            "Sound option '{name}' must be a number."
                        )));
                    }
                };
            }
        }
    }
    for (name, value, min, max) in [
        ("Frequency", sound.frequency, 1.0, 20_000.0),
        ("End frequency", sound.end_frequency, 1.0, 20_000.0),
        ("Gain", sound.gain, 0.0, 1.0),
        ("Pan", sound.pan, -1.0, 1.0),
        ("Attack", sound.attack, 0.0, 10.0),
        ("Decay", sound.decay, 0.0, 10.0),
        ("Hold", sound.hold, 0.0, 10.0),
        ("Release", sound.release, 0.0, 10.0),
        ("Sustain", sound.sustain, 0.0, 1.0),
        ("Noise", sound.noise, 0.0, 1.0),
        ("Overtone", sound.overtone, 0.0, 1.0),
        ("Cutoff", sound.cutoff, 20.0, 20_000.0),
    ] {
        if !value.is_finite() || !(min..=max).contains(&value) {
            return Err(mlua::Error::runtime(format!(
                "{name} must be finite and between {min} and {max}."
            )));
        }
    }
    let duration = sound.attack + sound.decay + sound.hold + sound.release;
    if !(0.001..=10.0).contains(&duration) {
        return Err(mlua::Error::runtime(
            "Sound envelope duration must be 0.001–10 seconds.",
        ));
    }
    if !["sine", "triangle", "noise"].contains(&sound.waveform.as_str()) {
        return Err(mlua::Error::runtime(
            "Sound waveform must be sine, triangle, or noise.",
        ));
    }
    Ok(sound)
}

pub(super) fn create(runtime: Rc<Runtime>, options: Table) -> Result<SoundHandle> {
    runtime.construction()?;
    super::runtime::engine(|| runtime.animator.borrow().assert_finite_scope())?;
    let sound = configure(Sound::default(), options)?;
    Ok(SoundHandle {
        runtime: Rc::downgrade(&runtime),
        samples: Default::default(),
        sound,
        audio: runtime.audio.clone(),
    })
}

impl UserData for SoundHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("play", |_, this, options: Option<Table>| {
            let runtime = this
                .runtime
                .upgrade()
                .ok_or_else(|| mlua::Error::runtime("Scene has been released."))?;
            let mut step = runtime.audio_step.borrow_mut();
            if step.is_none() {
                runtime.construction()?;
                super::runtime::engine(|| runtime.animator.borrow().assert_finite_scope())?;
            }
            let override_sound = options
                .map(|options| configure(this.sound.clone(), options))
                .transpose()?;
            let sound = override_sound.as_ref().unwrap_or(&this.sound);
            let key = step
                .as_ref()
                .map(|step| (step.entity, step.frame, step.update_index));
            let pending = step.as_ref().map_or(0, |step| {
                step.events.iter().map(|event| event.samples.len()).sum()
            });
            this.audio
                .borrow()
                .check_capacity(sound.sample_count(), key, pending)
                .map_err(mlua::Error::runtime)?;
            let samples = if override_sound.is_some() {
                sound.samples()
            } else {
                this.samples.get_or_init(|| sound.samples()).clone()
            };
            if let Some(step) = step.as_mut() {
                step.events.push(SoundEvent {
                    time: step.time as f64,
                    samples,
                });
            } else {
                this.audio.borrow_mut().scheduled.push(SoundEvent {
                    time: runtime.animator.borrow().time() as f64,
                    samples,
                });
            }
            Ok(())
        });
    }
}

pub(super) fn step<T>(
    runtime: &Runtime,
    entity: hecs::Entity,
    frame: u64,
    update_index: usize,
    time: f32,
    callback: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let previous = runtime.audio_step.replace(Some(AudioStep {
        entity,
        frame,
        update_index,
        time,
        events: Vec::new(),
    }));
    let result = callback();
    let step = runtime.audio_step.replace(previous).unwrap();
    if result.is_ok() {
        let mut audio = runtime.audio.borrow_mut();
        let key = (step.entity, step.frame, step.update_index);
        if step.events.is_empty() {
            audio.steps.remove(&key);
        } else {
            audio.steps.insert(key, step.events);
        }
    }
    result
}
