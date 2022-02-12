use anyhow::{Context, Result};
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub ignore_hidden: bool,
    #[serde(default)]
    pub ignore_paths: Vec<PathBuf>,
    #[serde(default)]
    pub template_ext: String,
    #[serde(default)]
    pub target_ext: String,
    #[serde(default)]
    pub content_ext: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: String::from("https://www.example.com"),
            ignore_hidden: true,
            ignore_paths: vec![PathBuf::from("renatic.yaml")],
            template_ext: String::from("html"),
            target_ext: String::from("html"),
            content_ext: String::from("md"),
        }
    }
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self> {
        let file_str =
            fs::read_to_string(&path).with_context(|| "Failed to read configuration file")?;
        let config = serde_yaml::from_str::<Config>(&file_str)
            .with_context(|| "Failed to deserialize configuration file")?;

        Ok(config)
    }
}

#[derive(Deserialize, Clone)]
pub struct CollectionConfig {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub templates: Option<Vec<PathBuf>>,
    #[serde(default)]
    pub connections: Option<Vec<PathBuf>>,
    #[serde(default)]
    pub rss: Option<PathBuf>,
}

impl CollectionConfig {
    pub fn load(path: &PathBuf) -> Result<Self> {
        let file_str = fs::read_to_string(&path)?;
        let config = serde_yaml::from_str(&file_str)?;
        Ok(config)
    }
}
