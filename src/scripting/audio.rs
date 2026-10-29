use super::runtime::Runtime;
use crate::core::audio::{SAMPLE_RATE, SceneAudio, Sound, SoundEvent};
use mlua::{Result, Table, UserData, UserDataFields, UserDataMethods};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    path::Path,
    rc::{Rc, Weak},
    sync::{Arc, Weak as SyncWeak},
};

pub(super) struct AudioStep {
    pub entity: hecs::Entity,
    pub frame: u64,
    pub update_index: usize,
    pub time: f32,
    pub events: Vec<SoundEvent>,
}

#[derive(Debug)]
enum Source {
    Synth(Sound),
    File(u64),
}

struct AudioAsset {
    source: Source,
    samples: Arc<Vec<[f32; 2]>>,
}

#[derive(Clone)]
pub(super) struct AudioResource(Arc<AudioAsset>);

impl std::fmt::Debug for AudioResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0.source {
            Source::Synth(sound) => f.debug_tuple("Synth").field(sound).finish(),
            Source::File(id) => f.debug_tuple("File").field(id).finish(),
        }
    }
}

impl UserData for AudioResource {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("duration", |_, this| {
            Ok(this.0.samples.len() as f64 / SAMPLE_RATE as f64)
        });
    }
}

thread_local! {
    static SYNTHS: RefCell<BTreeMap<String, SyncWeak<AudioAsset>>> = RefCell::new(BTreeMap::new());
    static FILES: RefCell<BTreeMap<Vec<u8>, SyncWeak<Vec<[f32; 2]>>>> = RefCell::new(BTreeMap::new());
}

pub(super) fn release_unused() {
    SYNTHS.with(|cache| {
        cache
            .borrow_mut()
            .retain(|_, value| value.strong_count() > 0)
    });
    FILES.with(|cache| {
        cache
            .borrow_mut()
            .retain(|_, value| value.strong_count() > 0)
    });
}

pub(crate) fn load_file(
    bytes: Vec<u8>,
    deadline: std::time::Instant,
) -> std::result::Result<Arc<Vec<[f32; 2]>>, String> {
    FILES.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.retain(|_, value| value.strong_count() > 0);
        if let Some(samples) = cache.get(&bytes).and_then(SyncWeak::upgrade) {
            return Ok(samples);
        }
        let samples = Arc::new(super::audio_file::decode(&bytes, deadline)?);
        cache.insert(bytes, Arc::downgrade(&samples));
        Ok(samples)
    })
}

pub(crate) fn clip_events(
    context: &crate::core::ProjectContext,
    sources: &super::Sources,
    clips: &[super::AudioClip],
    existing: &crate::core::audio::AudioEvents,
) -> std::result::Result<Vec<SoundEvent>, String> {
    let mut budget = existing.clone();
    budget.clips.clear();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let mut events = Vec::new();
    for clip in clips {
        clip.validate(context)?;
        let bytes = sources
            .read_audio(context, &clip.file)
            .map_err(|e| e.to_string())?;
        let samples = load_file(bytes, deadline)?;
        budget.check_capacity(None, events.len())?;
        events.push(clip_event(clip, samples)?);
    }
    Ok(events)
}

pub(crate) fn clip_event(
    clip: &super::AudioClip,
    samples: Arc<Vec<[f32; 2]>>,
) -> std::result::Result<SoundEvent, String> {
    let offset = (clip.offset * SAMPLE_RATE as f64).round() as usize;
    let end = ((clip.offset + clip.duration) * SAMPLE_RATE as f64).round() as usize;
    let frames = end.saturating_sub(offset);
    if frames == 0 || offset.saturating_add(frames) > samples.len() {
        return Err(format!(
            "Audio clip '{}' exceeds its source duration.",
            clip.file.display()
        ));
    }
    Ok(SoundEvent {
        time: clip.at,
        samples,
        offset,
        frames,
        gain: clip.gain,
        pan: clip.pan,
    })
}

