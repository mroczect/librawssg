#![cfg(feature = "tera")]

use librawssg_templates::{RenderContext, Renderer, TeraRenderer};
use tempfile as _;
use walkdir as _;

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

fn sample_context() -> tera::Context {
    let mut ctx = tera::Context::new();
    ctx.insert("title", "Hello");
    ctx.insert("items", &vec!["a", "b", "c"]);
    ctx.insert("number", &42);
    ctx
}

#[test]
fn render_raw_template_with_simple_variable() {
    let mut renderer = TeraRenderer::new();
    must!(
        renderer.add_raw_template("simple", "{{ title }}"),
        "add_raw_template"
    );

    let mut ctx = tera::Context::new();
    ctx.insert("title", "Hello World");
    let output = must!(renderer.render("simple", &ctx), "render");
    assert_eq!(output, "Hello World");
}

#[test]
fn render_raw_template_with_loop() {
    let mut renderer = TeraRenderer::new();
    must!(
        renderer.add_raw_template(
            "loop",
            "{% for item in items %}{{ item }}{% if not loop.last %},{% endif %}{% endfor %}"
        ),
        "add_raw_template"
    );

    let ctx = sample_context();
    let output = must!(renderer.render("loop", &ctx), "render");
    assert_eq!(output, "a,b,c");
}

#[test]
fn render_raw_template_with_filter() {
    let mut renderer = TeraRenderer::new();
    must!(
        renderer.add_raw_template("filter", "{{ title | upper }}"),
        "add_raw_template"
    );

    let ctx = sample_context();
    let output = must!(renderer.render("filter", &ctx), "render");
    assert_eq!(output, "HELLO");
}

#[test]
fn render_raw_template_with_condition() {
    let mut renderer = TeraRenderer::new();
    must!(
        renderer.add_raw_template(
            "condition",
            "{% if number > 40 %}high{% else %}low{% endif %}"
        ),
        "add_raw_template"
    );

    let ctx = sample_context();
    let output = must!(renderer.render("condition", &ctx), "render");
    assert_eq!(output, "high");
}

#[test]
fn render_missing_template_errors() {
    let renderer = TeraRenderer::new();
    let ctx = tera::Context::new();
    let result = renderer.render("missing", &ctx);
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err, librawssg_error::Error::Render(_)));
    }
}

#[test]
fn render_with_missing_variable_errors() {
    let mut renderer = TeraRenderer::new();
    must!(
        renderer.add_raw_template("missing_var", "{{ does_not_exist }}"),
        "add_raw_template"
    );
    let ctx = tera::Context::new();
    let result = renderer.render("missing_var", &ctx);
    assert!(result.is_err());
}

#[test]
fn add_invalid_template_errors() {
    let mut renderer = TeraRenderer::new();
    let result = renderer.add_raw_template("invalid", "{% if %}");
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err, librawssg_error::Error::Render(_)));
    }
}

#[test]
fn add_template_file_works() {
    let dir = must!(tempfile::TempDir::new(), "TempDir::new");
    let file_path = dir.path().join("hello.tera");
    must!(std::fs::write(&file_path, "{{ name }}"), "write template");

    let mut renderer = TeraRenderer::new();
    must!(renderer.add_template_file(&file_path), "add_template_file");

    let mut ctx = tera::Context::new();
    ctx.insert("name", "World");
    let output = must!(renderer.render("hello.tera", &ctx), "render");
    assert_eq!(output, "World");
}

#[test]
fn add_template_files_from_dir_works() {
    let dir = must!(tempfile::TempDir::new(), "TempDir::new");
    must!(std::fs::write(dir.path().join("a.tera"), "A"), "write a");
    must!(std::fs::write(dir.path().join("b.tera"), "B"), "write b");

    let mut renderer = TeraRenderer::new();
    must!(
        renderer.add_template_files_from_dir(dir.path()),
        "add from dir"
    );

    let ctx = tera::Context::new();
    let out_a = must!(renderer.render("a.tera", &ctx), "render a");
    let out_b = must!(renderer.render("b.tera", &ctx), "render b");
    assert_eq!(out_a, "A");
    assert_eq!(out_b, "B");
}

