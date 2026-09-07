use librawssg_config::Config;
use librawssg_error::Result;
use librawssg_handler::Document;
use librawssg_templates::RenderContext;

pub trait ContextBuilder: Send + Sync {
    fn build_context(&self, config: &Config, doc: &Document) -> Result<Box<dyn RenderContext>>;
}

#[cfg(feature = "tera")]
pub struct TeraContextBuilder;

#[cfg(feature = "tera")]
impl ContextBuilder for TeraContextBuilder {
    fn build_context(&self, config: &Config, doc: &Document) -> Result<Box<dyn RenderContext>> {
        let mut ctx = tera::Context::new();
        ctx.insert("site", &config.site);
        ctx.insert("page_title", &doc.metadata.title);
        ctx.insert("page_description", &doc.metadata.description);
        ctx.insert("page_author", &doc.metadata.author);
        ctx.insert("page_date", &doc.metadata.date);
        ctx.insert("page_tags", &doc.metadata.tags);
        ctx.insert("page_content", &doc.body);
        ctx.insert("page_url", &doc.url);
        ctx.insert("page_depth", &doc.depth);
        ctx.insert("page_type", &doc.content_type);
        ctx.insert("page_is_list", &doc.is_list);
        ctx.insert("page_list_items", &doc.list_items);
        Ok(Box::new(ctx))
    }
}
