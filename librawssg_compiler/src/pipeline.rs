use crate::ContextBuilder;
use crate::pattern::match_pattern;
use librawssg_config::Config;
use librawssg_error::{Error, Result};
use librawssg_fs::FileSystem;
use librawssg_handler::Document;
use librawssg_templates::Renderer;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct Pipeline {
    pub(crate) config: Config,
    pub(crate) fs: Box<dyn FileSystem>,
    pub(crate) renderer: Box<dyn Renderer>,
    pub(crate) processors: Vec<Box<dyn crate::processor::Processor>>,
    pub(crate) context_builder: Box<dyn ContextBuilder>,
    pub(crate) generators: Vec<Box<dyn crate::generator::Generator>>,
    pub(crate) content_dir: PathBuf,
    pub(crate) output_dir: PathBuf,
}

impl Pipeline {
    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    pub fn run(&self) -> Result<()> {
        let tmp_dir = self.output_dir.with_extension("tmp");
        if self.fs.exists(&tmp_dir) {
            self.fs.remove_dir_all(&tmp_dir)?;
        }
        self.fs.create_dir_all(&tmp_dir)?;

        self.generate_to(&tmp_dir)?;

        if self.fs.exists(&self.output_dir) {
            self.fs.remove_dir_all(&self.output_dir)?;
        }
        match self.fs.rename(&tmp_dir, &self.output_dir) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
                self.copy_dir_all(&tmp_dir, &self.output_dir)?;
                self.fs.remove_dir_all(&tmp_dir)?;
                Ok(())
            }
            Err(e) => Err(Error::Generation(format!("atomic rename failed: {e}"))),
        }
    }

    fn generate_to(&self, output_base: &Path) -> Result<()> {
        self.fs.create_dir_all(output_base)?;

        let documents = self.process_documents()?;

        let mut docs_by_type: HashMap<String, Vec<Document>> = HashMap::new();
        for doc in &documents {
            docs_by_type
                .entry(doc.content_type.clone())
                .or_default()
                .push(doc.clone());
        }

        for doc in &documents {
            if doc.is_list {
                continue;
            }
            self.render_document(output_base, doc)?;
        }

        for (content_type, docs) in &docs_by_type {
            if let Some(rule) = self
                .config
                .content_rules
                .iter()
                .find(|r| r.name == *content_type)
            {
                if rule.list_enabled && !docs.is_empty() {
                    if let Some(list_template) = &rule.list_template {
                        let list_doc = Document {
                            metadata: librawssg_handler::Metadata {
                                title: content_type.clone(),
                                ..Default::default()
                            },
                            body: String::new(),
                            url: format!("{}/index.html", content_type),
                            output_path: PathBuf::from(format!("{}/index.html", content_type)),
                            source_path: PathBuf::new(),
                            depth: 1,
                            content_type: content_type.clone(),
                            is_list: true,
                            list_items: Some(docs.clone()),
                            taxonomies: HashMap::new(),
                        };
                        self.render_document_with_template(output_base, &list_doc, list_template)?;
                    }
                }
            }
        }

        if self.fs.exists(Path::new(&self.config.build.static_dir)) {
            self.copy_dir_all(
                Path::new(&self.config.build.static_dir),
                &output_base.join(&self.config.build.static_dir),
            )?;
        }

        for generator in &self.generators {
            generator.generate(self)?;
        }

        Ok(())
    }

    fn process_documents(&self) -> Result<Vec<Document>> {
        let mut docs = Vec::new();
        if !self.fs.exists(&self.content_dir) {
            return Ok(docs);
        }
        let files = self.fs.walk_dir(&self.content_dir)?;
        for file_path in files {
            let rel = file_path
                .strip_prefix(&self.content_dir)
                .map_err(|e| Error::Generation(e.to_string()))?;
            for processor in &self.processors {
                if processor.can_process(rel, &file_path) {
                    if let Some(mut doc) = processor.process(&*self.fs, rel, &self.content_dir)? {
                        doc.content_type = self.determine_content_type(rel);
                        doc.depth = rel.components().count().saturating_sub(1);
                        docs.push(doc);
                    }
                    break;
                }
            }
        }
        Ok(docs)
    }

    fn determine_content_type(&self, rel: &Path) -> String {
        for rule in &self.config.content_rules {
            if match_pattern(&rule.pattern, rel) {
                return rule.name.clone();
            }
        }
        "page".into()
    }

    fn render_document(&self, output_base: &Path, doc: &Document) -> Result<()> {
        let template = self.template_for_document(doc)?;
        self.render_document_with_template(output_base, doc, &template)
    }

    fn render_document_with_template(
        &self,
        output_base: &Path,
        doc: &Document,
        template: &str,
    ) -> Result<()> {
        let ctx = self.context_builder.build_context(&self.config, doc)?;
        let html = self.renderer.render(template, &*ctx)?;
        self.write_output(output_base, doc, html.as_bytes())
    }

    fn template_for_document(&self, doc: &Document) -> Result<String> {
        for rule in &self.config.content_rules {
            if rule.name == doc.content_type {
                if doc.is_list
                    && let Some(ref list_template) = rule.list_template
                {
                    return Ok(list_template.clone());
                }
                return Ok(rule.template.clone());
            }
        }
        Err(Error::Generation(format!(
            "no content rule found for type '{}'",
            doc.content_type
        )))
    }

    fn write_output(&self, output_base: &Path, doc: &Document, content: &[u8]) -> Result<()> {
        let rel_out = Path::new(&doc.output_path);
        if let Some(parent) = rel_out.parent() {
            self.fs.create_dir_all(&output_base.join(parent))?;
        }
        let dest = self
            .fs
            .safe_join(output_base, rel_out)
            .map_err(|e| Error::Generation(format!("unsafe output path: {e}")))?;
        self.fs
            .write(&dest, content)
            .map_err(|e| Error::Generation(format!("write output failed: {e}")))
    }

    fn copy_dir_all(&self, from: &Path, to: &Path) -> Result<()> {
        if !self.fs.exists(from) {
            return Ok(());
        }
        self.fs.create_dir_all(to)?;
        for entry in self.fs.walk_dir(from)? {
            let rel = entry
                .strip_prefix(from)
                .map_err(|e| Error::Generation(e.to_string()))?;
            let dest = to.join(rel);
            if self.fs.is_dir(&entry) {
                self.fs.create_dir_all(&dest)?;
            } else {
                if let Some(parent) = dest.parent() {
                    self.fs.create_dir_all(parent)?;
                }
                let _ = self.fs.copy_file(&entry, &dest)?;
            }
        }
        Ok(())
    }
}