#[test]
fn load_templates_dir_recursive() {
    let dir = must!(tempfile::TempDir::new(), "TempDir::new");
    let sub_dir = dir.path().join("sub");
    must!(std::fs::create_dir(&sub_dir), "create subdir");
    must!(
        std::fs::write(sub_dir.join("nested.tera"), "Nested"),
        "write nested"
    );

    let mut renderer = TeraRenderer::new();
    must!(
        renderer.load_templates_dir(dir.path()),
        "load templates dir"
    );

    let ctx = tera::Context::new();
    let output = must!(renderer.render("sub/nested.tera", &ctx), "render nested");
    assert_eq!(output, "Nested");
}

#[test]
fn autoescape_escapes_html() {
    let mut renderer = TeraRenderer::new();
    renderer.enable_autoescape();
    must!(
        renderer.add_raw_template("esc.html", "{{ content }}"),
        "add_raw_template"
    );
    let mut ctx = tera::Context::new();
    ctx.insert("content", "<script>alert(1)</script>");
    let output = must!(renderer.render("esc.html", &ctx), "render");
    assert_eq!(output, "&lt;script&gt;alert(1)&lt;&#x2F;script&gt;");
}

#[test]
fn render_str_autoescapes_by_default() {
    let renderer = TeraRenderer::new();
    let mut ctx = tera::Context::new();
    ctx.insert("content", "<b>bold</b>");
    let output = must!(renderer.render_str("{{ content }}", &ctx), "render_str");
    assert_eq!(output, "&lt;b&gt;bold&lt;&#x2F;b&gt;");
}

#[test]
fn inheritance_and_blocks() {
    let mut renderer = TeraRenderer::new();
    must!(
        renderer.add_raw_template(
            "base",
            "<html>{% block content %}Default{% endblock %}</html>"
        ),
        "add base"
    );
    must!(
        renderer.add_raw_template(
            "child",
            "{% extends \"base\" %}{% block content %}Child content{% endblock %}"
        ),
        "add child"
    );

    let ctx = tera::Context::new();
    let output = must!(renderer.render("child", &ctx), "render child");
    assert_eq!(output, "<html>Child content</html>");
}

#[test]
fn macros_work() {
    let mut renderer = TeraRenderer::new();
    must!(
        renderer.add_raw_template(
            "macro",
            "{% macro hello(name) %}Hello, {{ name }}{% endmacro hello %}{{ self::hello(name=\"World\") }}"
        ),
        "add macro template"
    );
    let ctx = tera::Context::new();
    let output = must!(renderer.render("macro", &ctx), "render macro");
    assert_eq!(output, "Hello, World");
}

#[test]
fn include_partial_works() {
    let mut renderer = TeraRenderer::new();
    must!(
        renderer.add_raw_template("partial", "Partial content"),
        "add partial"
    );
    must!(
        renderer.add_raw_template("main", "{% include \"partial\" %}"),
        "add main"
    );
    let ctx = tera::Context::new();
    let output = must!(renderer.render("main", &ctx), "render main");
    assert_eq!(output, "Partial content");
}

#[test]
fn context_downcast_works() {
    let ctx = sample_context();
    let dyn_ctx: &dyn RenderContext = &ctx;
    assert!(dyn_ctx.as_any().is::<tera::Context>());
}

#[test]
fn context_mutable_downcast_works() {
    let mut ctx = sample_context();
    let dyn_ctx: &mut dyn RenderContext = &mut ctx;
    assert!(dyn_ctx.as_mut_any().is::<tera::Context>());
}

#[test]
fn as_tera_returns_underlying_engine() {
    let renderer = TeraRenderer::new();
    let tera = renderer.as_tera();
    let _ = tera;
}
#[test]
fn as_tera_mut_allows_mutation() {
    let mut renderer = TeraRenderer::new();
    let tera_mut = renderer.as_tera_mut();
    let add_result = tera_mut
        .add_raw_template("via_mut", "ok")
        .map_err(|e| librawssg_error::Error::Render(e.to_string()));
    must!(add_result, "add raw template via mutable access");
    let ctx = tera::Context::new();
    let output = must!(renderer.render("via_mut", &ctx), "render via mut");
    assert_eq!(output, "ok");
}
