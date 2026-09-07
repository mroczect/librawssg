use super::ContextBuilder;
use crate::generator::Generator;
use crate::pipeline::Pipeline;
use librawssg_config::Config;
use librawssg_error::Result;
use librawssg_fs::FileSystem;
use librawssg_fs::RealFs;
use librawssg_handler::Processor;
use librawssg_templates::Renderer;
use std::path::{Path, PathBuf};

#[allow(missing_debug_implementations)]
pub struct PipelineBuilder {
    config: Config,
    content_dir: PathBuf,
    output_dir: PathBuf,
    fs: Box<dyn FileSystem>,
    renderer: Option<Box<dyn Renderer>>,
    processors: Vec<Box<dyn Processor>>,
    context_builder: Option<Box<dyn ContextBuilder>>,
    generators: Vec<Box<dyn Generator>>,
}

impl PipelineBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            content_dir: PathBuf::from("content"),
            output_dir: PathBuf::from("dist"),
            fs: Box::new(RealFs),
            renderer: None,
            processors: Vec::new(),
            context_builder: None,
            generators: Vec::new(),
        }
    }

    #[must_use]
    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn load_config<P: AsRef<Path> + Send + Sync>(mut self, path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| librawssg_error::Error::Config(e.to_string()))?;
        self.config = Config::from_yaml_str(&content)?;
        Ok(self)
    }

    #[must_use]
    pub fn content_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.content_dir = dir.into();
        self
    }

    #[must_use]
    pub fn output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }

    #[must_use]
    pub fn with_fs(mut self, fs: Box<dyn FileSystem>) -> Self {
        self.fs = fs;
        self
    }

    #[must_use]
    pub fn with_renderer(mut self, renderer: Box<dyn Renderer>) -> Self {
        self.renderer = Some(renderer);
        self
    }

    #[must_use]
    pub fn add_processor(mut self, processor: Box<dyn Processor>) -> Self {
        self.processors.push(processor);
        self
    }

    #[must_use]
    pub fn with_context_builder(mut self, builder: Box<dyn ContextBuilder>) -> Self {
        self.context_builder = Some(builder);
        self
    }

    #[must_use]
    pub fn add_generator(mut self, generator: Box<dyn Generator>) -> Self {
        self.generators.push(generator);
        self
    }

    pub fn build(mut self) -> Result<Pipeline> {
        self.config.validate()?;

        if self.content_dir.as_path() == Path::new("content") {
            self.content_dir = PathBuf::from(&self.config.build.content_dir);
        }
        if self.output_dir.as_path() == Path::new("dist") {
            self.output_dir = PathBuf::from(&self.config.build.output_dir);
        }

        let renderer = self
            .renderer
            .take()
            .ok_or_else(|| librawssg_error::Error::Config("template renderer not set".into()))?;
        let context_builder = self
            .context_builder
            .take()
            .ok_or_else(|| librawssg_error::Error::Config("context builder not set".into()))?;

        Ok(Pipeline {
            config: self.config,
            fs: self.fs,
            renderer,
            processors: self.processors,
            context_builder,
            generators: self.generators,
            content_dir: self.content_dir,
            output_dir: self.output_dir,
        })
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
