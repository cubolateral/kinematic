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
    pub(crate) audio: BTreeMap<PathBuf, Vec<AudioClip>>,
    pub resolution: (u32, u32),
    pub fps: u32,
    pub margin: (u32, u32),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AudioClip {
    pub id: u64,
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
            audio,
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
                    .audio
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

        for clips in self.audio.values() {
            let mut ids = BTreeSet::new();
            if clips.len() > 100_000 {
                return Err("Scene exceeds 100000 audio clips.".into());
            }
            for clip in clips {
                clip.validate(context)?;
                if !ids.insert(clip.id) {
                    return Err("Audio clip IDs must be unique within a scene.".into());
                }
            }
        }
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
            audio: BTreeMap::new(),
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
                .and_then(|mut scene| {
                    scene.set_audio_clips(
                        self.manifest.audio.get(path).map_or(&[], Vec::as_slice),
                    )?;
                    Ok(scene)
                })
                .map_err(|e| e.to_string())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_and_audio_scene_entries_round_trip_without_losing_clips() {
        let legacy = include_str!("../../examples/lua/2d/kinematic/project.ron");
        let mut manifest: Manifest = ron::from_str(legacy).unwrap();
        assert!(manifest.audio.is_empty());
        let clip = AudioClip {
            id: 42,
            file: "assets/chime.wav".into(),
            at: 0.25,
            offset: 0.05,
            duration: 0.15,
            gain: 0.5,
            pan: -0.5,
        };
        manifest
            .audio
            .insert(manifest.scenes[0].clone(), vec![clip.clone()]);
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
            file: "assets/chime.wav".into(),
            at: 0.25,
            offset: 0.05,
            duration: 0.15,
            gain: 0.5,
            pan: -0.5,
        };
        project
            .manifest
            .audio
            .insert("scenes/intro.lua".into(), vec![clip.clone()]);
        project.save().unwrap();
        let project = LuaProject::open(&root).unwrap();
        assert_eq!(
            project.manifest.audio.values().next().unwrap(),
            &[clip.clone()]
        );
        let mut scene = project.load_scenes(&Sources::default()).unwrap().remove(0);
        {
            let audio = scene.scene.audio.borrow();
            assert!(std::sync::Arc::ptr_eq(
                &audio.clips[0].samples,
                &audio.scheduled[0].samples
            ));
        }
        let mix = |scene: &LuaScene| {
            let mut frames = vec![[0.0; 2]; 48_000];
            scene.scene.audio.borrow().mix(0.0, 0, &mut frames);
            frames
        };
        let expected = mix(&scene);
        scene.update(0.8).unwrap();
        scene.update(0.0).unwrap();
        let candidate = scene.prepare_seek().unwrap();
        scene.apply_reload(candidate);
        assert_eq!(scene.scene.audio.borrow().clips.len(), 1);
        assert_eq!(mix(&scene), expected);
        let mut clips = vec![clip.clone()];
        let right = clips[0].split(0.35, 2).unwrap();
        clips.push(right);
        scene.set_audio_clips(&clips).unwrap();
        assert_eq!(mix(&scene), expected);
        let mut invalid = clip.clone();
        invalid.offset = 200.0;
        assert!(scene.set_audio_clips(&[invalid]).is_err());
        assert_eq!(mix(&scene), expected);
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
