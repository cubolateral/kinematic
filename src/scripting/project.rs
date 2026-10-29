use super::{LuaScene, Sources};
use crate::core::{ProjectContext, ProjectSettings, atomic_write};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

pub const MANIFEST: &str = "kinematic/project.ron";

/// The single authoritative configuration for a Lua project.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(from = "ManifestData", into = "ManifestData")]
pub struct Manifest {
    pub version: u32,
    pub name: String,
    pub scenes: Vec<PathBuf>,
    pub(crate) audio: Vec<AudioClip>,
    pub(crate) legacy_audio: BTreeMap<PathBuf, Vec<AudioClip>>,
    pub resolution: (u32, u32),
    pub fps: u32,
    pub margin: (u32, u32),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AudioClip {
    pub id: u64,
    #[serde(default)]
    pub lane: usize,
    pub file: PathBuf,
    pub at: f64,
    pub offset: f64,
    pub duration: f64,
    pub gain: f32,
    pub pan: f32,
}

impl AudioClip {
    pub(crate) fn validate(&self, context: &ProjectContext) -> Result<(), String> {
        context.resolve(&self.file).map_err(|e| e.to_string())?;
        if ![
            self.at,
            self.offset,
            self.duration,
            self.at + self.duration,
            self.offset + self.duration,
        ]
        .iter()
        .all(|v| v.is_finite())
            || self.at + self.duration
                > i64::MAX as f64 / crate::core::audio::SAMPLE_RATE as f64 / 2.0
            || self.at < 0.0
            || self.offset < 0.0
            || self.duration <= 0.0
            || !self.gain.is_finite()
            || !(0.0..=1.0).contains(&self.gain)
            || !self.pan.is_finite()
            || !(-1.0..=1.0).contains(&self.pan)
        {
            return Err("Invalid audio clip times, gain or pan.".into());
        }
        Ok(())
    }

