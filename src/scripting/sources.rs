use crate::core::ProjectContext;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

/// One source revision. Unsaved text wins over disk; file contents are frozen on first read.
///
/// Clone this value to share a revision between scenes. Start a new value for a disk reload.
#[derive(Clone, Default)]
pub struct Sources {
    text: BTreeMap<PathBuf, String>,
    files: Rc<RefCell<BTreeMap<PathBuf, Arc<[u8]>>>>,
}
impl Sources {
    pub(crate) fn read(&self, context: &ProjectContext, path: &Path) -> std::io::Result<Vec<u8>> {
        self.read_resource(context, path, true)
    }

    pub(crate) fn read_audio(
        &self,
        context: &ProjectContext,
        path: &Path,
    ) -> std::io::Result<Vec<u8>> {
        self.read_resource(context, path, false)
    }

    fn read_resource(
        &self,
        context: &ProjectContext,
        path: &Path,
        limited: bool,
    ) -> std::io::Result<Vec<u8>> {
        let full = context.resolve(path)?;

        if let Some(text) = self.text.get(path) {
            return Ok(text.as_bytes().to_vec());
        }

        if let Some(bytes) = self.files.borrow().get(&full) {
            if limited && bytes.len() > 64 * 1024 * 1024 {
                return Err(std::io::Error::other("Resource exceeds 64 MiB."));
            }
            return Ok(bytes.to_vec());
        }

        let metadata = std::fs::metadata(&full)?;

        if limited && metadata.len() > 64 * 1024 * 1024 {
            return Err(std::io::Error::other("Resource exceeds 64 MiB."));
        }

        let bytes = std::fs::read(&full)?;

        if limited && bytes.len() > 64 * 1024 * 1024 {
            return Err(std::io::Error::other("Resource exceeds 64 MiB."));
        }

        self.files
            .borrow_mut()
            .insert(full, Arc::from(bytes.as_slice()));

        Ok(bytes)
    }
}
impl std::ops::Deref for Sources {
    type Target = BTreeMap<PathBuf, String>;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}
impl std::ops::DerefMut for Sources {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.text
    }
}
impl FromIterator<(PathBuf, String)> for Sources {
    fn from_iter<I: IntoIterator<Item = (PathBuf, String)>>(iter: I) -> Self {
        Self {
            text: iter.into_iter().collect(),
            ..Self::default()
        }
    }
}
impl<const N: usize> From<[(PathBuf, String); N]> for Sources {
    fn from(value: [(PathBuf, String); N]) -> Self {
        value.into_iter().collect()
    }
}
impl From<BTreeMap<PathBuf, String>> for Sources {
    fn from(text: BTreeMap<PathBuf, String>) -> Self {
        Self {
            text,
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn large_audio_does_not_bypass_the_limit_for_other_resources() {
        let root =
            std::env::temp_dir().join(format!("kinematic-audio-source-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let file = root.join("large.wav");
        let size = 64 * 1024 * 1024 + 1;
        std::fs::File::create(&file).unwrap().set_len(size).unwrap();
        let context = ProjectContext::new(&root).unwrap();
        let sources = Sources::default();
        let path = Path::new("large.wav");
        assert!(sources.read(&context, path).is_err());
        assert_eq!(
            sources.read_audio(&context, path).unwrap().len() as u64,
            size
        );
        assert!(sources.read(&context, path).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
