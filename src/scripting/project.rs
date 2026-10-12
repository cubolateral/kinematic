use super::{LuaScene, Sources};
use crate::core::{ProjectContext, ProjectSettings, atomic_write};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub const MANIFEST: &str = "kinematic.ron";

/// The single authoritative configuration for a Lua project.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: u32,
    pub name: String,
    pub scenes: Vec<PathBuf>,
    pub resolution: (u32, u32),
    pub fps: u32,
    pub margin: (u32, u32),
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
                .map_err(|e| e.to_string())
            })
            .collect()
    }
}
