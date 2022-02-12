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
    pub location: Location,
    pub content: String,
    pub collection: Option<CollectionBinding>,
}

impl Entry {
    pub fn load(location: Location, collection: Option<CollectionBinding>) -> Result<Entry> {
        let file_str = fs::read_to_string(&location.source_path)?;
        let (meta_str, source) = parser::parse_markdown_with_meta(&file_str)
            .with_context(|| format!("Failed to read file '{}'", location.source_path.display()))?;
        let meta = Meta::from_str(&meta_str).with_context(|| "Failed to parse content meta")?;

        Ok(Entry {
            location,
            meta,
            content: source,
            collection,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Location {
    /// Source path
    #[serde(skip)]
    pub source_path: PathBuf,
    /// Source child path
    #[serde(skip)]
    pub source_child_path: PathBuf,
    /// Source file name
    #[serde(skip)]
    pub source_file_name: String,

    /// File name without extension
    pub target_file_stem: String,

    /// Path of destination file
    #[serde(skip)]
    pub target_path: PathBuf,
    /// Child path of destination file
    #[serde(rename = "path")]
    pub target_child_path: PathBuf,

    pub route: PathBuf,
    pub short_route: PathBuf,
}

impl Location {
    pub fn new(
        source_path: &PathBuf,
        source_dir: &PathBuf,
        target_dir: &PathBuf,
        extension: &str,
        custom_file_stem: Option<&str>,
    ) -> Result<Self> {
        let source_path = source_path.to_owned();
        let source_child_path = source_path.strip_prefix(source_dir)?.to_owned();

        let source_file_name = source_child_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let target_file_stem = {
            if let Some(stem) = custom_file_stem {
                stem.to_owned()
            } else {
                source_child_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            }
        };

        let target_path = target_dir
            .join(&source_child_path)
            .with_file_name(&target_file_stem)
            .with_extension(extension);
        let target_child_path = target_path.strip_prefix(target_dir)?.to_path_buf();

        let route = PathBuf::from("/").join(&target_child_path);
        let short_route = route.with_extension("");

        Ok(Self {
            source_path,
            source_child_path,
            source_file_name,
            target_file_stem,
            target_path,
            target_child_path,
            route,
            short_route,
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
