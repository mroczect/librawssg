pub trait MarkdownRenderer: Send + Sync {
    fn render(&self, markdown: &str) -> String;
}

#[cfg(feature = "pulldown")]
pub struct PulldownMarkdown;

#[cfg(feature = "pulldown")]
impl MarkdownRenderer for PulldownMarkdown {
    fn render(&self, md: &str) -> String {
        use pulldown_cmark::{Options, Parser, html};
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        let parser = Parser::new_ext(md, options);
        let mut html_out = String::new();
        html::push_html(&mut html_out, parser);
        html_out
    }
}
