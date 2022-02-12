use crate::{
    config::Config,
    sources::TemplateSource,
    util::minifier::{self, MinificationLevel},
};
use anyhow::{Context, Result};
use log::info;
use std::path::PathBuf;
use tera::{Context as TemplateContext, Tera};

#[derive(Clone)]
pub struct ContentRenderer {
    mfc_level: MinificationLevel,
    tera: Tera,
}

impl ContentRenderer {
    pub fn load(
        parent_dir: PathBuf,
        config: &Config,
        mfc_level: MinificationLevel,
    ) -> Result<Self> {
        let dirs = format!("{}/**/*.{}", parent_dir.display(), config.template_ext);

        let mut tera = Tera::new(&dirs)?;
        tera.autoescape_on(vec![]);

        info!("Loaded {} template files", tera.templates.len());

        Ok(Self { tera, mfc_level })
    }

    pub fn render(&self, path: &PathBuf, template: Option<&dyn TemplateSource>) -> Result<String> {
        let context = {
            if let Some(template) = template {
                template.get_context()
            } else {
                TemplateContext::default()
            }
        };
        let html_output = self
            .tera
            .render(path.to_str().unwrap(), &context)
            .with_context(|| {
                format!(
                    "Failed to render template '{}' with context '{context:#?}'",
                    path.display(),
                )
            })?;
        Ok(minifier::minify_string(&html_output, &self.mfc_level))
    }
}