    pub(crate) fn split(&mut self, time: f64, id: u64) -> Option<Self> {
        let rate = crate::core::audio::SAMPLE_RATE as f64;
        let frames = ((self.offset + self.duration) * rate).round() - (self.offset * rate).round();
        let left_frames = ((time - self.at) * rate).round();
        if !left_frames.is_finite() || left_frames <= 0.0 || left_frames >= frames {
            return None;
        }
        let left = left_frames / rate;
        let mut right = self.clone();
        right.id = id;
        right.at += left;
        right.offset += left;
        right.duration -= left;
        self.duration = left;
        Some(right)
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestData {
    version: u32,
    name: String,
    scenes: Vec<SceneEntry>,
    #[serde(default)]
    audio: Vec<AudioClip>,
    resolution: (u32, u32),
    fps: u32,
    margin: (u32, u32),
}

#[derive(Clone, Serialize)]
#[serde(untagged)]
enum SceneEntry {
    Path(PathBuf),
    Audio {
        file: PathBuf,
        audio: Vec<AudioClip>,
    },
}

impl<'de> Deserialize<'de> for SceneEntry {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = SceneEntry;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("A Lua path or scene with audio clips.")
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(SceneEntry::Path(value.into()))
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                map: M,
            ) -> Result<Self::Value, M::Error> {
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Entry {
                    file: PathBuf,
                    #[serde(default)]
                    audio: Vec<AudioClip>,
                }
                let entry = Entry::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                Ok(SceneEntry::Audio {
                    file: entry.file,
                    audio: entry.audio,
                })
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}

impl From<ManifestData> for Manifest {
    fn from(data: ManifestData) -> Self {
        let mut audio = BTreeMap::new();
        let scenes = data
            .scenes
            .into_iter()
            .map(|entry| match entry {
                SceneEntry::Path(path) => path,
                SceneEntry::Audio { file, audio: clips } => {
                    audio.insert(file.clone(), clips);
                    file
                }
            })
            .collect();
        Self {
            version: data.version,
            name: data.name,
            scenes,
            audio: data.audio,
            legacy_audio: audio,
            resolution: data.resolution,
            fps: data.fps,
            margin: data.margin,
        }
    }
}

impl From<Manifest> for ManifestData {
    fn from(mut manifest: Manifest) -> Self {
        let scenes = manifest
            .scenes
            .into_iter()
            .map(|file| {
                match manifest
                    .legacy_audio
                    .remove(&file)
                    .filter(|clips| !clips.is_empty())
                {
                    Some(audio) => SceneEntry::Audio { file, audio },
                    None => SceneEntry::Path(file),
                }
            })
            .collect();
        Self {
            version: manifest.version,
            name: manifest.name,
            scenes,
            audio: manifest.audio,
            resolution: manifest.resolution,
            fps: manifest.fps,
            margin: manifest.margin,
        }
    }
}

impl Manifest {
    pub fn validate(&self, context: &ProjectContext) -> Result<(), String> {
        if self.version != 1 {
            return Err(format!("Unsupported project format {}.", self.version));
        }

        if self.name.is_empty()
            || self.name.contains(['/', '\\', '\0'])
            || self.name == "."
            || self.name == ".."
        {
            return Err("Project name must be a nonempty filename.".into());
        }

        if !ProjectSettings::is_valid(&self.settings()) {
            return Err("Invalid resolution, FPS or margins.".into());
        }

        if self.scenes.is_empty() {
            return Err("Project needs at least one scene.".into());
        }

        let mut paths = BTreeSet::new();

        for path in &self.scenes {
            context.resolve(path).map_err(|e| e.to_string())?;

            if path.extension().is_none_or(|e| e != "lua") || !paths.insert(path) {
                return Err("Scene paths must be unique Lua files.".into());
            }
        }

        for clips in std::iter::once(&self.audio).chain(self.legacy_audio.values()) {
            let mut ids = BTreeSet::new();
            if clips.len() > 100_000 {
                return Err("Project exceeds 100000 audio clips.".into());
            }
            for clip in clips {
                clip.validate(context)?;
                if !ids.insert(clip.id) {
                    return Err("Audio clip IDs must be unique.".into());
                }
            }
        }
        Ok(())
    }

    pub(crate) fn migrate_audio(&mut self, durations: &[f32]) -> Result<(), String> {
        if self.legacy_audio.is_empty() {
            return Ok(());
        }
        let mut clips = self.audio.clone();
        let mut ids = clips.iter().map(|clip| clip.id).collect::<BTreeSet<_>>();
        let mut next = 0;
        let mut start: f32 = 0.0;
        for (path, duration) in self.scenes.iter().zip(durations) {
            for clip in self.legacy_audio.get(path).into_iter().flatten() {
                let mut clip = clip.clone();
                clip.at += start as f64;
                if ids.contains(&clip.id) {
                    while ids.contains(&next) {
                        next += 1;
                    }
                    clip.id = next;
                }
                ids.insert(clip.id);
                clips.push(clip);
            }
            start += *duration;
        }
        if clips.len() > 100_000 {
            return Err("Project exceeds 100000 audio clips.".into());
        }
        self.audio = clips;
        self.legacy_audio.clear();
        Ok(())
    }

    pub(crate) fn settings(&self) -> ProjectSettings {
        ProjectSettings {
            resolution: self.resolution,
            fps: self.fps,
            margin: self.margin,
        }
    }
}

/// An opened project retains the manifest revision for conflict-aware saves.
pub struct LuaProject {
    pub context: ProjectContext,
    pub manifest: Manifest,
    saved: Vec<u8>,
}
impl LuaProject {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, String> {
        let context = ProjectContext::new(root).map_err(|e| e.to_string())?;
        let saved = std::fs::read(context.resolve(MANIFEST).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let manifest: Manifest = ron::de::from_bytes(&saved).map_err(|e| e.to_string())?;

        manifest.validate(&context)?;

        Ok(Self {
            context,
            manifest,
            saved,
        })
    }

    pub fn create(
        root: impl AsRef<Path>,
        name: &str,
        three_dimensional: bool,
    ) -> Result<Self, String> {
        let root = root.as_ref();

        if root.exists()
            && std::fs::read_dir(root)
                .map_err(|e| e.to_string())?
                .next()
                .is_some()
        {
            return Err("Create a project in an empty directory.".into());
        }

        std::fs::create_dir_all(root).map_err(|e| e.to_string())?;

        let context = ProjectContext::new(root).map_err(|e| e.to_string())?;
        let manifest = Manifest {
            version: 1,
            name: name.into(),
            scenes: vec!["scenes/intro.lua".into()],
            audio: Vec::new(),
            legacy_audio: BTreeMap::new(),
            resolution: (1280, 720),
            fps: 60,
            margin: (0, 0),
        };

        manifest.validate(&context)?;

        let script = if three_dimensional {
            include_str!("../../examples/lua/3d/scenes/intro.lua")
        } else {
            include_str!("../../examples/lua/2d/scenes/intro.lua")
        };

        atomic_write(
            &context
                .resolve("scenes/intro.lua")
                .map_err(|e| e.to_string())?,
            script.as_bytes(),
            None,
        )
        .map_err(|e| e.to_string())?;

        std::fs::create_dir_all(context.root().join("assets")).map_err(|e| e.to_string())?;

        let saved = ron::ser::to_string_pretty(&manifest, ron::ser::PrettyConfig::default())
            .map_err(|e| e.to_string())?
            .into_bytes();

        atomic_write(
            &context.resolve(MANIFEST).map_err(|e| e.to_string())?,
            &saved,
            None,
        )
        .map_err(|e| e.to_string())?;

        Ok(Self {
            context,
            manifest,
            saved,
        })
    }

    pub(crate) fn accept_manifest(&mut self, manifest: Manifest) {
        self.manifest = manifest;

        if let Ok(bytes) = std::fs::read(self.context.root().join(MANIFEST)) {
            self.saved = bytes;
        }
    }

    pub(crate) fn saved_manifest(&self) -> Result<Manifest, String> {
        ron::de::from_bytes(&self.saved).map_err(|error| error.to_string())
    }

    pub fn save(&mut self) -> Result<(), String> {
        self.manifest.validate(&self.context)?;

        let contents =
            ron::ser::to_string_pretty(&self.manifest, ron::ser::PrettyConfig::default())
                .map_err(|e| e.to_string())?
                .into_bytes();

        atomic_write(
            &self.context.resolve(MANIFEST).map_err(|e| e.to_string())?,
            &contents,
            Some(&self.saved),
        )
        .map_err(|e| e.to_string())?;

        self.saved = contents;

        Ok(())
    }

    pub fn load_scenes(&self, sources: &Sources) -> Result<Vec<LuaScene>, String> {
        self.manifest
            .scenes
            .iter()
            .map(|path| {
                LuaScene::load(
                    self.context.clone(),
                    path,
                    sources.clone(),
                    self.manifest.resolution,
                    self.manifest.fps,
                    self.manifest.margin,
                )
                .map_err(|e| e.to_string())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_audio_clips_default_to_the_first_lane() {
        let mut clip: AudioClip = ron::from_str(r#"(id: 1, file: "assets/a.wav", at: 0.0, offset: 0.0, duration: 1.0, gain: 1.0, pan: 0.0)"#).unwrap();
        assert_eq!(clip.lane, 0);
        clip.lane = 2;
        assert_eq!(clip.split(0.5, 2).unwrap().lane, 2);
    }

    #[test]
    fn legacy_audio_migrates_to_absolute_time_with_unique_ids() {
        let mut manifest: Manifest =
            ron::from_str(include_str!("../../examples/lua/2d/kinematic/project.ron")).unwrap();
        manifest.scenes = vec!["a.lua".into(), "b.lua".into()];
        let clip: AudioClip = ron::from_str(r#"(id: 1, file: "assets/a.wav", at: 0.25, offset: 0.0, duration: 10.0, gain: 1.0, pan: 0.0)"#).unwrap();
        manifest
            .legacy_audio
            .insert("a.lua".into(), vec![clip.clone()]);
        manifest.legacy_audio.insert("b.lua".into(), vec![clip]);
        let serialized = ron::to_string(&manifest).unwrap();
        let mut manifest: Manifest = ron::from_str(&serialized).unwrap();
        manifest.migrate_audio(&[2.0, 3.0]).unwrap();
        assert_eq!(
            manifest
                .audio
                .iter()
                .map(|clip| clip.at)
                .collect::<Vec<_>>(),
            [0.25, 2.25]
        );
        assert_ne!(manifest.audio[0].id, manifest.audio[1].id);
        assert!(manifest.legacy_audio.is_empty());
        let saved = ron::to_string(&manifest).unwrap();
        let restored: Manifest = ron::from_str(&saved).unwrap();
        assert_eq!(restored.audio, manifest.audio);
        assert!(restored.legacy_audio.is_empty());
    }

    #[test]
    fn legacy_and_audio_scene_entries_round_trip_without_losing_clips() {
        let legacy = include_str!("../../examples/lua/2d/kinematic/project.ron");
        let mut manifest: Manifest = ron::from_str(legacy).unwrap();
        assert!(manifest.audio.is_empty());
        let clip = AudioClip {
            id: 42,
            lane: 2,
            file: "assets/chime.wav".into(),
            at: 0.25,
            offset: 0.05,
            duration: 0.15,
            gain: 0.5,
            pan: -0.5,
        };
        manifest.audio.extend([clip.clone()]);
        manifest.scenes.push("other.lua".into());
        let text = ron::to_string(&manifest).unwrap();
        let restored: Manifest = ron::from_str(&text).unwrap();
        assert_eq!(restored.scenes, manifest.scenes);
        assert_eq!(restored.audio, manifest.audio);
        let mut left = clip.clone();
        assert!(left.split(left.at, 43).is_none());
        assert!(left.split(left.at + left.duration, 43).is_none());
        let right = left.split(0.35, 43).unwrap();
        assert_eq!(right.id, 43);
        assert_eq!(right.gain, clip.gain);
        assert_eq!(right.pan, clip.pan);
        assert!((left.offset + left.duration - right.offset).abs() < 1e-12);
        assert!((left.at + left.duration - right.at).abs() < 1e-12);
        assert!((left.duration + right.duration - clip.duration).abs() < 1e-12);
    }

    #[test]
    #[ignore = "Requires FFmpeg with WAV support."]
    fn ui_audio_survives_rebuilds_and_seek_and_reuses_lua_file_samples() {
        let root = std::env::temp_dir().join(format!("kinematic-ui-audio-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let mut project = LuaProject::create(&root, "Audio clips", false).unwrap();
        std::fs::write(root.join("scenes/intro.lua"), "return function(s) local f=k.audio_file('assets/chime.wav'); s:audio{source=f}:play(); s:wait(1) end").unwrap();
        std::fs::write(
            root.join("assets/chime.wav"),
            include_bytes!("../../examples/lua/audio/assets/chime.wav"),
        )
        .unwrap();
        let clip = AudioClip {
            id: 1,
            lane: 0,
            file: "assets/chime.wav".into(),
            at: 0.25,
            offset: 0.05,
            duration: 0.15,
            gain: 0.5,
            pan: -0.5,
        };
        project.manifest.audio.extend([clip.clone()]);
        project.save().unwrap();
        let project = LuaProject::open(&root).unwrap();
        assert_eq!(project.manifest.audio.as_slice(), &[clip.clone()]);
        let mut scene = project.load_scenes(&Sources::default()).unwrap().remove(0);
        let sources = Sources::default();
        let prepare = |clips: &[AudioClip]| {
            super::super::audio::clip_events(
                &project.context,
                &sources,
                clips,
                &crate::core::audio::AudioEvents::default(),
            )
        };
        let events = prepare(&project.manifest.audio).unwrap();
        assert!(scene.scene.audio.borrow().clips.is_empty());
        assert!(std::sync::Arc::ptr_eq(
            &events[0].samples,
            &scene.scene.audio.borrow().scheduled[0].samples
        ));
        scene.update(0.8).unwrap();
        scene.update(0.0).unwrap();
        let candidate = scene.prepare_seek().unwrap();
        scene.apply_reload(candidate);
        assert!(scene.scene.audio.borrow().clips.is_empty());
        assert!(std::sync::Arc::ptr_eq(
            &events[0].samples,
            &prepare(&project.manifest.audio).unwrap()[0].samples
        ));
        let mut invalid = clip.clone();
        invalid.offset = 200.0;
        assert!(prepare(&[invalid]).is_err());
        let mut invalid = clip;
        invalid.pan = f32::NAN;
        assert!(invalid.validate(&project.context).is_err());
        std::fs::write(root.join(MANIFEST), "External changes.").unwrap();
        let mut project = project;
        assert!(project.save().is_err());
        assert_eq!(
            std::fs::read_to_string(root.join(MANIFEST)).unwrap(),
            "External changes."
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
