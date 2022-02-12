use anyhow::{Context, Result};
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub base_url: String,
    pub ignore_hidden: bool,
    #[serde(rename="ignore")]
    pub ignore_paths: Vec<PathBuf>,
    pub template_ext: String,
    pub target_ext: String,
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

#[derive(Default, Deserialize, Clone)]
#[serde(default)]
pub struct CollectionConfig {
    pub title: String,
    pub description: String,
    pub template: Option<PathBuf>,
    pub connections: Vec<PathBuf>,
    pub rss: Option<PathBuf>,
}

impl CollectionConfig {
    pub fn load(path: &PathBuf) -> Result<Self> {
        let file_str = fs::read_to_string(&path)?;
        let config = serde_yaml::from_str(&file_str)?;
        Ok(config)
    }
}
