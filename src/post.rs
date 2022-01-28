use crate::meta::Meta;
use anyhow::{Context, Result};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct Post {
    pub location: PostLocation,
    pub meta: Meta,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostLocation {
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

impl PostLocation {
    pub fn from_paths(
        parent_dir: &PathBuf,
        source_dir: &PathBuf,
        source_path: &PathBuf,
        target_ext: &str,
    ) -> Result<Self> {
        let child_path = parent_dir.join(source_path.strip_prefix(source_dir)?.to_path_buf());

        let file_name = child_path.file_name().context("Invalid file name")?;
        let file_stem = child_path.file_name().context("Invalid file name")?;
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
