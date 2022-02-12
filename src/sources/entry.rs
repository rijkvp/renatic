use super::{meta::Meta, TemplateSource};
use crate::{config::CollectionConfig, util::parser};
use anyhow::{Context, Result};
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
    pub content: String,
    pub collection: Option<CollectionBinding>,
}

impl Entry {
    pub fn load(
        source_path: &PathBuf,
        source_dir: &PathBuf,
        out_dir: &PathBuf,
        target_ext: &str,
        collection: Option<CollectionBinding>,
    ) -> Result<Entry> {
        let file_str = fs::read_to_string(&source_path)?;
        let (meta_str, source) = parser::parse_markdown_with_meta(&file_str)
            .with_context(|| format!("Failed to read file '{}'", source_path.display()))?;
        let meta = Meta::from_str(&meta_str).with_context(|| "Failed to parse content meta")?;

        let location = EntryLocation::from_paths(&source_path, &source_dir, &out_dir, &target_ext)
            .with_context(|| "Failed to get entry location")?;

        Ok(Entry {
            location,
            meta,
            content: source,
            collection,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EntryLocation {
    /// Source path
    #[serde(skip)]
    pub source_path: PathBuf,
    /// Source child path
    #[serde(skip)]
    pub source_child_path: PathBuf,
    /// Source file name
    #[serde(skip)]
    pub source_file_name: String,

    /// Source file name without extension
    pub file_stem: String,

    /// Path of destination file
    #[serde(skip)]
    pub target_path: PathBuf,
    /// Child path of destination file
    #[serde(rename = "path")]
    pub target_child_path: PathBuf,

    pub route: PathBuf,
    pub short_route: PathBuf,
}

impl EntryLocation {
    pub fn from_paths(
        source_path: &PathBuf,
        source_dir: &PathBuf,
        out_dir: &PathBuf,
        target_ext: &str,
    ) -> Result<Self> {
        let source_child_path = source_path.strip_prefix(source_dir)?.to_path_buf();
        let source_file_name = source_child_path.file_name().context("Invalid file name")?;
        let file_stem = source_child_path.file_stem().context("Invalid file stem")?;

        let target_path = out_dir.join(&source_child_path).with_extension(target_ext);
        let target_child_path = target_path.strip_prefix(out_dir)?.to_path_buf();
        // Route with root /
        let route = PathBuf::from("/").join(&target_child_path);

        Ok(Self {
            source_path: source_path.clone(),
            source_child_path: source_child_path.to_path_buf(),
            source_file_name: source_file_name.to_str().unwrap().to_string(),
            file_stem: file_stem.to_str().unwrap().to_string(),
            target_path: target_path.clone(),
            target_child_path,
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
