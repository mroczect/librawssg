#[cfg(feature = "pulldown")]
mod markdown_tests {
    use librawssg::markdown::{MarkdownRenderer, PulldownMarkdown};

    #[test]
    fn basic_markdown() {
        let html = PulldownMarkdown.render("# Hello\nWorld");
        assert!(html.contains("<h1>"));
        assert!(html.contains("World"));
    }

    #[test]
    fn tables_rendered() {
        let input = "| A | B |\n|---|---|\n| 1 | 2 |";
        let html = PulldownMarkdown.render(input);
        assert!(html.contains("<table>"));
    }

    #[test]
    fn strikethrough() {
        let html = PulldownMarkdown.render("~~deleted~~");
        assert!(html.contains("<del>"));
    }

    #[test]
    fn task_list() {
        let html = PulldownMarkdown.render("- [ ] task\n- [x] done");
        assert!(html.contains("checkbox"));
    }

    #[test]
    fn empty_input() {
        assert_eq!(PulldownMarkdown.render(""), "");
    }
}