pub(super) fn register(runtime: &Rc<Runtime>, api: &Table) -> Result<()> {
    let weak = Rc::downgrade(runtime);
    api.set(
        "audio_synth",
        runtime.lua.create_function(move |_, options: Table| {
            let runtime = weak
                .upgrade()
                .ok_or_else(|| mlua::Error::runtime("Scene has been released."))?;
            runtime.construction()?;
            let sound = configure(Sound::default(), options)?;
            let resource = SYNTHS.with(|cache| {
                let mut cache = cache.borrow_mut();
                cache.retain(|_, value| value.strong_count() > 0);
                let key = format!("{sound:?}");
                if let Some(asset) = cache.get(&key).and_then(SyncWeak::upgrade) {
                    return AudioResource(asset);
                }
                let asset = Arc::new(AudioAsset {
                    samples: sound.samples(),
                    source: Source::Synth(sound),
                });
                cache.insert(key, Arc::downgrade(&asset));
                AudioResource(asset)
            });
            runtime.assets.borrow_mut().push(Box::new(resource.clone()));
            Ok(resource)
        })?,
    )?;
    let weak = Rc::downgrade(runtime);
    api.set(
        "audio_file",
        runtime.lua.create_function(move |_, path: String| {
            let runtime = weak
                .upgrade()
                .ok_or_else(|| mlua::Error::runtime("Scene has been released."))?;
            runtime.construction()?;
            runtime
                .dependencies
                .borrow_mut()
                .insert(Path::new(&path).to_owned());
            let bytes = runtime
                .sources
                .read_audio(&runtime.context, Path::new(&path))
                .map_err(mlua::Error::external)?;
            let samples = load_file(bytes, runtime.deadline.get()).map_err(mlua::Error::runtime)?;
            let resource = AudioResource(Arc::new(AudioAsset {
                source: Source::File(Arc::as_ptr(&samples) as u64),
                samples,
            }));
            runtime.assets.borrow_mut().push(Box::new(resource.clone()));
            Ok(resource)
        })?,
    )?;
    Ok(())
}

#[derive(Clone)]
pub(super) struct AudioHandle {
    pub runtime: Weak<Runtime>,
    source: AudioResource,
    gain: f32,
    pan: f32,
    audio: SceneAudio,
}

impl AudioHandle {
    pub fn fingerprint(&self) -> String {
        format!("{:?}|{}|{}", self.source, self.gain, self.pan)
    }
}

fn playback_options(
    options: &Table,
    mut gain: f32,
    mut pan: f32,
    source: bool,
) -> Result<(f32, f32)> {
    for pair in options.clone().pairs::<String, mlua::Value>() {
        let (name, _) = pair?;
        match name.as_str() {
            "source" if source => (),
            "gain" => gain = options.get("gain")?,
            "pan" => pan = options.get("pan")?,
            _ => {
                return Err(mlua::Error::runtime(format!(
                    "Unknown audio playback option '{name}'."
                )));
            }
        }
    }
    if !gain.is_finite()
        || !(0.0..=1.0).contains(&gain)
        || !pan.is_finite()
        || !(-1.0..=1.0).contains(&pan)
    {
        return Err(mlua::Error::runtime(
            "Audio gain must be 0–1 and pan must be -1–1; both must be finite.",
        ));
    }
    Ok((gain, pan))
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

pub(super) fn create(runtime: Rc<Runtime>, options: Table) -> Result<AudioHandle> {
    runtime.construction()?;
    super::runtime::engine(|| runtime.animator.borrow().assert_finite_scope())?;
    let source = options
        .get::<mlua::UserDataRef<AudioResource>>("source")?
        .clone();
    let (gain, pan) = playback_options(&options, 0.2, 0.0, true)?;
    Ok(AudioHandle {
        runtime: Rc::downgrade(&runtime),
        source,
        gain,
        pan,
        audio: runtime.audio.clone(),
    })
}

impl UserData for AudioHandle {
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
            let (gain, pan) = if let Some(options) = options {
                playback_options(&options, this.gain, this.pan, false)?
            } else {
                (this.gain, this.pan)
            };
            let key = step
                .as_ref()
                .map(|step| (step.entity, step.frame, step.update_index));
            let pending = step.as_ref().map_or(0, |step| step.events.len());
            this.audio
                .borrow()
                .check_capacity(key, pending)
                .map_err(mlua::Error::runtime)?;
            let event = SoundEvent {
                time: step
                    .as_ref()
                    .map_or_else(|| runtime.animator.borrow().time(), |step| step.time)
                    as f64,
                samples: this.source.0.samples.clone(),
                offset: 0,
                frames: this.source.0.samples.len(),
                gain,
                pan,
            };
            if let Some(step) = step.as_mut() {
                step.events.push(event);
            } else {
                this.audio.borrow_mut().scheduled.push(event);
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
