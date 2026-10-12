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
        let full = context.resolve(path)?;
        if let Some(text) = self.text.get(path) {
            return Ok(text.as_bytes().to_vec());
        }
        if let Some(bytes) = self.files.borrow().get(&full) {
            return Ok(bytes.to_vec());
        }
        let metadata = std::fs::metadata(&full)?;
        if metadata.len() > 64 * 1024 * 1024 {
            return Err(std::io::Error::other("Resource exceeds 64 MiB."));
        }
        let bytes = std::fs::read(&full)?;
        if bytes.len() > 64 * 1024 * 1024 {
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
