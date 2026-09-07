use super::FileSystem;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Default, Clone, Copy)]
pub struct RealFs;

impl FileSystem for RealFs {
    #[tracing::instrument(skip(self))]
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    #[tracing::instrument(skip(self))]
    fn read_bytes(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    #[tracing::instrument(skip(self, content))]
    fn write(&self, path: &Path, content: &[u8]) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            self.create_dir_all(parent)?;
        }
        fs::write(path, content)
    }

    #[tracing::instrument(skip(self))]
    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)
    }

    #[tracing::instrument(skip(self))]
    fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::remove_dir_all(path)
    }

    #[tracing::instrument(skip(self))]
    fn remove_file(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }

    #[tracing::instrument(skip(self))]
    fn create_dir(&self, path: &Path) -> io::Result<()> {
        fs::create_dir(path)
    }

    #[tracing::instrument(skip(self))]
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    #[tracing::instrument(skip(self))]
    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    #[tracing::instrument(skip(self))]
    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    #[tracing::instrument(skip(self))]
    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(path)? {
            entries.push(entry?.path());
        }
        Ok(entries)
    }

    #[tracing::instrument(skip(self))]
    fn copy_file(&self, from: &Path, to: &Path) -> io::Result<u64> {
        fs::copy(from, to)
    }

    #[tracing::instrument(skip(self))]
    fn copy_dir_all(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.create_dir_all(to)?;
        for entry in self.walk_dir(from)? {
            let rel = entry
                .strip_prefix(from)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            let dest = to.join(rel);
            if self.is_dir(&entry) {
                self.create_dir_all(&dest)?;
            } else {
                if let Some(parent) = dest.parent() {
                    self.create_dir_all(parent)?;
                }
                let _ = self.copy_file(&entry, &dest)?;
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn walk_dir(&self, root: &Path) -> io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for entry in WalkDir::new(root) {
            let entry = entry?;
            if entry.file_type().is_file() {
                files.push(entry.into_path());
            }
        }
        Ok(files)
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        path.canonicalize()
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }

    fn atomic_write(&self, path: &Path, content: &[u8]) -> io::Result<()> {
        let tmp = path.with_extension("tmp");
        self.write(&tmp, content)?;
        match self.rename(&tmp, path) {
            Ok(()) => Ok(()),
            Err(e) => {
                let _ = self.remove_file(&tmp);
                Err(e)
            }
        }
    }

    fn touch(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            self.create_dir_all(parent)?;
        }
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        file.sync_all()?;
        Ok(())
    }

    fn metadata(&self, path: &Path) -> io::Result<fs::Metadata> {
        fs::metadata(path)
    }

    fn symlink_metadata(&self, path: &Path) -> io::Result<fs::Metadata> {
        fs::symlink_metadata(path)
    }

    fn permissions(&self, path: &Path) -> io::Result<fs::Permissions> {
        fs::metadata(path).map(|m| m.permissions())
    }

    fn set_permissions(&self, path: &Path, permissions: fs::Permissions) -> io::Result<()> {
        fs::set_permissions(path, permissions)
    }

    fn read_link(&self, path: &Path) -> io::Result<PathBuf> {
        fs::read_link(path)
    }

    fn hard_link(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::hard_link(from, to)
    }
}
