use crate::renderer::{RenderContext, Renderer};
use librawssg_error::{Error, Result};
use std::path::Path;

#[derive(Debug)]
pub struct TeraRenderer {
    tera: tera::Tera,
}

impl TeraRenderer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tera: tera::Tera::default(),
        }
    }

    pub fn add_raw_template(&mut self, name: &str, content: &str) -> Result<()> {
        self.tera
            .add_raw_template(name, content)
            .map_err(|e| Error::Render(e.to_string()))
    }

    pub fn add_template_file(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Io(std::io::Error::other(format!("{e}"))))?;
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Error::Render("template file has no valid file name".into()))?;
        self.add_raw_template(name, &content)
    }

    pub fn add_template_files_from_dir(&mut self, dir: &Path) -> Result<()> {
        let entries =
            std::fs::read_dir(dir).map_err(|e| Error::Io(std::io::Error::other(format!("{e}"))))?;
        for entry in entries {
            let entry = entry.map_err(|e| Error::Io(std::io::Error::other(format!("{e}"))))?;
            let path = entry.path();
            if path.is_file() {
                self.add_template_file(&path)?;
            }
        }
        Ok(())
    }

    pub fn load_templates_dir(&mut self, dir: &Path) -> Result<()> {
        let dir_canon = dir
            .canonicalize()
            .map_err(|e| Error::Io(std::io::Error::other(format!("{e}"))))?;
        for entry in walkdir::WalkDir::new(&dir_canon) {
            let entry = entry.map_err(|e| Error::Io(std::io::Error::other(format!("{e}"))))?;
            if entry.file_type().is_file() {
                let abs_path = entry.path();
                let rel_path = abs_path
                    .strip_prefix(&dir_canon)
                    .map_err(|e| Error::Io(std::io::Error::other(format!("{e}"))))?;
                let template_name = rel_path_to_template_name(rel_path)?;
                let content = std::fs::read_to_string(abs_path)
                    .map_err(|e| Error::Io(std::io::Error::other(format!("{e}"))))?;
                self.add_raw_template(&template_name, &content)?;
            }
        }
        Ok(())
    }

    pub fn enable_autoescape(&mut self) {
        self.tera.autoescape_on(vec!["html", "htm", "xml"]);
    }

    pub fn render_str(&self, template_str: &str, context: &dyn RenderContext) -> Result<String> {
        let tera_ctx = context
            .as_any()
            .downcast_ref::<tera::Context>()
            .ok_or_else(|| Error::Render("invalid context type for Tera".into()))?;
        tera::Tera::one_off(template_str, tera_ctx, true).map_err(|e| Error::Render(e.to_string()))
    }

    #[must_use]
    pub const fn as_tera(&self) -> &tera::Tera {
        &self.tera
    }

    #[must_use]
    pub const fn as_tera_mut(&mut self) -> &mut tera::Tera {
        &mut self.tera
    }
}

fn rel_path_to_template_name(rel_path: &Path) -> Result<String> {
    let mut parts: Vec<&str> = Vec::new();
    for component in rel_path.components() {
        match component {
            std::path::Component::Normal(os_str) => parts.push(
                os_str
                    .to_str()
                    .ok_or_else(|| Error::Render("non-UTF-8 path".into()))?,
            ),
            std::path::Component::Prefix(_)
            | std::path::Component::RootDir
            | std::path::Component::CurDir
            | std::path::Component::ParentDir => {
                return Err(Error::Render(
                    "unexpected path component in template path".into(),
                ));
            }
        }
    }
    if parts.is_empty() {
        return Err(Error::Render("empty template name".into()));
    }
    Ok(parts.join("/"))
}

impl Default for TeraRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for TeraRenderer {
    fn render(&self, template_name: &str, context: &dyn RenderContext) -> Result<String> {
        let tera_ctx = context
            .as_any()
            .downcast_ref::<tera::Context>()
            .ok_or_else(|| Error::Render("invalid context type for Tera".into()))?;
        self.tera
            .render(template_name, tera_ctx)
            .map_err(|e| Error::Render(e.to_string()))
    }
}

impl RenderContext for tera::Context {
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn core::any::Any {
        self
    }
}
