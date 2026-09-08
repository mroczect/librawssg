pub mod real;

use std::io;
use std::path::{Path, PathBuf};

use tracing as _;
use walkdir as _;

pub trait FileSystem: Send + Sync {
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
    fn read_bytes(&self, path: &Path) -> io::Result<Vec<u8>>;
    fn write(&self, path: &Path, content: &[u8]) -> io::Result<()>;
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    fn remove_dir_all(&self, path: &Path) -> io::Result<()>;
    fn remove_file(&self, path: &Path) -> io::Result<()>;
    fn create_dir(&self, path: &Path) -> io::Result<()>;
    fn exists(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;
    fn is_file(&self, path: &Path) -> bool;
    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;
    fn copy_file(&self, from: &Path, to: &Path) -> io::Result<u64>;
    fn copy_dir_all(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn walk_dir(&self, root: &Path) -> io::Result<Vec<PathBuf>>;
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf>;
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn atomic_write(&self, path: &Path, content: &[u8]) -> io::Result<()>;
    fn touch(&self, path: &Path) -> io::Result<()>;
    fn metadata(&self, path: &Path) -> io::Result<std::fs::Metadata>;
    fn symlink_metadata(&self, path: &Path) -> io::Result<std::fs::Metadata>;
    fn permissions(&self, path: &Path) -> io::Result<std::fs::Permissions>;
    fn set_permissions(&self, path: &Path, permissions: std::fs::Permissions) -> io::Result<()>;
    fn read_link(&self, path: &Path) -> io::Result<PathBuf>;
    fn hard_link(&self, from: &Path, to: &Path) -> io::Result<()>;

    fn is_symlink(&self, path: &Path) -> bool {
        self.symlink_metadata(path)
            .is_ok_and(|meta| meta.file_type().is_symlink())
    }

    fn canonicalize_or_join(&self, base: &Path, candidate: &Path) -> io::Result<PathBuf> {
        let mut components: Vec<std::path::Component<'_>> = Vec::new();
        for comp in candidate.components() {
            match comp {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    if components.is_empty() {
                        return Err(io::Error::new(
                            io::ErrorKind::PermissionDenied,
                            "path traversal detected",
                        ));
                    }
                    if matches!(components.last(), Some(std::path::Component::Normal(_))) {
                        let _ = components.pop();
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::PermissionDenied,
                            "path traversal detected",
                        ));
                    }
                }
                std::path::Component::Prefix(_)
                | std::path::Component::RootDir
                | std::path::Component::Normal(_) => components.push(comp),
            }
        }
        let normalized_candidate: PathBuf = components.into_iter().collect();
        let joined = base.join(normalized_candidate);

        if self.exists(&joined) {
            self.canonicalize(&joined)
        } else {
            let parent = joined
                .parent()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no parent"))?;
            let parent_canon = self.canonicalize(parent)?;
            let file_name = joined.file_name().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "path has no file name")
            })?;
            Ok(parent_canon.join(file_name))
        }
    }

    fn safe_join(&self, base: &Path, candidate: &Path) -> io::Result<PathBuf> {
        let base_canon = self.canonicalize(base)?;
        let joined = self.canonicalize_or_join(&base_canon, candidate)?;
        if !joined.starts_with(&base_canon) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "path traversal detected",
            ));
        }
        Ok(joined)
    }

    fn copy(&self, from: &Path, to: &Path) -> io::Result<()> {
        if self.is_dir(from) {
            self.copy_dir_all(from, to)
        } else {
            self.copy_file(from, to).map(|_| ())
        }
    }

    fn rename_or_copy(&self, from: &Path, to: &Path) -> io::Result<()> {
        match self.rename(from, to) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::CrossesDevices => {
                self.copy_dir_all(from, to)?;
                self.remove_dir_all(from)
            }
            Err(e) => Err(e),
        }
    }
}

pub use real::RealFs;

#[cfg(test)]
use tempfile as _;
