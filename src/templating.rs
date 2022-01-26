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
    pub fn load(parent_dir: PathBuf) -> Result<Self> {
        let dirs = format!("{}/*/**.html", parent_dir.display());
        let tera = Tera::new(&dirs)?;

        info!("Loaded {} template files", tera.templates.len());

        Ok(Self { parent_dir, tera })
    }

    pub fn render_file(&self, path: &PathBuf, context: &TeraContext) -> Result<String> {
        let rel_path = path.strip_prefix(&self.parent_dir)?;
        self.tera
            .render(rel_path.to_str().unwrap(), context)
            .with_context(|| {
                format!(
                    "Failed to render template '{}' with context '{context:?}'",
                    path.display()
                )
            })
    }
}
