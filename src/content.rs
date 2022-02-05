use crate::meta::Meta;
use anyhow::{anyhow, Context, Result};
use log::trace;
use pulldown_cmark::{Options, Parser, html};
use serde::Serialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct Content {
    pub location: ContentLocation,
    pub meta: Meta,
    pub content: String,
}

impl Content {
    pub fn load(
        parent_dir: &PathBuf,
        source_dir: &PathBuf,
        path: &PathBuf,
        target_ext: &str,
    ) -> Result<Content> {
        let file_str = fs::read_to_string(&path)?;
        let (meta, content) = split_md_meta(&file_str)
            .with_context(|| format!("Failed to read file '{}'", path.display()))?;

        let location = ContentLocation::from_paths(&parent_dir, source_dir, &path, &target_ext)
            .with_context(|| "Failed to get post location")?;
        trace!("Loaded post '{}'", &location.child_path.display());

        Ok(Content {
            location,
            meta,
            content,
        })
    }
}

fn split_md_meta(input: &str) -> Result<(Meta, String)> {
    let splits: Vec<&str> = input.split("---").collect();
    if splits.len() != 3 {
        return Err(anyhow!("Invalid meta section!"));
    }
    let meta = Meta::from_str(&splits[1]).with_context(|| "Failed to read meta information")?;
    let contents = md_to_html(&splits[2]);
    Ok((meta, contents))
}

fn md_to_html(input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(input, options);

    // Write to String buffer.
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

#[derive(Debug, Clone, Serialize)]
pub struct ContentLocation {
    /// Source path
    #[serde(skip)]
    pub source_path: PathBuf,
    /// Source child path
    pub child_path: PathBuf,
    /// File name
    pub file_name: String,
    /// File name without extension
    pub file_stem: String,

    /// Path of destination file
    #[serde(rename = "path")]
    pub target_path: PathBuf,

    pub route: PathBuf,
    pub short_route: PathBuf,
}

impl ContentLocation {
    pub fn from_paths(
        parent_dir: &PathBuf,
        source_dir: &PathBuf,
        source_path: &PathBuf,
        target_ext: &str,
    ) -> Result<Self> {
        let child_path = parent_dir.join(source_path.strip_prefix(source_dir)?.to_path_buf());

        let file_name = child_path.file_name().context("Invalid file name")?;
        let file_stem = child_path.file_stem().context("Invalid file name")?;
        let target_path = child_path.with_extension(target_ext);
        // Route with root /
        let route = PathBuf::from("/").join(&target_path);
        Ok(Self {
            source_path: source_path.clone(),
            child_path: child_path.to_path_buf(),
            file_name: file_name.to_str().unwrap().to_string(),
            file_stem: file_stem.to_str().unwrap().to_string(),
            target_path: target_path.clone(),
            route: route.clone(),
            short_route: route.with_extension(""),
        })
    }
}
