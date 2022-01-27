use anyhow::{Result, Context};
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Deserialize)]
pub struct Config {
    #[serde(rename = "ignore")]
    pub ignore_paths: Vec<PathBuf>,
    pub ignore_hidden: bool,
    pub base_url: String,
    pub template_ext: String,
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self> {
        let file_str = fs::read_to_string(&path).with_context(|| "Failed to read configuration file")?;
        let config = serde_yaml::from_str(&file_str).with_context(|| "Failed to deserialize configuration file")?;
        Ok(config)
    }
}

#[derive(Deserialize, Clone)]
pub struct FeedConfig {
    pub title: String,
    pub description: String,
    pub rss_path: Option<PathBuf>,
    pub templates: Option<Vec<PathBuf>>,
    pub index_templates: Option<Vec<PathBuf>>,
}

impl FeedConfig {
    pub fn load(path: &PathBuf) -> Result<Self> {
        let file_str = fs::read_to_string(&path)?;
        let config = serde_yaml::from_str(&file_str)?;
        Ok(config)
    }
}
