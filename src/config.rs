use anyhow::{Context, Result};
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub base_url: String,
    pub ignore_hidden: Option<bool>,
    pub ignore: Option<Vec<PathBuf>>,
    pub template_ext: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    pub base_url: String,
    pub ignore_hidden: bool,
    pub ignore_paths: Vec<PathBuf>,
    pub template_ext: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: "https://www.example.com".to_string(),
            ignore_hidden: true,
            ignore_paths: vec![PathBuf::from("renatic.yaml")],
            template_ext: "html".to_string(),
        }
    }
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self> {
        let file_str =
            fs::read_to_string(&path).with_context(|| "Failed to read configuration file")?;
        let cfg_file = serde_yaml::from_str::<ConfigFile>(&file_str)
            .with_context(|| "Failed to deserialize configuration file")?;

        let mut ignore_paths = Config::default().ignore_paths;
        ignore_paths.append(&mut cfg_file.ignore.unwrap_or(vec![]));

        Ok(Self {
            base_url: cfg_file.base_url,
            ignore_hidden: cfg_file
                .ignore_hidden
                .unwrap_or(Config::default().ignore_hidden),
            ignore_paths,
            template_ext: cfg_file
                .template_ext
                .unwrap_or(Config::default().template_ext),
        })
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
