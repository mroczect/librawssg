use chrono as _;
use librawssg_error as _;
use librawssg_fs as _;
use serde as _;
use serde_json as _;
#[test]
fn reexports_are_available() {
    let _ = librawssg_handler::Metadata::default();
    let _ = librawssg_handler::Document::new(
        librawssg_handler::Metadata::default(),
        "",
        "",
        "",
        "",
        0,
        "",
        false,
    );
    let _: Option<&dyn librawssg_handler::Processor> = None;
}
