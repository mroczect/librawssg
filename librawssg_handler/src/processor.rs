use crate::Document;
use librawssg_error::Result;
use librawssg_fs::FileSystem;
use std::path::Path;

pub trait Processor: Send + Sync {
    fn name(&self) -> &str;

    fn priority(&self) -> i32 {
        0
    }

    fn can_process(&self, relative_path: &Path, original_path: &Path) -> bool;

    fn process(
        &self,
        fs: &dyn FileSystem,
        relative_path: &Path,
        content_dir: &Path,
    ) -> Result<Option<Document>>;
}
