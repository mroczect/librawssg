use super::FileSystem;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct RealFs;

impl FileSystem for RealFs {
    #[tracing::instrument(skip(self))]
    fn read_to_string(&self, path: &Path) -> io::Result<String> { fs::read_to_string(path) }

    #[tracing::instrument(skip(self))]
    fn read_bytes(&self, path: &Path) -> io::Result<Vec<u8>> { fs::read(path) }

    #[tracing::instrument(skip(self, content))]
    fn write(&self, path: &Path, content: &[u8]) -> io::Result<()> {
        if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
        fs::write(path, content)
    }

    #[tracing::instrument(skip(self))]
    fn create_dir_all(&self, path: &Path) -> io::Result<()> { fs::create_dir_all(path) }

    #[tracing::instrument(skip(self))]
    fn remove_dir_all(&self, path: &Path) -> io::Result<()> { fs::remove_dir_all(path) }

    #[tracing::instrument(skip(self))]
    fn exists(&self, path: &Path) -> bool { path.exists() }

    #[tracing::instrument(skip(self))]
    fn is_dir(&self, path: &Path) -> bool { path.is_dir() }

    #[tracing::instrument(skip(self))]
    fn is_file(&self, path: &Path) -> bool { path.is_file() }

    #[tracing::instrument(skip(self))]
    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(path)? {
            entries.push(entry?.path());
        }
        Ok(entries)
    }

    #[tracing::instrument(skip(self))]
    fn copy_file(&self, from: &Path, to: &Path) -> io::Result<u64> { fs::copy(from, to) }

    #[tracing::instrument(skip(self))]
    fn walk_dir(&self, root: &Path) -> io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for entry in WalkDir::new(root) {
            let entry = entry?;
            if entry.file_type().is_file() { files.push(entry.into_path()); }
        }
        Ok(files)
    }
}
