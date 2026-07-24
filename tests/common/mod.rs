#![allow(dead_code, unused_imports)]

use librawssg::config::ConfigLoader;
use librawssg::error::RawssgError;
use librawssg::fs::FileSystem;
use librawssg::markdown::MarkdownRenderer;
use librawssg::site::TemplateRenderer;
use librawssg::types::RawssgConfig;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use tera::Context;

pub struct MockFs {
    pub files: HashMap<PathBuf, Vec<u8>>,
    pub dirs: Vec<PathBuf>,
    pub read_error: Option<PathBuf>,
}

impl MockFs {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            dirs: vec![],
            read_error: None,
        }
    }

    pub fn add_file(&mut self, path: &str, content: &str) {
        self.files
            .insert(PathBuf::from(path), content.as_bytes().to_vec());
        if let Some(parent) = Path::new(path).parent() {
            let mut p = PathBuf::new();
            for comp in parent.components() {
                p.push(comp);
                if !self.dirs.contains(&p) {
                    self.dirs.push(p.clone());
                }
            }
        }
    }
}

impl FileSystem for MockFs {
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        if let Some(err_path) = &self.read_error {
            if err_path == path {
                return Err(io::Error::new(io::ErrorKind::NotFound, "mock error"));
            }
        }
        self.files
            .get(path)
            .map(|b| String::from_utf8_lossy(b).to_string())
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file not found"))
    }

    fn read_bytes(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file not found"))
    }

    fn write(&self, _path: &Path, _content: &[u8]) -> io::Result<()> {
        Ok(())
    }
    fn create_dir_all(&self, _path: &Path) -> io::Result<()> {
        Ok(())
    }
    fn remove_dir_all(&self, _path: &Path) -> io::Result<()> {
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        self.files.contains_key(path) || self.dirs.contains(&path.to_path_buf())
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.dirs.contains(&path.to_path_buf())
    }
    fn is_file(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        let mut entries = vec![];
        for f in self.files.keys() {
            if let Some(parent) = f.parent() {
                if parent == path {
                    entries.push(f.clone());
                }
            }
        }
        for d in &self.dirs {
            if let Some(parent) = d.parent() {
                if parent == path {
                    entries.push(d.clone());
                }
            }
        }
        Ok(entries)
    }

    fn copy_file(&self, _from: &Path, _to: &Path) -> io::Result<u64> {
        Ok(0)
    }

    fn walk_dir(&self, root: &Path) -> io::Result<Vec<PathBuf>> {
        Ok(self
            .files
            .keys()
            .filter(|p| p.starts_with(root))
            .cloned()
            .collect())
    }
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        // Start from a virtual root "/"
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            Path::new("/").join(path)
        };

        // Resolve . and .. manually
        let mut components = Vec::new();
        for comp in absolute.components() {
            match comp {
                std::path::Component::ParentDir => {
                    components.pop(); // go up one level
                }
                std::path::Component::CurDir => {}
                other => components.push(other),
            }
        }
        let canonical: PathBuf = components.iter().collect();

        // Check whether the canonical path is known (file or dir)
        if self.files.contains_key(&canonical) || self.dirs.contains(&canonical) {
            Ok(canonical)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "mock: path not found",
            ))
        }
    }
}

// ---------- Mock MarkdownRenderer ----------
pub struct MockMarkdownRenderer {
    pub render_fn: Box<dyn Fn(&str) -> String + Send + Sync>,
}

impl MockMarkdownRenderer {
    pub fn new<F: Fn(&str) -> String + Send + Sync + 'static>(f: F) -> Self {
        Self {
            render_fn: Box::new(f),
        }
    }
    pub fn identity() -> Self {
        Self::new(|s| s.to_string())
    }
}

impl MarkdownRenderer for MockMarkdownRenderer {
    fn render(&self, markdown: &str) -> String {
        (self.render_fn)(markdown)
    }
}

// ---------- Mock TemplateRenderer ----------
pub struct MockTemplateRenderer {
    pub templates: HashMap<String, String>,
    pub render_fn: Box<dyn Fn(&str, &Context) -> Result<String, RawssgError> + Send + Sync>,
}

impl MockTemplateRenderer {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            render_fn: Box::new(|name, _ctx| Ok(format!("rendered:{}", name))),
        }
    }

    pub fn with_fn<F: Fn(&str, &Context) -> Result<String, RawssgError> + Send + Sync + 'static>(
        f: F,
    ) -> Self {
        Self {
            templates: HashMap::new(),
            render_fn: Box::new(f),
        }
    }
}

impl TemplateRenderer for MockTemplateRenderer {
    fn render(&self, template: &str, context: &Context) -> Result<String, RawssgError> {
        (self.render_fn)(template, context)
    }

    fn add_raw_template(&mut self, name: &str, content: &str) -> Result<(), RawssgError> {
        self.templates.insert(name.to_string(), content.to_string());
        Ok(())
    }
}

// ---------- Mock ConfigLoader ----------
pub struct MockConfigLoader {
    pub config_fn: Box<dyn Fn() -> Result<RawssgConfig, RawssgError> + Send + Sync>,
}

impl MockConfigLoader {
    pub fn new<F: Fn() -> Result<RawssgConfig, RawssgError> + Send + Sync + 'static>(f: F) -> Self {
        Self {
            config_fn: Box::new(f),
        }
    }
}

impl ConfigLoader for MockConfigLoader {
    fn load(&self) -> Result<RawssgConfig, RawssgError> {
        (self.config_fn)()
    }
}

// Helper to create a default config for tests
pub fn default_test_config() -> RawssgConfig {
    RawssgConfig::default()
}
