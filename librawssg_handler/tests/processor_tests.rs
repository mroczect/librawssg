use chrono as _;
use librawssg_error::Result;
use librawssg_fs::FileSystem;
use librawssg_handler::{Document, Metadata, Processor};
use serde as _;
use serde_json as _;
use std::io;
use std::path::{Path, PathBuf};

macro_rules! must {
    ($result:expr, $context:expr) => {
        match $result {
            Ok(value) => value,
            Err(err) => {
                eprintln!("{} failed: {}", $context, err);
                std::process::exit(1);
            }
        }
    };
}

struct DummyFs;
impl FileSystem for DummyFs {
    fn read_to_string(&self, _path: &Path) -> io::Result<String> {
        Err(io::Error::other("not implemented"))
    }
    fn read_bytes(&self, _path: &Path) -> io::Result<Vec<u8>> {
        Err(io::Error::other("not implemented"))
    }
    fn write(&self, _path: &Path, _content: &[u8]) -> io::Result<()> {
        Err(io::Error::other("not implemented"))
    }
    fn create_dir_all(&self, _path: &Path) -> io::Result<()> {
        Err(io::Error::other("not implemented"))
    }
    fn remove_dir_all(&self, _path: &Path) -> io::Result<()> {
        Err(io::Error::other("not implemented"))
    }
    fn remove_file(&self, _path: &Path) -> io::Result<()> {
        Err(io::Error::other("not implemented"))
    }
    fn create_dir(&self, _path: &Path) -> io::Result<()> {
        Err(io::Error::other("not implemented"))
    }
    fn exists(&self, _path: &Path) -> bool {
        false
    }
    fn is_dir(&self, _path: &Path) -> bool {
        false
    }
    fn is_file(&self, _path: &Path) -> bool {
        false
    }
    fn read_dir(&self, _path: &Path) -> io::Result<Vec<PathBuf>> {
        Err(io::Error::other("not implemented"))
    }
    fn copy_file(&self, _from: &Path, _to: &Path) -> io::Result<u64> {
        Err(io::Error::other("not implemented"))
    }
    fn copy_dir_all(&self, _from: &Path, _to: &Path) -> io::Result<()> {
        Err(io::Error::other("not implemented"))
    }
    fn walk_dir(&self, _root: &Path) -> io::Result<Vec<PathBuf>> {
        Err(io::Error::other("not implemented"))
    }
    fn canonicalize(&self, _path: &Path) -> io::Result<PathBuf> {
        Err(io::Error::other("not implemented"))
    }
    fn rename(&self, _from: &Path, _to: &Path) -> io::Result<()> {
        Err(io::Error::other("not implemented"))
    }
    fn atomic_write(&self, _path: &Path, _content: &[u8]) -> io::Result<()> {
        Err(io::Error::other("not implemented"))
    }
    fn touch(&self, _path: &Path) -> io::Result<()> {
        Err(io::Error::other("not implemented"))
    }
    fn metadata(&self, _path: &Path) -> io::Result<std::fs::Metadata> {
        Err(io::Error::other("not implemented"))
    }
    fn symlink_metadata(&self, _path: &Path) -> io::Result<std::fs::Metadata> {
        Err(io::Error::other("not implemented"))
    }
    fn permissions(&self, _path: &Path) -> io::Result<std::fs::Permissions> {
        Err(io::Error::other("not implemented"))
    }
    fn set_permissions(&self, _path: &Path, _permissions: std::fs::Permissions) -> io::Result<()> {
        Err(io::Error::other("not implemented"))
    }
    fn read_link(&self, _path: &Path) -> io::Result<PathBuf> {
        Err(io::Error::other("not implemented"))
    }
    fn hard_link(&self, _from: &Path, _to: &Path) -> io::Result<()> {
        Err(io::Error::other("not implemented"))
    }
}

struct MockProcessor {
    name: String,
    priority: i32,
    can_process_result: bool,
    process_output: Option<Document>,
}

impl Processor for MockProcessor {
    fn name(&self) -> &str {
        &self.name
    }
    fn priority(&self) -> i32 {
        self.priority
    }
    fn can_process(&self, _relative_path: &Path, _original_path: &Path) -> bool {
        self.can_process_result
    }
    fn process(
        &self,
        _fs: &dyn FileSystem,
        _relative_path: &Path,
        _content_dir: &Path,
    ) -> Result<Option<Document>> {
        Ok(self.process_output.clone())
    }
}

fn sample_document() -> Document {
    let metadata = must!(Metadata::new("Sample", "Desc"), "Metadata::new");
    must!(
        Document::new(
            metadata,
            "<p>Body</p>",
            "sample.html",
            "sample/index.html",
            "content/sample.md",
            0,
            "page",
            false,
        ),
        "Document::new"
    )
}

#[test]
fn name_returns_set_name() {
    let p = MockProcessor {
        name: "test".to_string(),
        priority: 0,
        can_process_result: false,
        process_output: None,
    };
    assert_eq!(p.name(), "test");
}

#[test]
fn priority_default_zero() {
    let p = MockProcessor {
        name: "default".to_string(),
        priority: 0,
        can_process_result: false,
        process_output: None,
    };
    assert_eq!(p.priority(), 0);
}

#[test]
fn priority_custom() {
    let p = MockProcessor {
        name: "custom".to_string(),
        priority: 5,
        can_process_result: false,
        process_output: None,
    };
    assert_eq!(p.priority(), 5);
}

#[test]
fn can_process_returns_true() {
    let p = MockProcessor {
        name: "true".to_string(),
        priority: 0,
        can_process_result: true,
        process_output: None,
    };
    assert!(p.can_process(Path::new("a.md"), Path::new("content/a.md")));
}

#[test]
fn can_process_returns_false() {
    let p = MockProcessor {
        name: "false".to_string(),
        priority: 0,
        can_process_result: false,
        process_output: None,
    };
    assert!(!p.can_process(Path::new("a.md"), Path::new("content/a.md")));
}

#[test]
fn process_returns_document() {
    let doc = sample_document();
    let p = MockProcessor {
        name: "doc".to_string(),
        priority: 0,
        can_process_result: true,
        process_output: Some(doc.clone()),
    };
    let result = must!(
        p.process(&DummyFs, Path::new("a.md"), Path::new("content")),
        "process"
    );
    assert!(result.is_some());
    if let Some(processed) = result {
        assert_eq!(processed, doc);
    }
}

#[test]
fn process_returns_none() {
    let p = MockProcessor {
        name: "none".to_string(),
        priority: 0,
        can_process_result: true,
        process_output: None,
    };
    let result = must!(
        p.process(&DummyFs, Path::new("a.md"), Path::new("content")),
        "process"
    );
    assert!(result.is_none());
}

#[test]
fn process_error_case() {
    struct ErrorProcessor;
    impl Processor for ErrorProcessor {
        fn name(&self) -> &'static str {
            "error"
        }
        fn can_process(&self, _relative_path: &Path, _original_path: &Path) -> bool {
            true
        }
        fn process(
            &self,
            _fs: &dyn FileSystem,
            _relative_path: &Path,
            _content_dir: &Path,
        ) -> Result<Option<Document>> {
            Err(librawssg_error::Error::Processor("test error".into()))
        }
    }
    let p = ErrorProcessor;
    let result = p.process(&DummyFs, Path::new("a.md"), Path::new("content"));
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(
            err,
            librawssg_error::Error::Processor(ref msg) if msg == "test error"
        ));
    }
}
