pub mod meta;
pub mod entry;

use tera::Context as TemplateContext;

pub trait TemplateSource {
    fn get_context(&self) -> TemplateContext;
}