use crate::{config::Config, sources::TemplateSource};
use anyhow::{Context, Result};
use log::info;
use std::path::PathBuf;
use tera::{Context as TemplateContext, Tera};

#[derive(Clone)]
pub struct ContentRenderer {
    parent_dir: PathBuf,
    tera: Tera,
}

impl ContentRenderer {
    pub fn load(parent_dir: PathBuf, config: &Config) -> Result<Self> {
        let dirs = format!("{}/**/*.{}", parent_dir.display(), config.template_ext);

        let mut tera = Tera::new(&dirs)?;
        tera.autoescape_on(vec![]);

        info!("Loaded {} template files", tera.templates.len());

        Ok(Self { parent_dir, tera })
    }

    pub fn render(&self, path: &PathBuf, template: Option<&dyn TemplateSource>) -> Result<String> {
        let canonicalized_path = path.canonicalize()?;
        let child_path = canonicalized_path.strip_prefix(&self.parent_dir)?;
        let context = {
            if let Some(template) = template {
                template.get_context()
            } else {
                TemplateContext::default()
            }
        };
        self.tera
            .render(child_path.to_str().unwrap(), &context)
            .with_context(|| {
                format!(
                    "Failed to render template '{}' with context '{context:#?}'",
                    child_path.display(),
                )
            })
    }
}
