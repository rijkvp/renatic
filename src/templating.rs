use crate::config::Config;
use anyhow::{Context, Result};
use log::info;
use std::path::PathBuf;
use tera::{Context as TeraContext, Tera};

#[derive(Clone)]
pub struct TemplateEngine {
    parent_dir: PathBuf,
    tera: Tera,
}

impl TemplateEngine {
    pub fn load(parent_dir: PathBuf, config: &Config) -> Result<Self> {
        let dirs = format!("{}/**/*.{}", parent_dir.display(), config.template_ext);

        let mut tera = Tera::new(&dirs)?;
        tera.autoescape_on(vec![]);

        info!("Loaded {} template files", tera.templates.len());

        Ok(Self { parent_dir, tera })
    }

    pub fn render_file(&self, path: &PathBuf, context: &TeraContext) -> Result<String> {
        let canonicalized_path = path.canonicalize()?;
        let child_path = canonicalized_path.strip_prefix(&self.parent_dir)?;
        self.tera
            .render(child_path.to_str().unwrap(), context)
            .with_context(|| {
                format!(
                    "Failed to render template '{}' with context '{context:?}'",
                    child_path.display(),
                )
            })
    }
}
