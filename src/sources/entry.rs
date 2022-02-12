use super::{meta::Meta, TemplateSource};
use crate::{config::CollectionConfig, util::parser};
use anyhow::{Context, Result};
use log::trace;
use serde::Serialize;
use std::{fs, path::PathBuf};
use tera::Context as TemplateContext;

impl TemplateSource for Entry {
    fn get_context(&self) -> TemplateContext {
        TemplateContext::from_serialize(self).unwrap()
    }
}

impl TemplateSource for CollectionBinding {
    fn get_context(&self) -> TemplateContext {
        TemplateContext::from_serialize(self).unwrap()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    pub meta: Meta,
    pub location: EntryLocation,
    pub source: String,
    pub collection: Option<CollectionBinding>,
}

impl Entry {
    pub fn load(
        parent_dir: &PathBuf,
        source_dir: &PathBuf,
        path: &PathBuf,
        target_ext: &str,
    ) -> Result<Entry> {
        let file_str = fs::read_to_string(&path)?;
        let (meta_str, content) = parser::parse_markdown_with_meta(&file_str)
            .with_context(|| format!("Failed to read file '{}'", path.display()))?;
        let meta = Meta::from_str(&meta_str)?;

        let location = EntryLocation::from_paths(&parent_dir, source_dir, &path, &target_ext)
            .with_context(|| "Failed to get entry location")?;
        trace!("Loaded content of  '{}'", &location.child_path.display());

        Ok(Entry {
            location,
            meta,
            source: content,
            collection: None,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EntryLocation {
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

impl EntryLocation {
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

#[derive(Debug, Clone, Serialize)]
pub struct CollectionBinding {
    pub entries: Vec<Entry>,
    pub rss: Option<RssInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RssInfo {
    pub path: PathBuf,
    pub route: PathBuf,
}

impl CollectionBinding {
    pub fn new(entries: Vec<Entry>, config: &CollectionConfig) -> Self {
        let rss = {
            if let Some(rss_path) = &config.rss {
                Some(RssInfo {
                    path: rss_path.clone(),
                    route: PathBuf::from("/").join(&rss_path),
                })
            } else {
                None
            }
        };
        Self { entries, rss }
    }
}
